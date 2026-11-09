//! Explicit, read-only migration of legacy file state into the local archive.
use super::{
    catalog,
    storage::{self, AppError, Database},
};
use rusqlite::{OptionalExtension, TransactionBehavior, params};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};

const RECORD_LIMIT: usize = 128 * 1024;
const RECORD_COUNT: usize = 512;
const ENTRY_LIMIT: usize = RECORD_COUNT * 4;
const TOTAL_LIMIT: usize = 16 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    source: PathBuf,
    workspace: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Apply {
    source: PathBuf,
    workspace: Option<PathBuf>,
    manifest_sha256: String,
    writers_stopped: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordRef {
    source_id: String,
    path: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Link {
    source_id: String,
    path: String,
    binding_id: String,
    binding_revision: i64,
}

#[derive(Debug)]
struct Record {
    path: String,
    id: String,
    kind: &'static str,
    state: &'static str,
    revision: Option<i64>,
    digest: String,
    body: Value,
}

struct Inventory {
    source: PathBuf,
    workspace: Option<PathBuf>,
    id: String,
    digest: String,
    records: Vec<Record>,
}

pub fn dispatch(db: &mut Database, method: &str, params: Value) -> Result<Value, AppError> {
    match method {
        "legacy.preview" => {
            let input: Source = storage::decode(params)?;
            let archive = inventory(input)?;
            Ok(summary(&archive, "PREVIEW", None))
        }
        "legacy.apply" => apply(db, storage::decode(params)?),
        "legacy.inspect" => inspect(db, storage::decode(params)?),
        "legacy.link" => link(db, storage::decode(params)?),
        _ => Err(AppError::new(
            "UNKNOWN_METHOD",
            "Unknown legacy archive operation.",
        )),
    }
}

fn summary(archive: &Inventory, status: &str, backup: Option<&Value>) -> Value {
    let counts = archive
        .records
        .iter()
        .fold(BTreeMap::new(), |mut counts, record| {
            *counts.entry(record.kind).or_insert(0usize) += 1;
            counts
        });
    let files = archive.records.iter().take(100).map(|record| {
        json!({"path":record.path,"legacy_id":record.id,"kind":record.kind,"state":record.state,"revision":record.revision,"sha256":record.digest})
    }).collect::<Vec<_>>();
    json!({"status":status,"source_id":archive.id,"manifest_sha256":archive.digest,
        "counts":counts,"record_count":archive.records.len(),"sample":files,"sample_truncated":archive.records.len()>100,
        "active_storage_cutover":false,"source_retained":true,"backup":backup})
}

fn apply(db: &mut Database, input: Apply) -> Result<Value, AppError> {
    if !input.writers_stopped {
        return Err(AppError::new(
            "LEGACY_BUSY",
            "Stop legacy writers before archiving their state.",
        ));
    }
    let archive = inventory(Source {
        source: input.source,
        workspace: input.workspace,
    })?;
    if archive.digest != input.manifest_sha256 {
        return Err(AppError::new(
            "STALE_PREVIEW",
            "Legacy files or workspace selection changed. Preview again.",
        ));
    }
    let existing = |conn: &rusqlite::Connection| -> Result<Option<String>, AppError> {
        Ok(conn
            .query_row(
                "SELECT manifest_sha256 FROM legacy_imports WHERE id=?1",
                [&archive.id],
                |row| row.get(0),
            )
            .optional()?)
    };
    if let Some(digest) = existing(&db.conn)? {
        if digest != archive.digest {
            return Err(AppError::new(
                "CONFLICT",
                "This legacy source already has a different archived manifest.",
            ));
        }
        return Ok(summary(&archive, "UNCHANGED", None));
    }
    let backup = db.backup()?;
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(digest) = existing(&tx)? {
        if digest != archive.digest {
            return Err(AppError::new(
                "CONFLICT",
                "A concurrent import archived another manifest.",
            ));
        }
        return Ok(summary(&archive, "UNCHANGED", Some(&backup)));
    }
    tx.execute(
        "INSERT INTO legacy_imports(id,source_path,manifest_sha256,workspace_root,records,imported_at) VALUES(?1,?2,?3,?4,?5,?6)",
        params![archive.id,archive.source.to_str(),archive.digest,archive.workspace.as_ref().and_then(|path| path.to_str()),archive.records.len(),storage::now()?],
    )?;
    for record in &archive.records {
        tx.execute(
            "INSERT INTO legacy_records(import_id,relative_path,kind,legacy_id,legacy_revision,state,source_sha256,body) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![archive.id,record.path,record.kind,record.id,record.revision,record.state,record.digest,record.body.to_string()],
        )?;
    }
    tx.commit()?;
    Ok(summary(&archive, "ARCHIVED", Some(&backup)))
}

fn inspect(db: &Database, input: RecordRef) -> Result<Value, AppError> {
    let (kind, id, revision, state, digest, body): (String,String,Option<i64>,String,String,String) = db.conn.query_row(
        "SELECT kind,legacy_id,legacy_revision,state,source_sha256,body FROM legacy_records WHERE import_id=?1 AND relative_path=?2",
        params![input.source_id,input.path],
        |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?)),
    )?;
    let body: Value =
        serde_json::from_str(&body).map_err(|_| legacy_invalid("Archived record is malformed."))?;
    let mapped: Option<Value> = db.conn.query_row(
        "SELECT binding_id,binding_revision,repo_id FROM legacy_binding_links WHERE import_id=?1 AND relative_path=?2",
        params![input.source_id,input.path],
        |row| Ok(json!({"binding_id":row.get::<_,String>(0)?,"binding_revision":row.get::<_,i64>(1)?,"repo_id":row.get::<_,String>(2)?})),
    ).optional()?;
    Ok(
        json!({"kind":kind,"legacy_id":id,"revision":revision,"state":state,"sha256":digest,
        "body":body,"mapping":mapped,"authenticated_approval":false,"executable":false}),
    )
}

fn link(db: &mut Database, input: Link) -> Result<Value, AppError> {
    let (workspace, state, body): (Option<String>,String,String) = db.conn.query_row(
        "SELECT i.workspace_root,r.state,r.body FROM legacy_records r JOIN legacy_imports i ON i.id=r.import_id
         WHERE r.import_id=?1 AND r.relative_path=?2 AND r.kind='BINDING'",
        params![input.source_id,input.path], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?)),
    )?;
    let workspace = workspace
        .filter(|_| state == "REVIEW_REQUIRED")
        .ok_or_else(|| {
            AppError::new(
                "BINDING_REQUIRED",
                "The archived record has no verified local workspace namespace.",
            )
        })?;
    let body: Value = serde_json::from_str(&body)
        .map_err(|_| legacy_invalid("Archived binding is malformed."))?;
    let relative = body["relative"]
        .as_str()
        .filter(|path| safe_relative(path))
        .ok_or_else(|| legacy_invalid("Archived binding path is invalid."))?;
    let anchor = body["anchor"]
        .as_str()
        .filter(|anchor| valid_hex(anchor, &[40, 64]))
        .ok_or_else(|| legacy_invalid("Archived binding anchor is invalid."))?;
    let local = Path::new(&workspace).join(relative);
    storage::check_ancestors(&local, false)?;
    let local = local.canonicalize().map_err(|_| {
        AppError::new(
            "BINDING_REQUIRED",
            "The reviewed legacy checkout no longer exists.",
        )
    })?;
    let binding = catalog::resolve_binding(db, &input.binding_id, Some(input.binding_revision))?;
    if binding.root != local || !catalog::verifies_archived_anchor(&local, anchor)? {
        return Err(AppError::new(
            "REBIND_REQUIRED",
            "The registered binding does not match the reviewed legacy checkout and anchor.",
        ));
    }
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    let unchanged: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM bindings b JOIN repositories r ON b.repo_id=r.id
         WHERE b.id=?1 AND b.revision=?2 AND b.identity=?3 AND b.machine_id=?4 AND r.archived=0)",
        params![
            binding.id,
            binding.revision,
            binding.identity,
            db.machine_id
        ],
        |row| row.get(0),
    )?;
    if !unchanged {
        return Err(AppError::new(
            "CONFLICT",
            "The registered binding changed during archive linking.",
        ));
    }
    if tx.execute(
        "INSERT INTO legacy_binding_links(import_id,relative_path,binding_id,binding_revision,repo_id,linked_at)
         VALUES(?1,?2,?3,?4,?5,?6)
         ON CONFLICT(import_id,relative_path) DO UPDATE SET binding_revision=excluded.binding_revision,linked_at=excluded.linked_at
         WHERE legacy_binding_links.binding_id=excluded.binding_id AND legacy_binding_links.repo_id=excluded.repo_id",
        params![input.source_id,input.path,binding.id,binding.revision,binding.repo_id,storage::now()?],
    )? != 1 {
        return Err(AppError::new("CONFLICT", "This archive already links to a different registration."));
    }
    tx.commit()?;
    Ok(
        json!({"status":"LINKED","binding_id":binding.id,"repo_id":binding.repo_id,"binding_revision":binding.revision,
        "permissions_granted":false,"activated":false}),
    )
}

fn inventory(input: Source) -> Result<Inventory, AppError> {
    if !matches!(
        input.source.file_name().and_then(|name| name.to_str()),
        Some(".openrules" | ".cgacp")
    ) {
        return Err(legacy_invalid(
            "Select a supported legacy state directory explicitly.",
        ));
    }
    let source = ordinary_directory(input.source)?;
    let workspace = input.workspace.map(ordinary_directory).transpose()?;
    let namespace = workspace
        .as_ref()
        .and_then(|path| path.to_str())
        .map(|path| sha(path.as_bytes()));
    let mut records = Vec::new();
    let mut budget = TOTAL_LIMIT;
    let mut ids = BTreeSet::new();
    let mut entries_remaining = ENTRY_LIMIT;
    for path in entries(&source.join("tasks"), &mut entries_remaining)? {
        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let filename = file_name(&path)?;
        let (body, digest) = read_record(&path, &mut budget)?;
        let id = body["descriptor"]["id"]
            .as_str()
            .filter(|id| valid_id(id))
            .ok_or_else(|| legacy_invalid("A legacy task has an invalid ID."))?;
        let revision = body["revision"]
            .as_i64()
            .filter(|n| *n > 0)
            .ok_or_else(|| legacy_invalid("A legacy task has an invalid revision."))?;
        if !filename.eq_ignore_ascii_case(&format!("{id}.json"))
            || !ids.insert(id.to_ascii_lowercase())
            || !matches!(
                body["state"].as_str(),
                Some(
                    "PROPOSED"
                        | "READY"
                        | "IN_PROGRESS"
                        | "IMPLEMENTED"
                        | "VERIFIED"
                        | "ACCEPTED"
                        | "DOCS_SYNCED"
                        | "CLOSED"
                        | "BLOCKED"
                        | "CANCELLED"
                        | "SUPERSEDED"
                )
            )
        {
            return Err(legacy_invalid(
                "Legacy task identity, duplicate ID or state is invalid.",
            ));
        }
        let state = if body["state"] == "IN_PROGRESS" {
            "RECOVERY_REQUIRED"
        } else {
            "ARCHIVED"
        };
        records.push(Record {
            path: format!("tasks/{filename}"),
            id: id.to_owned(),
            kind: "TASK",
            state,
            revision: Some(revision),
            digest,
            body,
        });
    }
    for scope in entries(&source.join("bindings"), &mut entries_remaining)? {
        let scope_id = file_name(&scope)?;
        if !valid_hex(scope_id, &[64]) {
            continue;
        }
        for path in entries(&scope, &mut entries_remaining)? {
            if path.extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }
            let filename = file_name(&path)?;
            let id = filename
                .strip_suffix(".json")
                .filter(|name| valid_id(name))
                .ok_or_else(|| legacy_invalid("Invalid legacy repository ID."))?;
            if !ids.insert(format!("binding:{scope_id}:{}", id.to_ascii_lowercase())) {
                return Err(legacy_invalid(
                    "Duplicate legacy binding ID in one workspace.",
                ));
            }
            let (body, digest) = read_record(&path, &mut budget)?;
            if body["schema"] != 1
                || !body["relative"].as_str().is_some_and(safe_relative)
                || !body["anchor"]
                    .as_str()
                    .is_some_and(|anchor| valid_hex(anchor, &[40, 64]))
                || !body["identity"]
                    .as_str()
                    .is_some_and(|identity| valid_hex(identity, &[64]))
            {
                return Err(legacy_invalid("Invalid legacy worktree record."));
            }
            let state = if namespace.as_deref() == Some(scope_id) {
                "REVIEW_REQUIRED"
            } else {
                "BINDING_REQUIRED"
            };
            records.push(Record {
                path: format!("bindings/{scope_id}/{filename}"),
                id: id.to_owned(),
                kind: "BINDING",
                state,
                revision: None,
                digest,
                body,
            });
            check_count(&records)?;
        }
    }
    for dir in entries(&source.join("runs"), &mut entries_remaining)? {
        let task = file_name(&dir)?;
        if !valid_id(task) {
            continue;
        }
        for path in entries(&dir, &mut entries_remaining)? {
            let name = file_name(&path)?;
            if name != "active.json" && !name.ends_with(".expired.json") {
                continue;
            }
            let (mut body, digest) = read_record(&path, &mut budget)?;
            if !body["actor"].is_string()
                || !body["expires_at_ms"].is_u64()
                || !body["token"].is_string()
            {
                return Err(legacy_invalid("Invalid legacy run lease."));
            }
            if let Some(body) = body.as_object_mut() {
                body.remove("token");
            }
            records.push(Record {
                path: format!("runs/{task}/{name}"),
                id: task.to_owned(),
                kind: "LEASE",
                state: "RECOVERY_REQUIRED",
                revision: None,
                digest,
                body,
            });
            check_count(&records)?;
        }
    }
    records.sort_by(|left, right| left.path.cmp(&right.path));
    let manifest = records
        .iter()
        .map(|record| (&record.path, &record.digest))
        .collect::<Vec<_>>();
    let digest = sha(&serde_json::to_vec(&(&workspace, manifest))
        .map_err(|_| legacy_invalid("Cannot encode legacy manifest."))?);
    let id = format!(
        "legacy-{}",
        sha(source
            .to_str()
            .ok_or_else(|| legacy_invalid("Invalid local path."))?
            .as_bytes())
    );
    Ok(Inventory {
        source,
        workspace,
        id,
        digest,
        records,
    })
}

fn ordinary_directory(path: PathBuf) -> Result<PathBuf, AppError> {
    storage::check_ancestors(&path, false)?;
    if path
        .components()
        .any(|part| part == std::path::Component::ParentDir)
        || !path.is_dir()
    {
        return Err(legacy_invalid("Invalid local archive directory."));
    }
    path.canonicalize()
        .map_err(|_| legacy_invalid("Cannot resolve local archive directory."))
}

fn entries(dir: &Path, entries_remaining: &mut usize) -> Result<Vec<PathBuf>, AppError> {
    match fs::symlink_metadata(dir) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Ok(meta) if meta.is_dir() && !storage::redirect(&meta) => {}
        _ => {
            return Err(legacy_invalid(
                "A legacy metadata directory is invalid or redirected.",
            ));
        }
    }
    let read = fs::read_dir(dir).map_err(|_| legacy_invalid("Cannot list legacy metadata."))?;
    let limit = RECORD_COUNT.min(*entries_remaining);
    let mut paths = read
        .take(limit + 1)
        .map(|row| row.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|_| legacy_invalid("Cannot inspect legacy metadata."))?;
    if paths.len() > limit {
        return Err(legacy_invalid("Legacy archive exceeds the entry budget."));
    }
    *entries_remaining -= paths.len();
    paths.sort();
    Ok(paths)
}

fn read_record(path: &Path, remaining: &mut usize) -> Result<(Value, String), AppError> {
    let meta = fs::symlink_metadata(path)
        .map_err(|_| legacy_invalid("Cannot inspect a legacy record."))?;
    let limit = RECORD_LIMIT.min(*remaining);
    if !meta.is_file() || storage::redirect(&meta) || meta.len() > limit as u64 {
        return Err(legacy_invalid(
            "Legacy record is redirected or exceeds the byte budget.",
        ));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .and_then(|file| file.take(limit as u64 + 1).read_to_end(&mut bytes))
        .map_err(|_| legacy_invalid("Cannot read a legacy record."))?;
    if bytes.len() > limit {
        return Err(legacy_invalid("Legacy record exceeds the byte budget."));
    }
    *remaining -= bytes.len();
    let body: Value =
        serde_json::from_slice(&bytes).map_err(|_| legacy_invalid("Legacy JSON is malformed."))?;
    // YAML's mapping parser also rejects duplicate keys. The JSON parser above
    // remains the authority for JSON syntax and value types.
    let _: serde_yaml::Value = serde_yaml::from_slice(&bytes)
        .map_err(|_| legacy_invalid("Legacy JSON contains ambiguous mappings."))?;
    if !body.is_object() || contains_credential(&body) {
        return Err(legacy_invalid(
            "Legacy record must be an object without identifiable plaintext credentials.",
        ));
    }
    Ok((body, sha(&bytes)))
}

fn contains_credential(value: &Value) -> bool {
    match value {
        Value::String(value) => {
            let value = value.to_ascii_lowercase();
            [
                "-----begin private key-----",
                "-----begin rsa private key-----",
                "authorization: bearer ",
                "github_pat_",
                "ghp_",
                "sk-proj-",
            ]
            .iter()
            .any(|marker| value.contains(marker))
        }
        Value::Array(items) => items.iter().any(contains_credential),
        Value::Object(items) => items.iter().any(|(name, value)| {
            matches!(
                name.to_ascii_lowercase().as_str(),
                "api_key"
                    | "password"
                    | "access_token"
                    | "refresh_token"
                    | "private_key"
                    | "client_secret"
            ) && value.as_str().is_some_and(|value| !value.is_empty())
                || contains_credential(value)
        }),
        _ => false,
    }
}

fn check_count(records: &[Record]) -> Result<(), AppError> {
    if records.len() > RECORD_COUNT {
        return Err(legacy_invalid("Legacy archive exceeds the record budget."));
    }
    Ok(())
}

fn file_name(path: &Path) -> Result<&str, AppError> {
    path.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| legacy_invalid("Legacy filename is not valid UTF-8."))
}

fn valid_id(id: &str) -> bool {
    (1..=96).contains(&id.len())
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
}

fn valid_hex(value: &str, lengths: &[usize]) -> bool {
    lengths.contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn safe_relative(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 1024
        && !path.contains(['\\', ':'])
        && !path.chars().any(char::is_control)
        && !path
            .split('/')
            .any(|part| matches!(part, "" | "." | "..") || part.ends_with(['.', ' ']))
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn legacy_invalid(message: &str) -> AppError {
    AppError::new("INVALID_LEGACY_DATA", message)
}
