use super::{documents, files};
use crate::docs::sha256;
use rusqlite::{
    Connection, OpenFlags,
    backup::{Backup, StepResult},
    types::ValueRef,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fmt::Write as _,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub(super) const MIGRATIONS: &[(&str, &str)] = &[
    (
        "001_catalog.sql",
        include_str!("../../migrations/sqlite/001_catalog.sql"),
    ),
    (
        "002_sessions.sql",
        include_str!("../../migrations/sqlite/002_sessions.sql"),
    ),
    (
        "003_legacy.sql",
        include_str!("../../migrations/sqlite/003_legacy.sql"),
    ),
    (
        "004_desktop_preferences.sql",
        include_str!("../../migrations/sqlite/004_desktop_preferences.sql"),
    ),
    (
        "005_conversation_lifecycle.sql",
        include_str!("../../migrations/sqlite/005_conversation_lifecycle.sql"),
    ),
    (
        "006_automation_retirement.sql",
        include_str!("../../migrations/sqlite/006_automation_retirement.sql"),
    ),
    (
        "007_governance.sql",
        include_str!("../../migrations/sqlite/007_governance.sql"),
    ),
];
pub(super) const HISTORY_TABLE: &str = "CREATE TABLE schema_migrations (revision INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE, checksum TEXT NOT NULL, applied_at INTEGER NOT NULL) STRICT;";
const EXPORT_TABLES: &[(&str, &str)] = &[
    ("repositories", "id"),
    ("bindings", "id"),
    ("governance_assets", "id"),
    ("governance_revisions", "asset_id, revision"),
    ("governance_dependencies", "asset_id, layer, target_id"),
    ("governance_assignments", "binding_id, asset_id"),
    ("governance_defaults", "singleton"),
    ("governance_reviews", "asset_id"),
    ("governance_events", "id"),
    ("governance_mutations", "key"),
    ("governance_session_snapshots", "conversation_id"),
    ("governance_run_snapshots", "run_id"),
];
const MANIFEST: &str = "archive.json";
const SNAPSHOT: &str = "snapshot.sqlite";
const EXPORT: &str = "governance.jsonl";
const MANIFEST_LIMIT: u64 = 16 * 1024 * 1024;
pub(super) const GOVERNANCE_SCHEMA: usize = 7;
// The original Windows working copy of 007 had mixed line endings, while
// the Git blob is LF. Both were embedded verbatim by the retired runtime.
const KNOWN_WINDOWS_007: &str = "3b0f5974923ef09b5f05ae70035ce1d08c05d1bbe220c893563ae85280ca7b6b";

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: u32,
    source: PathBuf,
    schema: usize,
    snapshot_sha256: String,
    export_sha256: String,
    inventory: Inventory,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Inventory {
    counts: BTreeMap<String, u64>,
    drafts: u64,
    archived: u64,
    deleted: u64,
    findings: Vec<documents::Finding>,
}

impl Inventory {
    fn report(&self, schema: usize) -> Value {
        let mut issues = BTreeMap::new();
        for finding in &self.findings {
            *issues.entry(finding.code.as_str()).or_insert(0_u64) += 1;
        }
        json!({
            "status": "PASS", "schema": schema, "counts": self.counts,
            "drafts": self.drafts, "archived": self.archived, "deleted": self.deleted, "issues": issues
        })
    }
}

impl Manifest {
    fn report(&self, receipt: &str) -> Value {
        let mut value = self.inventory.report(self.schema);
        value["snapshot_sha256"] = json!(self.snapshot_sha256);
        value["export_sha256"] = json!(self.export_sha256);
        value["receipt_sha256"] = json!(receipt);
        value
    }
}

fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut file = path.as_os_str().to_owned();
    file.push(suffix);
    PathBuf::from(file)
}

fn check_files(path: &Path, snapshot: bool) -> Result<bool, String> {
    let exists = files::ordinary_file(path)?;
    for suffix in ["-wal", "-shm", "-journal"] {
        if files::ordinary_file(&sidecar(path, suffix))? && (!exists || snapshot) {
            return Err("Unexpected migration database sidecar".into());
        }
    }
    Ok(exists)
}

struct Source {
    path: PathBuf,
    db: Option<Connection>,
    version: usize,
    _shared_memory: Option<File>,
}

fn open_source(path: &Path) -> Result<Option<Source>, String> {
    let absolute = files::absolute(path)?;
    if !check_files(&absolute, false)? {
        return Ok(None);
    }
    let absolute = absolute
        .canonicalize()
        .map_err(|_| "Cannot resolve the legacy database")?;
    let mut header = [0; 64];
    files::open(&absolute)?
        .read_exact(&mut header)
        .map_err(|_| "Cannot read the legacy SQLite header")?;
    if &header[..16] != b"SQLite format 3\0" {
        return Err("Invalid legacy SQLite header".into());
    }
    let shared_memory = if header[18..20] == [2, 2] {
        let shared = sidecar(&absolute, "-shm");
        let wal = files::ordinary_file(&sidecar(&absolute, "-wal"))?;
        let shm = files::ordinary_file(&shared)?;
        if !wal && !shm {
            require_no_sidecars(&absolute)?;
            let version = u32::from_be_bytes(header[60..64].try_into().unwrap()) as usize;
            if !(1..=MIGRATIONS.len()).contains(&version) {
                return Err(format!(
                    "Unsupported legacy SQLite schema revision {version}"
                ));
            }
            return Ok(Some(Source {
                path: absolute,
                db: None,
                version,
                _shared_memory: None,
            }));
        }
        if !wal || !shm {
            return Err(
                "Read-only WAL access requires both existing sidecars when either is present"
                    .into(),
            );
        }
        // Windows SQLite uses OPEN_ALWAYS for its SHM handle even with
        // readonly_shm. Keep the existing file from being unlinked while
        // opening the source. On Unix readonly_shm uses O_RDONLY alone.
        Some(files::open_existing_shared(&shared)?)
    } else {
        None
    };
    let uri = source_uri(&absolute)?;
    let db = open_reader(Path::new(&uri), OpenFlags::SQLITE_OPEN_URI)?;
    let version = check_history(&db).map_err(|reason| {
        if shared_memory.is_some() {
            format!("Could not verify the legacy WAL source in read-only mode: {reason}")
        } else {
            reason
        }
    })?;
    Ok(Some(Source {
        path: absolute,
        db: Some(db),
        version,
        _shared_memory: shared_memory,
    }))
}

fn require_no_sidecars(path: &Path) -> Result<(), String> {
    for suffix in ["-wal", "-shm", "-journal"] {
        if files::ordinary_file(&sidecar(path, suffix))? {
            return Err(
                "Legacy database sidecars appeared during the checkpointed WAL copy".into(),
            );
        }
    }
    Ok(())
}

fn copy_checkpointed_wal(source: &Path, snapshot: &Path, mut output: &File) -> Result<(), String> {
    require_no_sidecars(source)?;
    let before = files::digest_file(source)?;
    io::copy(&mut files::open(source)?, &mut output)
        .map_err(|_| "Cannot copy checkpointed WAL database")?;
    output
        .sync_all()
        .map_err(|_| "Cannot sync checkpointed WAL copy")?;
    require_no_sidecars(source)?;
    if files::digest_file(snapshot)? != before || files::digest_file(source)? != before {
        return Err("Checkpointed WAL database changed during the copy".into());
    }
    require_no_sidecars(source)
}

fn source_uri(path: &Path) -> Result<String, String> {
    let path = path.to_str().ok_or("Legacy database path must be UTF-8")?;
    #[cfg(windows)]
    let path = format!(
        "/{}",
        path.strip_prefix(r"\\?\")
            .unwrap_or(path)
            .replace('\\', "/")
    );
    let mut uri = String::from("file:");
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~/:".contains(&byte) {
            uri.push(char::from(byte));
        } else {
            let _ = write!(uri, "%{byte:02X}");
        }
    }
    uri.push_str("?mode=ro&readonly_shm=1");
    Ok(uri)
}

fn open_reader(path: &Path, extra_flags: OpenFlags) -> Result<Connection, String> {
    let db = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW | extra_flags,
    )
    .map_err(sqlite_error)?;
    db.busy_timeout(Duration::from_secs(5))
        .map_err(sqlite_error)?;
    db.execute_batch("PRAGMA query_only=ON; PRAGMA trusted_schema=OFF; BEGIN;")
        .map_err(sqlite_error)?;
    Ok(db)
}

fn sqlite_error(error: rusqlite::Error) -> String {
    // SQLite errors may contain arbitrary input data. Only expose its
    // numeric extended status when available.
    if let rusqlite::Error::SqliteFailure(details, _) = error {
        return format!(
            "Cannot read or verify migration database (SQLite {})",
            details.extended_code
        );
    }
    "Cannot read or verify migration database fields".into()
}

pub(super) fn inspect(source: &Path) -> Result<Value, String> {
    let Some(source) = open_source(source)? else {
        return Ok(
            json!({"status":"NOT_FOUND","reason":"The explicit legacy database was not found"}),
        );
    };
    match source.db {
        Some(db) => Ok(inventory(&db, source.version)?.report(source.version)),
        None => Ok(json!({
            "status":"BLOCKED", "schema":source.version,
            "reason":"Checkpointed WAL database has no sidecars; use export to verify a private copy",
        })),
    }
}

pub(super) fn export(root: &Path, source: &Path, destination: &Path) -> Result<Value, String> {
    let Some(source) = open_source(source)? else {
        return Ok(
            json!({"status":"NOT_FOUND","reason":"The explicit legacy database was not found"}),
        );
    };
    let destination = files::destination(root, destination)?;
    files::private_dir(&destination)?;
    let snapshot = destination.join(SNAPSHOT);
    let result = write_archive(&source, &snapshot, &destination);
    if result.is_err() {
        let _ = files::publish(
            &destination.join("incomplete.json"),
            b"{\"status\":\"INCOMPLETE\"}\n",
        );
    }
    result
}

fn write_archive(source: &Source, snapshot: &Path, destination: &Path) -> Result<Value, String> {
    let snapshot_file = files::new_file(snapshot)?;
    if source.db.is_none() {
        copy_checkpointed_wal(&source.path, snapshot, &snapshot_file)?;
    }
    let mut copy = Connection::open_with_flags(
        snapshot,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .map_err(sqlite_error)?;
    if let Some(source) = &source.db {
        let backup = Backup::new(source, &mut copy).map_err(sqlite_error)?;
        let deadline = Instant::now() + Duration::from_secs(120);
        loop {
            match backup.step(256).map_err(sqlite_error)? {
                StepResult::Done => break,
                StepResult::More => {}
                _ => std::thread::sleep(Duration::from_millis(10)),
            }
            if Instant::now() >= deadline {
                return Err("Legacy SQLite backup did not finish before its deadline".into());
            }
        }
    }
    let journal: String = copy
        .query_row("PRAGMA journal_mode=DELETE", [], |row| row.get(0))
        .map_err(sqlite_error)?;
    if journal != "delete" {
        return Err("Migration snapshot could not be made standalone".into());
    }
    // SQLite omits CHECK expressions when it parses a read-only database.
    // Check the privately owned copy with full schema checks and no writes.
    copy.execute_batch("PRAGMA query_only=ON; PRAGMA trusted_schema=OFF; BEGIN;")
        .map_err(sqlite_error)?;
    let version = check_history(&copy)?;
    let inventory = inventory(&copy, version)?;
    let export_path = destination.join(EXPORT);
    let mut export_file = files::new_file(&export_path)?;
    let export_sha256 = rows(&copy, version, &mut export_file)?;
    drop(copy);
    snapshot_file
        .sync_all()
        .map_err(|_| "Cannot sync the migration snapshot")?;
    drop(snapshot_file);
    if !check_files(snapshot, true)? {
        return Err("Migration snapshot was lost".into());
    }
    export_file
        .sync_all()
        .map_err(|_| "Cannot sync the migration export")?;
    let manifest = Manifest {
        format: 1,
        source: source.path.clone(),
        schema: version,
        snapshot_sha256: files::digest_file(snapshot)?,
        export_sha256,
        inventory,
    };
    let bytes =
        serde_json::to_vec_pretty(&manifest).map_err(|_| "Cannot encode migration receipt")?;
    if bytes.len() as u64 > MANIFEST_LIMIT {
        return Err("Migration findings exceed the receipt budget; snapshot retained".into());
    }
    files::publish(&destination.join(MANIFEST), &bytes)?;
    Ok(manifest.report(&sha256(&bytes)))
}

pub(super) struct Archive {
    pub(super) connection: Connection,
    pub(super) schema: usize,
    pub(super) receipt_sha256: String,
    pub(super) snapshot_sha256: String,
    pub(super) export_sha256: String,
    pub(super) issues: Vec<documents::Finding>,
    directory: PathBuf,
    snapshot: PathBuf,
}

impl Archive {
    pub(super) fn ensure_unchanged(&self) -> Result<(), String> {
        if !check_files(&self.snapshot, true)?
            || files::digest_file(&self.snapshot)? != self.snapshot_sha256
            || files::digest_file(&self.directory.join(EXPORT))? != self.export_sha256
            || files::digest_file(&self.directory.join(MANIFEST))? != self.receipt_sha256
        {
            return Err("Migration archive changed while it was being read".into());
        }
        Ok(())
    }
}

pub(super) fn open(path: &Path) -> Result<(Archive, Value), String> {
    let path = files::directory(path)?;
    let snapshot = path.join(SNAPSHOT);
    let manifest_path = path.join(MANIFEST);
    let bytes = files::read(&manifest_path, MANIFEST_LIMIT)?;
    let manifest: Manifest =
        serde_json::from_slice(&bytes).map_err(|_| "Invalid migration archive receipt")?;
    if manifest.format != 1 || !check_files(&snapshot, true)? {
        return Err("Unknown archive format, legacy schema, or missing snapshot".into());
    }
    let mut header = [0; 20];
    files::open(&snapshot)?
        .read_exact(&mut header)
        .map_err(|_| "Cannot read the archived SQLite header")?;
    if &header[..16] != b"SQLite format 3\0" || header[18..20] != [1, 1] {
        return Err("Archive requires a standalone SQLite snapshot".into());
    }
    if files::digest_file(&snapshot)? != manifest.snapshot_sha256
        || files::digest_file(&path.join(EXPORT))? != manifest.export_sha256
    {
        return Err("Migration archive digests changed".into());
    }
    let db = open_reader(&snapshot, OpenFlags::empty())?;
    let version = check_history(&db)?;
    if version != manifest.schema
        || inventory(&db, version)? != manifest.inventory
        || rows(&db, version, &mut io::sink())? != manifest.export_sha256
    {
        return Err("Migration archive does not reproduce its manifest".into());
    }
    let receipt_sha256 = sha256(&bytes);
    let report = manifest.report(&receipt_sha256);
    let archive = Archive {
        connection: db,
        schema: version,
        receipt_sha256,
        snapshot_sha256: manifest.snapshot_sha256,
        export_sha256: manifest.export_sha256,
        issues: manifest.inventory.findings,
        directory: path,
        snapshot,
    };
    archive.ensure_unchanged()?;
    Ok((archive, report))
}

fn check_history(db: &Connection) -> Result<usize, String> {
    let version: usize = db
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(sqlite_error)?;
    if !(1..=MIGRATIONS.len()).contains(&version) {
        return Err(format!(
            "Unsupported legacy SQLite schema revision {version}"
        ));
    }
    let mut statement = db
        .prepare("SELECT revision, name, checksum FROM schema_migrations ORDER BY revision")
        .map_err(sqlite_error)?;
    let mut rows = statement.query([]).map_err(sqlite_error)?;
    for (index, (name, sql)) in MIGRATIONS.iter().take(version).enumerate() {
        let row = rows
            .next()
            .map_err(sqlite_error)?
            .ok_or("Incomplete legacy SQLite migration history")?;
        let revision: usize = row.get(0).map_err(sqlite_error)?;
        let recorded_name: String = row.get(1).map_err(sqlite_error)?;
        let checksum: String = row.get(2).map_err(sqlite_error)?;
        if revision != index + 1
            || recorded_name != *name
            || (checksum != sha256(sql.as_bytes())
                && checksum != sha256(sql.replace("\r\n", "\n").as_bytes())
                && !(*name == "007_governance.sql" && checksum == KNOWN_WINDOWS_007))
        {
            return Err("Legacy SQLite migration history checksum mismatch".into());
        }
    }
    if rows.next().map_err(sqlite_error)?.is_some() {
        return Err("Unexpected legacy SQLite migration history entries".into());
    }
    Ok(version)
}

type Schema = Vec<(String, String, Option<String>)>;

fn schema(db: &Connection) -> Result<Schema, String> {
    let mut statement = db
        .prepare("SELECT type,name,sql FROM sqlite_schema ORDER BY type,name")
        .map_err(sqlite_error)?;
    statement
        .query_map([], |row| {
            let sql: Option<String> = row.get(2)?;
            Ok((
                row.get(0)?,
                row.get(1)?,
                sql.map(|value| value.replace("\r\n", "\n")),
            ))
        })
        .map_err(sqlite_error)?
        .collect::<Result<_, _>>()
        .map_err(sqlite_error)
}

fn inventory(db: &Connection, version: usize) -> Result<Inventory, String> {
    let reference = Connection::open_in_memory().map_err(sqlite_error)?;
    reference
        .execute_batch(HISTORY_TABLE)
        .map_err(sqlite_error)?;
    for (_, sql) in &MIGRATIONS[..version] {
        reference.execute_batch(sql).map_err(sqlite_error)?;
    }
    let expected = schema(&reference)?;
    if schema(db)? != expected {
        return Err("Legacy SQLite schema differs from the retained migrations".into());
    }
    let integrity: String = db
        .query_row("PRAGMA integrity_check(1)", [], |row| row.get(0))
        .map_err(sqlite_error)?;
    if integrity != "ok" {
        return Err("Legacy SQLite integrity check failed".into());
    }
    let mut statement = db
        .prepare("PRAGMA foreign_key_check")
        .map_err(sqlite_error)?;
    if statement
        .query([])
        .map_err(sqlite_error)?
        .next()
        .map_err(sqlite_error)?
        .is_some()
    {
        return Err("Legacy SQLite foreign key check failed".into());
    }
    let mut counts = BTreeMap::new();
    for (kind, table, _) in expected {
        if kind == "table" && !table.starts_with("sqlite_") {
            // The name is from the embedded, known schema.
            let count = db
                .query_row(&format!("SELECT count(*) FROM \"{table}\""), [], |row| {
                    row.get(0)
                })
                .map_err(sqlite_error)?;
            counts.insert(table, count);
        }
    }
    if version < GOVERNANCE_SCHEMA {
        return Ok(Inventory {
            counts,
            drafts: 0,
            archived: 0,
            deleted: 0,
            findings: Vec::new(),
        });
    }
    let (drafts, archived, deleted) = db.query_row(
        "SELECT count(draft_json), count(CASE WHEN archived=1 THEN 1 END), count(deleted_at) FROM governance_assets",
        [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).map_err(sqlite_error)?;
    Ok(Inventory {
        counts,
        drafts,
        archived,
        deleted,
        findings: documents::findings(db)?,
    })
}

fn rows(db: &Connection, version: usize, output: &mut dyn Write) -> Result<String, String> {
    let mut hash = Sha256::new();
    let mut buffer = Vec::new();
    for (table, order) in EXPORT_TABLES {
        if version < GOVERNANCE_SCHEMA && table.starts_with("governance_") {
            continue;
        }
        let mut statement = db
            .prepare(&format!("SELECT * FROM {table} ORDER BY {order}"))
            .map_err(sqlite_error)?;
        let columns: Vec<_> = statement
            .column_names()
            .into_iter()
            .map(str::to_owned)
            .collect();
        let mut rows = statement.query([]).map_err(sqlite_error)?;
        while let Some(row) = rows.next().map_err(sqlite_error)? {
            let mut fields = BTreeMap::new();
            for (index, name) in columns.iter().enumerate() {
                let value = match row.get_ref(index).map_err(sqlite_error)? {
                    ValueRef::Null => Value::Null,
                    ValueRef::Integer(value) => json!(value),
                    ValueRef::Real(value) => {
                        json!({"real_bits": format!("{:016x}", value.to_bits())})
                    }
                    ValueRef::Text(bytes) => match std::str::from_utf8(bytes) {
                        Ok(text) => json!(text),
                        Err(_) => json!({"text_hex": hex(bytes)}),
                    },
                    ValueRef::Blob(bytes) => json!({"blob_hex": hex(bytes)}),
                };
                fields.insert(name, value);
            }
            buffer.clear();
            serde_json::to_writer(&mut buffer, &(table, fields))
                .map_err(|_| "Cannot encode governance archive row")?;
            buffer.push(b'\n');
            output
                .write_all(&buffer)
                .map_err(|_| "Cannot write governance archive row")?;
            hash.update(&buffer);
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(result, "{byte:02x}");
    }
    result
}
