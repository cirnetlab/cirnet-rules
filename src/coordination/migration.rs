use super::{record_digest, record_event, revision};
use crate::{docs, tasks, workspace};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::{ErrorKind, Read, Write},
    path::Path,
};

pub(super) fn migrate(
    root: &Path,
    registry: &Value,
    id: &str,
    args: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let actor = args.get("actor").ok_or("Missing --actor")?;
    let expected = args
        .get("revision")
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or("Missing or invalid --revision")?;
    let confirmation = args.get("confirm").ok_or("Use --confirm preview first")?;
    if confirmation == "preview" {
        return preview(root, id, actor, expected).map(|(_, _, report)| report);
    }
    let _coordinator = tasks::lock(root, "_coordination")?;
    let _task = tasks::lock(root, &id.to_ascii_lowercase())?;
    let (mut task, raw, report) = preview(root, id, actor, expected)?;
    if report["review_sha256"] != *confirmation {
        return Err("Task changed or migration review digest is invalid".into());
    }
    super::ensure_quiescent(root, id)?;
    let mut descriptor = task["descriptor"].clone();
    descriptor["authorization"]["actions"] = json!([]);
    descriptor["authorization"]
        .as_object_mut()
        .ok_or("Invalid legacy authorization")?
        .remove("expires_at");
    let before = record_digest(&task)?;
    tasks::reset(root, &mut task, descriptor, registry)?;
    let archive = format!(
        ".cirnetrules/task-archive/{}-{}.json",
        id.to_ascii_lowercase(),
        docs::sha256(&raw)
    );
    let _directory = tasks::directory(root, ".cirnetrules/task-archive", true)?;
    let path = workspace::scoped_path(root, &archive)?;
    let mut options = fs::OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    match options.open(&path) {
        Ok(mut file) => {
            if file.write_all(&raw).and_then(|_| file.sync_all()).is_err() {
                drop(file);
                let _ = fs::remove_file(&path);
                return Err("Cannot persist old task snapshot".into());
            }
        }
        Err(error) if error.kind() == ErrorKind::AlreadyExists && snapshot(&path)? == raw => {}
        Err(_) => return Err("Cannot reserve old task snapshot".into()),
    }
    task["revision"] = json!(expected.checked_add(1).ok_or("Task revision exhausted")?);
    task["schema_migration"] = json!({"snapshot":archive,"snapshot_sha256":docs::sha256(&raw)});
    record_event(&mut task, "migrate", Some(actor), Some(before))?;
    tasks::persist(root, id, &task)?;
    Ok(
        json!({"status":"PASS","task":id,"state":task["state"],"revision":task["revision"],"archive":archive}),
    )
}

fn preview(
    root: &Path,
    id: &str,
    actor: &str,
    expected: u64,
) -> Result<(Value, Vec<u8>, Value), String> {
    let task = tasks::load(root, id)?;
    tasks::actor_is(&task, actor, true)?;
    if revision(&task)? != expected || task["task_schema"] == 2 {
        return Err("Expected revision must refer to an unmigrated task".into());
    }
    if matches!(
        task["state"].as_str(),
        Some("IN_PROGRESS" | "IMPLEMENTED" | "VERIFIED" | "ACCEPTED" | "DOCS_SYNCED")
    ) {
        return Err(
            "Stop and block active or pending task work before migrating its record".into(),
        );
    }
    let path = workspace::scoped_path(
        root,
        &format!(".cirnetrules/tasks/{}.json", id.to_ascii_lowercase()),
    )?;
    let raw = snapshot(&path)?;
    let source = std::str::from_utf8(&raw).map_err(|_| "Invalid prior task UTF-8")?;
    if tasks::parse_json(source)? != task {
        return Err("Task changed while preparing migration review".into());
    }
    let mut report = json!({
        "status":"PASS","task":id,"source_revision":expected,"source_state":task["state"],
        "source_sha256":docs::sha256(&raw),"target_state":"PROPOSED",
        "target_schema":2,"target_grants":[]
    });
    report["review_sha256"] = json!(super::digest(&report)?);
    Ok((task, raw, report))
}

fn snapshot(path: &Path) -> Result<Vec<u8>, String> {
    const LIMIT: u64 = 4 * 1024 * 1024;
    let meta = fs::symlink_metadata(path).map_err(|_| "Cannot inspect prior task snapshot")?;
    if !meta.is_file() || crate::policy::is_link(&meta) || meta.len() > LIMIT {
        return Err("Prior task snapshot must be an ordinary bounded file".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .and_then(|file| file.take(LIMIT + 1).read_to_end(&mut bytes))
        .map_err(|_| "Cannot read prior task snapshot")?;
    if bytes.len() as u64 > LIMIT {
        return Err("Prior task snapshot exceeds the byte budget".into());
    }
    Ok(bytes)
}
