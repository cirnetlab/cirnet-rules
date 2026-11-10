use super::*;
use std::io::Read;

fn source_record(root: &Path, id: &str) -> Result<Value, String> {
    let mut source = None;
    for store in LEGACY_STORES {
        let relative = format!("{store}/tasks/{}.json", id.to_ascii_lowercase());
        match fs::symlink_metadata(root.join(store)) {
            Ok(meta) if meta.is_dir() && !policy::is_link(&meta) => {}
            Ok(_) => return Err("Legacy task store must be an ordinary directory".into()),
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(_) => return Err("Cannot inspect the legacy task store".into()),
        }
        let path = root.join(&relative);
        match fs::symlink_metadata(root.join(store).join("tasks")) {
            Ok(meta) if meta.is_dir() && !policy::is_link(&meta) => {}
            Ok(_) => return Err("Legacy tasks must be in an ordinary directory".into()),
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(_) => return Err("Cannot inspect the legacy tasks directory".into()),
        }
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_file() && !policy::is_link(&meta) => {}
            Ok(_) => return Err("Legacy task source must be an ordinary file".into()),
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(_) => return Err("Cannot inspect the legacy task record".into()),
        }
        if source.is_some() {
            return Err(
                "Task ID exists in multiple legacy stores; resolve the conflict before recovery"
                    .into(),
            );
        }
        let path = policy::safe_path(root, &relative)?;
        let mut bytes = Vec::new();
        File::open(path)
            .and_then(|file| file.take(RECORD_LIMIT as u64 + 1).read_to_end(&mut bytes))
            .map_err(|_| "Cannot read the legacy task record")?;
        if bytes.len() > RECORD_LIMIT {
            return Err("Legacy task record exceeds its byte budget".into());
        }
        let old = parse_json(std::str::from_utf8(&bytes).map_err(|_| "Legacy task is not UTF-8")?)?;
        if old["descriptor"]["id"] != id
            || !old["revision"].as_u64().is_some_and(|rev| rev > 0)
            || !STATES.contains(&text(&old["state"]))
        {
            return Err("Legacy task identity, revision or state is invalid".into());
        }
        source = Some(json!({
            "path": relative, "sha256": docs::sha256(&bytes),
            "revision": old["revision"], "state": old["state"]
        }));
    }
    source.ok_or_else(|| "No legacy task matches the reviewed descriptor ID".into())
}

fn preview(root: &Path, descriptor: Value) -> Result<Value, String> {
    let source = source_record(root, text(&descriptor["id"]))?;
    let proposal = new_record(root, descriptor)?;
    let mut review = json!({"source": source, "proposal": proposal});
    review["review_sha256"] = json!(descriptor_hash(&review)?);
    Ok(review)
}

pub(super) fn recover(
    root: &Path,
    record: Value,
    registry: &Value,
    review: &str,
) -> Result<Value, String> {
    descriptor(root, &record, registry)?;
    if record["authorization"]["actions"] != json!([])
        || record["authorization"].get("expires_at").is_some()
    {
        return Err(
            "Recovery requires an ungranted descriptor; revise the task to authorize it later"
                .into(),
        );
    }
    let mut checked = preview(root, record.clone())?;
    if review == "preview" {
        checked["status"] = json!("PASS");
        return Ok(checked);
    }
    if checked["review_sha256"] != review {
        return Err("Legacy record or recovery proposal changed; review a fresh preview".into());
    }
    let id = text(&record["id"]).to_owned();
    let key = id.to_ascii_lowercase();
    let _lock = lock(root, &key)?;
    let mut checked = preview(root, record)?;
    if checked["review_sha256"] != review {
        return Err("Legacy record or recovery proposal changed; review a fresh preview".into());
    }
    match fs::symlink_metadata(root.join(format!("{STORE}/{key}.json"))) {
        Ok(_) => {
            let current = load(root, &id)?;
            if current["recovery"]["review_sha256"] != review {
                return Err("Task ID is already in use by a different record".into());
            }
            return Ok(json!({"status": "PASS", "task": current}));
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(_) => return Err("Cannot inspect the recovery destination".into()),
    }
    let mut proposal = checked["proposal"].take();
    proposal["recovery"] = json!({"source": checked["source"], "review_sha256": review});
    persist(root, &id, &proposal)?;
    Ok(json!({"status": "PASS", "task": proposal}))
}
