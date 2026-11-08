use super::*;
use std::{
    io::Read,
    path::Component,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const MAX_MARKER: u64 = 4096;

#[derive(Debug, Serialize)]
pub(super) struct Checkout {
    pub selected_path: PathBuf,
    pub root: PathBuf,
    pub git_dir: PathBuf,
    pub common_dir: PathBuf,
    pub branch: Option<String>,
    pub commit: String,
    pub file_hint: Option<String>,
    pub linked_worktree: bool,
    pub suggested_name: String,
    pub common_identity: String,
    pub identity: String,
}

pub(super) fn inspect(path: &Path) -> Result<Checkout, AppError> {
    storage::validate_local(path)?;
    if path
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err(AppError::new(
            "INVALID_PATH",
            "The selected path contains traversal.",
        ));
    }
    let selected = path.canonicalize().map_err(|err| match err.kind() {
        std::io::ErrorKind::NotFound => AppError::new("MISSING", "The selected path is missing."),
        _ => AppError::new("INACCESSIBLE", "The selected path is not readable."),
    })?;
    storage::check_ancestors(&selected, false)?;
    let metadata = fs::metadata(&selected)
        .map_err(|_| AppError::new("INACCESSIBLE", "The selected path is not readable."))?;
    let start = if metadata.is_file() {
        selected.parent().ok_or_else(invalid_metadata)?
    } else if metadata.is_dir() {
        &selected
    } else {
        return Err(AppError::new(
            "INVALID_PATH",
            "Select a directory or regular file.",
        ));
    };
    let (root, marker_metadata) = discover(start)?;
    let marker = root.join(".git");
    if storage::redirect(&marker_metadata) {
        return Err(invalid_metadata());
    }
    let git_dir = if marker_metadata.is_dir() {
        marker.clone()
    } else {
        let declaration = read_marker(&marker)?;
        let target = declaration
            .strip_prefix("gitdir: ")
            .ok_or_else(invalid_metadata)?;
        resolve_metadata(root, target)?
    };
    let common_marker = git_dir.join("commondir");
    let (common_dir, linked_worktree) = match fs::symlink_metadata(&common_marker) {
        Ok(_) => {
            let common = resolve_metadata(&git_dir, &read_marker(&common_marker)?)?;
            let backlink = resolve_metadata(&git_dir, &read_marker(&git_dir.join("gitdir"))?)?;
            if backlink != marker || git_dir.parent() != Some(common.join("worktrees").as_path()) {
                return Err(invalid_metadata());
            }
            (common, true)
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => (git_dir.clone(), false),
        Err(_) => return Err(invalid_metadata()),
    };
    if !git_dir.is_dir() || !common_dir.is_dir() {
        return Err(invalid_metadata());
    }
    let executable = git_executable(root)?;
    if executable.starts_with(&git_dir) || executable.starts_with(&common_dir) {
        return Err(AppError::new(
            "GIT_UNAVAILABLE",
            "Git cannot be started from the selected repository.",
        ));
    }
    let declared = run_git(
        &executable,
        root,
        &[
            "rev-parse",
            "--path-format=absolute",
            "--show-toplevel",
            "--git-dir",
            "--git-common-dir",
            "--is-inside-work-tree",
        ],
    )?
    .ok_or_else(invalid_metadata)?;
    let mut fields = declared.lines();
    for path in [root, git_dir.as_path(), common_dir.as_path()] {
        let output = fields.next().ok_or_else(invalid_metadata)?;
        if !Path::new(output).is_absolute() || resolve_metadata(root, output)? != path {
            return Err(invalid_metadata());
        }
    }
    if fields.next() != Some("true") || fields.next().is_some() {
        return Err(invalid_metadata());
    }
    let head = read_marker(&git_dir.join("HEAD"))?;
    let branch = head.strip_prefix("ref: refs/heads/").map(str::to_owned);
    let commit = match run_git(
        &executable,
        root,
        &["rev-parse", "--verify", "HEAD^{commit}"],
    )? {
        Some(commit) if valid_git_id(commit.trim()) => commit.trim().to_owned(),
        None if branch.is_some() => {
            let reference = head.strip_prefix("ref: ").ok_or_else(invalid_metadata)?;
            // An absent branch on a newly initialized checkout is allowed.
            // A present or malformed reference that cannot resolve is not.
            if !valid_head_ref(reference)
                || git_dir.join(reference).exists()
                || common_dir.join(reference).exists()
                || run_git(&executable, root, &["show-ref", "--verify", reference])?.is_some()
            {
                return Err(invalid_metadata());
            }
            String::new()
        }
        _ => return Err(invalid_metadata()),
    };
    let file_hint = if metadata.is_file() {
        let relative = selected
            .strip_prefix(root)
            .map_err(|_| invalid_metadata())?;
        let hint = relative
            .to_str()
            .ok_or_else(invalid_metadata)?
            .replace('\\', "/");
        (!hint
            .split('/')
            .any(|part| part.eq_ignore_ascii_case(".git")))
        .then_some(hint)
    } else {
        None
    };
    let suggested_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("Repository")
        .chars()
        .take(120)
        .collect();
    let common_identity = file_identity(&common_dir)?;
    let identity =
        digest(format!("{}:{}", file_identity(root)?, file_identity(&git_dir)?).as_bytes());
    Ok(Checkout {
        selected_path: path.to_path_buf(),
        root: root.to_path_buf(),
        git_dir,
        common_dir,
        branch,
        commit,
        file_hint,
        linked_worktree,
        suggested_name,
        common_identity,
        identity,
    })
}

fn discover(start: &Path) -> Result<(&Path, fs::Metadata), AppError> {
    for path in start.ancestors().take(64) {
        match fs::symlink_metadata(path.join(".git")) {
            Ok(meta) if meta.is_dir() || meta.is_file() => return Ok((path, meta)),
            Ok(_) => return Err(invalid_metadata()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => {
                return Err(AppError::new(
                    "INACCESSIBLE",
                    "Cannot inspect Git checkout metadata.",
                ));
            }
        }
    }
    Err(AppError::new(
        "NOT_A_REPOSITORY",
        "The selected path is not inside a Git worktree.",
    ))
}

fn read_marker(path: &Path) -> Result<String, AppError> {
    let meta = fs::symlink_metadata(path).map_err(|_| invalid_metadata())?;
    if !meta.is_file() || storage::redirect(&meta) || meta.len() > MAX_MARKER {
        return Err(invalid_metadata());
    }
    let mut value = String::new();
    fs::File::open(path)
        .map_err(|_| invalid_metadata())?
        .take(MAX_MARKER + 1)
        .read_to_string(&mut value)
        .map_err(|_| invalid_metadata())?;
    let line = value.trim_end_matches(['\r', '\n']);
    if line.is_empty() || line.len() as u64 > MAX_MARKER || line.chars().any(char::is_control) {
        return Err(invalid_metadata());
    }
    Ok(line.to_owned())
}

fn resolve_metadata(base: &Path, value: &str) -> Result<PathBuf, AppError> {
    let declared = Path::new(value);
    if declared.is_absolute() {
        storage::validate_local(declared).map_err(|_| invalid_metadata())?;
    }
    let mut result = if declared.is_absolute() {
        PathBuf::new()
    } else {
        base.to_path_buf()
    };
    for component in declared.components() {
        match component {
            Component::Prefix(_) | Component::RootDir if !declared.is_absolute() => {
                return Err(invalid_metadata());
            }
            Component::Prefix(_) | Component::RootDir => result.push(component),
            Component::CurDir => {}
            Component::ParentDir if result.pop() => {}
            Component::ParentDir => return Err(invalid_metadata()),
            Component::Normal(segment) => {
                result.push(segment);
                let meta = fs::symlink_metadata(&result).map_err(|_| invalid_metadata())?;
                if storage::redirect(&meta) {
                    return Err(invalid_metadata());
                }
            }
        }
    }
    storage::check_ancestors(&result, false).map_err(|_| invalid_metadata())?;
    result.canonicalize().map_err(|_| invalid_metadata())
}

fn invalid_metadata() -> AppError {
    AppError::new(
        "REBIND_REQUIRED",
        "The Git worktree metadata does not identify a readable local checkout.",
    )
}

pub(super) fn git_executable(root: &Path) -> Result<PathBuf, AppError> {
    let paths = std::env::var_os("PATH")
        .ok_or_else(|| AppError::new("GIT_UNAVAILABLE", "Git is unavailable."))?;
    let name = if cfg!(windows) { "git.exe" } else { "git" };
    for dir in std::env::split_paths(&paths).filter(|dir| dir.is_absolute()) {
        if let Ok(candidate) = dir.join(name).canonicalize()
            && !candidate.starts_with(root)
            && candidate.is_file()
            && storage::validate_local(&candidate).is_ok()
        {
            return Ok(candidate);
        }
    }
    Err(AppError::new(
        "GIT_UNAVAILABLE",
        "No local Git executable was found outside this checkout.",
    ))
}

fn run_git(executable: &Path, root: &Path, args: &[&str]) -> Result<Option<String>, AppError> {
    let mut cmd = Command::new(executable);
    cmd.current_dir(root)
        .env_clear()
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_ATTR_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .args([
            "--no-pager",
            "--no-lazy-fetch",
            "--no-optional-locks",
            "--no-replace-objects",
            "-c",
            "core.fsmonitor=false",
        ])
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
        for name in ["SystemRoot", "WINDIR"] {
            if let Some(value) = std::env::var_os(name) {
                cmd.env(name, value);
            }
        }
    }
    let mut child = cmd.spawn().map_err(|_| {
        AppError::new(
            "GIT_UNAVAILABLE",
            "Cannot start the local Git metadata check.",
        )
    })?;
    let stdout = child.stdout.take().ok_or_else(invalid_metadata)?;
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let mut output = Vec::new();
        let result = stdout
            .take(128 * 1024 + 1)
            .read_to_end(&mut output)
            .map(|_| output);
        let _ = sender.send(result);
    });
    let limit = Instant::now() + Duration::from_secs(4);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < limit => thread::sleep(Duration::from_millis(10)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(AppError::new(
                    "GIT_TIMEOUT",
                    "Local Git metadata verification could not finish.",
                ));
            }
        }
    };
    let output = receiver
        .recv_timeout(limit.saturating_duration_since(Instant::now()))
        .map_err(|_| AppError::new("GIT_TIMEOUT", "Local Git metadata output did not finish."))?
        .map_err(|_| invalid_metadata())?;
    if output.len() > 128 * 1024 {
        return Err(invalid_metadata());
    }
    if !status.success() {
        return Ok(None);
    }
    String::from_utf8(output)
        .map(Some)
        .map_err(|_| invalid_metadata())
}

fn valid_git_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_head_ref(value: &str) -> bool {
    value.starts_with("refs/heads/")
        && value.len() <= 1024
        && !value.contains(['\\', ':', '~', '^', '?', '*', '[', ' '])
        && !value.contains("..")
        && !value.split('/').any(|part| {
            part.is_empty() || part == "." || part.starts_with('.') || part.ends_with(".lock")
        })
}

pub(super) fn contains_commit(checkout: &Checkout, anchor: &str) -> Result<bool, AppError> {
    if anchor.is_empty() || anchor == checkout.commit {
        return Ok(true);
    }
    if !valid_git_id(anchor) {
        return Ok(false);
    }
    let git = git_executable(&checkout.root)?;
    if git.starts_with(&checkout.git_dir) || git.starts_with(&checkout.common_dir) {
        return Err(invalid_metadata());
    }
    Ok(run_git(&git, &checkout.root, &["cat-file", "-t", anchor])?
        .is_some_and(|kind| kind.trim() == "commit"))
}

#[cfg(unix)]
fn file_identity(path: &Path) -> Result<String, AppError> {
    use std::os::unix::fs::MetadataExt;
    let meta = fs::symlink_metadata(path).map_err(|_| invalid_metadata())?;
    if !meta.is_dir() || storage::redirect(&meta) {
        return Err(invalid_metadata());
    }
    Ok(format!("{:x}-{:x}", meta.dev(), meta.ino()))
}

#[cfg(windows)]
fn file_identity(path: &Path) -> Result<String, AppError> {
    use std::{ffi::c_void, os::windows::ffi::OsStrExt, ptr};
    type Handle = *mut c_void;
    #[repr(C)]
    struct FileIdInfo {
        volume: u64,
        id: [u8; 16],
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateFileW(
            name: *const u16,
            access: u32,
            share: u32,
            security: *const c_void,
            creation: u32,
            flags: u32,
            template: Handle,
        ) -> Handle;
        fn GetFileInformationByHandleEx(
            handle: Handle,
            class: i32,
            info: *mut c_void,
            size: u32,
        ) -> i32;
        fn CloseHandle(handle: Handle) -> i32;
    }
    let meta = fs::symlink_metadata(path).map_err(|_| invalid_metadata())?;
    if !meta.is_dir() || storage::redirect(&meta) {
        return Err(invalid_metadata());
    }
    let name = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    // FILE_READ_ATTRIBUTES, shared access, OPEN_EXISTING, directory and reparse-safe flags.
    let handle = unsafe {
        CreateFileW(
            name.as_ptr(),
            0x80,
            7,
            ptr::null(),
            3,
            0x02200000,
            ptr::null_mut(),
        )
    };
    if handle as isize == -1 {
        return Err(invalid_metadata());
    }
    let mut info = FileIdInfo {
        volume: 0,
        id: [0; 16],
    };
    // FileIdInfo writes exactly the fixed layout above. The owned handle is closed once.
    let success = unsafe {
        let result = GetFileInformationByHandleEx(
            handle,
            18,
            (&mut info as *mut FileIdInfo).cast(),
            std::mem::size_of::<FileIdInfo>() as u32,
        );
        CloseHandle(handle);
        result
    };
    if success == 0 {
        return Err(invalid_metadata());
    }
    Ok(format!("{:x}-{}", info.volume, digest(&info.id)))
}
