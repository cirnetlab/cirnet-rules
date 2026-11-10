use super::{Selection, run_native};
use crate::tooling;
use std::{
    env,
    ffi::{OsStr, OsString},
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
    process::Command,
};

pub(super) fn executable(path: &Path) -> Result<PathBuf, String> {
    reject_batch(path)?;
    let actual = path
        .canonicalize()
        .map_err(|_| "Executable is unavailable")?;
    reject_batch(&actual)?;
    let meta = actual
        .metadata()
        .map_err(|_| "Cannot inspect the executable")?;
    if !meta.is_file() {
        return Err("Expected an executable file".into());
    }
    #[cfg(windows)]
    if !actual
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
    {
        return Err("Windows child processes require a native .exe".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o111 == 0 {
            return Err("The file is not executable".into());
        }
    }
    Ok(actual)
}

fn reject_batch(path: &Path) -> Result<(), String> {
    if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("cmd") || ext.eq_ignore_ascii_case("bat"))
    {
        return Err(
            "Windows batch files require a shell and cannot be used as native executables".into(),
        );
    }
    Ok(())
}

pub(super) fn plain_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

pub(super) fn explicit(root: &Path, name: &str) -> Result<PathBuf, String> {
    let path = Path::new(name);
    reject_batch(path)?;
    if path.is_absolute() {
        return executable(path);
    }
    if !plain_relative(path) && !path.starts_with(".") {
        return Err("Use an absolute executable or a path relative to the selected project".into());
    }
    if path.components().any(|part| {
        matches!(
            part,
            Component::ParentDir | Component::Prefix(_) | Component::RootDir
        )
    }) {
        return Err("Relative executable paths cannot leave the selected project".into());
    }
    if path.components().count() == 1 && !name.contains(['/', '\\']) {
        return discover(root, name);
    }
    let actual = executable(&root.join(path))?;
    if !actual.starts_with(root) {
        return Err("Relative executable resolves outside the selected project".into());
    }
    Ok(actual)
}

pub(super) fn discover(root: &Path, name: &str) -> Result<PathBuf, String> {
    #[cfg(windows)]
    let name = name
        .rsplit_once('.')
        .filter(|(_, ext)| ext.eq_ignore_ascii_case("exe"))
        .map_or(name, |(name, _)| name);
    let path = tooling::find_native(name, Some(root))
        .ok_or_else(|| format!("Required native executable {name} is unavailable"))?;
    executable(&path)
}

pub(super) fn selected(root: &Path, selection: &Selection, tool: &str) -> Result<PathBuf, String> {
    let pin = selection.get(tool).map(String::as_str);
    if let Some(version) = pin
        && let Some(path) = tooling::resolved(root, tool, version)?
    {
        return executable(&path);
    }
    let name = match tool {
        "rust" => "rustc",
        "python" if cfg!(unix) => "python3",
        _ => tool,
    };
    let mut path = discover(root, name)?;
    if tool == "rust"
        && tooling::find_native("rustup", Some(root))
            .and_then(|shim| shim.canonicalize().ok())
            .is_some_and(|shim| shim.parent() == path.parent())
    {
        path = installed_rustup(pin)?;
    }
    if let Some(version) = pin {
        verify_version(&path, &[], version, root)
            .map_err(|reason| format!("Selected {tool}@{version} is unavailable. {reason}"))?;
    }
    Ok(path)
}

pub(super) fn verify_version(
    program: &Path,
    prefix: &[OsString],
    expected: &str,
    root: &Path,
) -> Result<(), String> {
    let mut probe = command(program, program.parent().unwrap_or(root), &[program], root);
    probe.args(prefix).arg("--version");
    let result = run_native(&mut probe, 5000)?;
    let stream = if result.output.is_empty() {
        &result.error
    } else {
        &result.output
    };
    let matches = result.success()
        && !result.output_truncated
        && String::from_utf8_lossy(stream)
            .lines()
            .next()
            .is_some_and(|line| {
                let mut words = line.split_ascii_whitespace();
                match words.next() {
                    Some("rustc" | "Python" | "uv" | "node" | "ninja") => {
                        words.next() == Some(expected)
                    }
                    Some("cmake") => {
                        words.next() == Some("version") && words.next() == Some(expected)
                    }
                    Some(version) => version.strip_prefix('v').unwrap_or(version) == expected,
                    None => false,
                }
            });
    if !matches {
        return Err("Installed executable does not report the exact selected version".into());
    }
    Ok(())
}

fn installed_rustup(pin: Option<&str>) -> Result<PathBuf, String> {
    let home = env::var_os("RUSTUP_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
                .map(|home| PathBuf::from(home).join(".rustup"))
        })
        .filter(|home| home.is_absolute())
        .ok_or("Rustup home is unavailable")?;
    let mut config = String::new();
    File::open(home.join("settings.toml"))
        .and_then(|file| file.take(65537).read_to_string(&mut config))
        .map_err(|_| "Cannot read local Rustup settings")?;
    if config.len() > 65536 {
        return Err("Local Rustup settings exceed the size limit".into());
    }
    let config = config
        .parse::<toml::Table>()
        .map_err(|_| "Invalid local Rustup settings")?;
    let default = config
        .get("default_toolchain")
        .and_then(toml::Value::as_str);
    let host = config
        .get("default_host_triple")
        .and_then(toml::Value::as_str)
        .or_else(|| {
            default
                .and_then(|value| value.split_once('-'))
                .map(|(_, host)| host)
        });
    let directory = if let Some(version) = pin {
        let host = host.ok_or("Rustup has no configured host triple")?;
        format!("{version}-{host}")
    } else {
        default
            .ok_or("Rustup has no installed default selection")?
            .to_owned()
    };
    if directory.is_empty()
        || !directory
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_.".contains(&byte))
        || matches!(directory.as_str(), "." | "..")
    {
        return Err("Unsafe local Rustup toolchain identifier".into());
    }
    executable(
        &home
            .join("toolchains")
            .join(directory)
            .join("bin")
            .join(exe_name("rustc")),
    )
    .map_err(|_| "The selected Rust toolchain is not installed locally".into())
}

pub(super) fn sibling(program: &Path, name: &str) -> Result<PathBuf, String> {
    executable(
        &program
            .parent()
            .ok_or("Executable has no parent directory")?
            .join(exe_name(name)),
    )
    .map_err(|_| format!("{name} is unavailable beside the selected executable"))
}

pub(super) fn npm_cli(root: &Path, node: &Path, pinned: bool) -> Result<PathBuf, String> {
    let bin = node
        .parent()
        .ok_or("Node executable has no parent directory")?;
    for entry in [
        "node_modules/npm/bin/npm-cli.js",
        "../lib/node_modules/npm/bin/npm-cli.js",
        "../share/nodejs/npm/bin/npm-cli.js",
    ] {
        if let Ok(path) = bin.join(entry).canonicalize()
            && path.is_file()
        {
            return Ok(path);
        }
    }
    if !pinned
        && let Some(path) = tooling::find_native("npm", Some(root))
        && path.file_name() == Some(OsStr::new("npm-cli.js"))
    {
        return Ok(path);
    }
    Err("The selected Node installation has no native npm JavaScript entrypoint".into())
}

fn exe_name(name: &str) -> String {
    format!("{name}{}", env::consts::EXE_SUFFIX)
}

/// Some native tools treat a Windows verbatim drive prefix as a distinct
/// volume and cannot walk parent directories correctly.
pub(super) fn provider_path(path: &Path) -> OsString {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        if let Some(Component::Prefix(prefix)) = path.components().next()
            && matches!(prefix.kind(), std::path::Prefix::VerbatimDisk(_))
        {
            let path: Vec<_> = path.as_os_str().encode_wide().collect();
            return OsString::from_wide(&path[4..]);
        }
    }
    path.as_os_str().to_owned()
}

pub(super) fn command(program: &Path, cwd: &Path, prepend: &[&Path], root: &Path) -> Command {
    let mut command = Command::new(program);
    command.current_dir(cwd).env_clear();
    for name in [
        "HOME",
        "USERPROFILE",
        "LOCALAPPDATA",
        "APPDATA",
        "SYSTEMROOT",
        "SYSTEMDRIVE",
        "WINDIR",
        "COMSPEC",
        "PATHEXT",
        "TEMP",
        "TMP",
        "TMPDIR",
        "LANG",
        "LC_ALL",
        "TERM",
        "NO_COLOR",
        "INCLUDE",
        "LIB",
        "LIBPATH",
        "VCINSTALLDIR",
        "VCToolsInstallDir",
        "WindowsSdkDir",
        "WindowsSDKVersion",
        "UniversalCRTSdkDir",
        "UCRTVersion",
        "CMAKE_PREFIX_PATH",
    ] {
        if let Some(value) = env::var_os(name) {
            command.env(name, value);
        }
    }
    let mut paths = Vec::new();
    for path in prepend.iter().filter_map(|path| path.parent()) {
        if !paths.iter().any(|other| other == path) {
            paths.push(path.to_path_buf());
        }
    }
    if let Some(path) = env::var_os("PATH") {
        paths.extend(env::split_paths(&path).take(256).filter_map(|path| {
            if !path.is_absolute() {
                return None;
            }
            let path = path.canonicalize().ok()?;
            (path.is_dir() && !path.starts_with(root)).then_some(path)
        }));
    }
    if let Ok(path) = env::join_paths(paths.iter().map(|path| provider_path(path))) {
        command.env("PATH", path);
    }
    command.env("RUSTUP_AUTO_INSTALL", "0");
    command
}
