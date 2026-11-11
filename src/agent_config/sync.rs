use super::{Bundle, checked, paths};
use crate::{docs, policy, runner, tasks, workspace};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const STORE: &str = ".cirnetrules/agent-config";
const MAX_OUTPUTS: usize = 1024;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Ownership {
    files: BTreeMap<String, String>,
    head: Option<String>,
    pending: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Change {
    path: String,
    before: paths::Observation,
    after: Option<String>,
    conflict: bool,
}

impl Change {
    fn changes(&self) -> bool {
        self.before.sha256 != self.after
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    schema: u8,
    repository: String,
    binding: String,
    target_commit: String,
    bundle_sha256: String,
    authority_sha256: String,
    ownership: Ownership,
    files: Vec<Change>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    schema: u8,
    plan: Plan,
    actor: String,
    task: String,
    started_ns: u128,
    pid: u32,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Progress {
    intents: BTreeMap<String, paths::Observation>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    rollback_intents: BTreeMap<String, paths::Observation>,
    applied_ns: Option<u128>,
    rollback: Option<Rollback>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Rollback {
    actor: String,
    task: String,
    completed_ns: u128,
}

fn serial(value: &impl Serialize) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value).map_err(|_| "Cannot encode agent configuration record".into())
}

fn digest(value: &impl Serialize) -> Result<String, String> {
    Ok(docs::sha256(&serial(value)?))
}

fn read<T: DeserializeOwned + Serialize>(root: &Path, relative: &str) -> Result<T, String> {
    let file = workspace::scoped_path(root, relative)?;
    let parent = file
        .parent()
        .ok_or("Agent configuration record has no parent")?;
    paths::directory(parent)?;
    let bytes = paths::read(&file)?;
    let value = serde_json::from_slice(&bytes).map_err(|_| "Invalid agent configuration record")?;
    if serial(&value)? != bytes {
        return Err("Agent configuration record encoding changed".into());
    }
    Ok(value)
}

fn exists(root: &Path, relative: &str) -> Result<bool, String> {
    let path = workspace::scoped_path(root, relative)?;
    path.try_exists()
        .map_err(|_| "Cannot inspect agent configuration state".into())
}

fn persist(root: &Path, relative: &str, value: &impl Serialize) -> Result<(), String> {
    let prior = paths::observe(root, relative)?;
    paths::replace(root, relative, &prior, Some(&serial(value)?))
}

fn valid_hash(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn output_path(path: &str) -> Result<(), String> {
    paths::validate(path)?;
    let skill = [".agents/skills/", ".claude/skills/"].iter().any(|prefix| {
        path.strip_prefix(prefix)
            .and_then(|tail| tail.strip_suffix("/SKILL.md"))
            .is_some_and(|name| {
                !name.is_empty()
                    && !name.contains('/')
                    && name
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            })
    });
    let reference = path.starts_with("docs/cirnetrules/en/")
        && [".md", ".txt", ".json", ".yaml", ".yml"]
            .iter()
            .any(|suffix| path.ends_with(suffix));
    if matches!(path, "AGENTS.md" | "CLAUDE.md") || skill || reference {
        Ok(())
    } else {
        Err("Path is outside managed agent outputs".into())
    }
}

fn state_path(binding: &str) -> Result<String, String> {
    if !valid_hash(binding) {
        return Err("Invalid agent configuration binding identity".into());
    }
    Ok(format!("{STORE}/bindings/{binding}/state.json"))
}

fn receipt_dir(id: &str) -> Result<String, String> {
    if !valid_hash(id) {
        return Err("Invalid agent configuration receipt ID".into());
    }
    Ok(format!("{STORE}/receipts/{id}"))
}

fn load_ownership(root: &Path, binding: &str) -> Result<Ownership, String> {
    let path = state_path(binding)?;
    let state = if exists(root, &path)? {
        read(root, &path)?
    } else {
        Ownership::default()
    };
    validate_ownership(&state)?;
    Ok(state)
}

fn validate_ownership(state: &Ownership) -> Result<(), String> {
    if state.files.len() > MAX_OUTPUTS
        || state
            .head
            .iter()
            .chain(&state.pending)
            .any(|id| !valid_hash(id))
        || state.files.values().any(|sha| !valid_hash(sha))
    {
        return Err("Invalid agent configuration ownership state".into());
    }
    paths::validate_names(state.files.keys().map(String::as_str))?;
    for path in state.files.keys() {
        output_path(path)?;
    }
    Ok(())
}

fn plan(
    bundle: &Bundle,
    binding: &workspace::Binding,
    ownership: Ownership,
) -> Result<Plan, String> {
    let filenames: BTreeSet<_> = ownership.files.keys().chain(bundle.files.keys()).collect();
    if filenames.len() > MAX_OUTPUTS {
        return Err("Agent sync exceeds its file count limit".into());
    }
    paths::validate_names(filenames.iter().map(|path| path.as_str()))?;
    let mut files = Vec::new();
    for path in filenames {
        output_path(path)?;
        let before = paths::observe(&binding.path, path)?;
        let after = bundle.files.get(path).map(|file| docs::sha256(&file.bytes));
        let owned = ownership.files.get(path);
        let conflict = before.sha256.as_ref() != owned;
        files.push(Change {
            path: path.clone(),
            before,
            after,
            conflict,
        });
    }
    Ok(Plan {
        schema: 1,
        repository: bundle.manifest.repository.clone(),
        binding: binding.identity.clone(),
        target_commit: binding.commit.clone(),
        bundle_sha256: docs::sha256(&bundle.manifest_bytes()?),
        authority_sha256: bundle.manifest.authority_sha256.clone(),
        ownership,
        files,
    })
}

fn plan_report(bundle: &Bundle, plan: &Plan, warnings: &[Value]) -> Result<Value, String> {
    let mut files = Vec::new();
    let mut conflict = false;
    for change in &plan.files {
        let action = if change.conflict {
            conflict = true;
            "CONFLICT"
        } else if !change.changes() {
            "UNCHANGED"
        } else if change.before.sha256.is_none() {
            "CREATE"
        } else if change.after.is_none() {
            "DELETE"
        } else {
            "UPDATE"
        };
        let patch = if change.conflict {
            bundle.files.get(&change.path).map(|file| super::render::guarded(&file.bytes)).transpose()?
                .map(|body| json!({"path":change.path,"expected_sha256":change.before.sha256,"replacement":body}))
        } else {
            None
        };
        files.push(json!({
            "path":change.path,"action":action,"before_sha256":change.before.sha256,
            "after_sha256":change.after,"patch":patch
        }));
    }
    let blocked = conflict || plan.ownership.pending.is_some();
    Ok(json!({
        "status": if blocked {"BLOCKED"} else {"PASS"},
        "state": if plan.ownership.pending.is_some() {"RECOVERY_REQUIRED"} else if conflict {"CONFLICT"} else if plan.files.iter().all(|row| !row.changes()) {"UNCHANGED"} else {"DRY_RUN"},
        "plan_sha256":digest(plan)?,"binding":plan.binding,"commit":plan.target_commit,
        "bundle_sha256":plan.bundle_sha256,"repository":plan.repository,"files":files,
        "pending_receipt":plan.ownership.pending,"warnings":warnings,"native_discovery":"NOT_RUN"
    }))
}

fn warnings(binding: &workspace::Binding, bundle: &Bundle) -> Result<Vec<Value>, String> {
    let mut warnings = Vec::new();
    let output = policy::git(
        &binding.path,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            ":(glob)**/AGENTS.md",
            ":(glob)**/AGENTS.override.md",
            ":(glob)**/CLAUDE.md",
            ":(glob)**/GEMINI.md",
            ":(glob)**/.agents/rules/**",
            ":(glob)**/.claude/rules/**",
        ],
    )?;
    let mut names = BTreeSet::new();
    for path in output.split_terminator('\0') {
        if !bundle.files.contains_key(path) {
            names.insert(path.to_owned());
        }
    }
    for path in [
        "AGENTS.override.md",
        "GEMINI.md",
        ".agents/AGENTS.md",
        ".claude/CLAUDE.md",
        ".agents/rules",
        ".claude/rules",
    ] {
        if binding.path.join(path).symlink_metadata().is_ok() {
            names.insert(path.to_owned());
        }
    }
    for name in names.into_iter().take(128) {
        if paths::validate(&name).is_ok() {
            warnings.push(json!({"path":name,"reason":"Existing discovery rules can add to or shadow the exported instructions"}));
        }
    }
    Ok(warnings)
}

struct Operation<'a> {
    root: &'a Path,
    work: Option<&'a Path>,
    registry: &'a Value,
    target: PathBuf,
    args: &'a BTreeMap<String, String>,
    apply: bool,
}

impl<'a> Operation<'a> {
    fn new(
        root: &'a Path,
        work: Option<&'a Path>,
        registry: &'a Value,
        args: &'a BTreeMap<String, String>,
    ) -> Result<Self, String> {
        let dry = args.get("dry-run");
        let yes = args.get("yes");
        let apply = match (dry.map(String::as_str), yes.map(String::as_str)) {
            (Some("true"), None) if !args.contains_key("expect") => false,
            (None, Some("true")) if args.get("expect").is_some_and(|value| valid_hash(value)) => {
                true
            }
            _ => {
                return Err("Choose --dry-run, or --yes with the reviewed --expect SHA-256".into());
            }
        };
        let target =
            paths::directory(Path::new(args.get("target").ok_or("--target is required")?))?;
        if apply && target == root {
            return Err("Agent sync cannot write into its governance authority checkout".into());
        }
        let operation = Self {
            root,
            work,
            registry,
            target,
            args,
            apply,
        };
        if apply {
            operation.identity()?;
        }
        Ok(operation)
    }

    fn bind(
        &self,
        repository: &str,
        fallback_task: Option<&str>,
    ) -> Result<workspace::Binding, String> {
        let task = self.args.get("task").map(String::as_str).or(fallback_task);
        let binding = if let Some(task) = task {
            let state = tasks::load_current(self.root, task)?;
            tasks::authorize(&state, "read")?;
            workspace::resolve_task(self.root, self.work, repository, self.registry, &state)?
        } else {
            workspace::resolve(self.root, self.work, repository, self.registry)?
        };
        if binding.path != self.target {
            return Err(
                "Requested target does not match the exact confirmed worktree binding".into(),
            );
        }
        Ok(binding)
    }

    fn identity(&self) -> Result<(&str, &str), String> {
        let task = self
            .args
            .get("task")
            .map(String::as_str)
            .ok_or("--task is required for agent configuration writes")?;
        let actor = self
            .args
            .get("actor")
            .map(String::as_str)
            .ok_or("--actor is required for agent configuration writes")?;
        if tasks::legacy_operation_blocked("grant-evaluate", self.root, self.args)? {
            return Err("Existing legacy maintenance state requires reviewed recovery before agent configuration writes".into());
        }
        if actor.is_empty()
            || actor.len() > 96
            || !actor
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"-_.@".contains(&c))
        {
            return Err("Invalid declared agent configuration operator".into());
        }
        Ok((task, actor))
    }

    fn review(&self, hash: &str) -> Result<(), String> {
        if self.args.get("expect").map(String::as_str) != Some(hash) {
            return Err("Agent configuration preview changed. Review a new --dry-run.".into());
        }
        Ok(())
    }

    fn grant(
        &self,
        repository: &str,
        binding: &workspace::Binding,
        path: &str,
    ) -> Result<u64, String> {
        let (task, actor) = self.identity()?;
        let checked = self.bind(repository, None)?;
        if checked.identity != binding.identity || checked.commit != binding.commit {
            return Err("Bound worktree changed during agent configuration apply".into());
        }
        let mut args = BTreeMap::from([
            ("task".into(), task.into()),
            ("actor".into(), actor.into()),
            ("repo".into(), repository.into()),
            ("path".into(), path.into()),
            ("action".into(), "write".into()),
        ]);
        if let Some(workspace) = self.args.get("workspace") {
            args.insert("workspace".into(), workspace.clone());
        }
        let outcome = runner::run("grant-evaluate", self.root, &args, self.registry)?;
        if outcome["status"] != "PASS" || outcome["worktree_identity"] != binding.identity {
            return Err(format!(
                "Agent configuration write grant blocked for {path}: {}",
                outcome["reason"].as_str().unwrap_or("Binding changed")
            ));
        }
        outcome["task_revision"]
            .as_u64()
            .ok_or_else(|| "Agent configuration grant has no task revision".into())
    }

    fn lock(&self, binding: &str) -> Result<fs::File, String> {
        state_path(binding)?;
        let path = paths::directories(self.root, &format!("{STORE}/bindings/{binding}"))?;
        paths::lock(&path)
    }

    fn grant_all<'b>(
        &self,
        repository: &str,
        binding: &workspace::Binding,
        paths: impl Iterator<Item = &'b str>,
    ) -> Result<Option<u64>, String> {
        let mut revision = None;
        for path in paths {
            let current = self.grant(repository, binding, path)?;
            if revision.is_some_and(|prior| prior != current) {
                return Err(
                    "Task revision changed while reviewing agent configuration grant".into(),
                );
            }
            revision = Some(current);
        }
        Ok(revision)
    }
}

fn desired_ownership(plan: &Plan, id: &str) -> Ownership {
    Ownership {
        files: plan
            .files
            .iter()
            .filter_map(|file| {
                file.after
                    .as_ref()
                    .map(|hash| (file.path.clone(), hash.clone()))
            })
            .collect(),
        head: Some(id.into()),
        pending: None,
    }
}

fn backup(root: &Path, binding: &workspace::Binding, dir: &str, plan: &Plan) -> Result<(), String> {
    for change in plan.files.iter().filter(|change| change.changes()) {
        if let Some(expected) = &change.before.sha256 {
            let bytes = paths::read(&binding.path.join(&change.path))?;
            super::render::guarded(&bytes)?;
            if &docs::sha256(&bytes) != expected {
                return Err("Owned output changed before its rollback backup".into());
            }
            let relative = format!("{dir}/before/{}", change.path);
            let (parent, _) = relative
                .rsplit_once('/')
                .ok_or("Invalid rollback backup path")?;
            paths::directories(root, parent)?;
            paths::write_new(&root.join(relative), &bytes)?;
        }
    }
    Ok(())
}

fn save_progress(root: &Path, dir: &str, progress: &Progress) -> Result<(), String> {
    persist(root, &format!("{dir}/progress.json"), progress)
}

fn stage_path(path: &str, receipt: &str, rollback: bool) -> String {
    let phase = if rollback { "undo" } else { "apply" };
    let suffix = docs::sha256(path.as_bytes());
    let name = format!(".cnr-stage-{phase}-{receipt}-{}", &suffix[..16]);
    match path.rsplit_once('/') {
        Some((parent, _)) => format!("{parent}/{name}"),
        None => name,
    }
}

fn completed_retry(
    operation: &Operation<'_>,
    binding: &workspace::Binding,
    preview: &Plan,
    task: &str,
    actor: &str,
) -> Result<bool, String> {
    let Some(id) = &preview.ownership.head else {
        return Ok(false);
    };
    let (journal, progress) = load_journal(operation.root, id)?;
    if progress.applied_ns.is_none()
        || progress.rollback.is_some()
        || journal.task != task
        || journal.actor != actor
        || journal.plan.repository != preview.repository
        || journal.plan.binding != preview.binding
        || journal.plan.target_commit != preview.target_commit
        || journal.plan.bundle_sha256 != preview.bundle_sha256
        || desired_ownership(&journal.plan, id) != preview.ownership
        || operation.args.get("expect") != Some(&digest(&journal.plan)?)
    {
        return Ok(false);
    }
    for row in journal.plan.files.iter().filter(|row| row.changes()) {
        let current = paths::observe(&binding.path, &row.path)?;
        if progress.intents.get(&row.path) != Some(&current)
            || (row.after.is_some()
                && paths::observe(&binding.path, &stage_path(&row.path, id, false))?
                    .sha256
                    .is_some())
        {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) fn sync(
    root: &Path,
    work: Option<&Path>,
    registry: &Value,
    args: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let operation = Operation::new(root, work, registry, args)?;
    let input = Path::new(args.get("bundle").ok_or("--bundle is required")?);
    let bundle = checked(root, work, registry, input)?;
    let repository = bundle.manifest.repository.as_str();
    let binding = operation.bind(repository, bundle.manifest.task.as_deref())?;
    if bundle.manifest.binding.as_deref() != Some(&binding.identity) {
        return Err("Export the bundle again for this confirmed worktree binding".into());
    }
    let warnings = warnings(&binding, &bundle)?;
    let preview = plan(&bundle, &binding, load_ownership(root, &binding.identity)?)?;
    if !operation.apply {
        return plan_report(&bundle, &preview, &warnings);
    }
    let preview_digest = digest(&preview)?;
    if preview.ownership.pending.is_some() || preview.files.iter().any(|row| row.conflict) {
        operation.review(&preview_digest)?;
        return plan_report(&bundle, &preview, &warnings);
    }
    let (task, actor) = operation.identity()?;
    if bundle
        .manifest
        .task
        .as_deref()
        .is_some_and(|selector| selector != task)
    {
        return Err("Apply requires the bundle's selected task".into());
    }
    if preview.files.iter().all(|row| !row.changes()) {
        if operation.review(&preview_digest).is_err()
            && !completed_retry(&operation, &binding, &preview, task, actor)?
        {
            operation.review(&preview_digest)?;
        }
        return plan_report(&bundle, &preview, &warnings);
    }
    operation.review(&preview_digest)?;
    let _lock = operation.lock(&binding.identity)?;
    let rechecked = checked(root, work, registry, input)?;
    let confirmed = operation.bind(repository, bundle.manifest.task.as_deref())?;
    let plan = plan(
        &rechecked,
        &confirmed,
        load_ownership(root, &binding.identity)?,
    )?;
    operation.review(&digest(&plan)?)?;
    let revision = operation.grant_all(
        repository,
        &binding,
        plan.files
            .iter()
            .filter(|row| row.changes())
            .map(|row| row.path.as_str()),
    )?;
    let journal = Journal {
        schema: 1,
        plan,
        task: task.into(),
        actor: actor.into(),
        started_ns: paths::nonce()?,
        pid: std::process::id(),
    };
    let id = digest(&journal)?;
    let dir = receipt_dir(&id)?;
    let receipts = paths::directories(root, &format!("{STORE}/receipts"))?;
    fs::create_dir(receipts.join(&id))
        .map_err(|_| "Cannot create fresh agent configuration receipt directory")?;
    backup(root, &binding, &dir, &journal.plan)?;
    paths::write_new(
        &root.join(format!("{dir}/journal.json")),
        &serial(&journal)?,
    )?;
    let mut progress = Progress::default();
    save_progress(root, &dir, &progress)?;
    let state_path = state_path(&binding.identity)?;
    let mut pending = journal.plan.ownership.clone();
    pending.pending = Some(id.clone());
    persist(root, &state_path, &pending)?;
    let applied = (|| -> Result<(), String> {
        for change in journal.plan.files.iter().filter(|row| row.changes()) {
            if Some(operation.grant(repository, &binding, &change.path)?) != revision {
                return Err("Task revision changed during agent configuration apply".into());
            }
            let current = paths::observe(&binding.path, &change.path)?;
            if !change.before.matches_after_directory_creation(&current) {
                return Err("Destination changed after agent configuration preview".into());
            }
            let bytes = rechecked
                .files
                .get(&change.path)
                .map(|file| file.bytes.as_slice());
            let stage = stage_path(&change.path, &id, false);
            paths::replace_journaled(
                &binding.path,
                &change.path,
                &stage,
                &current,
                bytes,
                |post| {
                    if post.sha256 != change.after {
                        return Err(
                            "Agent configuration stage does not match its reviewed digest".into(),
                        );
                    }
                    progress.intents.insert(change.path.clone(), post);
                    save_progress(root, &dir, &progress)
                },
            )?;
            let next = paths::observe(&binding.path, &change.path)?;
            if progress.intents.get(&change.path) != Some(&next) {
                return Err(
                    "Agent configuration write was not observed at its expected digest".into(),
                );
            }
        }
        progress.applied_ns = Some(paths::nonce()?);
        save_progress(root, &dir, &progress)?;
        persist(root, &state_path, &desired_ownership(&journal.plan, &id))
    })();
    if let Err(reason) = applied {
        return Ok(
            json!({"status":"BLOCKED","state":"RECOVERY_REQUIRED","receipt":id,"reason":reason,"journaled_paths":progress.intents.keys().collect::<Vec<_>>()}),
        );
    }
    Ok(
        json!({"status":"PASS","state":"APPLIED","receipt":id,"binding":binding.identity,"files":progress.intents.keys().collect::<Vec<_>>(),"native_discovery":"NOT_RUN"}),
    )
}

fn load_journal(root: &Path, id: &str) -> Result<(Journal, Progress), String> {
    let dir = receipt_dir(id)?;
    let journal: Journal = read(root, &format!("{dir}/journal.json"))?;
    let progress: Progress = read(root, &format!("{dir}/progress.json"))?;
    if journal.schema != 1
        || journal.plan.schema != 1
        || digest(&journal)? != id
        || journal.plan.files.len() > MAX_OUTPUTS
        || !valid_hash(&journal.plan.binding)
        || !valid_hash(&journal.plan.bundle_sha256)
        || !valid_hash(&journal.plan.authority_sha256)
    {
        return Err("Agent configuration journal is invalid or changed".into());
    }
    validate_ownership(&journal.plan.ownership)?;
    paths::validate_names(journal.plan.files.iter().map(|row| row.path.as_str()))?;
    for row in &journal.plan.files {
        output_path(&row.path)?;
        if row.conflict
            || row.before.sha256.as_ref() != journal.plan.ownership.files.get(&row.path)
            || row.after.as_deref().is_some_and(|hash| !valid_hash(hash))
            || progress
                .intents
                .get(&row.path)
                .is_some_and(|next| !row.changes() || next.sha256 != row.after)
            || progress
                .rollback_intents
                .get(&row.path)
                .is_some_and(|next| {
                    !row.changes()
                        || row.before.sha256.is_none()
                        || next.sha256 != row.before.sha256
                })
        {
            return Err("Agent configuration journal ownership or progress is invalid".into());
        }
    }
    if progress
        .intents
        .keys()
        .chain(progress.rollback_intents.keys())
        .any(|key| !journal.plan.files.iter().any(|row| &row.path == key))
        || (progress.applied_ns.is_some()
            && progress.intents.len()
                != journal
                    .plan
                    .files
                    .iter()
                    .filter(|row| row.changes())
                    .count())
    {
        return Err("Agent configuration progress includes an unknown path".into());
    }
    Ok((journal, progress))
}

#[derive(Serialize)]
struct Restore {
    path: String,
    current: paths::Observation,
    before: Option<String>,
    conflict: bool,
}

fn restoration(
    binding: &workspace::Binding,
    journal: &Journal,
    progress: &Progress,
    pending: bool,
) -> Result<Vec<Restore>, String> {
    let mut files = Vec::new();
    for row in journal.plan.files.iter().filter(|row| row.changes()) {
        let current = paths::observe(&binding.path, &row.path)?;
        let conflict = if pending && current.sha256 == row.before.sha256 {
            // A rollback retry may already have restored this path.
            false
        } else {
            progress.intents.get(&row.path) != Some(&current)
        };
        files.push(Restore {
            path: row.path.clone(),
            current,
            before: row.before.sha256.clone(),
            conflict,
        });
    }
    Ok(files)
}

#[derive(Serialize)]
struct PendingStage {
    path: String,
    output: String,
    current: paths::Observation,
    conflict: bool,
}

fn pending_stages(
    binding: &workspace::Binding,
    id: &str,
    journal: &Journal,
    progress: &Progress,
) -> Result<Vec<PendingStage>, String> {
    let mut found = Vec::new();
    for row in journal.plan.files.iter().filter(|row| row.changes()) {
        for (rollback, expected, has_stage) in [
            (false, progress.intents.get(&row.path), row.after.is_some()),
            (
                true,
                progress.rollback_intents.get(&row.path),
                row.before.sha256.is_some(),
            ),
        ] {
            if has_stage {
                let path = stage_path(&row.path, id, rollback);
                let current = paths::observe(&binding.path, &path)
                    .map_err(|error| format!("Cannot inspect journal stage {path}: {error}"))?;
                if current.sha256.is_some() {
                    found.push(PendingStage {
                        path,
                        output: row.path.clone(),
                        conflict: expected != Some(&current),
                        current,
                    });
                }
            }
        }
    }
    Ok(found)
}

pub(super) fn rollback(
    root: &Path,
    work: Option<&Path>,
    registry: &Value,
    args: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let operation = Operation::new(root, work, registry, args)?;
    let id = args.get("receipt").ok_or("--receipt is required")?;
    let dir = receipt_dir(id)?;
    let (journal, mut progress) = load_journal(root, id)?;
    let repository = journal.plan.repository.as_str();
    let binding = operation.bind(repository, Some(&journal.task))?;
    if binding.identity != journal.plan.binding {
        return Err("Agent configuration journal refers to a different worktree binding".into());
    }
    let _lock = if operation.apply {
        Some(operation.lock(&binding.identity)?)
    } else {
        None
    };
    if operation.apply {
        (_, progress) = load_journal(root, id)?;
    }
    let state = load_ownership(root, &binding.identity)?;
    let pending = state.pending.as_deref() == Some(id);
    if !pending && progress.rollback.is_some() {
        return Ok(json!({"status":"PASS","state":"UNCHANGED","receipt":id}));
    }
    if !pending && (state.pending.is_some() || state.head.as_deref() != Some(id)) {
        return Err(
            "Rollback requires the latest applied or pending agent configuration receipt".into(),
        );
    }
    let restore = restoration(&binding, &journal, &progress, pending)?;
    let stages = pending_stages(&binding, id, &journal, &progress)?;
    let hash = digest(&(id, &binding.commit, &state, &restore, &stages))?;
    let conflict =
        restore.iter().any(|row| row.conflict) || stages.iter().any(|stage| stage.conflict);
    if !operation.apply || conflict {
        return Ok(json!({
            "status":if conflict {"BLOCKED"} else {"PASS"},"state":if conflict {"CONFLICT"} else {"DRY_RUN"},
            "receipt":id,"plan_sha256":hash,"binding":binding.identity,"files":restore.iter().map(|row| json!({
                "path":row.path,"before_sha256":row.current.sha256,"after_sha256":row.before,
                "action":if row.conflict {"CONFLICT"} else if row.current.sha256 == row.before {"UNCHANGED"} else if row.before.is_none() {"DELETE"} else {"RESTORE"}
            })).collect::<Vec<_>>(),
            "stages":stages.iter().map(|stage| json!({
                "path":stage.path,"output":stage.output,
                "action":if stage.conflict {"CONFLICT"} else {"DELETE"},
                "reason":if stage.conflict {Some("Stage identity was not recorded or has changed; preserve it for local recovery review")} else {None}
            })).collect::<Vec<_>>(),
        }));
    }
    operation.review(&hash)?;
    let (task, actor) = operation.identity()?;
    tasks::load_current(root, task)?;
    let grant_paths: BTreeSet<_> = restore
        .iter()
        .filter(|row| row.current.sha256 != row.before)
        .map(|row| row.path.as_str())
        .chain(stages.iter().map(|stage| stage.output.as_str()))
        .collect();
    let revision = operation.grant_all(repository, &binding, grant_paths.into_iter())?;
    let mut backups = BTreeMap::new();
    for row in restore
        .iter()
        .filter(|row| row.current.sha256 != row.before)
    {
        if let Some(before) = &row.before {
            let backup = workspace::scoped_path(root, &format!("{dir}/before/{}", row.path))?;
            let bytes = paths::read(&backup)?;
            super::render::guarded(&bytes)?;
            if &docs::sha256(&bytes) != before {
                return Err("Rollback backup changed".into());
            }
            backups.insert(row.path.as_str(), bytes);
        }
    }
    let mut active = state;
    active.pending = Some(id.clone());
    let state_path = state_path(&binding.identity)?;
    persist(root, &state_path, &active)?;
    let restored = (|| -> Result<(), String> {
        for stage in &stages {
            if Some(operation.grant(repository, &binding, &stage.output)?) != revision {
                return Err("Task revision changed before journal stage cleanup".into());
            }
            paths::replace(&binding.path, &stage.path, &stage.current, None)?;
        }
        for row in restore
            .iter()
            .rev()
            .filter(|row| row.current.sha256 != row.before)
        {
            if Some(operation.grant(repository, &binding, &row.path)?) != revision {
                return Err("Task revision changed during agent configuration rollback".into());
            }
            let staged = stage_path(&row.path, id, true);
            let bytes = backups.get(row.path.as_str()).map(Vec::as_slice);
            paths::replace_journaled(
                &binding.path,
                &row.path,
                &staged,
                &row.current,
                bytes,
                |post| {
                    if let Some(before) = &row.before {
                        if post.sha256.as_ref() != Some(before) {
                            return Err("Rollback stage does not match its expected digest".into());
                        }
                        progress.rollback_intents.insert(row.path.clone(), post);
                        save_progress(root, &dir, &progress)?;
                    }
                    Ok(())
                },
            )?;
        }
        progress.rollback = Some(Rollback {
            actor: actor.into(),
            task: task.into(),
            completed_ns: paths::nonce()?,
        });
        save_progress(root, &dir, &progress)?;
        persist(root, &state_path, &journal.plan.ownership)
    })();
    if let Err(reason) = restored {
        return Ok(
            json!({"status":"BLOCKED","state":"RECOVERY_REQUIRED","receipt":id,"reason":reason}),
        );
    }
    Ok(json!({"status":"PASS","state":"ROLLED_BACK","receipt":id,"binding":binding.identity}))
}

pub(super) fn audit(root: &Path, id: &str) -> Result<Value, String> {
    let root = paths::directory(root)?;
    let (journal, progress) = load_journal(&root, id)?;
    let state = load_ownership(&root, &journal.plan.binding)?;
    let outcome = if state.pending.as_deref() == Some(id) {
        "RECOVERY_REQUIRED"
    } else if progress.rollback.is_some() {
        "ROLLED_BACK"
    } else if progress.applied_ns.is_some() {
        "APPLIED"
    } else {
        "NOT_APPLIED"
    };
    Ok(json!({
        "schema":1,"status":if outcome == "RECOVERY_REQUIRED" {"BLOCKED"} else {"PASS"},
        "operation":"agent-sync","result":outcome,"receipt_sha256":id,
        "repository":journal.plan.repository,"binding":journal.plan.binding,"task":journal.task,"actor":journal.actor,
        "authority_sha256":journal.plan.authority_sha256,"bundle_sha256":journal.plan.bundle_sha256,
        "before_sha256":digest(&journal.plan.ownership.files)?,
        "after_sha256":digest(&desired_ownership(&journal.plan,id).files)?,
        "journaled_paths":progress.intents.len(),
        "files_written":progress.applied_ns.map(|_| progress.intents.len()),
        "time_unix_ns":journal.started_ns,
        "applied_unix_ns":progress.applied_ns,"rollback":progress.rollback,
        "evidence_source":"local agent configuration journal"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_preserves_a_stage_without_a_recorded_identity() {
        let target = crate::test_support::temporary_directory("cirnet-agent-stage").unwrap();
        let binding = workspace::Binding {
            path: target.clone(),
            identity: "1".repeat(64),
            commit: "2".repeat(40),
        };
        let path = "AGENTS.md";
        let before = paths::observe(&target, path).unwrap();
        let bytes = b"pending configuration";
        let journal = Journal {
            schema: 1,
            plan: Plan {
                schema: 1,
                repository: "fixture".into(),
                binding: binding.identity.clone(),
                target_commit: binding.commit.clone(),
                bundle_sha256: "3".repeat(64),
                authority_sha256: "4".repeat(64),
                ownership: Ownership::default(),
                files: vec![Change {
                    path: path.into(),
                    before,
                    after: Some(docs::sha256(bytes)),
                    conflict: false,
                }],
            },
            actor: "worker".into(),
            task: "config".into(),
            started_ns: 1,
            pid: 1,
        };
        let id = digest(&journal).unwrap();
        let stage = stage_path(path, &id, false);
        paths::write_new(&target.join(&stage), bytes).unwrap();
        let mut progress = Progress::default();
        let unresolved = pending_stages(&binding, &id, &journal, &progress).unwrap();
        assert_eq!(unresolved.len(), 1);
        assert!(unresolved[0].conflict);
        assert_eq!(fs::read(target.join(&stage)).unwrap(), bytes);

        let identity = paths::observe(&target, &stage).unwrap();
        progress.intents.insert(path.into(), identity);
        let recoverable = pending_stages(&binding, &id, &journal, &progress).unwrap();
        assert_eq!(recoverable.len(), 1);
        assert!(!recoverable[0].conflict);
        paths::replace(&target, &stage, &recoverable[0].current, None).unwrap();
        assert!(
            pending_stages(&binding, &id, &journal, &progress)
                .unwrap()
                .is_empty()
        );
        assert!(!target.join(path).exists());
        fs::remove_dir(&target).unwrap();
    }
}
