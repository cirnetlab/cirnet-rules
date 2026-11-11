use super::{
    Step, blocked, boundary,
    process::{process_receipt, run_process},
    rows, safe_relative, scoped, sensitive, snapshot,
    snapshot::{git_command, git_version, load_input, no_filters},
    step, text, valid_actor,
};
use crate::{docs, policy, tasks, workspace};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn now_ms() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "Local clock is unavailable for the run lease")?
        .as_millis()
        .try_into()
        .map_err(|_| "Local run clock is outside the supported range".into())
}

fn run_dir(root: &Path, id: &str, create: bool) -> Result<PathBuf, String> {
    if id.len() > 80 || !valid_actor(id) || id.contains(['.', '@']) {
        return Err("Invalid task ID for a managed run".into());
    }
    let mut selected = root.to_path_buf();
    for relative in [
        ".cirnetrules",
        ".cirnetrules/runs",
        &format!(".cirnetrules/runs/{id}"),
    ] {
        let path = workspace::scoped_path(root, relative)?;
        if create {
            match fs::create_dir(&path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(_) => return Err("Unable to initialize the local run directory".into()),
            }
        }
        let meta = fs::symlink_metadata(&path).map_err(|_| "Managed run directory unavailable")?;
        if !meta.is_dir() || policy::is_link(&meta) {
            return Err("Managed run directory is redirected".into());
        }
        selected = path;
    }
    Ok(selected)
}

fn marker(dir: &Path, token: &str) -> Result<PathBuf, String> {
    if token.is_empty()
        || token.len() > 64
        || !token.bytes().all(|c| c.is_ascii_digit() || c == b'-')
    {
        return Err("Invalid owned run marker".into());
    }
    workspace::scoped_path(dir, &format!("{token}.cancel"))
}

fn read_run(dir: &Path) -> Result<Value, String> {
    let path = workspace::scoped_path(dir, "active.json")?;
    let meta = fs::symlink_metadata(&path).map_err(|_| "No active managed run")?;
    if !meta.is_file() || policy::is_link(&meta) || meta.len() > 2048 {
        return Err("Invalid active run lease".into());
    }
    let bytes = fs::read(path).map_err(|_| "Unable to read the active run lease")?;
    serde_json::from_slice(&bytes).map_err(|_| "Unreviewed active run lease".into())
}

struct RunLock {
    root: PathBuf,
    task: String,
    token: String,
    cancel: PathBuf,
}

impl RunLock {
    fn acquire(root: &Path, task: &str, actor: &str, timeout_ms: u64) -> Result<Self, String> {
        let dir = run_dir(root, task, true)?;
        let now = now_ms()?;
        let token = format!("{}-{now}", std::process::id());
        let cancel = marker(&dir, &token)?;
        let active = workspace::scoped_path(&dir, "active.json")?;
        let record = json!({"token":token,"actor":actor,"expires_at_ms":now + timeout_ms + 30_000});
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&active)
            .map_err(|_| "Task already owns an active or abandoned run. Inspect the lease before recovery.")?;
        file.write_all(record.to_string().as_bytes())
            .and_then(|_| file.sync_all())
            .map_err(|_| "Unable to persist the owned run lease")?;
        Ok(Self {
            root: root.into(),
            task: task.into(),
            token,
            cancel,
        })
    }
}

impl Drop for RunLock {
    fn drop(&mut self) {
        if let Ok(dir) = run_dir(&self.root, &self.task, false)
            && read_run(&dir).is_ok_and(|record| record["token"] == self.token)
        {
            if let Ok(marker) = marker(&dir, &self.token) {
                let _ = fs::remove_file(marker);
            }
            if let Ok(active) = workspace::scoped_path(&dir, "active.json") {
                let _ = fs::remove_file(active);
            }
        }
    }
}

fn cancel_step(root: &Path, task: &Value, actor: &str) -> Result<Value, String> {
    let id = text(&task["descriptor"]["id"], "Missing task identity")?;
    let dir = run_dir(root, id, false)?;
    let record = read_run(&dir)?;
    if !valid_actor(actor) || (record["actor"] != actor && task["descriptor"]["owner"] != actor) {
        return Ok(blocked(
            "Cancellation requires the running operator or task owner",
        ));
    }
    let token = text(&record["token"], "Missing run token")?;
    let cancel = marker(&dir, token)?;
    if record["expires_at_ms"]
        .as_u64()
        .is_some_and(|expiry| now_ms().is_ok_and(|now| now >= expiry))
    {
        let active = workspace::scoped_path(&dir, "active.json")?;
        let retained = workspace::scoped_path(&dir, &format!("{token}.expired.json"))?;
        if retained.exists() || !read_run(&dir).is_ok_and(|current| current == record) {
            return Ok(blocked("The run lease changed during recovery"));
        }
        fs::rename(&active, &retained).map_err(|_| "Unable to retire the expired owned lease")?;
        return Ok(json!({"status":"PASS","state":"EXPIRED_LEASE_RETAINED","signalled":false}));
    }
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&cancel)
    {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(_) => return Err("Unable to request cancellation of the owned run".into()),
    }
    Ok(json!({"status":"PASS","state":"CANCELLATION_REQUESTED","signalled":false}))
}

fn built_in(step: &Step<'_>, checkout: &Path, cancel: &Path) -> Result<Value, String> {
    let start = Instant::now();
    let mut records = Vec::new();
    let mut valid = true;
    let mut cancelled = false;
    let mut timed_out = false;
    for path in &step.paths {
        cancelled = cancel.is_file();
        timed_out = !cancelled && start.elapsed() >= Duration::from_millis(step.timeout_ms);
        if cancelled || timed_out {
            break;
        }
        let bytes = load_input(checkout, path)?;
        let passed = bytes.as_deref().is_some_and(|bytes| match step.program {
            "check-json" => serde_json::from_slice::<Value>(bytes).is_ok(),
            "check-utf8" => std::str::from_utf8(bytes).is_ok(),
            _ => false,
        });
        valid &= passed;
        records.push(json!({"path":path,"passed":passed,"bytes":bytes.map(|value| value.len())}));
    }
    cancelled |= cancel.is_file();
    timed_out |= !cancelled && start.elapsed() >= Duration::from_millis(step.timeout_ms);
    let digest = serde_json::to_vec(&records).map_err(|_| "Unable to serialize check receipt")?;
    let truncated = digest.len() > step.log_bytes;
    let status = if cancelled || timed_out || truncated {
        "BLOCKED"
    } else if valid {
        "PASS"
    } else {
        "FAIL"
    };
    Ok(json!({
        "status":status,"exit_code":if status == "BLOCKED" {None} else {Some(i32::from(!valid))},
        "elapsed_ms":start.elapsed().as_millis() as u64,
        "output_bytes":digest.len().min(step.log_bytes),"output_sha256":docs::sha256(&digest[..digest.len().min(step.log_bytes)]),
        "output_retained":false,"output_truncated":truncated,"timed_out":timed_out,"cancelled":cancelled
    }))
}

pub fn execute_step(
    root: &Path,
    task: &Value,
    registry: &Value,
    workspace: Option<&Path>,
    id: &str,
    actor: &str,
) -> Result<Value, String> {
    let step = match step(task, registry, id, actor, false) {
        Ok(step) => step,
        Err(reason) => return Ok(blocked(&reason)),
    };
    let baseline = &task["baseline"];
    if baseline.is_null() {
        return Ok(blocked("Task has no approved start snapshot"));
    }
    let binding = workspace::resolve_task(root, workspace, step.repo, registry, task)?;
    if !rows(baseline).any(|row| {
        row["repo"] == step.repo
            && row["identity"] == binding.identity
            && row["commit"] == binding.commit
    }) {
        return Ok(blocked("The approved worktree identity or commit changed"));
    }
    for path in &step.paths {
        workspace::scoped_path(&binding.path, path)?;
    }
    let lock = RunLock::acquire(
        root,
        text(&task["descriptor"]["id"], "Missing task ID")?,
        actor,
        step.timeout_ms,
    )?;
    crate::coordination::check_run(root, task, step.mode == "WORK")?;
    let tool_identity;
    let mut result = if step.program == "git" {
        tool_identity = git_version(root, &binding.path)?;
        let mut command = if step.args == ["diff", "--check"] {
            let mut command = no_filters(root, &binding.path)?;
            command.args([
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                "--check",
                "HEAD",
                "--",
            ]);
            command.args(&step.paths);
            command
        } else {
            let mut command = git_command(root, &binding.path)?;
            command.args(["rev-parse", "--verify", "HEAD"]);
            command
        };
        process_receipt(run_process(
            &mut command,
            Duration::from_millis(step.timeout_ms),
            step.log_bytes,
            Some(&lock.cancel),
        )?)
    } else {
        tool_identity = format!("cirnet-governance {}", env!("CARGO_PKG_VERSION"));
        built_in(&step, &binding.path, &lock.cancel)?
    };
    result["plan"] = step.plan();
    result["actor"] = json!(actor);
    result["step"] = json!(step.id);
    result["repository"] = json!(step.repo);
    result["tool_identity"] = json!(tool_identity);
    result["environment"] = json!({"arch":env::consts::ARCH,"os":env::consts::OS});
    result["limits"] = json!(
        "Read-only fixed operations. Raw output redacted. No arbitrary command, network, child tree, RAM or CPU sandbox."
    );
    Ok(result)
}

fn path_grant(
    root: &Path,
    task: &Value,
    registry: &Value,
    workspace: Option<&Path>,
    options: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let action = options.get("action").map(String::as_str).unwrap_or("read");
    let scope = match action {
        "read" => "read_scope",
        "write" => "write_scope",
        "verify" => "verification_scope",
        "docs" => "documentation_scope",
        _ => {
            return Ok(blocked(
                "The runner has no direct filesystem grant for this action",
            ));
        }
    };
    if let Err(reason) = tasks::authorize(task, action) {
        return Ok(blocked(&reason));
    }
    let descriptor = &task["descriptor"];
    let repo = options
        .get("repo")
        .map(String::as_str)
        .ok_or("Missing --repo")?;
    let path = options
        .get("path")
        .map(String::as_str)
        .ok_or("Missing --path")?;
    if !safe_relative(path) || sensitive(path) || !scoped(descriptor, scope, repo, path) {
        return Ok(blocked("Path is protected or outside the exact task grant"));
    }
    if matches!(action, "write" | "docs")
        && options
            .get("actor")
            .is_none_or(|actor| descriptor["owner"] != actor.as_str())
    {
        return Ok(blocked("File changes require the declared owner"));
    }
    if matches!(action, "write" | "docs") {
        crate::coordination::check_run(root, task, action == "write")?;
    }
    let binding = workspace::resolve_task(root, workspace, repo, registry, task)?;
    workspace::scoped_path(&binding.path, path)?;
    if action == "write" {
        if task["state"] != "IN_PROGRESS"
            || !rows(&task["baseline"]).any(|row| {
                row["repo"] == repo
                    && row["identity"] == binding.identity
                    && row["commit"] == binding.commit
            })
        {
            return Ok(blocked(
                "File changes require the original bound worktree and an active task",
            ));
        }
        if boundary(registry, path) && matches!(descriptor["risk"].as_str(), Some("low" | "normal"))
        {
            return Ok(blocked(
                "Boundary path requires risk reassessment before edits",
            ));
        }
    }
    Ok(json!({
        "status":"PASS","action":action,"repo":repo,"path":path,"worktree_identity":binding.identity,
        "task_revision":task["revision"],"assessment_only":true
    }))
}

pub fn run(
    command: &str,
    root: &Path,
    options: &BTreeMap<String, String>,
    registry: &Value,
) -> Result<Value, String> {
    let allowed: &[&str] = match command {
        "diff-check" => &["task", "workspace"],
        "cancel-step" => &["task", "actor"],
        "grant-evaluate" if options.contains_key("step") => &["task", "workspace", "actor", "step"],
        "grant-evaluate" => &["task", "workspace", "actor", "action", "repo", "path"],
        "run-step" => &["task", "workspace", "actor", "step", "action"],
        _ => return Err("Unknown runner command".into()),
    };
    if options.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err("Unsupported option for the runner command".into());
    }
    let id = options.get("task").ok_or("Missing --task")?;
    let task = if command == "cancel-step" {
        tasks::load(root, id)?
    } else {
        tasks::load_current(root, id)?
    };
    let workspace = options.get("workspace").map(Path::new);
    let actor = options.get("actor").map(String::as_str).unwrap_or_default();
    match command {
        "diff-check" => {
            tasks::authorize(&task, "read")?;
            if task["baseline"].is_null() {
                return Ok(blocked("Task has no approved start snapshot"));
            }
            snapshot(root, &task, registry, workspace)
        }
        "cancel-step" => cancel_step(root, &task, actor),
        "grant-evaluate" if !options.contains_key("step") => {
            path_grant(root, &task, registry, workspace, options)
        }
        "grant-evaluate" | "run-step" => {
            let id = options.get("step").ok_or("Missing --step")?;
            let action = options.get("action").map(String::as_str);
            if !matches!(action, None | Some("dry-run")) {
                return Err("Unsupported run-step action".into());
            }
            let dry_run = command == "grant-evaluate" || action == Some("dry-run");
            let prepared = match step(&task, registry, id, actor, dry_run) {
                Ok(step) => step,
                Err(reason) => return Ok(blocked(&reason)),
            };
            if dry_run {
                let binding =
                    workspace::resolve_task(root, workspace, prepared.repo, registry, &task)?;
                let baseline = &task["baseline"];
                if !baseline.is_null()
                    && !rows(baseline).any(|row| {
                        row["repo"] == prepared.repo
                            && row["identity"] == binding.identity
                            && row["commit"] == binding.commit
                    })
                {
                    return Ok(blocked("The approved worktree identity or commit changed"));
                }
                for path in &prepared.paths {
                    workspace::scoped_path(&binding.path, path)?;
                }
                return Ok(json!({
                    "status":"PASS","state":"DRY_RUN","plan":prepared.plan(),
                    "worktree_identity":binding.identity,"task_revision":task["revision"]
                }));
            }
            let current = snapshot(root, &task, registry, workspace)?;
            if current["status"] != "PASS" {
                return Ok(current);
            }
            execute_step(root, &task, registry, workspace, id, actor)
        }
        _ => Err("Unknown runner command".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_is_owned_and_expired_lease_is_retained() {
        let parent = env::temp_dir().canonicalize().unwrap();
        let root = parent.join(format!(
            "cirnetrules-run-lease-{}-{}",
            std::process::id(),
            now_ms().unwrap()
        ));
        fs::create_dir(&root).unwrap();
        let task = json!({"descriptor":{"id":"owned","owner":"owner"}});
        let lock = RunLock::acquire(&root, "owned", "verifier", 100).unwrap();
        assert_eq!(
            cancel_step(&root, &task, "unrelated").unwrap()["status"],
            "BLOCKED"
        );
        assert!(!lock.cancel.exists());
        assert_eq!(
            cancel_step(&root, &task, "owner").unwrap()["state"],
            "CANCELLATION_REQUESTED"
        );
        assert!(lock.cancel.is_file());
        drop(lock);

        let dir = run_dir(&root, "owned", false).unwrap();
        let active = dir.join("active.json");
        fs::write(
            &active,
            br#"{"token":"123-456","actor":"verifier","expires_at_ms":1}"#,
        )
        .unwrap();
        assert_eq!(
            cancel_step(&root, &task, "verifier").unwrap()["state"],
            "EXPIRED_LEASE_RETAINED"
        );
        assert!(!active.exists());
        assert!(dir.join("123-456.expired.json").is_file());
        assert!(root.canonicalize().unwrap().starts_with(parent));
        let _ = fs::remove_dir_all(root);
    }
}
