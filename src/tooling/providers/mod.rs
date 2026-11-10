use crate::execution::{NativeResult, run_native};
use std::{
    env,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    process::Command,
};

use super::{catalog::Tool, store};

pub(super) fn native_name(name: &str) -> OsString {
    if cfg!(windows) {
        format!("{name}.exe").into()
    } else {
        name.into()
    }
}

pub(super) fn trusted_path(project: Option<&Path>) -> Vec<PathBuf> {
    env::var_os("PATH")
        .into_iter()
        .flat_map(|value| env::split_paths(&value).collect::<Vec<_>>())
        .filter(|directory| directory.is_absolute())
        .filter_map(|directory| directory.canonicalize().ok())
        .filter(|directory| !project.is_some_and(|root| directory.starts_with(root)))
        .take(256)
        .collect()
}

pub(super) fn find(name: &str, project: Option<&Path>) -> Option<PathBuf> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return None;
    }
    trusted_path(project).into_iter().find_map(|directory| {
        #[cfg(windows)]
        if directory
            .components()
            .any(|part| part.as_os_str().eq_ignore_ascii_case("WindowsApps"))
        {
            // App execution aliases may launch the Store instead of reporting a version.
            return None;
        }
        let file = directory.join(native_name(name)).canonicalize().ok()?;
        (file.is_file()
            && !project.is_some_and(|root| file.starts_with(root))
            && !cfg!(windows)
                .then(|| file.extension())
                .flatten()
                .is_some_and(|ext| !ext.eq_ignore_ascii_case("exe")))
        .then_some(file)
    })
}

pub(super) fn command(
    binary: &Path,
    cwd: &Path,
    project: Option<&Path>,
    private_home: Option<&Path>,
) -> Command {
    let mut command = Command::new(binary);
    command.env_clear().current_dir(cwd);
    if let Ok(path) = env::join_paths(trusted_path(project)) {
        command.env("PATH", path);
    }
    for name in [
        "SystemRoot",
        "WINDIR",
        "COMSPEC",
        "PATHEXT",
        "PROCESSOR_ARCHITECTURE",
    ] {
        if let Some(value) = env::var_os(name) {
            command.env(name, value);
        }
    }
    if let Some(home) = private_home {
        command
            .env("HOME", home)
            .env("USERPROFILE", home)
            .env("LOCALAPPDATA", home)
            .env("APPDATA", home)
            .env("XDG_DATA_HOME", home)
            .env("XDG_CONFIG_HOME", home)
            .env("XDG_CACHE_HOME", home)
            .env("TEMP", home)
            .env("TMP", home)
            .env("TMPDIR", home);
    }
    command
        .env("NO_COLOR", "1")
        .env("RUSTUP_AUTO_INSTALL", "0")
        .env("PYTHONNOUSERSITE", "1")
        .env("PYTHONSAFEPATH", "1");
    command
}

pub(super) fn run(command: &mut Command, timeout_ms: u64) -> Result<NativeResult, String> {
    run_native(command, timeout_ms)
}

fn require_success(result: &NativeResult, what: &str) -> Result<(), String> {
    if result.success() {
        Ok(())
    } else if result.cancelled {
        Err(format!("{what} was cancelled."))
    } else if result.timed_out {
        Err(format!("{what} exceeded its time limit."))
    } else {
        Err(format!(
            "{what} failed (child exit code {}).",
            result
                .exit_code
                .map_or_else(|| "unavailable".to_owned(), |code| code.to_string())
        ))
    }
}

pub(super) fn version_matches(
    binary: &Path,
    name: &str,
    version: &str,
    cwd: &Path,
    project: &Path,
) -> Result<(), String> {
    let mut cmd = command(binary, cwd, Some(project), None);
    if name == "python" {
        cmd.arg("-I");
    }
    let result = run(cmd.arg("--version"), 5000)?;
    require_success(&result, "Executable version check")?;
    if result.output_truncated {
        return Err("Executable version evidence is truncated".into());
    }
    let output = String::from_utf8_lossy(if result.output.is_empty() {
        &result.error
    } else {
        &result.output
    });
    let token = match name {
        "python" => output
            .trim()
            .strip_prefix("Python ")
            .and_then(|rest| rest.split_whitespace().next()),
        "rust" => output
            .trim()
            .strip_prefix("rustc ")
            .and_then(|rest| rest.split_whitespace().next()),
        "node" => output.trim().strip_prefix('v'),
        _ => None,
    };
    if token != Some(version) {
        return Err("The installed executable did not report the exact requested version".into());
    }
    Ok(())
}

fn rustup(target: &Path, project: &Path, rustup: &Path, version: &str) -> Result<PathBuf, String> {
    let mut command = command(rustup, target, Some(project), Some(target));
    command
        .env("RUSTUP_HOME", target.join("rustup"))
        .env("CARGO_HOME", target.join("cargo"))
        .args([
            "toolchain",
            "install",
            version,
            "--profile",
            "minimal",
            "--no-self-update",
            "--no-update",
        ]);
    require_success(&run(&mut command, 300_000)?, "Rust toolchain installation")?;
    let mut command = self::command(rustup, target, Some(project), Some(target));
    command
        .env("RUSTUP_HOME", target.join("rustup"))
        .env("CARGO_HOME", target.join("cargo"))
        .args(["which", "--toolchain", version, "rustc"]);
    executable_output(target, run(&mut command, 5000)?)
}

fn executable_output(target: &Path, result: NativeResult) -> Result<PathBuf, String> {
    require_success(&result, "Provider executable lookup")?;
    if result.output_truncated {
        return Err("Provider executable lookup exceeded its output budget".into());
    }
    let stdout = std::str::from_utf8(&result.output)
        .map_err(|_| "Provider returned a non-UTF-8 executable path")?
        .trim();
    if stdout.is_empty() || stdout.lines().count() != 1 {
        return Err("Provider did not return one executable path".into());
    }
    let path = Path::new(stdout);
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        target.join(path)
    };
    store::checked_executable(target, &path)
}

fn python(
    target: &Path,
    project: &Path,
    uv: &Path,
    cache: &Path,
    version: &str,
) -> Result<PathBuf, String> {
    let mut command = command(uv, target, Some(project), Some(target));
    command
        .env("UV_PYTHON_INSTALL_DIR", target.join("python"))
        .env("UV_PYTHON_BIN_DIR", target.join("bin"))
        .env("UV_CACHE_DIR", cache)
        .args([
            "--no-config",
            "--no-progress",
            "python",
            "install",
            "--no-bin",
            "--no-registry",
            version,
        ]);
    require_success(&run(&mut command, 300_000)?, "Python installation")?;
    let mut command = self::command(uv, target, Some(project), Some(target));
    command
        .env("UV_PYTHON_INSTALL_DIR", target.join("python"))
        .env("UV_CACHE_DIR", cache)
        .args([
            "--offline",
            "--no-config",
            "python",
            "find",
            "--no-project",
            "--managed-python",
            "--no-python-downloads",
            "--resolve-links",
            version,
        ]);
    executable_output(target, run(&mut command, 5000)?)
}

fn node(target: &Path, project: &Path, fnm: &Path, version: &str) -> Result<PathBuf, String> {
    let managed = target.join("fnm");
    let mut command = command(fnm, target, Some(project), Some(target));
    command
        .args([OsStr::new("--fnm-dir"), managed.as_os_str()])
        .args(["install", "--progress", "never", version]);
    require_success(&run(&mut command, 300_000)?, "Node.js installation")?;
    let prefix = managed
        .join("node-versions")
        .join(format!("v{version}"))
        .join("installation");
    let bin = if cfg!(windows) {
        prefix
    } else {
        prefix.join("bin")
    };
    store::checked_executable(target, &bin.join(native_name("node")))
}

pub(super) fn install(
    tool: &Tool,
    version: &str,
    binary: &Path,
    target: &Path,
    project: &Path,
    cache: Option<&Path>,
) -> Result<PathBuf, String> {
    let path = match tool.id {
        "rust" => rustup(target, project, binary, version),
        "python" => python(
            target,
            project,
            binary,
            cache.ok_or("Python provider cache is missing")?,
            version,
        ),
        "node" => node(target, project, binary, version),
        _ => return Err("No installation adapter for this tool".into()),
    }?;
    version_matches(&path, tool.id, version, target, project)?;
    Ok(path)
}

pub(super) fn venv(
    uv: &Path,
    python: &Path,
    target: &Path,
    root: &Path,
    cache: &Path,
) -> Result<PathBuf, String> {
    let venv = target.join("env");
    let mut command = command(uv, target, Some(root), Some(target));
    command
        .env("UV_PYTHON_INSTALL_DIR", target.join("python"))
        .env("UV_CACHE_DIR", cache)
        .args([
            "--offline",
            "--no-config",
            "--no-progress",
            "venv",
            "--no-project",
            "--no-python-downloads",
        ])
        .arg("--python")
        .arg(python)
        .arg(&venv);
    require_success(&run(&mut command, 60_000)?, "Virtual environment creation")?;
    Ok(venv
        .join(if cfg!(windows) { "Scripts" } else { "bin" })
        .join(native_name("python")))
}

pub(super) fn native_version(path: &Path, name: &str, project: Option<&Path>) -> Option<String> {
    let cwd = path.parent()?;
    let mut command = command(path, cwd, project, None);
    let args: &[&str] = match name {
        "python" => &["-I", "--version"],
        "qt" if path
            .file_stem()
            .is_some_and(|stem| stem.to_string_lossy().starts_with("qmake")) =>
        {
            &["-query", "QT_VERSION"]
        }
        "qt" => &["--qt-version"],
        "msvc" => &["/?"],
        _ => &["--version"],
    };
    let result = run(command.args(args), 1800).ok()?;
    if result.timed_out
        || result.cancelled
        || result.output_truncated
        || (!result.success() && name != "msvc")
    {
        return None;
    }
    let output = String::from_utf8_lossy(if result.output.is_empty() {
        &result.error
    } else {
        &result.output
    });
    output
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| line.chars().filter(|c| !c.is_control()).take(160).collect())
}

#[cfg(windows)]
fn native_at(path: &Path, program: &str) -> Option<PathBuf> {
    let path = path.join(native_name(program)).canonicalize().ok()?;
    path.is_file().then_some(path)
}

#[cfg(windows)]
fn windows_sdk() -> Option<(PathBuf, String)> {
    let base = env::var_os("ProgramFiles(x86)").or_else(|| env::var_os("ProgramFiles"))?;
    let include = PathBuf::from(base)
        .join("Windows Kits")
        .join("10")
        .join("Include");
    let mut versions = Vec::new();
    for path in store::entries(&include).ok()? {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Ok(numbers) = name
            .split('.')
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>()
        else {
            continue;
        };
        if numbers.len() == 4
            && path.join("um").join("Windows.h").is_file()
            && path.join("ucrt").join("stdio.h").is_file()
        {
            versions.push((numbers, path.clone(), name.to_owned()));
        }
    }
    versions
        .into_iter()
        .max_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, path, version)| (path, version))
}

#[cfg(windows)]
fn msvc(project: Option<&Path>) -> Option<PathBuf> {
    let architecture = match env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        _ => "x86",
    };
    let host = match architecture {
        "x64" => "Hostx64",
        "arm64" => "Hostarm64",
        _ => "Hostx86",
    };
    let locate = |tools: PathBuf| native_at(&tools.join("bin").join(host).join(architecture), "cl");
    if let Some(path) = env::var_os("VCToolsInstallDir").and_then(|tools| locate(tools.into())) {
        return Some(path);
    }
    let base = PathBuf::from(env::var_os("ProgramFiles(x86)")?);
    let vswhere = native_at(
        &base.join("Microsoft Visual Studio").join("Installer"),
        "vswhere",
    )?;
    let mut command = command(&vswhere, vswhere.parent()?, project, None);
    let out = run(
        command.args([
            "-latest",
            "-products",
            "*",
            "-requires",
            "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
            "-property",
            "installationPath",
            "-utf8",
        ]),
        1800,
    )
    .ok()?;
    if !out.success() || out.output_truncated {
        return None;
    }
    let root = PathBuf::from(std::str::from_utf8(&out.output).ok()?.trim());
    if !root.is_absolute() {
        return None;
    }
    let version_file = root
        .join("VC")
        .join("Auxiliary")
        .join("Build")
        .join("Microsoft.VCToolsVersion.default.txt");
    if std::fs::metadata(&version_file).ok()?.len() > 128 {
        return None;
    }
    let version = std::fs::read_to_string(version_file).ok()?;
    let version = version.trim();
    if version.is_empty() || !version.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return None;
    }
    locate(root.join("VC").join("Tools").join("MSVC").join(version))
}

pub(super) fn discover(tool: &Tool, project: Option<&Path>) -> Option<(PathBuf, Option<String>)> {
    #[cfg(windows)]
    if tool.id == "windows-sdk" {
        return windows_sdk().map(|(path, version)| (path, Some(version)));
    }
    #[cfg(windows)]
    let msvc = || (tool.id == "msvc").then(|| msvc(project)).flatten();
    #[cfg(not(windows))]
    let msvc = || None;
    if matches!(tool.id, "rust" | "cargo") && find("rustup", project).is_some() {
        return installed_rust(tool.programs[0], project);
    }
    let binary = tool
        .programs
        .iter()
        .find_map(|name| find(name, project))
        .or_else(msvc)
        .or_else(|| match tool.id {
            "qt" => qt(tool.programs, project),
            #[cfg(windows)]
            "ninja" => native_at(&windows_qt_root()?.join("Tools").join("Ninja"), "ninja"),
            _ => None,
        })
        .filter(|path| !project.is_some_and(|root| path.starts_with(root)))?;
    let version = native_version(&binary, tool.id, project);
    Some((binary, version))
}

fn installed_rust(program: &str, project: Option<&Path>) -> Option<(PathBuf, Option<String>)> {
    // Rustup proxies may auto-install a pinned toolchain. Inspect the manager
    // state as data and probe only the installed executable.
    let home = env::var_os("RUSTUP_HOME").map(PathBuf::from).or_else(|| {
        let variable = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
        env::var_os(variable).map(|home| PathBuf::from(home).join(".rustup"))
    })?;
    if !home.is_absolute() {
        return None;
    }
    let lookup = |directory: &Path| {
        let path = directory
            .join("bin")
            .join(native_name(program))
            .canonicalize()
            .ok()?;
        (path.is_file() && !project.is_some_and(|root| path.starts_with(root))).then_some(path)
    };
    let toolchains = home.join("toolchains");
    let default = (|| {
        let path = home.join("settings.toml");
        let metadata = std::fs::symlink_metadata(&path).ok()?;
        if !metadata.is_file() || !store::ordinary(&metadata) || metadata.len() > 16 * 1024 {
            return None;
        }
        let config: toml::Value = toml::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
        let name = config.get("default_toolchain")?.as_str()?;
        if name.is_empty()
            || !name.as_bytes()[0].is_ascii_alphanumeric()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b".-_".contains(&byte))
        {
            return None;
        }
        lookup(&toolchains.join(name))
    })();
    let path = default.or_else(|| {
        store::entries(&toolchains)
            .ok()?
            .into_iter()
            .rev()
            .find_map(|entry| lookup(&entry))
    })?;
    let version = native_version(&path, program, project);
    Some((path, version))
}

fn qt(programs: &[&str], project: Option<&Path>) -> Option<PathBuf> {
    let lookup = |root: &Path| {
        if !root.is_absolute() {
            return None;
        }
        programs.iter().find_map(|program| {
            let path = root
                .join("bin")
                .join(native_name(program))
                .canonicalize()
                .ok()?;
            (path.is_file() && !project.is_some_and(|root| path.starts_with(root))).then_some(path)
        })
    };
    if let Some(path) = env::var_os("QTDIR")
        .as_deref()
        .and_then(|path| lookup(Path::new(path)))
    {
        return Some(path);
    }
    if let Some(prefixes) = env::var_os("CMAKE_PREFIX_PATH")
        && let Some(path) = env::split_paths(&prefixes)
            .take(64)
            .find_map(|prefix| lookup(&prefix))
    {
        return Some(path);
    }
    #[cfg(windows)]
    {
        let mut versions: Vec<_> = store::entries(&windows_qt_root()?)
            .ok()?
            .into_iter()
            .filter_map(|path| {
                let version = path
                    .file_name()?
                    .to_str()?
                    .split('.')
                    .map(str::parse::<u32>)
                    .collect::<Result<Vec<_>, _>>()
                    .ok()?;
                (version.len() == 3 && path.is_dir()).then_some((version, path))
            })
            .collect();
        versions.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        for (_, path) in versions {
            let kits = store::entries(&path).ok()?;
            if let Some(binary) = kits.into_iter().rev().find_map(|kit| lookup(&kit)) {
                return Some(binary);
            }
        }
    }
    None
}

#[cfg(windows)]
fn windows_qt_root() -> Option<PathBuf> {
    let drive = env::var_os("SystemDrive")?;
    let mut root = drive;
    root.push("\\Qt");
    let root = PathBuf::from(root);
    root.is_absolute().then_some(root)
}
