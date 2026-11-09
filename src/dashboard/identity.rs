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

const LEGACY_FILE: &str = "cgacp.sqlite3";
const STAGED: &str = ".identity-candidate.sqlite3";
const STARTED: &str = "identity-prepared";
const RECEIPT: &str = "identity-verified.json";
const COMPLETE: &str = "identity-activated";
const SELECTED: &str = "identity-current-selected";
const LOCK: &str = "identity.lock";

#[derive(Debug, PartialEq)]
enum State {
    Ready,
    Available,
    Resume,
    Conflict,
}

impl State {
    fn wire(&self) -> &'static str {
        match self {
            Self::Ready => "READY",
            Self::Available => "MIGRATION_AVAILABLE",
            Self::Resume => "RESUME",
            Self::Conflict => "CONFLICT",
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    source: PathBuf,
    digest: String,
    backup: String,
}

pub struct Locations {
    data_dir: PathBuf,
    active: PathBuf,
    legacy: PathBuf,
}

impl Locations {
    pub fn new(data_dir: PathBuf, use_default: bool) -> Self {
        let legacy = if use_default {
            data_dir.with_file_name("CGACP").join(LEGACY_FILE)
        } else {
            data_dir.join(LEGACY_FILE)
        };
        Self {
            active: data_dir.join(DATABASE_FILE),
            data_dir,
            legacy,
        }
    }

    pub fn preview(&self) -> Result<Value, AppError> {
        let state = self.state()?;
        Ok(json!({
            "status":state.wire(),
            "source":self.legacy,
            "destination":self.active
        }))
    }

    pub fn open(&self) -> Result<Database, AppError> {
        let _lock = self.lock()?;
        match self.state()? {
            State::Ready => Database::open(&self.data_dir),
            State::Conflict => Err(conflict()),
            State::Available | State::Resume => Err(AppError::new(
                "IDENTITY_MIGRATION_REQUIRED",
                "Review the local database upgrade in OPENRULES Desktop before opening it.",
            )),
        }
    }

    pub fn migrate(&self) -> Result<Value, AppError> {
        let _lock = self.lock()?;
        match self.state()? {
            State::Ready => return self.preview(),
            State::Conflict => return Err(conflict()),
            State::Resume if ordinary_file(&self.active)? => {
                self.complete()?;
                return self.preview();
            }
            State::Available | State::Resume => {}
        }

        // The legacy writer used WAL. Exclusive locking must be acquired before
        // reading it, so a live reader/writer cannot continue through cutover.
        check_database_files(&self.legacy)?;
        let source = Connection::open_with_flags(
            &self.legacy,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        source.busy_timeout(Duration::from_millis(800))?;
        source.pragma_update(None, "locking_mode", "EXCLUSIVE")?;
        source.execute_batch("BEGIN EXCLUSIVE; COMMIT;")?;
        verify(&source)?;
        verify_history(&source)?;
        if !ordinary_file(&self.data_dir.join(STARTED))? {
            self.write_marker(STARTED, b"migration-prepared\n")?;
        }

        let staged = self.data_dir.join(STAGED);
        clear_file(&staged)?;
        for extension in ["-wal", "-shm", "-journal"] {
            clear_file(&self.data_dir.join(format!("{STAGED}{extension}")))?;
        }
        clear_file(&self.data_dir.join(RECEIPT))?;

        let backup = Database::backup_from(&source, &self.data_dir)?;
        let backup_name = backup["backup"].as_str().ok_or_else(journal_error)?;
        super::validate_backup_name(backup_name)?;
        let backup_path = self.data_dir.join("backups").join(backup_name);
        ordinary_file(&backup_path)?;
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

        let digest = file_digest(&staged)?;
        let candidate = Connection::open_with_flags(
            &staged,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        verify(&candidate)?;
        verify_history(&candidate)?;
        candidate.close().map_err(|(_, error)| error)?;

        let receipt = serde_json::to_vec(&Receipt {
            source: self.legacy.clone(),
            digest,
            backup: backup_name.to_owned(),
        })
        .map_err(|_| journal_error())?;
        self.write_marker(RECEIPT, &receipt)?;

        // Install only after the backup and the journal are durable.
        if check_database_files(&self.active)? {
            return Err(conflict());
        }
        publish(&staged, &self.active)?;
        self.complete()?;
        self.preview()
    }

    pub fn use_current(&self) -> Result<Value, AppError> {
        let _lock = self.lock()?;
        if self.state()? != State::Conflict {
            return Err(conflict());
        }
        let active = Connection::open_with_flags(
            &self.active,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        verify(&active)?;
        verify_history(&active)?;
        self.write_marker(SELECTED, b"retain-current\n")?;
        self.preview()
    }

    fn state(&self) -> Result<State, AppError> {
        check_ancestors(&self.data_dir, true)?;
        let active = check_database_files(&self.active)?;
        let legacy = check_database_files(&self.legacy)?;
        let complete = ordinary_file(&self.data_dir.join(COMPLETE))?;
        let selected = ordinary_file(&self.data_dir.join(SELECTED))?;
        let receipt = ordinary_file(&self.data_dir.join(RECEIPT))?;
        let started = ordinary_file(&self.data_dir.join(STARTED))?;
        if complete || selected {
            // A missing active file after migration is a recovery incident, not a fresh install.
            if !active || (complete && !receipt) {
                return Err(journal_error());
            }
            return Ok(State::Ready);
        }
        if started && !legacy && !active {
            return Err(journal_error());
        }
        Ok(match (active, legacy, receipt) {
            (_, _, true) => State::Resume,
            (true, true, false) => State::Conflict,
            (false, true, false) => State::Available,
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
                "Another OPENRULES process is opening or upgrading this data directory.",
            )
        })?;
        Ok(file)
    }

    fn complete(&self) -> Result<(), AppError> {
        let path = self.data_dir.join(RECEIPT);
        let mut bytes = Vec::new();
        File::open(path)
            .map_err(storage_error)?
            .take(64 * 1024)
            .read_to_end(&mut bytes)
            .map_err(storage_error)?;
        let receipt: Receipt = serde_json::from_slice(&bytes).map_err(|_| journal_error())?;
        super::validate_backup_name(&receipt.backup)?;
        let backup = self.data_dir.join("backups").join(receipt.backup);
        if receipt.source != self.legacy
            || !ordinary_file(&backup)?
            || file_digest(&self.active)? != receipt.digest
            || file_digest(&backup)? != receipt.digest
        {
            return Err(journal_error());
        }
        self.write_marker(COMPLETE, b"migration-activated\n")
    }

    fn write_marker(&self, name: &str, data: &[u8]) -> Result<(), AppError> {
        let temp = self.data_dir.join(format!(".{name}.pending"));
        clear_file(&temp)?;
        let mut file = private_options()
            .create_new(true)
            .open(&temp)
            .map_err(storage_error)?;
        file.write_all(data).map_err(storage_error)?;
        file.sync_all().map_err(storage_error)?;
        drop(file);
        publish(&temp, &self.data_dir.join(name))
    }
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
    let (source, destination) = (wide(source), wide(destination));
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

fn storage_error(_: io::Error) -> AppError {
    AppError::new(
        "STORAGE_UNAVAILABLE",
        "Could not access the local migration files.",
    )
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
