//! SQLite adapter for storage inventory and operator maintenance.
use super::{
    AppError, DATABASE_FILE, Database, check_ancestors, now, redirect, validate_backup_name, verify,
};
use crate::operations::{Compact, JobControl};
use rusqlite::{Connection, OpenFlags};
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    path::Path,
    sync::Arc,
    time::Duration,
};

const INVENTORY_LIMIT: usize = 4096;
const BACKUP_LIST_LIMIT: usize = 32;

fn open(path: &Path, mode: OpenFlags, control: &Arc<JobControl>) -> Result<Connection, AppError> {
    check_ancestors(path, false)?;
    for suffix in ["-wal", "-shm", "-journal"] {
        let mut name = path.as_os_str().to_owned();
        name.push(suffix);
        let sidecar = std::path::PathBuf::from(name);
        match fs::symlink_metadata(sidecar) {
            Ok(meta) if meta.is_file() && !redirect(&meta) => {}
            Ok(_) => {
                return Err(AppError::new(
                    "INVALID_PATH",
                    "A database sidecar is redirected.",
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => {
                return Err(AppError::new(
                    "INACCESSIBLE",
                    "Cannot inspect database sidecars.",
                ));
            }
        }
    }
    control.check()?;
    let conn = Connection::open_with_flags(path, mode | OpenFlags::SQLITE_OPEN_NOFOLLOW)?;
    conn.busy_timeout(Duration::from_millis(200))?;
    let guard = Arc::clone(control);
    conn.progress_handler(1000, Some(move || guard.check().is_err()));
    Ok(conn)
}

pub(crate) fn online_backup(data: &Path, control: &Arc<JobControl>) -> Result<Value, AppError> {
    let conn = open(
        &data.join(DATABASE_FILE),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
        control,
    )?;
    Database::controlled_backup(&conn, data, Some(control))
}

pub(crate) fn verify_job(
    data: &Path,
    backup: Option<&str>,
    control: &Arc<JobControl>,
) -> Result<Value, AppError> {
    let path = if let Some(name) = backup {
        validate_backup_name(name)?;
        data.join("backups").join(name)
    } else {
        data.join(DATABASE_FILE)
    };
    control.update("VERIFY_SQLITE", 0, None);
    verify(&open(&path, OpenFlags::SQLITE_OPEN_READ_ONLY, control)?)
}

struct Sizes {
    seen: HashSet<(u64, u64)>,
    skipped: u64,
}

impl Sizes {
    fn record(&mut self, path: &Path) -> Option<u64> {
        match fs::symlink_metadata(path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
            Ok(meta) if meta.is_file() && !redirect(&meta) => {}
            _ => {
                self.skipped += 1;
                return None;
            }
        }
        let result = (|| {
            check_ancestors(path, false).ok()?;
            let mut options = OpenOptions::new();
            options.read(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
            }
            #[cfg(windows)]
            {
                use std::os::windows::fs::OpenOptionsExt;
                use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
                options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
            }
            let file = options.open(path).ok()?;
            let meta = file.metadata().ok()?;
            if !meta.is_file() || redirect(&meta) {
                return None;
            }
            Some((file_id(&file, &meta)?, meta.len()))
        })();
        match result {
            Some((id, bytes)) if self.seen.insert(id) => Some(bytes),
            Some(_) => None,
            None => {
                self.skipped += 1;
                None
            }
        }
    }
}

#[cfg(unix)]
fn file_id(_: &File, meta: &fs::Metadata) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    Some((meta.dev(), meta.ino()))
}

#[cfg(windows)]
fn file_id(file: &File, _: &fs::Metadata) -> Option<(u64, u64)> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, GetFileInformationByHandle,
    };
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    // The File owns a valid handle and info is writable for the duration of the call.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return None;
    }
    Some((
        u64::from(info.dwVolumeSerialNumber),
        (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
    ))
}

fn pages(conn: &Connection) -> Result<(u64, u64, u64), AppError> {
    Ok((
        conn.pragma_query_value(None, "page_size", |row| row.get(0))?,
        conn.pragma_query_value(None, "page_count", |row| row.get(0))?,
        conn.pragma_query_value(None, "freelist_count", |row| row.get(0))?,
    ))
}

fn chat_sizes(conn: &Connection, state: &str) -> Result<(u64, u64), AppError> {
    let count = conn.query_row(
        "SELECT count(*) FROM conversations WHERE state=?1",
        [state],
        |row| row.get(0),
    )?;
    let bytes = conn.query_row(
        "SELECT coalesce(sum(octet_length(m.content)),0) FROM messages m
         JOIN conversations c ON c.id=m.conversation_id WHERE c.state=?1",
        [state],
        |row| row.get(0),
    )?;
    Ok((count, bytes))
}

pub(crate) fn chat_usage(data: &Path, control: &Arc<JobControl>) -> Result<Value, AppError> {
    control.update("MEASURE_CHATS", 0, None);
    let mut conn = open(
        &data.join(DATABASE_FILE),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
        control,
    )?;
    let transaction = conn.transaction()?;
    let (active_chats, active_bytes) = chat_sizes(&transaction, "ACTIVE")?;
    let (archived_chats, archived_bytes) = chat_sizes(&transaction, "ARCHIVED")?;
    transaction.commit()?;
    Ok(json!({
        "measured_at": now()?,
        "active": {"conversations":active_chats,"message_bytes":active_bytes},
        "archived": {"conversations":archived_chats,"message_bytes":archived_bytes},
    }))
}

pub(crate) fn inventory(data: &Path, control: &Arc<JobControl>) -> Result<Value, AppError> {
    check_ancestors(data, false)?;
    control.update("MEASURE_FILES", 0, None);
    let mut sizes = Sizes {
        seen: HashSet::new(),
        skipped: 0,
    };
    let [database, wal, shm, journal] = ["", "-wal", "-shm", "-journal"].map(|suffix| {
        sizes
            .record(&data.join(format!("{DATABASE_FILE}{suffix}")))
            .unwrap_or(0)
    });
    let mut backup_bytes = 0u64;
    let mut pending_bytes = 0u64;
    let mut backup_count = 0;
    let mut backups = Vec::new();
    let dir = data.join("backups");
    if let Ok(metadata) = fs::symlink_metadata(&dir) {
        if metadata.is_dir() && !redirect(&metadata) && check_ancestors(&dir, false).is_ok() {
            match fs::read_dir(&dir) {
                Ok(entries) => {
                    for (index, entry) in entries.enumerate() {
                        control.check()?;
                        if index == INVENTORY_LIMIT {
                            sizes.skipped += 1;
                            break;
                        }
                        if index % 32 == 0 {
                            control.update("MEASURE_FILES", (index + 4) as u64, None);
                        }
                        let Ok(entry) = entry else {
                            sizes.skipped += 1;
                            continue;
                        };
                        let name = entry.file_name();
                        let Some(name) = name.to_str() else {
                            continue;
                        };
                        if validate_backup_name(name).is_ok() {
                            if let Some(bytes) = sizes.record(&entry.path()) {
                                backup_bytes = backup_bytes.saturating_add(bytes);
                                backup_count += 1;
                                if backups.len() < BACKUP_LIST_LIMIT {
                                    backups.push(json!({"name":name,"bytes":bytes}));
                                }
                            }
                        } else if name
                            .strip_prefix('.')
                            .and_then(|value| value.strip_suffix(".pending"))
                            .is_some_and(|name| validate_backup_name(name).is_ok())
                            && let Some(bytes) = sizes.record(&entry.path())
                        {
                            pending_bytes = pending_bytes.saturating_add(bytes);
                        }
                    }
                }
                Err(_) => sizes.skipped += 1,
            }
        } else {
            sizes.skipped += 1;
        }
    } else if fs::symlink_metadata(&dir)
        .is_err_and(|error| error.kind() != std::io::ErrorKind::NotFound)
    {
        sizes.skipped += 1;
    }
    control.check()?;
    control.update("MEASURE_SQLITE", 0, None);
    let mut conn = open(
        &data.join(DATABASE_FILE),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
        control,
    )?;
    let transaction = conn.transaction()?;
    let (page_size, page_count, free_pages) = pages(&transaction)?;
    let (active_chats, active_message_bytes) = chat_sizes(&transaction, "ACTIVE")?;
    let (archived_chats, archived_message_bytes) = chat_sizes(&transaction, "ARCHIVED")?;
    transaction.commit()?;
    let total = [database, wal, shm, journal, backup_bytes, pending_bytes]
        .into_iter()
        .fold(0u64, u64::saturating_add);
    Ok(json!({
        "data_root":data,"measured_at":now()?,"partial":sizes.skipped>0,"skipped":sizes.skipped,
        "physical":{"total_bytes":total,"database_bytes":database,"wal_bytes":wal,
            "shm_bytes":shm,"journal_bytes":journal,"backup_bytes":backup_bytes,"pending_backup_bytes":pending_bytes},
        "logical":{"active_chats":active_chats,"archived_chats":archived_chats,
            "active_message_bytes":active_message_bytes,"archived_message_bytes":archived_message_bytes,
            "page_size":page_size,"page_count":page_count,"free_pages":free_pages},
        "backup_count":backup_count,"backups":backups
    }))
}

#[cfg(unix)]
fn free_bytes(path: &Path) -> Option<u64> {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    let path = CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // statvfs writes stats on success and the NUL-terminated path stays live across the call.
    if unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) } != 0 {
        return None;
    }
    // A successful call initialized all of stats.
    let stats = unsafe { stats.assume_init() };
    Some(stats.f_bavail.saturating_mul(stats.f_frsize))
}

#[cfg(windows)]
fn free_bytes(path: &Path) -> Option<u64> {
    use std::{os::windows::ffi::OsStrExt, ptr};
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    let path = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut available = 0;
    // The path is NUL-terminated, the one requested output is writable and other outputs are optional.
    let status = unsafe {
        GetDiskFreeSpaceExW(
            path.as_ptr(),
            &mut available,
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    (status != 0).then_some(available)
}

pub(crate) fn compact(
    data: &Path,
    input: Compact,
    control: &Arc<JobControl>,
) -> Result<Value, AppError> {
    control.update("CHECK_MAINTENANCE", 0, None);
    let conn = open(
        &data.join(DATABASE_FILE),
        OpenFlags::SQLITE_OPEN_READ_WRITE,
        control,
    )?;
    conn.pragma_update(None, "synchronous", "FULL")?;
    let (page_size, page_count, free_pages) = pages(&conn)?;
    if input.expected_page_count != page_count || input.expected_free_pages != free_pages {
        return Err(AppError::new(
            "CONFLICT",
            "Storage changed after review. Measure it again.",
        ));
    }
    if free_pages == 0 {
        return Err(AppError::new(
            "NO_FREE_PAGES",
            "The database has no reusable pages to compact.",
        ));
    }
    let required = page_size.saturating_mul(page_count).saturating_mul(2);
    if free_bytes(data).is_none_or(|available| available < required) {
        return Err(AppError::new(
            "STORAGE_FULL",
            "Cannot establish enough disk space for compaction.",
        ));
    }
    control.check()?;
    control.update("COMPACT_SQLITE", 0, None);
    conn.execute_batch("VACUUM")?;
    // Once SQLite commits, a late cancel must not label the operation cancelled.
    conn.progress_handler(0, None::<fn() -> bool>);
    let (_, after_pages, after_free) = pages(&conn)?;
    Ok(
        json!({"page_size":page_size,"before_page_count":page_count,"before_free_pages":free_pages,
        "after_page_count":after_pages,"after_free_pages":after_free}),
    )
}
