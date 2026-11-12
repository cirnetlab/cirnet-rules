use crate::{docs, policy, workspace};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    fs::{self, File, Metadata, OpenOptions},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf, Prefix},
    time::{SystemTime, UNIX_EPOCH},
};

pub(super) const MAX_FILE: u64 = 4 * 1024 * 1024;

pub(super) fn validate(relative: &str) -> Result<(), String> {
    crate::tasks::relative(relative)
}

pub(super) fn validate_names<'a>(paths: impl IntoIterator<Item = &'a str>) -> Result<(), String> {
    let mut names = BTreeMap::new();
    let mut files = BTreeSet::new();
    let mut directories = BTreeSet::new();
    for path in paths {
        validate(path)?;
        let mut cursor = path;
        while let Some((parent, _)) = cursor.rsplit_once('/') {
            directories.insert(parent.to_ascii_lowercase());
            cursor = parent;
        }
        if !files.insert(path.to_ascii_lowercase()) {
            return Err("Duplicate or case-colliding agent output file".into());
        }
        let mut cursor = path;
        loop {
            let key = cursor.to_ascii_lowercase();
            if names
                .insert(key, cursor)
                .is_some_and(|prior| prior != cursor)
            {
                return Err("Case-colliding agent output paths".into());
            }
            let Some((parent, _)) = cursor.rsplit_once('/') else {
                break;
            };
            cursor = parent;
        }
    }
    if !directories.is_disjoint(&files) {
        return Err("Agent output file conflicts with a directory".into());
    }
    Ok(())
}

fn metadata(path: &Path) -> Result<Option<Metadata>, String> {
    match fs::symlink_metadata(path) {
        Ok(meta) if !policy::is_link(&meta) => Ok(Some(meta)),
        Ok(_) => Err("Agent configuration path crosses a symbolic link or reparse point".into()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("Cannot inspect agent configuration path".into()),
    }
}

fn ordinary_ancestors(path: &Path) -> Result<(), String> {
    for ancestor in path.ancestors().filter(|p| p.parent().is_some()) {
        if !metadata(ancestor)?.is_some_and(|meta| meta.is_dir()) {
            return Err("Agent configuration directory is missing or occupied".into());
        }
    }
    Ok(())
}

pub(super) fn explicit(input: &Path) -> Result<PathBuf, String> {
    if input.as_os_str().is_empty()
        || (input.has_root() && !input.is_absolute())
        || matches!(input.components().next(), Some(Component::Prefix(prefix))
            if !input.is_absolute() || !matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
        || input.components().any(|c| c == Component::ParentDir)
    {
        return Err("Select a local filesystem path without traversal".into());
    }
    if input.is_absolute() {
        Ok(input.to_path_buf())
    } else {
        Ok(env::current_dir()
            .map_err(|_| "Cannot inspect current directory")?
            .join(input))
    }
}

pub(super) fn directory(input: &Path) -> Result<PathBuf, String> {
    let path = explicit(input)?;
    ordinary_ancestors(&path)?;
    if !metadata(&path)?.is_some_and(|meta| meta.is_dir()) {
        return Err("Expected an ordinary agent configuration directory".into());
    }
    path.canonicalize()
        .map_err(|_| "Cannot resolve selected directory".into())
}

pub(super) fn contains(root: &Path, path: &Path) -> bool {
    let mut parts = path.components();
    root.components().all(|expected| {
        parts.next().is_some_and(|actual| {
            if cfg!(windows) {
                expected
                    .as_os_str()
                    .eq_ignore_ascii_case(actual.as_os_str())
            } else {
                expected == actual
            }
        })
    })
}

pub(crate) fn export_path(input: &Path) -> Result<PathBuf, String> {
    let path = explicit(input)?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Invalid bundle directory name")?;
    validate(name)?;
    let parent = directory(path.parent().ok_or("Bundle output has no parent")?)?;
    let path = parent.join(name);
    let parts: Vec<_> = path
        .iter()
        .map(|part| part.to_string_lossy().to_ascii_lowercase())
        .collect();
    if parts.iter().enumerate().any(|(i, part)| {
        matches!(
            part.as_str(),
            ".agents" | ".agent" | ".claude" | ".gemini" | ".antigravity" | ".config"
        ) || (part == ".codex" && parts.get(i + 1).is_none_or(|next| next != "visualizations"))
            || (part == "antigravity" && parts[..i].iter().any(|previous| previous == "appdata"))
    }) {
        return Err("Bundle output cannot be a native agent configuration directory".into());
    }
    for ancestor in path.ancestors() {
        if ancestor.join(".git").symlink_metadata().is_ok()
            || (ancestor.join("HEAD").is_file() && ancestor.join("objects").is_dir())
        {
            return Err("Bundle output must be outside Git checkouts".into());
        }
    }
    for key in ["CODEX_HOME", "CLAUDE_CONFIG_DIR", "XDG_CONFIG_HOME"] {
        if let Some(config) =
            env::var_os(key).and_then(|value| PathBuf::from(value).canonicalize().ok())
            && contains(&config, &path)
            && !(key == "CODEX_HOME" && contains(&config.join("visualizations"), &path))
        {
            return Err("Bundle output cannot overlap a configured agent directory".into());
        }
    }
    check_case(&parent, name)?;
    let _ = metadata(&path)?;
    Ok(path)
}

#[derive(PartialEq, Eq, Serialize)]
pub(super) struct ExportLocation {
    pub(super) path: PathBuf,
    pub(super) parent_identity: [u64; 2],
    pub(super) existing: Option<[u64; 2]>,
}

pub(super) fn export_location(input: &Path) -> Result<ExportLocation, String> {
    let path = export_path(input)?;
    let parent = path.parent().ok_or("Bundle output has no parent")?;
    let parent_meta = metadata(parent)?
        .filter(Metadata::is_dir)
        .ok_or("Bundle output parent is missing")?;
    let existing = metadata(&path)?
        .map(|meta| {
            if !meta.is_dir() {
                return Err("Bundle output is occupied by a file".into());
            }
            file_id(&path, &meta)
        })
        .transpose()?;
    Ok(ExportLocation {
        parent_identity: file_id(parent, &parent_meta)?,
        path,
        existing,
    })
}

fn check_case(parent: &Path, expected: &str) -> Result<(), String> {
    let mut exact = false;
    for entry in fs::read_dir(parent).map_err(|_| "Cannot inspect destination filename case")? {
        let entry = entry.map_err(|_| "Cannot inspect destination filename case")?;
        if let Some(actual) = entry.file_name().to_str() {
            if actual == expected {
                exact = true;
            } else if actual.eq_ignore_ascii_case(expected) {
                return Err("Case-colliding agent configuration path".into());
            }
        }
    }
    if !exact && metadata(&parent.join(expected))?.is_some() {
        return Err("Agent configuration path resolves through a filesystem alias".into());
    }
    Ok(())
}

fn file_id(path: &Path, meta: &Metadata) -> Result<[u64; 2], String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let _ = path;
        Ok([meta.dev(), meta.ino()])
    }
    #[cfg(windows)]
    {
        use std::{
            mem::MaybeUninit,
            os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
        };
        use windows_sys::Win32::Storage::FileSystem::{
            BY_HANDLE_FILE_INFORMATION, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
            GetFileInformationByHandle,
        };
        let _ = meta;
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .map_err(|_| "Cannot open agent configuration file identity")?;
        let mut info = MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
        // The handle remains open for the call, and success initializes the entire structure.
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), info.as_mut_ptr()) } == 0 {
            return Err("Cannot obtain agent configuration file identity".into());
        }
        let info = unsafe { info.assume_init() };
        if info.dwFileAttributes & 0x400 != 0 {
            return Err("Agent configuration path changed to a reparse point".into());
        }
        Ok([
            u64::from(info.dwVolumeSerialNumber),
            (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
        ])
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Observation {
    pub sha256: Option<String>,
    ancestors: Vec<[u64; 2]>,
    file: Option<[u64; 2]>,
}

impl Observation {
    pub(super) fn matches_after_directory_creation(&self, other: &Self) -> bool {
        self.sha256 == other.sha256
            && self.file == other.file
            && other.ancestors.starts_with(&self.ancestors)
    }
}

pub(super) fn observe(root: &Path, relative: &str) -> Result<Observation, String> {
    let (_, state) = inspect(root, relative, true)?;
    Ok(state)
}

fn inspect(
    root: &Path,
    relative: &str,
    read_contents: bool,
) -> Result<(PathBuf, Observation), String> {
    workspace::scoped_path(root, relative)?;
    ordinary_ancestors(root)?;
    let root_meta = metadata(root)?
        .filter(Metadata::is_dir)
        .ok_or("Agent configuration root is unavailable")?;
    let mut state = Observation {
        sha256: None,
        ancestors: vec![file_id(root, &root_meta)?],
        file: None,
    };
    let mut current = root.to_path_buf();
    let mut parts = relative.split('/').peekable();
    while let Some(part) = parts.next() {
        check_case(&current, part)?;
        current.push(part);
        let Some(meta) = metadata(&current)? else {
            return Ok((root.join(relative), state));
        };
        if parts.peek().is_none() {
            if !meta.is_file() {
                return Err(
                    "Agent configuration file is occupied by a directory or special file".into(),
                );
            }
            state.file = Some(file_id(&current, &meta)?);
            if read_contents {
                state.sha256 = Some(docs::sha256(&read(&current)?));
                if !metadata(&current)?
                    .is_some_and(|next| file_id(&current, &next).ok() == state.file)
                {
                    return Err("Agent configuration destination changed while being read".into());
                }
            }
        } else {
            if !meta.is_dir() {
                return Err("Agent configuration parent is occupied by a file".into());
            }
            state.ancestors.push(file_id(&current, &meta)?);
        }
    }
    Ok((current, state))
}

pub(super) fn read(path: &Path) -> Result<Vec<u8>, String> {
    ordinary_ancestors(
        path.parent()
            .ok_or("Agent configuration file has no parent")?,
    )?;
    let meta = metadata(path)?
        .filter(Metadata::is_file)
        .ok_or("Missing ordinary agent configuration file")?;
    if meta.len() > MAX_FILE {
        return Err("Agent configuration file exceeds its size limit".into());
    }
    let mut bytes = Vec::new();
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
    let file = options
        .open(path)
        .map_err(|_| "Cannot open agent configuration file")?;
    if file
        .metadata()
        .ok()
        .is_none_or(|meta| !meta.is_file() || policy::is_link(&meta))
    {
        return Err("Agent configuration file changed during open".into());
    }
    file.take(MAX_FILE + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Cannot read agent configuration file")?;
    if bytes.len() as u64 > MAX_FILE {
        return Err("Agent configuration file exceeds its size limit".into());
    }
    Ok(bytes)
}

pub(super) fn directories(root: &Path, relative: &str) -> Result<PathBuf, String> {
    workspace::scoped_path(root, relative)?;
    ordinary_ancestors(root)?;
    let mut current = root.to_path_buf();
    for part in relative.split('/') {
        check_case(&current, part)?;
        current.push(part);
        if metadata(&current)?.is_none() {
            match fs::create_dir(&current) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(_) => {
                    return Err("Cannot create an ordinary agent configuration directory".into());
                }
            }
        }
        if !metadata(&current)?.is_some_and(|meta| meta.is_dir()) {
            return Err("Agent configuration directory is occupied".into());
        }
    }
    Ok(current)
}

pub(super) fn nonce() -> Result<u128, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "Local clock unavailable")?
        .as_nanos())
}

pub(crate) fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() as u64 > MAX_FILE {
        return Err("Agent configuration data exceeds its file size limit".into());
    }
    let parent = path
        .parent()
        .ok_or("Invalid agent configuration destination")?;
    ordinary_ancestors(parent)?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|_| "Agent configuration file already exists or is not writable")?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| "Cannot persist agent configuration file".into())
}

pub(super) fn rename_new(from: &Path, to: &Path) -> Result<(), String> {
    if metadata(to)?.is_some() {
        return Err("Agent configuration destination already exists".into());
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::MoveFileExW;
        let source: Vec<_> = from.as_os_str().encode_wide().chain([0]).collect();
        let destination: Vec<_> = to.as_os_str().encode_wide().chain([0]).collect();
        // Both pointers refer to live NUL-terminated wide strings.
        if unsafe { MoveFileExW(source.as_ptr(), destination.as_ptr(), 0) } == 0 {
            return Err("Cannot install a new agent configuration file".into());
        }
    }
    #[cfg(not(windows))]
    {
        fs::hard_link(from, to).map_err(|_| "Cannot install a new agent configuration file")?;
        fs::remove_file(from)
            .map_err(|_| "Installed agent configuration file but retained its stage")?;
    }
    Ok(())
}

pub(super) fn replace(
    root: &Path,
    relative: &str,
    expected: &Observation,
    bytes: Option<&[u8]>,
) -> Result<(), String> {
    replace_with_intent(root, relative, expected, bytes, |_| Ok(()))
}

pub(super) fn replace_with_intent(
    root: &Path,
    relative: &str,
    expected: &Observation,
    bytes: Option<&[u8]>,
    record: impl FnOnce(Observation) -> Result<(), String>,
) -> Result<(), String> {
    let name = format!(".cnr-stage-{}-{}", std::process::id(), nonce()?);
    let staged = relative
        .rsplit_once('/')
        .map_or(name.clone(), |(parent, _)| format!("{parent}/{name}"));
    replace_journaled(root, relative, &staged, expected, bytes, record)
}

pub(super) fn replace_journaled(
    root: &Path,
    relative: &str,
    staged: &str,
    expected: &Observation,
    bytes: Option<&[u8]>,
    record: impl FnOnce(Observation) -> Result<(), String>,
) -> Result<(), String> {
    replace_journaled_with(root, relative, staged, expected, bytes, write_new, record)
}

pub(super) fn replace_private_journaled(
    root: &Path,
    relative: &str,
    staged: &str,
    expected: &Observation,
    bytes: Option<&[u8]>,
    record: impl FnOnce(Observation) -> Result<(), String>,
) -> Result<(), String> {
    replace_journaled_with(
        root,
        relative,
        staged,
        expected,
        bytes,
        write_private,
        record,
    )
}

pub(super) fn write_private(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if bytes.len() as u64 > MAX_FILE {
        return Err("Agent configuration data exceeds its file size limit".into());
    }
    ordinary_ancestors(path.parent().ok_or("Invalid private file destination")?)?;
    let mut file = crate::coordination::create_private_file(path)
        .map_err(|_| "Cannot create an owner-restricted configuration file")?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| "Cannot persist owner-restricted configuration data".into())
}

fn replace_journaled_with(
    root: &Path,
    relative: &str,
    staged: &str,
    expected: &Observation,
    bytes: Option<&[u8]>,
    write_stage: impl FnOnce(&Path, &[u8]) -> Result<(), String>,
    record: impl FnOnce(Observation) -> Result<(), String>,
) -> Result<(), String> {
    if &observe(root, relative)? != expected {
        return Err("Agent configuration destination changed since preview".into());
    }
    let path = root.join(relative);
    if let Some(bytes) = bytes {
        let parent = relative.rsplit_once('/').map(|(parent, _)| parent);
        if staged.rsplit_once('/').map(|(parent, _)| parent) != parent
            || staged == relative
            || observe(root, staged)?.sha256.is_some()
        {
            return Err(
                "Agent configuration stage is occupied or not a sibling of the destination".into(),
            );
        }
        if let Some(parent) = parent {
            directories(root, parent)?;
        }
        let stage = root.join(staged);
        write_stage(&stage, bytes)?;
        let staged_state = observe(root, staged)?;
        let result = (|| {
            let current = observe(root, relative)?;
            if !expected.matches_after_directory_creation(&current)
                || staged_state.ancestors != current.ancestors
                || staged_state.sha256.as_deref() != Some(&docs::sha256(bytes))
            {
                return Err(
                    "Agent configuration stage or destination changed before replacement".into(),
                );
            }
            record(staged_state.clone())?;
            if observe(root, relative)? != current || observe(root, staged)? != staged_state {
                return Err(
                    "Agent configuration stage or destination changed before replacement".into(),
                );
            }
            if expected.file.is_some() {
                fs::rename(&stage, path)
                    .map_err(|_| "Cannot replace owned agent configuration file".to_owned())
            } else {
                rename_new(&stage, &path)
            }
        })();
        if result.is_err() && observe(root, staged).ok().as_ref() == Some(&staged_state) {
            let _ = fs::remove_file(stage);
        }
        result?;
    } else if expected.file.is_some() {
        record(Observation {
            sha256: None,
            ancestors: expected.ancestors.clone(),
            file: None,
        })?;
        if &observe(root, relative)? != expected {
            return Err("Agent configuration destination changed before removal".into());
        }
        fs::remove_file(path).map_err(|_| "Cannot remove owned agent configuration file")?;
    }
    Ok(())
}

pub(super) fn lock(root: &Path) -> Result<File, String> {
    ordinary_ancestors(root)?;
    let path = root.join("sync.lock");
    if metadata(&path)?.is_some_and(|meta| !meta.is_file()) {
        return Err("Agent configuration lock path is occupied".into());
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .mode(0o600);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options
        .open(path)
        .map_err(|_| "Cannot open agent configuration lock")?;
    if file
        .metadata()
        .ok()
        .is_none_or(|meta| !meta.is_file() || policy::is_link(&meta))
    {
        return Err("Agent configuration lock path changed".into());
    }
    file.try_lock()
        .map_err(|_| "Another agent configuration operation holds this binding lock")?;
    Ok(file)
}
