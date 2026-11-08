//! SQLite storage for the local operator service.
use rusqlite::{
    Connection, OpenFlags, TransactionBehavior,
    backup::{Backup, StepResult},
    params,
};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fmt, fs,
    path::{Component, Path, PathBuf},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const DATABASE_FILE: &str = "cgacp.sqlite3";
const MIGRATIONS: &[(&str, &str)] = &[
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
];

#[derive(Debug)]
pub struct AppError {
    pub code: &'static str,
    pub message: String,
}

impl AppError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        if let rusqlite::Error::SqliteFailure(detail, _) = &error {
            return match detail.extended_code & 0xff {
                5 | 6 => Self::new(
                    "STORAGE_BUSY",
                    "Local storage is busy. Retry the operation.",
                ),
                19 => Self::new(
                    "CONFLICT",
                    "The requested change conflicts with existing records.",
                ),
                11 | 26 => Self::new(
                    "INTEGRITY",
                    "Local storage could not be read as a valid database.",
                ),
                13 => Self::new(
                    "STORAGE_FULL",
                    "There is insufficient disk space for the transaction.",
                ),
                3 | 8 | 23 => {
                    Self::new("PERMISSION_DENIED", "Local storage denied this operation.")
                }
                _ => Self::new(
                    "STORAGE_UNAVAILABLE",
                    "The local database operation failed.",
                ),
            };
        }
        match error {
            rusqlite::Error::QueryReturnedNoRows => {
                Self::new("NOT_FOUND", "The requested record was not found.")
            }
            _ => Self::new(
                "STORAGE_UNAVAILABLE",
                "The local database operation failed.",
            ),
        }
    }
}

pub struct Database {
    pub(crate) conn: Connection,
    pub(crate) machine_id: String,
    pub(crate) preview_secret: Vec<u8>,
    data_dir: PathBuf,
}

impl Database {
    pub fn open(data_dir: &Path) -> Result<Self, AppError> {
        let data_dir = ensure_directory(data_dir)?;
        let path = data_dir.join(DATABASE_FILE);
        for suffix in ["", "-wal", "-shm", "-journal"] {
            let file = data_dir.join(format!("{DATABASE_FILE}{suffix}"));
            match fs::symlink_metadata(&file) {
                Ok(meta) if meta.is_file() && !redirect(&meta) => {}
                Ok(_) => {
                    return Err(AppError::new(
                        "INVALID_PATH",
                        "Database files must be ordinary files.",
                    ));
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => {
                    return Err(AppError::new(
                        "STORAGE_UNAVAILABLE",
                        "Cannot inspect local database files.",
                    ));
                }
            }
        }
        let mut conn = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        conn.busy_timeout(Duration::from_secs(2))?;
        conn.pragma_update(None, "foreign_keys", true)?;
        ensure_supported(&conn)?;
        let mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
        if !mode.eq_ignore_ascii_case("wal") {
            return Err(AppError::new(
                "STORAGE_UNAVAILABLE",
                "SQLite WAL is unavailable in this data directory.",
            ));
        }
        conn.pragma_update(None, "synchronous", "FULL")?;
        migrate(&mut conn, &data_dir, &path)?;
        let machine_id = conn.query_row(
            "SELECT value FROM catalog_settings WHERE key = 'machine_id'",
            [],
            |row| row.get(0),
        )?;
        let preview_secret = conn.query_row("SELECT randomblob(32)", [], |row| row.get(0))?;
        Ok(Self {
            conn,
            machine_id,
            preview_secret,
            data_dir,
        })
    }

    pub(super) fn backup(&self) -> Result<Value, AppError> {
        Self::backup_from(&self.conn, &self.data_dir)
    }

    fn backup_from(conn: &Connection, data_dir: &Path) -> Result<Value, AppError> {
        let dir = ensure_directory(&data_dir.join("backups"))?;
        let nonce: String =
            conn.query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))?;
        let name = format!("snapshot-{nonce}.sqlite3");
        let path = dir.join(&name);
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|_| {
                AppError::new("STORAGE_UNAVAILABLE", "Cannot reserve a local backup file.")
            })?;
        let result = (|| {
            let mut dest = Connection::open_with_flags(
                &path,
                OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
            )?;
            dest.busy_timeout(Duration::from_millis(250))?;
            {
                let backup = Backup::new(conn, &mut dest)?;
                let until = Instant::now() + Duration::from_secs(10);
                loop {
                    if Instant::now() >= until {
                        return Err(AppError::new(
                            "STORAGE_BUSY",
                            "The online backup exceeded the retry window.",
                        ));
                    }
                    match backup.step(128)? {
                        StepResult::Done => break,
                        StepResult::More => {}
                        StepResult::Busy | StepResult::Locked => {
                            thread::sleep(Duration::from_millis(20))
                        }
                        _ => {
                            return Err(AppError::new(
                                "STORAGE_UNAVAILABLE",
                                "Unknown SQLite backup state.",
                            ));
                        }
                    }
                }
            }
            dest.pragma_update(None, "journal_mode", "DELETE")?;
            let integrity = verify(&dest)?;
            Ok(json!({"backup":name,"path":path,"integrity":integrity}))
        })();
        if result.is_err() {
            let _ = fs::remove_file(path);
        }
        result
    }
}

pub fn default_data_dir() -> Result<PathBuf, AppError> {
    #[cfg(windows)]
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join("Library/Application Support"));
    #[cfg(all(not(windows), not(target_os = "macos")))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));
    Ok(base
        .ok_or_else(|| {
            AppError::new(
                "INVALID_PATH",
                "The OS local application data directory is unavailable.",
            )
        })?
        .join("CIRNETLAB/CGACP"))
}

pub fn dispatch(db: &mut Database, method: &str, params: Value) -> Result<Value, AppError> {
    match method {
        "storage.backup" => {
            let _: Empty = decode(params)?;
            db.backup()
        }
        "storage.verify" => {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Verify {
                backup: Option<String>,
            }
            let input: Verify = decode(params)?;
            if let Some(name) = input.backup {
                let hex = name
                    .strip_prefix("snapshot-")
                    .and_then(|name| name.strip_suffix(".sqlite3"));
                if !hex.is_some_and(|hex| {
                    hex.len() == 32 && hex.bytes().all(|b| b.is_ascii_hexdigit())
                }) {
                    return Err(AppError::new(
                        "INVALID_INPUT",
                        "Select a saved backup filename.",
                    ));
                }
                let path = db.data_dir.join("backups").join(name);
                check_ancestors(&path, false)?;
                let saved = Connection::open_with_flags(
                    path,
                    OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
                )?;
                verify(&saved)
            } else {
                verify(&db.conn)
            }
        }
        _ => Err(AppError::new(
            "UNKNOWN_METHOD",
            "Unknown storage operation.",
        )),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Empty {}

pub(crate) fn decode<T: DeserializeOwned>(params: Value) -> Result<T, AppError> {
    serde_json::from_value(params)
        .map_err(|_| AppError::new("INVALID_INPUT", "Request fields are invalid."))
}

pub(crate) fn now() -> Result<i64, AppError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| duration.as_secs().try_into().ok())
        .ok_or_else(|| {
            AppError::new(
                "INVALID_CLOCK",
                "The local clock is outside the supported range.",
            )
        })
}

fn ensure_supported(conn: &Connection) -> Result<i64, AppError> {
    let current: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if !(0..=MIGRATIONS.len() as i64).contains(&current) {
        return Err(AppError::new(
            "MIGRATION_REQUIRED",
            "This database requires a newer application.",
        ));
    }
    Ok(current)
}

fn migrate(conn: &mut Connection, data_dir: &Path, path: &Path) -> Result<(), AppError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let current = ensure_supported(&tx)?;
    if current == 0 {
        let objects: i64 = tx.query_row(
            "SELECT count(*) FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )?;
        if objects != 0 {
            return Err(AppError::new(
                "MIGRATION_REQUIRED",
                "Existing unversioned data requires reviewed import.",
            ));
        }
        tx.execute_batch(
            "CREATE TABLE schema_migrations (revision INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE, checksum TEXT NOT NULL, applied_at INTEGER NOT NULL) STRICT;",
        )?;
    } else {
        let count: i64 = tx.query_row("SELECT count(*) FROM schema_migrations", [], |row| {
            row.get(0)
        })?;
        if count != current {
            return Err(AppError::new(
                "INTEGRITY",
                "SQLite migration history has an unexpected number of entries.",
            ));
        }
    }
    for (index, (name, sql)) in MIGRATIONS.iter().enumerate() {
        let revision = (index + 1) as i64;
        let checksum = format!("{:x}", Sha256::digest(sql.as_bytes()));
        if revision <= current {
            let stored: (String, String) = tx.query_row(
                "SELECT name, checksum FROM schema_migrations WHERE revision = ?1",
                [revision],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            if stored != (name.to_string(), checksum) {
                return Err(AppError::new(
                    "INTEGRITY",
                    "SQLite migration history checksum mismatch.",
                ));
            }
        } else {
            if current > 0 && revision == current + 1 {
                // The write transaction prevents another runtime changing the old schema
                // between this snapshot and the upgrade. WAL allows a separate read connection.
                let source = Connection::open_with_flags(
                    path,
                    OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
                )?;
                Database::backup_from(&source, data_dir)?;
            }
            tx.execute_batch(sql)?;
            tx.execute(
                "INSERT INTO schema_migrations(revision, name, checksum, applied_at) VALUES (?1, ?2, ?3, ?4)",
                params![revision, name, checksum, now()?],
            )?;
            tx.pragma_update(None, "user_version", revision)?;
        }
    }
    tx.commit()?;
    Ok(())
}

fn verify(conn: &Connection) -> Result<Value, AppError> {
    let revision = ensure_supported(conn)?;
    let check: String = conn.query_row("PRAGMA integrity_check(1)", [], |row| row.get(0))?;
    let mut fk = conn.prepare("PRAGMA foreign_key_check")?;
    if check != "ok" || fk.query([])?.next()?.is_some() {
        return Err(AppError::new(
            "INTEGRITY",
            "SQLite integrity or foreign key check failed.",
        ));
    }
    let repositories: i64 =
        conn.query_row("SELECT count(*) FROM repositories", [], |row| row.get(0))?;
    let bindings: i64 = conn.query_row("SELECT count(*) FROM bindings", [], |row| row.get(0))?;
    Ok(
        json!({"status":"PASS","schema_revision":revision,"repositories":repositories,"bindings":bindings}),
    )
}

pub(crate) fn validate_local(path: &Path) -> Result<(), AppError> {
    if !path.is_absolute()
        || path
            .as_os_str()
            .to_str()
            .is_none_or(|s| s.chars().any(char::is_control))
    {
        return Err(AppError::new(
            "INVALID_PATH",
            "Select an absolute local path.",
        ));
    }
    #[cfg(windows)]
    {
        use std::{os::windows::ffi::OsStrExt, path::Prefix};
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetDriveTypeW(root: *const u16) -> u32;
        }
        let Some(Component::Prefix(prefix)) = path.components().next() else {
            return Err(AppError::new("INVALID_PATH", "Select a local drive path."));
        };
        let (Prefix::Disk(drive) | Prefix::VerbatimDisk(drive)) = prefix.kind() else {
            return Err(AppError::new(
                "INVALID_PATH",
                "Network and device paths are unsupported.",
            ));
        };
        if path.components().any(|component| {
            let Component::Normal(part) = component else {
                return false;
            };
            part.to_str().is_none_or(|part| {
                let base = part
                    .split('.')
                    .next()
                    .unwrap_or_default()
                    .to_ascii_uppercase();
                part.ends_with(['.', ' '])
                    || part.contains(['<', '>', ':', '"', '|', '?', '*'])
                    || matches!(
                        base.as_str(),
                        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
                    )
                    || base
                        .strip_prefix("COM")
                        .or_else(|| base.strip_prefix("LPT"))
                        .is_some_and(|digit| {
                            matches!(
                                digit,
                                "0" | "1"
                                    | "2"
                                    | "3"
                                    | "4"
                                    | "5"
                                    | "6"
                                    | "7"
                                    | "8"
                                    | "9"
                                    | "¹"
                                    | "²"
                                    | "³"
                            )
                        })
            })
        }) {
            return Err(AppError::new(
                "INVALID_PATH",
                "The selected path contains a Windows device name or ambiguous component.",
            ));
        }
        let root = std::ffi::OsStr::new(&format!("{}:\\", char::from(drive)))
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        // The fixed-length buffer is NUL-terminated and retained across the call.
        if !matches!(unsafe { GetDriveTypeW(root.as_ptr()) }, 2 | 3) {
            return Err(AppError::new(
                "INVALID_PATH",
                "The selected path is not on a local writable drive.",
            ));
        }
    }
    Ok(())
}

pub(crate) fn redirect(meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    meta.file_type().is_symlink()
}

pub(crate) fn check_ancestors(path: &Path, allow_missing: bool) -> Result<(), AppError> {
    validate_local(path)?;
    for component in path.ancestors() {
        match fs::symlink_metadata(component) {
            Ok(meta) if !redirect(&meta) => {}
            Ok(_) => {
                return Err(AppError::new(
                    "INVALID_PATH",
                    "A redirected path crossed the local storage boundary.",
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && allow_missing => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(AppError::new("MISSING", "The selected path is missing."));
            }
            Err(_) => {
                return Err(AppError::new(
                    "INACCESSIBLE",
                    "The selected path is not readable.",
                ));
            }
        }
    }
    Ok(())
}

fn ensure_directory(path: &Path) -> Result<PathBuf, AppError> {
    if path.components().any(|part| part == Component::ParentDir) {
        return Err(AppError::new(
            "INVALID_PATH",
            "The local data path contains traversal.",
        ));
    }
    check_ancestors(path, true)?;
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path).map_err(|_| {
        AppError::new(
            "STORAGE_UNAVAILABLE",
            "Cannot initialize the local data directory.",
        )
    })?;
    check_ancestors(path, false)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if fs::metadata(path)
            .map_err(|_| {
                AppError::new(
                    "STORAGE_UNAVAILABLE",
                    "Cannot inspect the local data directory.",
                )
            })?
            .mode()
            & 0o077
            != 0
        {
            return Err(AppError::new(
                "PERMISSION_DENIED",
                "The local data directory must be private to its owner.",
            ));
        }
    }
    path.canonicalize().map_err(|_| {
        AppError::new(
            "STORAGE_UNAVAILABLE",
            "Cannot resolve the local data directory.",
        )
    })
}
