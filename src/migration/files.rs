use crate::policy::is_link;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, Metadata, OpenOptions},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub(super) fn digest_file(path: &Path) -> Result<String, String> {
    let mut file = open(path)?;
    let mut hash = Sha256::new();
    io::copy(&mut file, &mut hash).map_err(|_| "Cannot hash the migration file")?;
    Ok(format!("{:x}", hash.finalize()))
}

pub(super) fn metadata(path: &Path) -> Result<Option<Metadata>, String> {
    match fs::symlink_metadata(path) {
        Ok(meta) if is_link(&meta) => {
            Err("Migration paths cannot cross links or reparse points".into())
        }
        Ok(meta) => Ok(Some(meta)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("Migration path is inaccessible".into()),
    }
}

pub(super) fn absolute(path: &Path) -> Result<PathBuf, String> {
    for part in path.components() {
        match part {
            Component::ParentDir => {
                return Err("Migration paths cannot contain parent traversal".into());
            }
            Component::Normal(part) if !part.to_str().is_some_and(portable_part) => {
                return Err("Migration path has an unsupported filename".into());
            }
            #[cfg(windows)]
            Component::Prefix(prefix) => {
                use std::path::Prefix;
                if !path.is_absolute()
                    || !matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_))
                {
                    return Err("Migration requires a local drive path".into());
                }
            }
            _ => {}
        }
    }
    if path.as_os_str().is_empty() || (path.has_root() && !path.is_absolute()) {
        return Err("Migration requires an explicit file or directory path".into());
    }
    std::path::absolute(path).map_err(|_| "Cannot resolve the migration path".into())
}

pub(super) fn portable_part(part: &str) -> bool {
    let stem = part
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end()
        .to_ascii_uppercase();
    !part.is_empty()
        && !part.ends_with(['.', ' '])
        && !part
            .chars()
            .any(|c| c.is_control() || r#"<>:"/\|?*"#.contains(c))
        && !matches!(
            stem.as_str(),
            "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
        )
        && !["COM", "LPT"].iter().any(|prefix| {
            stem.strip_prefix(prefix).is_some_and(|digit| {
                matches!(
                    digit,
                    "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                )
            })
        })
}

pub(super) fn ancestors(path: &Path) -> Result<(), String> {
    let mut chain = path.ancestors().collect::<Vec<_>>();
    chain.reverse();
    for parent in chain.into_iter().filter(|path| path.has_root()) {
        match metadata(parent)? {
            Some(meta) if meta.is_dir() => {}
            Some(_) => return Err("Migration path ancestor is not a directory".into()),
            None => break,
        }
    }
    Ok(())
}

pub(super) fn ordinary_file(path: &Path) -> Result<bool, String> {
    ancestors(path.parent().ok_or("Migration file has no parent")?)?;
    match metadata(path)? {
        Some(meta) if meta.is_file() => Ok(true),
        Some(_) => Err("Migration requires an ordinary file".into()),
        None => Ok(false),
    }
}

pub(super) fn directory(path: &Path) -> Result<PathBuf, String> {
    let absolute = absolute(path)?;
    ancestors(&absolute)?;
    if !metadata(&absolute)?.is_some_and(|meta| meta.is_dir()) {
        return Err("Migration directory is missing".into());
    }
    absolute
        .canonicalize()
        .map_err(|_| "Cannot resolve the migration directory".into())
}

pub(super) fn destination(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let path = absolute(path)?;
    let parent = directory(path.parent().ok_or("Migration destination has no parent")?)?;
    let path = parent.join(
        path.file_name()
            .ok_or("Migration destination needs a name")?,
    );
    if case_alias(&path)? || metadata(&path)?.is_some() {
        return Err("Migration export destination already exists or has a case alias".into());
    }
    for ancestor in parent.ancestors() {
        if metadata(&ancestor.join(".git"))?.is_some() {
            if ancestor != directory(root)? {
                return Err("Migration export cannot write inside another Git checkout".into());
            }
            let relative = path
                .strip_prefix(ancestor)
                .map_err(|_| "Invalid archive scope")?;
            if !relative.starts_with(".cirnetrules") {
                return Err(
                    "Migration archives belong outside Git or in ignored .cirnetrules state".into(),
                );
            }
            require_ignored(ancestor, relative)?;
            break;
        }
    }
    Ok(path)
}

pub(super) fn require_ignored(root: &Path, relative: &Path) -> Result<(), String> {
    let relative = relative
        .to_str()
        .ok_or("Local migration path is not UTF-8")?
        .replace('\\', "/");
    let probe = format!("{relative}/archive.json");
    if !crate::policy::git(root, &["ls-files", "--", &relative])?.is_empty()
        || crate::policy::git(root, &["check-ignore", "--quiet", "--", &probe]).is_err()
    {
        return Err("Migration state must be ignored and untracked".into());
    }
    Ok(())
}

pub(super) fn case_alias(path: &Path) -> Result<bool, String> {
    let Some((parent, name)) = path
        .parent()
        .zip(path.file_name().and_then(|name| name.to_str()))
    else {
        return Err("Invalid migration path".into());
    };
    if !metadata(parent)?.is_some_and(|meta| meta.is_dir()) {
        return Ok(false);
    }
    for entry in fs::read_dir(parent).map_err(|_| "Cannot enumerate migration directory")? {
        let entry = entry.map_err(|_| "Cannot inspect migration filename")?;
        if entry
            .file_name()
            .to_str()
            .is_some_and(|other| other != name && other.eq_ignore_ascii_case(name))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn private_dir(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    let result = {
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new().mode(0o700).create(path)
    };
    #[cfg(not(unix))]
    let result = fs::create_dir(path);
    result.map_err(|_| "Cannot create a new migration directory".into())
}

pub(super) fn open(path: &Path) -> Result<File, String> {
    open_read(path, false)
}

pub(super) fn open_existing_shared(path: &Path) -> Result<File, String> {
    open_read(path, true)
}

fn open_read(path: &Path, preserve_path: bool) -> Result<File, String> {
    if !ordinary_file(path)? {
        return Err("Migration file is missing".into());
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    if preserve_path {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{FILE_SHARE_READ, FILE_SHARE_WRITE};
        options.share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE);
    }
    #[cfg(not(windows))]
    let _ = preserve_path;
    nofollow(&mut options);
    let file = options
        .open(path)
        .map_err(|_| "Cannot read the migration file")?;
    if !file
        .metadata()
        .is_ok_and(|meta| meta.is_file() && !is_link(&meta))
    {
        return Err("Migration requires an ordinary file".into());
    }
    Ok(file)
}

fn nofollow(options: &mut OpenOptions) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT);
    }
}

pub(super) fn read(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Cannot read migration data")?;
    if bytes.len() as u64 > limit {
        return Err("Migration file exceeds its byte budget".into());
    }
    Ok(bytes)
}

pub(super) fn new_file(path: &Path) -> Result<File, String> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(path)
        .map_err(|_| "Cannot create new migration data".into())
}

pub(super) fn publish(path: &Path, bytes: &[u8]) -> Result<(), String> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = directory(path.parent().ok_or("Migration file has no parent")?)?;
    let path = parent.join(path.file_name().ok_or("Migration file has no name")?);
    if case_alias(&path)? {
        return Err("Migration file has a case alias".into());
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| "Local time is unavailable")?
        .as_nanos();
    let stage = parent.join(format!(
        ".stage-{}-{stamp}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = new_file(&stage)?;
    let result = file.write_all(bytes).and_then(|_| file.sync_all());
    drop(file);
    let result = result.and_then(|()| publish_new(&stage, &path));
    let _ = fs::remove_file(&stage);
    result.map_err(|_| "Cannot publish new migration data; existing files were preserved")?;
    sync_dir(&parent)
}

#[cfg(not(windows))]
fn publish_new(stage: &Path, path: &Path) -> io::Result<()> {
    fs::hard_link(stage, path)
}

#[cfg(windows)]
fn publish_new(stage: &Path, path: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{MOVEFILE_WRITE_THROUGH, MoveFileExW};
    let wide = |name: &Path| {
        name.as_os_str()
            .encode_wide()
            .chain([0])
            .collect::<Vec<_>>()
    };
    let stage = wide(stage);
    let path = wide(path);
    if unsafe { MoveFileExW(stage.as_ptr(), path.as_ptr(), MOVEFILE_WRITE_THROUGH) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub(super) fn sync_dir(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        File::open(path)
            .and_then(|dir| dir.sync_all())
            .map_err(|_| "Cannot sync migration directory")?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

pub(super) fn lock(path: &Path) -> Result<File, String> {
    ordinary_file(path)?;
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    nofollow(&mut options);
    let lock = options
        .open(path)
        .map_err(|_| "Cannot open migration operation lock")?;
    if !lock
        .metadata()
        .is_ok_and(|meta| meta.is_file() && !is_link(&meta))
    {
        return Err("Migration lock must be an ordinary file".into());
    }
    lock.try_lock()
        .map_err(|_| "Migration operation is busy; retry using the same preview digest")?;
    Ok(lock)
}
