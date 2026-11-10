//! Cutover from the previous Desktop database identity. Startup and migration
//! share a process lock. The retired database and its existing backups are
//! kept in place as recovery data.
use super::{
    AppError, Connection, DATABASE_FILE, Database, Duration, OpenFlags, Path, PathBuf, Sha256,
    check_ancestors, ensure_directory, ensure_supported, redirect, verify,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::Digest;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
};

const PREVIOUS_PRODUCT: &str = "OPENRULES";
const PREVIOUS_FILE: &str = "openrules.sqlite3";
const LEGACY_PRODUCT: &str = "CGACP";
const LEGACY_FILE: &str = "cgacp.sqlite3";
const JOURNAL: &str = "identity-cutover-v2";
const JOURNAL_REVISION: u8 = 2;
const STAGED: &str = "candidate.sqlite3";
const STARTED: &str = "prepared.json";
const RECEIPT: &str = "verified.json";
const COMPLETE: &str = "activated";
const SELECTED: &str = "current-selected";
const LOCK: &str = "identity-cutover.lock";

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
enum Generation {
    #[serde(rename = "OPENRULES")]
    Previous,
    #[serde(rename = "CGACP")]
    Legacy,
}

impl Generation {
    fn name(self) -> &'static str {
        match self {
            Self::Previous => PREVIOUS_PRODUCT,
            Self::Legacy => LEGACY_PRODUCT,
        }
    }
}

#[derive(Debug, PartialEq)]
enum State {
    Ready,
    Available(Generation),
    Resume(Generation),
    Conflict,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Intent {
    revision: u8,
    source_product: Generation,
    target_product: String,
    source: PathBuf,
    target: PathBuf,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    intent: Intent,
    snapshot_sha256: String,
    source_schema: i64,
    backup: String,
    // Previous receipts used one digest for the backup and the unmodified candidate.
    #[serde(default)]
    backup_sha256: Option<String>,
}

// Read-only compatibility with the previous cutover receipt.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PreviousReceipt {
    source: PathBuf,
    digest: String,
    backup: String,
}

pub struct Locations {
    data_dir: PathBuf,
    journal: PathBuf,
    active: PathBuf,
    previous: PathBuf,
    legacy: PathBuf,
}

impl Locations {
    pub fn new(data_dir: PathBuf, use_default: bool) -> Self {
        let source = |folder, file| {
            if use_default {
                data_dir.with_file_name(folder).join(file)
            } else {
                data_dir.join(file)
            }
        };
        Self {
            active: data_dir.join(DATABASE_FILE),
            journal: data_dir.join(JOURNAL),
            previous: source(PREVIOUS_PRODUCT, PREVIOUS_FILE),
            legacy: source(LEGACY_PRODUCT, LEGACY_FILE),
            data_dir,
        }
    }

    pub fn preview(&self) -> Result<Value, AppError> {
        let state = self.state()?;
        let (status, source) = match state {
            State::Ready => ("READY", None),
            State::Available(source) => ("MIGRATION_AVAILABLE", Some(source)),
            State::Resume(source) => ("RESUME", Some(source)),
            State::Conflict => ("CONFLICT", None),
        };
        let mut candidates = Vec::new();
        if state != State::Ready {
            for generation in [Generation::Previous, Generation::Legacy] {
                let path = self.source(generation);
                if check_database_files(path)? {
                    let bytes = |p| {
                        fs::metadata(p)
                            .map(|meta| meta.len())
                            .map_err(storage_error)
                    };
                    let mut wal = path.as_os_str().to_owned();
                    wal.push("-wal");
                    let wal = Path::new(&wal);
                    let wal_bytes = if ordinary_file(wal)? { bytes(wal)? } else { 0 };
                    candidates.push(json!({"product":generation.name(),"path":path,
                        "database_bytes":bytes(path)?,"wal_bytes":wal_bytes}));
                }
            }
        }
        Ok(json!({
            "status":status,
            "source":source.map(|source| self.source(source)),
            "source_product":source.map(Generation::name),
            "destination":self.active,
            "candidates":candidates,
            "can_use_current":state == State::Conflict && check_database_files(&self.active)?
        }))
    }

    pub fn open(&self) -> Result<Database, AppError> {
        let _lock = self.lock()?;
        match self.state()? {
            State::Ready => Database::open(&self.data_dir),
            State::Conflict => Err(conflict()),
            State::Available(_) | State::Resume(_) => Err(AppError::new(
                "IDENTITY_MIGRATION_REQUIRED",
                format!(
                    "Review the local database upgrade in {} before opening it.",
                    crate::application_identity::DISPLAY_NAME
                ),
            )),
        }
    }

    #[cfg(test)]
    pub fn migrate(&self) -> Result<Value, AppError> {
        self.migrate_checked(&std::sync::atomic::AtomicBool::new(false), |_| {})
    }

    pub fn migrate_checked(
        &self,
        cancelled: &std::sync::atomic::AtomicBool,
        progress: impl Fn(&str),
    ) -> Result<Value, AppError> {
        let check_cancel = || {
            if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
                Err(AppError::new(
                    "CANCELLED",
                    "Upgrade paused before activation. Reopen Desktop to resume.",
                ))
            } else {
                Ok(())
            }
        };
        let _lock = self.lock()?;
        let generation = match self.state()? {
            State::Ready => return self.preview(),
            State::Conflict => return Err(conflict()),
            State::Resume(source) if ordinary_file(&self.active)? => {
                progress("ACTIVATE");
                self.complete(source)?;
                return self.preview();
            }
            State::Available(source) | State::Resume(source) => source,
        };
        let intent = self.intent(generation);
        ensure_directory(&self.journal)?;
        check_cancel()?;
        progress("CHECK_SOURCE");
        // Fence the older writer too. A prior receipt proves ancestry, not
        // that the retired application stopped writing after its cutover.
        let _ancestor = if generation == Generation::Previous && check_database_files(&self.legacy)?
        {
            let (path, digest) = self.previous_backup()?.ok_or_else(conflict)?;
            let ancestor = lock_source(&self.legacy)?;
            let historical = Connection::open_with_flags(
                &path,
                OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
            )?;
            if file_digest(&path)? != digest
                || logical_fingerprint(&ancestor)? != logical_fingerprint(&historical)?
            {
                return Err(conflict());
            }
            Some(ancestor)
        } else {
            None
        };
        let source = lock_source(&intent.source)?;
        verify(&source)?;
        verify_history(&source)?;
        let source_schema = ensure_supported(&source)?;
        if !ordinary_file(&self.journal.join(STARTED))? {
            self.write_json(STARTED, &intent)?;
        }

        // Rebuild any unpublished stage from the exclusively locked source.
        // Prior backups stay available for recovery.
        check_cancel()?;
        progress("BACKUP");
        let staged = self.journal.join(STAGED);
        clear_file(&staged)?;
        for extension in ["-wal", "-shm", "-journal"] {
            clear_file(&self.journal.join(format!("{STAGED}{extension}")))?;
        }
        clear_file(&self.journal.join(RECEIPT))?;
        let backup = Database::backup_from(&source, &self.data_dir)?;
        let backup_name = backup["backup"].as_str().ok_or_else(journal_error)?;
        super::validate_backup_name(backup_name)?;
        let backup_path = self.data_dir.join("backups").join(backup_name);
        if !ordinary_file(&backup_path)? {
            return Err(journal_error());
        }
        check_cancel()?;
        progress("VERIFY");
        let mut backup_file = private_options()
            .open(&backup_path)
            .map_err(storage_error)?;
        backup_file.sync_all().map_err(storage_error)?;
        let mut output = private_options()
            .create_new(true)
            .open(&staged)
            .map_err(storage_error)?;
        io::copy(&mut backup_file, &mut output).map_err(storage_error)?;
        output.sync_all().map_err(storage_error)?;
        drop(output);
        let backup_sha256 = file_digest(&staged)?;
        let mut candidate = Connection::open_with_flags(
            &staged,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        candidate.pragma_update(None, "foreign_keys", true)?;
        candidate.pragma_update(None, "synchronous", "FULL")?;
        // The raw backup is already durable. Upgrade the copy before publishing it.
        super::migrate(&mut candidate, None).map_err(|mut error| {
            error.message = format!("Could not upgrade the staged database. {}", error.message);
            error
        })?;
        verify(&candidate)?;
        verify_history(&candidate)?;
        candidate.close().map_err(|(_, error)| error)?;
        private_options()
            .open(&staged)
            .and_then(|file| file.sync_all())
            .map_err(storage_error)?;
        let snapshot_sha256 = file_digest(&staged)?;
        self.write_json(
            RECEIPT,
            &Receipt {
                intent,
                snapshot_sha256,
                source_schema,
                backup: backup_name.to_owned(),
                backup_sha256: Some(backup_sha256),
            },
        )?;
        check_cancel()?;
        progress("ACTIVATE");
        if check_database_files(&self.active)? {
            return Err(conflict());
        }
        publish(&staged, &self.active)?;
        self.complete(generation)?;
        self.preview()
    }

    pub fn use_current(&self) -> Result<Value, AppError> {
        let _lock = self.lock()?;
        if self.state()? != State::Conflict || !check_database_files(&self.active)? {
            return Err(conflict());
        }
        let active = Connection::open_with_flags(
            &self.active,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        verify(&active)?;
        verify_history(&active)?;
        ensure_directory(&self.journal)?;
        self.write_marker(SELECTED, b"retain-current\n")?;
        self.preview()
    }

    fn source(&self, generation: Generation) -> &Path {
        match generation {
            Generation::Previous => &self.previous,
            Generation::Legacy => &self.legacy,
        }
    }

    fn intent(&self, generation: Generation) -> Intent {
        Intent {
            revision: JOURNAL_REVISION,
            source_product: generation,
            target_product: crate::application_identity::PRODUCT_SYMBOL.to_owned(),
            source: self.source(generation).to_owned(),
            target: self.active.clone(),
        }
    }

    fn previous_backup(&self) -> Result<Option<(PathBuf, String)>, AppError> {
        let directory = self.previous.parent().ok_or_else(journal_error)?;
        let completed = directory.join("identity-activated");
        let receipt = directory.join("identity-verified.json");
        if !ordinary_file(&completed)? || !ordinary_file(&receipt)? {
            return Ok(None);
        }
        let old: PreviousReceipt = read_json(&receipt)?;
        super::validate_backup_name(&old.backup)?;
        let backup = directory.join("backups").join(&old.backup);
        if old.source != self.legacy || !valid_digest(&old.digest) || !ordinary_file(&backup)? {
            return Ok(None);
        }
        Ok(Some((backup, old.digest)))
    }

    fn state(&self) -> Result<State, AppError> {
        check_ancestors(&self.data_dir, true)?;
        let active = check_database_files(&self.active)?;
        if ordinary_file(&self.journal.join(SELECTED))? {
            return if active {
                Ok(State::Ready)
            } else {
                Err(journal_error())
            };
        }
        let complete = ordinary_file(&self.journal.join(COMPLETE))?;
        let started = ordinary_file(&self.journal.join(STARTED))?;
        let receipt = ordinary_file(&self.journal.join(RECEIPT))?;
        if complete || started || receipt {
            if !started || (complete && (!active || !receipt)) {
                return Err(journal_error());
            }
            let intent: Intent = read_json(&self.journal.join(STARTED))?;
            let generation = intent.source_product;
            if intent != self.intent(generation) {
                return Err(journal_error());
            }
            if receipt {
                self.receipt(generation)?;
            }
            if complete {
                return Ok(State::Ready);
            }
            if active && !receipt {
                return Ok(State::Conflict);
            }
            if !active && !check_database_files(self.source(generation))? {
                return Err(journal_error());
            }
            // A newly discovered competing predecessor must never be ignored.
            if !active && generation == Generation::Legacy && check_database_files(&self.previous)?
            {
                return Ok(State::Conflict);
            }
            return Ok(State::Resume(generation));
        }

        let previous = check_database_files(&self.previous)?;
        let legacy = check_database_files(&self.legacy)?;
        let previous_directory = self.previous.parent().ok_or_else(journal_error)?;
        if !previous
            && (ordinary_file(&previous_directory.join("identity-activated"))?
                || ordinary_file(&previous_directory.join("identity-current-selected"))?)
        {
            return Err(journal_error());
        }
        Ok(match (active, previous, legacy) {
            (true, true, _) | (true, _, true) => State::Conflict,
            (false, true, true) if self.previous_backup()?.is_none() => State::Conflict,
            (false, true, _) => State::Available(Generation::Previous),
            (false, false, true) => State::Available(Generation::Legacy),
            _ => State::Ready,
        })
    }

    fn lock(&self) -> Result<File, AppError> {
        let dir = ensure_directory(&self.data_dir)?;
        let path = dir.join(LOCK);
        ordinary_file(&path)?;
        let file = private_options()
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(storage_error)?;
        file.try_lock().map_err(|_| {
            AppError::new(
                "STORAGE_BUSY",
                "Another Desktop process is opening or upgrading this data directory.",
            )
        })?;
        Ok(file)
    }

    fn receipt(&self, generation: Generation) -> Result<Receipt, AppError> {
        let receipt: Receipt = read_json(&self.journal.join(RECEIPT))?;
        super::validate_backup_name(&receipt.backup)?;
        if receipt.intent != self.intent(generation)
            || !valid_digest(&receipt.snapshot_sha256)
            || receipt
                .backup_sha256
                .as_deref()
                .is_some_and(|digest| !valid_digest(digest))
            || !(1..=super::MIGRATIONS.len() as i64).contains(&receipt.source_schema)
        {
            return Err(journal_error());
        }
        Ok(receipt)
    }

    fn complete(&self, generation: Generation) -> Result<(), AppError> {
        let receipt = self.receipt(generation)?;
        let backup = self.data_dir.join("backups").join(receipt.backup);
        let backup_digest = receipt
            .backup_sha256
            .as_deref()
            .unwrap_or(&receipt.snapshot_sha256);
        if !ordinary_file(&backup)?
            || file_digest(&self.active)? != receipt.snapshot_sha256
            || file_digest(&backup)? != backup_digest
        {
            return Err(journal_error());
        }
        self.write_marker(COMPLETE, b"migration-activated\n")
    }

    fn write_json(&self, name: &str, record: &impl Serialize) -> Result<(), AppError> {
        let bytes = serde_json::to_vec(record).map_err(|_| journal_error())?;
        self.write_marker(name, &bytes)
    }

    fn write_marker(&self, name: &str, data: &[u8]) -> Result<(), AppError> {
        let temp = self.journal.join(format!(".{name}.pending"));
        clear_file(&temp)?;
        let mut file = private_options()
            .create_new(true)
            .open(&temp)
            .map_err(storage_error)?;
        file.write_all(data).map_err(storage_error)?;
        file.sync_all().map_err(storage_error)?;
        drop(file);
        publish(&temp, &self.journal.join(name))
    }
}

fn lock_source(path: &Path) -> Result<Connection, AppError> {
    check_database_files(path)?;
    let source = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )?;
    source.busy_timeout(Duration::from_millis(800))?;
    source.pragma_update(None, "locking_mode", "EXCLUSIVE")?;
    source.execute_batch("BEGIN EXCLUSIVE; COMMIT;")?;
    Ok(source)
}

fn logical_fingerprint(connection: &Connection) -> Result<String, AppError> {
    use rusqlite::types::ValueRef;
    use std::time::Instant;

    let deadline = Instant::now() + Duration::from_secs(10);
    let mut hash = Sha256::new();
    let mut update = |tag: u8, bytes: &[u8]| {
        hash.update([tag]);
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    };
    let mut tables = connection.prepare(
        "SELECT name, sql FROM sqlite_schema WHERE type='table' AND name NOT GLOB 'sqlite_*' ORDER BY name",
    )?;
    let mut list = tables.query([])?;
    while let Some(table) = list.next()? {
        let name: String = table.get(0)?;
        let schema: String = table.get(1)?;
        update(0, name.as_bytes());
        update(1, schema.as_bytes());
        let mut query = connection.prepare(&format!(
            "SELECT * FROM \"{}\" ORDER BY rowid",
            name.replace('"', "\"\""),
        ))?;
        let columns = query.column_count();
        let mut rows = query.query([])?;
        while let Some(row) = rows.next()? {
            if Instant::now() >= deadline {
                return Err(AppError::new(
                    "STORAGE_BUSY",
                    "The predecessor comparison exceeded its time budget.",
                ));
            }
            update(2, &[]);
            for index in 0..columns {
                match row.get_ref(index)? {
                    ValueRef::Null => update(3, &[]),
                    ValueRef::Integer(value) => update(4, &value.to_le_bytes()),
                    ValueRef::Real(value) => update(5, &value.to_le_bytes()),
                    ValueRef::Text(value) => update(6, value),
                    ValueRef::Blob(value) => update(7, value),
                }
            }
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, AppError> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(storage_error)?
        .take(64 * 1024)
        .read_to_end(&mut bytes)
        .map_err(storage_error)?;
    serde_json::from_slice(&bytes).map_err(|_| journal_error())
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn verify_history(connection: &Connection) -> Result<(), AppError> {
    let revision = ensure_supported(connection)?;
    if revision == 0 {
        return Err(journal_error());
    }
    let mut statement = connection
        .prepare("SELECT revision, name, checksum FROM schema_migrations ORDER BY revision")?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let history = rows.collect::<Result<Vec<_>, _>>()?;
    if history.len() != revision as usize
        || history.iter().zip(super::MIGRATIONS).enumerate().any(
            |(index, ((number, name, digest), (expected_name, sql)))| {
                *number != (index + 1) as i64
                    || name != expected_name
                    || *digest != format!("{:x}", Sha256::digest(sql.as_bytes()))
            },
        )
    {
        return Err(journal_error());
    }
    Ok(())
}

fn file_digest(path: &Path) -> Result<String, AppError> {
    let mut file = File::open(path).map_err(storage_error)?;
    let mut hash = Sha256::new();
    let mut bytes = vec![0; 64 * 1024];
    loop {
        let size = file.read(&mut bytes).map_err(storage_error)?;
        if size == 0 {
            return Ok(format!("{:x}", hash.finalize()));
        }
        hash.update(&bytes[..size]);
    }
}

fn check_database_files(path: &Path) -> Result<bool, AppError> {
    let exists = ordinary_file(path)?;
    for suffix in ["-wal", "-shm", "-journal"] {
        let mut filename = path.as_os_str().to_owned();
        filename.push(suffix);
        if ordinary_file(Path::new(&filename))? && !exists {
            return Err(journal_error());
        }
    }
    Ok(exists)
}

fn ordinary_file(path: &Path) -> Result<bool, AppError> {
    check_ancestors(path, true)?;
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_file() && !redirect(&meta) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        _ => Err(journal_error()),
    }
}

fn clear_file(path: &Path) -> Result<(), AppError> {
    if ordinary_file(path)? {
        fs::remove_file(path).map_err(storage_error)?;
    }
    Ok(())
}

fn private_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    options
}

#[cfg(unix)]
fn publish(source: &Path, destination: &Path) -> Result<(), AppError> {
    fs::hard_link(source, destination).map_err(storage_error)?;
    let directory = destination.parent().ok_or_else(journal_error)?;
    File::open(directory)
        .and_then(|dir| dir.sync_all())
        .map_err(storage_error)?;
    fs::remove_file(source).map_err(storage_error)?;
    Ok(())
}

#[cfg(windows)]
fn publish(source: &Path, destination: &Path) -> Result<(), AppError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{MOVEFILE_WRITE_THROUGH, MoveFileExW};
    let wide = |path: &Path| {
        path.as_os_str()
            .encode_wide()
            .chain([0])
            .collect::<Vec<_>>()
    };
    let source = fs::canonicalize(source).map_err(storage_error)?;
    let parent = destination.parent().ok_or_else(journal_error)?;
    let destination = fs::canonicalize(parent)
        .map_err(storage_error)?
        .join(destination.file_name().ok_or_else(journal_error)?);
    let (source, destination) = (wide(&source), wide(&destination));
    // Omitting MOVEFILE_REPLACE_EXISTING makes this atomic publication fail on collision.
    if unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_WRITE_THROUGH,
        )
    } == 0
    {
        return Err(storage_error(io::Error::last_os_error()));
    }
    Ok(())
}

fn storage_error(error: io::Error) -> AppError {
    AppError::storage_io(error, "Could not access the local migration files.")
}

fn journal_error() -> AppError {
    AppError::new(
        "IDENTITY_INTEGRITY",
        "The migration files require recovery. Existing data was retained.",
    )
}

fn conflict() -> AppError {
    AppError::new(
        "IDENTITY_CONFLICT",
        "The legacy and current databases both exist. Choose which data to open.",
    )
}

#[cfg(test)]
#[path = "identity_tests.rs"]
mod tests;
