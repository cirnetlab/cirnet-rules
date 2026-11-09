use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

pub fn safe_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = Path::new(relative);
    if relative.contains(['\\', ':'])
        || relative
            .split('/')
            .any(|p| p.is_empty() || matches!(p, "." | ".."))
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("Unsafe relative path".into());
    }
    let mut resolved = root.to_path_buf();
    for part in path.components() {
        resolved.push(part);
        let metadata =
            fs::symlink_metadata(&resolved).map_err(|_| format!("Missing path: {relative}"))?;
        if is_link(&metadata) {
            return Err(format!(
                "Link or reparse point is outside the policy boundary: {relative}"
            ));
        }
    }
    let resolved = resolved
        .canonicalize()
        .map_err(|_| format!("Cannot resolve {relative}"))?;
    if !resolved.starts_with(root) {
        return Err("Path leaves the policy root".into());
    }
    Ok(resolved)
}

pub fn is_link(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    metadata.file_type().is_symlink()
}

fn read_bytes(path: &Path) -> Result<Vec<u8>, String> {
    let meta = fs::symlink_metadata(path).map_err(|_| format!("Cannot read {}", path.display()))?;
    if !meta.is_file() || is_link(&meta) || meta.len() > 4 * 1024 * 1024 {
        return Err(format!(
            "Expected an ordinary file within 4 MiB: {}",
            path.display()
        ));
    }
    fs::read(path).map_err(|_| format!("Cannot read {}", path.display()))
}

pub fn read(path: &Path) -> Result<String, String> {
    let text = String::from_utf8(read_bytes(path)?)
        .map_err(|_| format!("Invalid UTF-8 or unreadable file: {}", path.display()))?;
    if text.starts_with('\u{feff}') || text.replace("\r\n", "").contains('\r') {
        return Err(format!("Invalid BOM or line endings: {}", path.display()));
    }
    Ok(text.replace("\r\n", "\n"))
}

pub fn registry(root: &Path) -> Result<Value, String> {
    let contents = read(&safe_path(root, "governance.yaml")?)?;
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(&contents).map_err(|_| "Invalid governance YAML or duplicate key")?;
    let registry =
        serde_json::to_value(yaml).map_err(|_| "Registry must be JSON-compatible YAML")?;
    let schema: Value = serde_json::from_str(&read(&safe_path(
        root,
        "en/schemas/governance.schema.json",
    )?)?)
    .map_err(|_| "Invalid governance schema JSON")?;
    let validator =
        jsonschema::validator_for(&schema).map_err(|_| "Cannot compile the governance schema")?;
    validator
        .validate(&registry)
        .map_err(|error| format!("Governance schema violation at {}", error.instance_path))?;
    let control_schema: Value =
        serde_json::from_str(include_str!("../en/schemas/cirnetrules-policy.schema.json"))
            .map_err(|_| "Invalid control plane schema")?;
    jsonschema::validator_for(&control_schema)
        .map_err(|_| "Cannot compile the control plane schema")?
        .validate(&registry["control_plane"])
        .map_err(|error| format!("Control plane schema violation at {}", error.instance_path))?;
    let version = read(&safe_path(root, "VERSION")?)?;
    if registry["governance"]["version"].as_str() != Some(version.trim())
        || version.trim() != env!("CARGO_PKG_VERSION")
    {
        return Err("Governance registry, VERSION and executable versions differ".into());
    }
    Ok(registry)
}

fn english(root: &Path, registry: &Value, path: &str) -> Result<PathBuf, String> {
    if !path.starts_with("en/")
        || !path.ends_with(".md")
        || path.split('/').any(|part| matches!(part, "history" | "vi"))
        || matches!(
            registry["document_states"][path]["state"].as_str(),
            Some("SUPERSEDED" | "RETIRED")
        )
    {
        return Err(format!("Invalid active English authority: {path}"));
    }
    safe_path(root, path)
}

fn visit<'a>(
    id: &'a str,
    graph: &BTreeMap<&'a str, Vec<&'a str>>,
    stack: &mut BTreeSet<&'a str>,
    visited: &mut BTreeSet<&'a str>,
) -> Result<(), String> {
    if visited.contains(id) {
        return Ok(());
    }
    if !stack.insert(id) {
        return Err(format!("Dependency cycle at {id}"));
    }
    for next in graph
        .get(id)
        .ok_or_else(|| format!("Missing dependency: {id}"))?
    {
        visit(next, graph, stack, visited)?;
    }
    stack.remove(id);
    visited.insert(id);
    Ok(())
}

fn validate_graph(graph: &BTreeMap<&str, Vec<&str>>) -> Result<(), String> {
    let mut visited = BTreeSet::new();
    for id in graph.keys() {
        visit(id, graph, &mut BTreeSet::new(), &mut visited)?;
    }
    Ok(())
}

fn text(value: &Value) -> &str {
    value.as_str().expect("registry validated by schema")
}

pub fn validate(root: &Path, registry: &Value) -> Result<(), String> {
    crate::release::validate_profile(registry)?;
    let mut migrated = BTreeSet::new();
    for entry in registry["migration"].as_array().unwrap() {
        let from = text(&entry["from"]);
        let to = text(&entry["to"]);
        if to != format!("en/{from}") || !migrated.insert(from) {
            return Err(format!("Invalid or duplicate migration for {from}"));
        }
        safe_path(root, to)?;
        safe_path(root, &format!("vi/{from}"))?;
        if has_old_content(&root.join(from), 0)? {
            return Err(format!("Old authority still exists under {from}"));
        }
    }
    for authority in registry["authority"].as_object().unwrap().values() {
        // Relative to the owning product checkout, inspected separately by doctor.
        let path = text(&authority["path"]);
        if registry["repositories"]
            .get(text(&authority["repository"]))
            .is_none()
            || path.contains(['\\', ':'])
            || path
                .split('/')
                .any(|part| part.is_empty() || matches!(part, "." | ".."))
        {
            return Err("Unsafe product authority path".into());
        }
    }
    for path in registry["context"]["common"].as_array().unwrap() {
        english(root, registry, text(path))?;
    }
    for task in registry["context"]["tasks"].as_object().unwrap().values() {
        for role in ["profile", "skill"] {
            english(root, registry, text(&task[role]))?;
        }
    }
    let mut ids = BTreeSet::new();
    for rule in registry["rules"].as_array().unwrap() {
        let id = text(&rule["id"]);
        if !ids.insert(id) {
            return Err(format!("Duplicate rule ID: {id}"));
        }
        english(root, registry, text(&rule["path"]))?;
    }
    let mut repos = BTreeMap::new();
    for (repo, entry) in registry["repositories"].as_object().unwrap() {
        if entry.get("plan").is_some() || entry.get("spec").is_some() {
            for (key, dir) in [("plan", "plans"), ("spec", "specs")] {
                let path = entry[key]
                    .as_str()
                    .ok_or("Historical plan/spec pair is incomplete")?;
                if !path.starts_with(&format!("en/{dir}/{repo}/")) {
                    return Err(format!("Misassigned {key} for {repo}"));
                }
                english(root, registry, path)?;
            }
            let spec = read(&english(root, registry, text(&entry["spec"]))?)?;
            for identity in ["commit", "tree"] {
                let value = entry["snapshot"][identity]
                    .as_str()
                    .ok_or("Historical snapshot is incomplete")?;
                if !spec.contains(&format!("`{value}`")) {
                    return Err(format!("Audit {identity} differs from registry for {repo}"));
                }
            }
        }
        repos.insert(
            repo.as_str(),
            entry["depends_on"]
                .as_array()
                .unwrap()
                .iter()
                .map(|dep| text(&dep["repository"]))
                .collect(),
        );
    }
    validate_graph(&repos)?;
    let control = &registry["control_plane"];
    english(root, registry, text(&control["entrypoint"]))?;
    for skill in control["skills"].as_object().unwrap().values() {
        english(root, registry, text(skill))?;
    }
    let known = |repo: &str| registry["repositories"].get(repo).is_some();
    for (repo, routes) in control["routes"].as_object().unwrap() {
        if !known(repo) {
            return Err(format!(
                "Routing refers to an unregistered repository: {repo}"
            ));
        }
        let mut route_ids = BTreeSet::new();
        for route in routes.as_array().unwrap() {
            if !route_ids.insert(text(&route["id"])) {
                return Err(format!("Duplicate route ID for {repo}"));
            }
            if route["impact"]
                .as_array()
                .unwrap()
                .iter()
                .any(|edge| !known(text(&edge["repository"])))
            {
                return Err(format!("Routing impact is not registered for {repo}"));
            }
        }
    }
    for rule in control["commands"].as_object().unwrap().values() {
        if !known(text(&rule["repository"])) {
            return Err("Command repository is not registered".into());
        }
    }
    for (path, entry) in registry["document_states"].as_object().unwrap() {
        safe_path(root, path)?;
        if entry["state"] == "APPROVED" && entry.get("approval_ref").is_none() {
            return Err(format!("Approval evidence is absent for {path}"));
        }
    }
    let mut tasks = BTreeMap::new();
    for task in registry["tasks"].as_array().unwrap() {
        let id = text(&task["id"]);
        let repo = text(&task["repository"]);
        let selection = &registry["repositories"][repo];
        if tasks
            .insert(
                id,
                task["depends_on"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(text)
                    .collect(),
            )
            .is_some()
        {
            return Err(format!("Duplicate task ID: {id}"));
        }
        if !ids.contains(text(&task["requirement"]))
            || task["plan"] != selection["plan"]
            || task["spec"] != selection["spec"]
        {
            return Err(format!("Broken traceability for {id}"));
        }
        let plan = read(&english(root, registry, text(&task["plan"]))?)?;
        if !plan
            .lines()
            .any(|line| line == format!("<!-- doc-id: {id} -->"))
        {
            return Err(format!("Plan document ID differs from task {id}"));
        }
        let evidence = task["evidence"].as_array().unwrap();
        if task["status"] == "VERIFIED" && evidence.is_empty() {
            return Err(format!("Task {id} claims completion without evidence"));
        }
        if task["status"] == "RELEASED"
            && (registry["baseline"]["activation"] != "ACTIVATED"
                || !evidence.iter().any(|item| item["kind"] == "publication"))
        {
            return Err(format!("Task {id} lacks activated publication evidence"));
        }
        for item in evidence {
            if item["commit"] != selection["snapshot"]["commit"] {
                return Err(format!("Evidence revision differs for {id}"));
            }
            let reference = format!("{}^{{commit}}", text(&item["governance_commit"]));
            git(root, &["cat-file", "-e", &reference])?;
            let result = safe_path(root, text(&item["result"]))?;
            if crate::docs::sha256(&read_bytes(&result)?) != item["sha256"] {
                return Err(format!("Evidence digest differs for {id}"));
            }
        }
    }
    validate_graph(&tasks)?;
    if !registry["exceptions"].as_array().unwrap().is_empty() {
        // Exception records are deliberately closed until the owner approves
        // an enforcement/clock policy and an actual scoped exception.
        return Err("No policy exceptions are authorized for this governance baseline".into());
    }
    if registry["baseline"]["activation"] == "ACTIVATED"
        && [
            "approved_matrix_sha256",
            "approved_evidence",
            "approved_publication_sha256",
        ]
        .iter()
        .any(|approval| registry["release_policy"][approval].is_null())
    {
        return Err("Activation requires compatibility, evidence and publication approvals".into());
    }
    Ok(())
}

fn has_old_content(path: &Path, depth: usize) -> Result<bool, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(_) => return Err("Cannot inspect retired location".into()),
    };
    if !metadata.is_dir() || is_link(&metadata) || depth > 8 {
        return Ok(true);
    }
    for entry in fs::read_dir(path).map_err(|_| "Cannot read retired location")? {
        let entry = entry.map_err(|_| "Cannot inspect retired location")?;
        if has_old_content(&entry.path(), depth + 1)? {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let caller = std::env::current_dir().unwrap_or_else(|_| root.to_path_buf());
    let mut command = Command::new(crate::runner::git_executable(root, &caller)?);
    for (key, _) in std::env::vars_os() {
        if key
            .to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("GIT_")
        {
            command.env_remove(key);
        }
    }
    let output = command
        .env("GIT_NO_LAZY_FETCH", "1")
        .args([
            "--no-optional-locks",
            "--no-replace-objects",
            "-c",
            "core.fsmonitor=false",
            "-C",
        ])
        .arg(root)
        .args(args)
        .output()
        .map_err(|_| "Git executable unavailable")?;
    if !output.status.success() || output.stdout.len() > 4 * 1024 * 1024 {
        return Err(format!(
            "Git {} could not verify {}",
            args[0],
            root.display()
        ));
    }
    String::from_utf8(output.stdout).map_err(|_| "Git output was not UTF-8".into())
}

fn checkout_commit(root: &Path) -> Result<String, String> {
    let actual = git(root, &["rev-parse", "--show-toplevel"])?;
    if Path::new(actual.trim()).canonicalize().ok().as_deref() != Some(root) {
        return Err("Git checkout root does not match the selected directory".into());
    }
    Ok(git(root, &["rev-parse", "--verify", "HEAD^{commit}"])?
        .trim()
        .to_owned())
}

pub fn checkout(root: &Path) -> Result<(String, bool), String> {
    let commit = checkout_commit(root)?;
    let clean = crate::release::worktree_is_clean(root)?;
    Ok((commit, clean))
}

fn origin_matches(root: &Path, expected: &str) -> Result<bool, String> {
    let origin = git(
        root,
        &[
            "config",
            "--local",
            "--no-includes",
            "--get-all",
            "remote.origin.url",
        ],
    )?;
    Ok(origin.trim().trim_end_matches(".git") == expected.trim_end_matches(".git"))
}

pub fn governance_origin_matches(root: &Path) -> Result<bool, String> {
    origin_matches(root, "https://github.com/cirnetlab/cirnet-rules")
}

fn identify(workspace: &Path, repo: &str, registry: &Value) -> Result<(PathBuf, String), String> {
    let expected = registry["repositories"][repo]["remote"]
        .as_str()
        .ok_or("Repository outside CIRNET scope")?;
    let workspace = workspace
        .canonicalize()
        .map_err(|_| "Product workspace unavailable")?;
    let product = safe_path(&workspace, repo)?;
    let commit = checkout_commit(&product)?;
    if !origin_matches(&product, expected)? {
        return Err(format!("Git remote does not match registry for {repo}"));
    }
    Ok((product, commit))
}

pub fn context(
    root: &Path,
    workspace: &Path,
    registry: &Value,
    repo: &str,
    task: &str,
    risk: &str,
) -> Result<Value, String> {
    let selection = registry["context"]["tasks"]
        .get(task)
        .ok_or("Unknown governance task")?;
    if !matches!(risk, "normal" | "high" | "hardware") {
        return Err("Unknown risk classification".into());
    }
    let (product, target_commit) = identify(workspace, repo, registry)?;
    if !governance_origin_matches(root)? {
        return Err("Governance origin does not match CIRNETLAB".into());
    }
    let (commit, clean) = checkout(root)?;
    let mut paths: Vec<&str> = registry["context"]["common"]
        .as_array()
        .unwrap()
        .iter()
        .map(text)
        .collect();
    paths.extend([text(&selection["profile"]), text(&selection["skill"])]);
    if matches!(risk, "high" | "hardware") {
        paths.push("en/profiles/reviewer.md");
    }
    if risk == "hardware" {
        paths.push("en/skills/hardware-evidence/SKILL.md");
    }
    let evidence = &registry["repositories"][repo];
    paths.extend([text(&evidence["plan"]), text(&evidence["spec"])]);
    let mut seen = BTreeSet::new();
    let mut documents = Vec::new();
    for path in paths {
        if seen.insert(path) {
            let source = read(&english(root, registry, path)?)?;
            documents.push(json!({"path":path, "sha256":crate::docs::sha256(source.as_bytes())}));
        }
    }
    Ok(json!({
        "governanceRoot":root,"productRoot":product,"repository":repo,"task":task,"risk":risk,
        "governanceCommit":commit,"governanceClean":clean,"targetCommit":target_commit,
        "scope":"read-only","documents":documents,"profile":evidence["profile"],
        "planReference":evidence["plan"],"auditReference":evidence["spec"]
    }))
}

pub fn doctor(root: &Path, workspace: Option<&Path>, registry: &Value) -> Result<Value, String> {
    let (commit, clean) = checkout(root)?;
    let version = |program| {
        Command::new(program)
            .current_dir(root)
            .env("RUSTUP_AUTO_INSTALL", "0")
            .arg("--version")
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|value| value.trim().to_owned())
    };
    let rust = version("rustc");
    let cargo = version("cargo");
    let required = "1.99.1";
    let matches_version = |value: &Option<String>| {
        value
            .as_deref()
            .and_then(|value| value.split_whitespace().nth(1))
            == Some(required)
    };
    let mut rows = Vec::new();
    let mut missing = !matches_version(&rust) || !matches_version(&cargo) || workspace.is_none();
    if let Some(workspace) = workspace {
        for repo in registry["repositories"]
            .as_object()
            .unwrap()
            .iter()
            .filter_map(|(repo, entry)| entry["remote"].is_string().then_some(repo))
        {
            match identify(workspace, repo, registry) {
                Ok((path, commit)) => {
                    rows.push(json!({"repo":repo,"status":"PASS","commit":commit,"path":path}))
                }
                Err(reason) => {
                    missing = true;
                    rows.push(json!({"repo":repo,"status":"BLOCKED","reason":reason}));
                }
            }
        }
        for authority in registry["authority"].as_object().unwrap().values() {
            let repo = text(&authority["repository"]);
            let relative = text(&authority["path"]);
            if identify(workspace, repo, registry)
                .and_then(|(path, _)| safe_path(&path, relative))
                .is_err()
            {
                missing = true;
                rows.push(json!({"repo":repo,"path":relative,"status":"BLOCKED","reason":"Authority not readable"}));
            }
        }
    }
    Ok(json!({
        "status":if missing{"BLOCKED"}else{"PASS"},
        "governance_commit":commit,"governance_clean":clean,"rust":rust,"cargo":cargo,"required_rust":required,
        "workspace_supplied":workspace.is_some(),"repositories":rows,"live_agent_discovery":"NOT_RUN"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_requires_reviewed_compatibility_evidence_and_publication() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .canonicalize()
            .unwrap();
        let mut registry = registry(&root).unwrap();
        registry["baseline"]["activation"] = json!("ACTIVATED");
        assert!(
            validate(&root, &registry)
                .unwrap_err()
                .contains("Activation requires")
        );
        registry["release_policy"]["approved_matrix_sha256"] = json!("a".repeat(64));
        registry["release_policy"]["approved_evidence"] =
            json!({"governance_commit":"a".repeat(40),"index_sha256":"b".repeat(64)});
        registry["release_policy"]["approved_publication_sha256"] = json!("c".repeat(64));
        validate(&root, &registry).unwrap();
    }

    #[test]
    fn updating_only_the_snapshot_cannot_hide_a_stale_audit() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .canonicalize()
            .unwrap();
        let mut registry = registry(&root).unwrap();
        registry["repositories"]["cirnet-remote"]["snapshot"]["commit"] = json!("a".repeat(40));
        assert!(
            validate(&root, &registry)
                .unwrap_err()
                .contains("Audit commit differs")
        );
    }
}
