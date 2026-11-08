use crate::{docs, policy};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

const SCHEMA: &str = include_str!("../en/schemas/cgacp-task.schema.json");
const STORE: &str = ".cgacp/tasks";
const RECORD_LIMIT: usize = 4 * 1024 * 1024;
const TASK_LIMIT: usize = 1024;
const CLAIM_SCAN_BYTES: u64 = 16 * 1024 * 1024;
const SCOPES: [&str; 4] = [
    "read_scope",
    "write_scope",
    "verification_scope",
    "documentation_scope",
];
const STATES: &[&str] = &[
    "PROPOSED",
    "READY",
    "IN_PROGRESS",
    "IMPLEMENTED",
    "VERIFIED",
    "ACCEPTED",
    "DOCS_SYNCED",
    "CLOSED",
    "BLOCKED",
    "CANCELLED",
    "SUPERSEDED",
];
static NEXT_STAGE: AtomicU64 = AtomicU64::new(0);

fn text(value: &Value) -> &str {
    value.as_str().unwrap_or_default()
}

fn items(value: &Value) -> impl Iterator<Item = &Value> {
    value.as_array().into_iter().flatten()
}

fn parse_json(content: &str) -> Result<Value, String> {
    let parsed: Value = serde_json::from_str(content).map_err(|_| "Invalid task JSON")?;
    let unambiguous: serde_yaml::Value =
        serde_yaml::from_str(content).map_err(|_| "Invalid or duplicate task JSON keys")?;
    if serde_json::to_value(unambiguous).ok().as_ref() != Some(&parsed) {
        return Err("Ambiguous task JSON".into());
    }
    Ok(parsed)
}

fn valid_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.len() > 64
        || !id.as_bytes()[0].is_ascii_alphanumeric()
        || !id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
        || reserved_component(id)
    {
        return Err("Invalid task or actor ID".into());
    }
    Ok(())
}

fn reserved_component(part: &str) -> bool {
    let name = part.split('.').next().unwrap_or("").to_ascii_uppercase();
    matches!(
        name.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) || name
        .strip_prefix("COM")
        .or_else(|| name.strip_prefix("LPT"))
        .is_some_and(|suffix| {
            matches!(
                suffix,
                "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
}

fn plain_relative(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.len() > 512
        || path
            .chars()
            .any(|c| c.is_control() || "\\:<>|?*\"".contains(c))
        || path.split('/').any(|part| {
            part.is_empty()
                || part.starts_with(' ')
                || part.ends_with([' ', '.'])
                || part.eq_ignore_ascii_case(".git")
                || reserved_component(part)
        })
    {
        return Err("Unsafe task-relative path".into());
    }
    Ok(())
}

pub fn relative(path: &str) -> Result<(), String> {
    plain_relative(path)?;
    if path
        .split('/')
        .any(|part| part.eq_ignore_ascii_case(".cgacp"))
    {
        return Err("Control-plane state is outside task file scope".into());
    }
    Ok(())
}

fn key(path: &str) -> String {
    if cfg!(windows) {
        path.to_lowercase()
    } else {
        path.to_owned()
    }
}

fn known_repo(registry: &Value, repo: &str) -> bool {
    repo == "cirnet-rules"
        || registry["repositories"].get(repo).is_some()
        || registry["control_plane"]["repositories"]
            .get(repo)
            .is_some()
}

fn descriptor(root: &Path, record: &Value, registry: &Value) -> Result<(), String> {
    static VALIDATOR: OnceLock<jsonschema::Validator> = OnceLock::new();
    let validator = VALIDATOR.get_or_init(|| {
        let schema = serde_json::from_str(SCHEMA).expect("valid embedded descriptor schema");
        jsonschema::validator_for(&schema).expect("compilable embedded descriptor schema")
    });
    validator
        .validate(record)
        .map_err(|error| format!("Task descriptor violates schema at {}", error.instance_path))?;
    reject_credentials(record)?;
    let id = text(&record["id"]);
    valid_id(id)?;
    let repo = text(&record["primary_repo"]);
    if repo != "_workspace" && !known_repo(registry, repo) {
        return Err("Primary repository is not registered".into());
    }
    for (field, dir) in [("spec", "specs"), ("plan", "plans")] {
        let path = text(&record[field]);
        relative(path)?;
        if !path.starts_with(&format!("en/{dir}/{repo}/{id}/")) || !path.ends_with(".md") {
            return Err(format!(
                "Task {field} must be in its central English record"
            ));
        }
        read_authority(root, path)?;
    }
    for hint in items(&record["hints"]) {
        relative(text(hint))?;
    }
    for scope in SCOPES {
        for (name, paths) in record[scope].as_object().unwrap() {
            if !known_repo(registry, name) {
                return Err(format!("Unknown repository in {scope}: {name}"));
            }
            let mut seen = BTreeSet::new();
            for path in items(paths) {
                let path = text(path);
                relative(path)?;
                if matches!(scope, "write_scope" | "documentation_scope")
                    && name == "cirnet-rules"
                    && policy_input(path)
                {
                    return Err("Ordinary task writes cannot modify governance authority".into());
                }
                if !seen.insert(key(path)) {
                    return Err(format!("Aliased path in {scope}: {name}"));
                }
            }
        }
    }
    if record["mode"] != "WORK" && !record["write_scope"].as_object().unwrap().is_empty() {
        return Err("Code write scope requires WORK mode".into());
    }
    if matches!(text(&record["role"]), "reviewer" | "verifier")
        && items(&record["authorization"]["actions"])
            .any(|action| matches!(text(action), "write" | "docs" | "network" | "release"))
    {
        return Err("Review and verification roles cannot hold write or release authority".into());
    }
    for command in items(&record["commands"]) {
        if registry["control_plane"]["commands"]
            .get(text(command))
            .is_none()
        {
            return Err("Task requests a command absent from the policy registry".into());
        }
    }
    let mut destinations = BTreeSet::new();
    for target in items(&record["docs_targets"]) {
        let name = text(&target["repo"]);
        let path = text(&target["path"]);
        let locale = text(&target["locale"]);
        relative(path)?;
        let prefix = if name == "cirnet-rules" {
            format!("{locale}/")
        } else {
            format!("docs/{locale}/")
        };
        if !path.starts_with(&prefix)
            || !path.ends_with(".md")
            || !items(&record["documentation_scope"][name]).any(|allowed| allowed == path)
            || !destinations.insert((name, key(path)))
        {
            return Err("Document target must be a unique scoped Markdown locale file".into());
        }
    }
    if destinations.is_empty() == record["docs_not_required"].is_null() {
        return Err(
            "Specify documentation targets or a reason documentation is unnecessary".into(),
        );
    }
    for field in ["parent", "depends_on", "required_children"] {
        let refs = if field == "parent" {
            record.get(field).into_iter().collect::<Vec<_>>()
        } else {
            items(&record[field]).collect()
        };
        for other in refs {
            valid_id(text(other))?;
            if id.eq_ignore_ascii_case(text(other)) {
                return Err("A task cannot depend on itself".into());
            }
        }
    }
    if items(&record["required_children"]).next().is_some() && repo != "_workspace" {
        return Err("Cross-repository parent tasks must use the workspace initiative scope".into());
    }
    if let Some(parent) = record["parent"].as_str() {
        let parent = load(root, parent)?;
        if parent["descriptor"]["primary_repo"] != "_workspace" {
            return Err("Parent task is not a workspace initiative".into());
        }
    }
    for dependency in items(&record["depends_on"]) {
        load(root, text(dependency))?;
    }
    validate_dependencies(root, record)?;
    Ok(())
}

fn policy_input(path: &str) -> bool {
    let normalized = path.to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "agents.md" | "governance.yaml" | "en/agents.md"
    ) || [
        "en/policies/",
        "en/rules/",
        "en/plans/",
        "en/specs/",
        "en/schemas/",
    ]
    .iter()
    .any(|prefix| normalized.starts_with(prefix))
}

fn reject_credentials(value: &Value) -> Result<(), String> {
    fn obvious_credential(value: &Value) -> bool {
        match value {
            Value::Object(values) => values.values().any(obvious_credential),
            Value::Array(values) => values.iter().any(obvious_credential),
            Value::String(text) => {
                let private_key = text.contains("-----BEGIN PRIVATE KEY-----")
                    || text.contains("-----BEGIN OPENSSH PRIVATE KEY-----")
                    || text.contains("-----BEGIN RSA PRIVATE KEY-----");
                let token = text
                    .split(|c: char| c.is_whitespace() || "`'\"=,:;()[]{}<>".contains(c))
                    .any(|word| {
                        ["ghp_", "github_pat_", "sk-proj-", "xoxb-"]
                            .iter()
                            .any(|prefix| {
                                word.strip_prefix(prefix).is_some_and(|suffix| {
                                    suffix.len() >= 16
                                        && suffix.bytes().all(|byte| {
                                            byte.is_ascii_alphanumeric() || b"-_".contains(&byte)
                                        })
                                })
                            })
                    });
                private_key || token
            }
            _ => false,
        }
    }
    if obvious_credential(value) {
        return Err("Possible credential in task input; provide a redacted reference".into());
    }
    Ok(())
}

fn validate_dependencies(root: &Path, record: &Value) -> Result<(), String> {
    let id = text(&record["id"]);
    let mut pending = items(&record["depends_on"])
        .chain(items(&record["required_children"]))
        .map(text)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    while let Some(dependency) = pending.pop() {
        if dependency.eq_ignore_ascii_case(id) {
            return Err("Task dependencies or required children form a cycle".into());
        }
        if !visited.insert(dependency.clone()) {
            continue;
        }
        if visited.len() > TASK_LIMIT {
            return Err("Task dependency traversal budget exceeded".into());
        }
        valid_id(&dependency)?;
        let path = root.join(format!("{STORE}/{}.json", dependency.to_ascii_lowercase()));
        match fs::symlink_metadata(path) {
            Ok(_) => {
                let state = load(root, &dependency)?;
                let descriptor = &state["descriptor"];
                pending.extend(
                    items(&descriptor["depends_on"])
                        .chain(items(&descriptor["required_children"]))
                        .map(text)
                        .map(str::to_owned),
                );
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                // A parent may declare children before their task files exist.
            }
            Err(_) => return Err("Cannot inspect a declared task dependency".into()),
        }
    }
    Ok(())
}

fn read_authority(root: &Path, path: &str) -> Result<String, String> {
    policy::read(&policy::safe_path(root, path)?)
}

fn descriptor_hash(record: &Value) -> Result<String, String> {
    serde_json::to_vec(record)
        .map(|bytes| docs::sha256(&bytes))
        .map_err(|_| "Cannot identify task descriptor".into())
}

fn authority_hashes(root: &Path, record: &Value) -> Result<[String; 3], String> {
    let spec = read_authority(root, text(&record["spec"]))?;
    let plan = read_authority(root, text(&record["plan"]))?;
    let mut policy = read_authority(root, "governance.yaml")?;
    let notes = "en/policies/governance.md";
    match fs::symlink_metadata(root.join(notes)) {
        Ok(_) => {
            policy.push('\0');
            policy.push_str(&read_authority(root, notes)?);
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(_) => return Err("Cannot inspect governance policy".into()),
    }
    Ok([
        docs::sha256(spec.as_bytes()),
        docs::sha256(plan.as_bytes()),
        docs::sha256(policy.as_bytes()),
    ])
}

fn directory(root: &Path, relative: &str, create: bool) -> Result<PathBuf, String> {
    let mut path = root.to_path_buf();
    for part in relative.split('/') {
        path.push(part);
        if create {
            #[cfg(unix)]
            let result = {
                use std::os::unix::fs::DirBuilderExt;
                fs::DirBuilder::new().mode(0o700).create(&path)
            };
            #[cfg(not(unix))]
            let result = fs::create_dir(&path);
            match result {
                Ok(()) => {}
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                Err(_) => return Err("Cannot create local task directory".into()),
            }
        }
        let metadata =
            fs::symlink_metadata(&path).map_err(|_| "Local task store is unavailable")?;
        if !metadata.is_dir() || policy::is_link(&metadata) {
            return Err("Local task store must be an ordinary directory".into());
        }
    }
    Ok(path)
}

struct TaskLock(File);

impl Drop for TaskLock {
    fn drop(&mut self) {
        // A concurrent child can inherit the descriptor until it executes.
        // Release the lock before closing, even if a duplicate remains open.
        let _ = self.0.unlock();
    }
}

fn lock(root: &Path, id: &str) -> Result<TaskLock, String> {
    let dir = directory(root, STORE, true)?;
    let path = dir.join(format!("{id}.lock"));
    if let Ok(meta) = fs::symlink_metadata(&path)
        && (!meta.is_file() || policy::is_link(&meta))
    {
        return Err("Unsafe local task lock".into());
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|_| "Cannot open local task lock")?;
    file.try_lock()
        .map_err(|_| "Task state is busy; resume with the current revision")?;
    Ok(TaskLock(file))
}

fn persist(root: &Path, id: &str, value: &Value) -> Result<(), String> {
    let directory = directory(root, STORE, false)?;
    let key = id.to_ascii_lowercase();
    let bytes = serde_json::to_vec_pretty(value).map_err(|_| "Cannot encode task state")?;
    if bytes.len() > RECORD_LIMIT {
        return Err("Task state exceeds its byte budget".into());
    }
    let stage = directory.join(format!(
        "{key}.stage-{}-{}",
        std::process::id(),
        NEXT_STAGE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&stage)
        .map_err(|_| "Cannot create task state stage")?;
    output
        .write_all(&bytes)
        .and_then(|_| output.sync_all())
        .map_err(|_| "Cannot persist staged task state")?;
    drop(output);
    let target = directory.join(format!("{key}.json"));
    if let Ok(meta) = fs::symlink_metadata(&target)
        && (!meta.is_file() || policy::is_link(&meta))
    {
        return Err("Unsafe local task state target".into());
    }
    let validated_directory = self::directory(root, STORE, false)?;
    if validated_directory != directory {
        return Err("Local task store changed during the update".into());
    }
    fs::rename(stage, target).map_err(|_| "Cannot atomically replace local task state".into())
}

pub fn load(root: &Path, id: &str) -> Result<Value, String> {
    valid_id(id)?;
    let path = format!("{STORE}/{}.json", id.to_ascii_lowercase());
    let record = parse_json(&read_authority(root, &path)?)?;
    if record["descriptor"]["id"] != id
        || !record["revision"].as_u64().is_some_and(|rev| rev > 0)
        || !STATES.contains(&text(&record["state"]))
    {
        return Err("Task state identity or revision is invalid".into());
    }
    Ok(record)
}

fn current(root: &Path, record: &Value) -> Result<(), String> {
    if record["descriptor_sha256"] != descriptor_hash(&record["descriptor"])? {
        return Err("Task authorization descriptor is stale; request explicit revision".into());
    }
    let actual = authority_hashes(root, &record["descriptor"])
        .map_err(|_| "Task authority is stale or unreadable; request explicit revision")?;
    for (field, hash) in ["spec_sha256", "plan_sha256", "policy_sha256"]
        .into_iter()
        .zip(actual)
    {
        if record[field] != hash {
            return Err(format!(
                "Task authority is stale ({field}); request explicit revision"
            ));
        }
    }
    Ok(())
}

pub fn load_current(root: &Path, id: &str) -> Result<Value, String> {
    let record = load(root, id)?;
    current(root, &record)?;
    check_expiry(&record["descriptor"]["authorization"])?;
    Ok(record)
}

fn authority_fields(record: &Value) -> Value {
    json!([
        record["descriptor"],
        record["spec_sha256"],
        record["plan_sha256"],
        record["policy_sha256"],
        record["descriptor_sha256"],
        record["approval_epoch"]
    ])
}

fn update_impl(
    root: &Path,
    id: &str,
    revision: u64,
    check_current: bool,
    apply: impl FnOnce(&mut Value) -> Result<(), String>,
) -> Result<Value, String> {
    valid_id(id)?;
    let _lock = lock(root, &id.to_ascii_lowercase())?;
    let mut record = load(root, id)?;
    if record["revision"] != revision {
        return Err("Task revision conflict; reload and resume".into());
    }
    if check_current {
        current(root, &record)?;
    }
    let before = authority_fields(&record);
    let old_state = text(&record["state"]).to_owned();
    apply(&mut record)?;
    if check_current && before != authority_fields(&record) {
        return Err("A task update cannot revise its authorization or acceptance criteria".into());
    }
    let state = text(&record["state"]);
    if !STATES.contains(&state) {
        return Err("Invalid new task state".into());
    }
    let _coordinator;
    if state == "CLOSED" && old_state != "CLOSED" {
        _coordinator = lock(root, "_coordination")?;
        ensure_children_closed(root, &record)?;
    }
    record["revision"] = json!(revision.checked_add(1).ok_or("Task revision exhausted")?);
    persist(root, id, &record)?;
    Ok(record)
}

pub fn update(
    root: &Path,
    id: &str,
    revision: u64,
    apply: impl FnOnce(&mut Value) -> Result<(), String>,
) -> Result<Value, String> {
    update_impl(root, id, revision, true, apply)
}

fn new_record(root: &Path, record: Value) -> Result<Value, String> {
    let [spec, plan, policy] = authority_hashes(root, &record)?;
    let descriptor = descriptor_hash(&record)?;
    Ok(json!({
        "descriptor": record,
        "revision": 1, "approval_epoch": 1, "state": "PROPOSED",
        "spec_sha256": spec, "plan_sha256": plan, "policy_sha256": policy, "descriptor_sha256":descriptor,
        "checkpoint": null, "baseline": null, "evidence": [], "docs": {},
        "review": null, "acceptance": null, "blockers": [], "history": []
    }))
}

fn create(root: &Path, record: Value, registry: &Value) -> Result<Value, String> {
    descriptor(root, &record, registry)?;
    let id = text(&record["id"]).to_owned();
    let id_key = id.to_ascii_lowercase();
    let _lock = lock(root, &id_key)?;
    let path = root.join(format!("{STORE}/{id_key}.json"));
    match fs::symlink_metadata(path) {
        Ok(_) => return Err("Task ID is already in use".into()),
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(_) => return Err("Cannot inspect task ID".into()),
    }
    let value = new_record(root, record)?;
    persist(root, &id, &value)?;
    Ok(value)
}

fn check_expiry(authorization: &Value) -> Result<(), String> {
    if let Some(expires) = authorization["expires_at"].as_u64()
        && SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "Cannot validate authorization expiry")?
            .as_secs()
            >= expires
    {
        return Err("Task authorization has expired".into());
    }
    Ok(())
}

pub fn authorize(task: &Value, action: &str) -> Result<(), String> {
    let descriptor = &task["descriptor"];
    let authorization = &descriptor["authorization"];
    check_expiry(authorization)?;
    if !items(&authorization["actions"]).any(|allowed| allowed == action) {
        return Err(format!("Task action is not authorized: {action}"));
    }
    if action == "write" && descriptor["mode"] != "WORK" {
        return Err("Code writes are only authorized in WORK mode".into());
    }
    if matches!(text(&descriptor["role"]), "reviewer" | "verifier")
        && matches!(action, "write" | "docs" | "network" | "release")
    {
        return Err("Review and verification roles are read-only".into());
    }
    Ok(())
}

pub fn ensure_children_closed(root: &Path, task: &Value) -> Result<(), String> {
    let id = &task["descriptor"]["id"];
    for child in items(&task["descriptor"]["required_children"]) {
        let name = text(child);
        let record = load(root, name).map_err(|_| format!("Required child is pending: {name}"))?;
        if record["descriptor"]["parent"] != *id || record["state"] != "CLOSED" {
            return Err(format!("Required child is pending: {name}"));
        }
        current(root, &record).map_err(|_| format!("Required child authority is stale: {name}"))?;
    }
    Ok(())
}

fn ready(root: &Path, task: &Value, registry: &Value) -> Result<(), String> {
    let descriptor = &task["descriptor"];
    check_expiry(&descriptor["authorization"])?;
    if !descriptor["write_scope"]
        .as_object()
        .is_some_and(|map| map.is_empty())
    {
        authorize(task, "write")?;
    }
    if !descriptor["read_scope"]
        .as_object()
        .is_some_and(|map| map.is_empty())
    {
        authorize(task, "read")?;
    }
    for command in items(&descriptor["commands"]) {
        let spec = &registry["control_plane"]["commands"][text(command)];
        match text(&spec["mode"]) {
            "WORK" if descriptor["mode"] == "WORK" => authorize(task, "run")?,
            "VERIFY" => authorize(task, "verify")?,
            _ => return Err("Task command mode is unavailable for its grant".into()),
        }
    }
    for dependency in items(&descriptor["depends_on"]) {
        let name = text(dependency);
        if load(root, name)?["state"] != "CLOSED" {
            return Err(format!("Dependency is pending: {name}"));
        }
    }
    Ok(())
}

fn overlapping(root: &Path, task: &Value) -> Result<(), String> {
    let dir = directory(root, STORE, false)?;
    let own = &task["descriptor"];
    let count = fs::read_dir(dir).map_err(|_| "Cannot inspect active task claims")?;
    let mut bytes = 0_u64;
    for (index, entry) in count.enumerate() {
        if index >= TASK_LIMIT * 4 {
            return Err("Task store inventory budget exceeded; review local records".into());
        }
        let entry = entry.map_err(|_| "Cannot inspect active task claim")?;
        let name = entry.file_name();
        if !name.to_str().is_some_and(|name| name.ends_with(".json")) {
            continue;
        }
        let size = entry
            .metadata()
            .map_err(|_| "Cannot size task claim")?
            .len();
        bytes = bytes
            .checked_add(size)
            .ok_or("Task claim scan byte budget exceeded")?;
        if bytes > CLAIM_SCAN_BYTES {
            return Err("Task claim scan byte budget exceeded; review local records".into());
        }
        let other = parse_json(&policy::read(&entry.path())?)?;
        if own["id"] == other["descriptor"]["id"]
            || !matches!(
                text(&other["state"]),
                "IN_PROGRESS" | "IMPLEMENTED" | "VERIFIED" | "ACCEPTED" | "DOCS_SYNCED"
            )
        {
            continue;
        }
        let claims = ["write_scope", "documentation_scope"];
        for scope in claims {
            for (repo, paths) in own[scope].as_object().unwrap() {
                let selected = claims
                    .into_iter()
                    .flat_map(|scope| items(&other["descriptor"][scope][repo]))
                    .map(text)
                    .map(key)
                    .collect::<BTreeSet<_>>();
                if items(paths).any(|path| selected.contains(&key(text(path)))) {
                    return Err(format!(
                        "Task scope overlaps active task {}",
                        text(&other["descriptor"]["id"])
                    ));
                }
            }
        }
    }
    Ok(())
}

fn required<'a>(options: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str, String> {
    options
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| format!("--{key} is required"))
}

fn input(root: &Path, path: &str) -> Result<Value, String> {
    plain_relative(path)?;
    parse_json(&read_authority(root, path)?)
}

fn reset(root: &Path, task: &mut Value, descriptor: Value) -> Result<(), String> {
    if let Some(parent) = task["descriptor"]["parent"].as_str() {
        let parent = load(root, parent)?;
        if parent["state"] == "CLOSED"
            && items(&parent["descriptor"]["required_children"])
                .any(|child| child == &task["descriptor"]["id"])
        {
            return Err(
                "Reopen the accepted parent initiative before reopening a required child".into(),
            );
        }
    }
    let mut replacement = new_record(root, descriptor)?;
    let epoch = task["approval_epoch"]
        .as_u64()
        .ok_or("Invalid authorization revision")?;
    replacement["approval_epoch"] = json!(
        epoch
            .checked_add(1)
            .ok_or("Authorization revision exhausted")?
    );
    let mut history = task["history"]
        .as_array()
        .ok_or("Invalid task history")?
        .clone();
    if history.len() >= 64 {
        return Err("Task history budget exhausted".into());
    }
    let previous = serde_json::to_vec(task).map_err(|_| "Cannot identify previous task state")?;
    history.push(json!({
        "revision":task["revision"], "state":task["state"], "previous_sha256":docs::sha256(&previous),
        "evidence":task["evidence"], "review":task["review"], "acceptance":task["acceptance"]
    }));
    replacement["history"] = json!(history);
    *task = replacement;
    Ok(())
}

fn actor_is(task: &Value, actor: &str, authority: bool) -> Result<(), String> {
    valid_id(actor)?;
    let expected = if authority {
        &task["descriptor"]["authorization"]["owner"]
    } else {
        &task["descriptor"]["owner"]
    };
    if expected != actor {
        return Err("Task actor lacks authority for this transition".into());
    }
    Ok(())
}

fn checkpoint(record: Value) -> Result<Value, String> {
    reject_credentials(&record)?;
    let allowed = ["changed_paths", "evidence", "blockers", "next_action"];
    if !record
        .as_object()
        .is_some_and(|object| object.keys().all(|name| allowed.contains(&name.as_str())))
        || !record["next_action"]
            .as_str()
            .is_some_and(|value| !value.trim().is_empty() && value.len() <= 4096)
    {
        return Err("Checkpoint must have a bounded next action and only checkpoint fields".into());
    }
    if let Some(changes) = record.get("changed_paths") {
        let repos = changes
            .as_object()
            .filter(|map| map.len() <= 32)
            .ok_or("Checkpoint changed_paths must be a repository map")?;
        for (repo, files) in repos {
            valid_id(repo)?;
            for path in files
                .as_array()
                .filter(|files| files.len() <= 128)
                .ok_or("Checkpoint file list exceeds bounds")?
            {
                relative(text(path))?;
            }
        }
    }
    for field in ["evidence", "blockers"] {
        if let Some(list) = record.get(field)
            && !list.as_array().is_some_and(|rows| {
                rows.len() <= 64
                    && rows.iter().all(|value| {
                        value
                            .as_str()
                            .is_some_and(|value| !value.trim().is_empty() && value.len() <= 512)
                    })
            })
        {
            return Err(format!(
                "Checkpoint {field} must contain bounded reference strings"
            ));
        }
    }
    Ok(record)
}

pub fn run(
    command: &str,
    root: &Path,
    options: &BTreeMap<String, String>,
    registry: &Value,
) -> Result<Value, String> {
    let permitted: &[&str] = match command {
        "task-create" => &["descriptor"],
        "task-show" | "task-resume" => &["task"],
        "task-revise" => &["task", "revision", "actor", "descriptor"],
        "task-checkpoint" => &["task", "revision", "actor", "checkpoint"],
        "task-transition" => &["task", "revision", "actor", "to", "reason", "workspace"],
        _ => return Err("Unknown task command".into()),
    };
    if options
        .keys()
        .any(|option| !permitted.contains(&option.as_str()))
    {
        return Err("Unexpected task option".into());
    }
    if command == "task-create" {
        let record = input(root, required(options, "descriptor")?)?;
        return Ok(json!({"status":"PASS", "task":create(root, record, registry)?}));
    }
    let id = required(options, "task")?;
    if command == "task-show" {
        return Ok(json!({"status":"PASS","task":load(root, id)?}));
    }
    if command == "task-resume" {
        return match load_current(root, id) {
            Ok(task) => Ok(json!({"status":"PASS","task":task})),
            Err(reason) => Ok(json!({"status":"BLOCKED","reason":reason})),
        };
    }
    let revision = required(options, "revision")?
        .parse::<u64>()
        .map_err(|_| "Invalid --revision")?;
    let actor = required(options, "actor")?;
    let task = match command {
        "task-revise" => {
            let proposed = input(root, required(options, "descriptor")?)?;
            descriptor(root, &proposed, registry)?;
            let _coordinator = lock(root, "_coordination")?;
            update_impl(root, id, revision, false, |task| {
                actor_is(task, actor, true)?;
                if task["state"] == "IN_PROGRESS" {
                    return Err(
                        "Stop the active execution and block the task before revising its grant"
                            .into(),
                    );
                }
                if proposed["id"] != id
                    || proposed["authorization"]["owner"]
                        != task["descriptor"]["authorization"]["owner"]
                    || proposed["parent"] != task["descriptor"]["parent"]
                    || proposed["primary_repo"] != task["descriptor"]["primary_repo"]
                {
                    return Err(
                        "Task revision cannot change its identity or authorization owner".into(),
                    );
                }
                reset(root, task, proposed)
            })?
        }
        "task-checkpoint" => {
            let record = checkpoint(input(root, required(options, "checkpoint")?)?)?;
            update(root, id, revision, |task| {
                actor_is(task, actor, false)?;
                if !matches!(
                    text(&task["state"]),
                    "IN_PROGRESS" | "IMPLEMENTED" | "BLOCKED"
                ) {
                    return Err("Task state does not accept a checkpoint".into());
                }
                task["checkpoint"] = record;
                Ok(())
            })?
        }
        "task-transition" => {
            let next = required(options, "to")?;
            let _coordinator;
            if matches!(next, "IN_PROGRESS" | "REOPEN") {
                _coordinator = lock(root, "_coordination")?;
            }
            let uses_active_grant =
                !matches!(next, "REOPEN" | "CANCELLED" | "SUPERSEDED" | "BLOCKED");
            update_impl(root, id, revision, uses_active_grant, |task| {
                let previous = text(&task["state"]);
                match (previous, next) {
                    ("PROPOSED", "READY") => {
                        actor_is(task, actor, true)?;
                        ready(root, task, registry)?;
                    }
                    ("READY", "IN_PROGRESS") => {
                        actor_is(task, actor, false)?;
                        ready(root, task, registry)?;
                        overlapping(root, task)?;
                        task["baseline"] = crate::runner::capture_baseline(
                            root,
                            task,
                            registry,
                            options.get("workspace").map(Path::new),
                        )?;
                    }
                    ("IN_PROGRESS", "IMPLEMENTED") => actor_is(task, actor, false)?,
                    (
                        "PROPOSED" | "READY" | "IN_PROGRESS" | "IMPLEMENTED" | "VERIFIED"
                        | "ACCEPTED",
                        "BLOCKED",
                    ) => {
                        if actor_is(task, actor, false).is_err() {
                            actor_is(task, actor, true)?;
                        }
                        let reason = required(options, "reason")?;
                        if reason.trim().is_empty() || reason.len() > 512 {
                            return Err("A bounded blocker reason is required".into());
                        }
                        task["blockers"] = json!([reason]);
                    }
                    (
                        "PROPOSED" | "READY" | "IN_PROGRESS" | "IMPLEMENTED" | "VERIFIED"
                        | "ACCEPTED" | "BLOCKED",
                        "CANCELLED" | "SUPERSEDED",
                    ) => {
                        actor_is(task, actor, true)?;
                        let reason = required(options, "reason")?;
                        if reason.trim().is_empty() || reason.len() > 512 {
                            return Err("A bounded disposition reason is required".into());
                        }
                        task["blockers"] = json!([reason]);
                    }
                    ("BLOCKED" | "CANCELLED" | "SUPERSEDED" | "CLOSED", "REOPEN") => {
                        actor_is(task, actor, true)?;
                        current(root, task)?;
                        let descriptor = task["descriptor"].clone();
                        return reset(root, task, descriptor);
                    }
                    (_, "VERIFIED" | "ACCEPTED" | "DOCS_SYNCED" | "CLOSED") => {
                        return Err("Use the dedicated evidence, acceptance, documentation or closure command".into());
                    }
                    _ => {
                        return Err(format!(
                            "Task transition {previous} → {next} is unavailable"
                        ));
                    }
                }
                task["state"] = json!(next);
                Ok(())
            })?
        }
        _ => return Err("Unknown task command".into()),
    };
    Ok(json!({"status":"PASS","task":task}))
}

#[cfg(test)]
#[path = "tasks/tests.rs"]
mod tests;
