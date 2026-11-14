use crate::{docs, workspace};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use super::process::run_process;

const META_BYTES: usize = 2 * 1024 * 1024;
const FILE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 64 * 1024 * 1024;

pub(super) fn safe_relative(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 512
        && !path.contains(['\\', ':'])
        && !path.chars().any(char::is_control)
        && path.split('/').all(|part| {
            !matches!(part, "" | "." | "..")
                && !part.ends_with([' ', '.'])
                && !part.eq_ignore_ascii_case(".git")
                && !part.eq_ignore_ascii_case(".cirnetrules")
                && !part.eq_ignore_ascii_case(".openrules")
                && !part.eq_ignore_ascii_case(".cgacp")
        })
}

pub(super) fn sensitive(path: &str) -> bool {
    path.split('/').any(|part| {
        let name = part.to_ascii_lowercase();
        name == ".env"
            || name.starts_with(".env.")
            || name.ends_with(".pem")
            || name.ends_with(".key")
            || matches!(
                name.as_str(),
                ".netrc" | ".npmrc" | ".ssh" | "credentials.json" | "id_rsa" | "id_ed25519"
            )
    })
}

pub(super) fn is_boundary(path: &str) -> bool {
    let path = path.to_ascii_lowercase();
    path.split('/').any(|part| {
        matches!(
            part,
            ".github"
                | "cargo.lock"
                | "package-lock.json"
                | "pnpm-lock.yaml"
                | "dockerfile"
                | "build.rs"
                | "integration"
                | "deployment"
        ) || part.split(['.', '-', '_']).any(|word| {
            matches!(
                word,
                "auth"
                    | "authentication"
                    | "authorization"
                    | "permissions"
                    | "permission"
                    | "api"
                    | "run"
                    | "stop"
                    | "safety"
                    | "rpc"
                    | "serializer"
                    | "schema"
                    | "deploy"
            )
        })
    })
}

pub(super) fn boundary_signals(bytes: &[u8]) -> Option<String> {
    let source = std::str::from_utf8(bytes).ok()?;
    let mut digest = Sha256::new();
    let mut found = false;
    for line in source.lines().map(str::trim) {
        let lower = line.to_ascii_lowercase();
        if [
            "fetch(",
            "api.",
            "httpclient",
            "authorization",
            "authenticate(",
            "authorize(",
            "permission.",
            "session.token",
            "cookie=",
            "grpc.",
            "rpc(",
            "websocket(",
            "startrun(",
            "stoprun(",
            "startexperiment(",
            "stopexperiment(",
            "runexperiment(",
            "safetygate",
            "safe_state",
            "hardware.enable(",
        ]
        .iter()
        .any(|token| lower.contains(token))
        {
            digest.update((line.len() as u64).to_le_bytes());
            digest.update(line.as_bytes());
            found = true;
        }
    }
    found.then(|| format!("{:x}", digest.finalize()))
}

pub(super) fn parse_dirty(bytes: &[u8]) -> Result<BTreeMap<String, String>, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "Git status is not UTF-8")?;
    if !text.is_empty() && !text.ends_with('\0') {
        return Err("Incomplete Git status".into());
    }
    let mut paths = BTreeMap::new();
    for entry in text.split_terminator('\0') {
        let status = entry.get(..2).ok_or("Invalid Git status")?;
        let path = entry.get(3..).ok_or("Invalid Git status")?;
        if entry.as_bytes().get(2) != Some(&b' ')
            || !status.bytes().all(|byte| b" MADRCU?!T".contains(&byte))
            || status.contains(['R', 'C'])
            || !safe_relative(path)
            || paths.len() >= 4096
            || paths.insert(path.to_owned(), status.to_owned()).is_some()
        {
            return Err("Unreviewed, duplicate or oversized Git path metadata".into());
        }
    }
    Ok(paths)
}

pub(crate) fn git_executable(governance: &Path, checkout: &Path) -> Result<PathBuf, String> {
    let path = env::var_os("PATH").ok_or("Git is unavailable in the approved environment")?;
    let name = if cfg!(windows) { "git.exe" } else { "git" };
    let governance = governance
        .canonicalize()
        .map_err(|_| "Governance checkout is unavailable")?;
    let checkout = checkout
        .canonicalize()
        .map_err(|_| "Git checkout is unavailable")?;
    let workspace = governance.parent().unwrap_or(&governance);
    let product_workspace = checkout.parent().unwrap_or(&checkout);
    for directory in env::split_paths(&path).filter(|dir| dir.is_absolute()) {
        if let Ok(candidate) = directory.join(name).canonicalize()
            && !candidate.starts_with(workspace)
            && !candidate.starts_with(product_workspace)
            && candidate.is_file()
        {
            return Ok(candidate);
        }
    }
    Err("No trusted Git executable exists outside the workspace".into())
}

pub(super) fn git_command(governance: &Path, checkout: &Path) -> Result<Command, String> {
    let mut command = Command::new(git_executable(governance, checkout)?);
    command
        .env_clear()
        .current_dir(checkout)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_ATTR_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .args([
            "--no-pager",
            "--no-lazy-fetch",
            "--no-optional-locks",
            "--no-replace-objects",
            "--literal-pathspecs",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "status.submoduleSummary=false",
        ]);
    if cfg!(windows) {
        for key in ["SystemRoot", "WINDIR"] {
            if let Some(value) = env::var_os(key) {
                command.env(key, value);
            }
        }
    }
    Ok(command)
}

fn metadata(command: &mut Command) -> Result<Vec<u8>, String> {
    let output = run_process(command, Duration::from_secs(5), META_BYTES, None)?;
    if !output.exit.success() || output.timed_out || output.output.truncated {
        return Err("Required Git metadata is unavailable or exceeds the resource bound".into());
    }
    Ok(output.output.prefix)
}

pub(super) fn git_version(governance: &Path, checkout: &Path) -> Result<String, String> {
    let mut command = git_command(governance, checkout)?;
    let output = metadata(command.arg("--version"))?;
    let output = std::str::from_utf8(&output)
        .map_err(|_| "Invalid Git version output")?
        .trim();
    let token = output
        .strip_prefix("git version ")
        .and_then(|value| value.split_whitespace().next())
        .filter(|version| {
            version.len() <= 48
                && version.as_bytes().first().is_some_and(u8::is_ascii_digit)
                && version
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b".-+".contains(&byte))
        });
    let Some(token) = token else {
        return Err("Unreviewed Git version identity".into());
    };
    Ok(format!("git version {token}"))
}

pub(super) fn system_autocrlf(mut config: Command) -> Result<Option<&'static str>, String> {
    config.args([
        "config",
        "--system",
        "--no-includes",
        "--type=bool-or-str",
        "--get",
        "core.autocrlf",
    ]);
    let output = run_process(&mut config, Duration::from_secs(5), 64, None)?;
    if output.timed_out || output.output.truncated || output.error.truncated {
        return Err("System Git line ending configuration exceeds the resource bound".into());
    }
    if output.exit.code() == Some(1)
        && output.output.prefix.is_empty()
        && output.error.prefix.is_empty()
    {
        return Ok(None);
    }
    if !output.exit.success() {
        return Err("Cannot read system Git line ending configuration".into());
    }
    let value = std::str::from_utf8(&output.output.prefix)
        .map_err(|_| "Invalid system Git line ending configuration")?
        .trim_end_matches(['\r', '\n']);
    ["true", "false", "input"]
        .into_iter()
        .find(|allowed| value.eq_ignore_ascii_case(allowed))
        .map(Some)
        .ok_or_else(|| "Unsupported system Git line ending configuration".into())
}

pub(super) fn no_filters(governance: &Path, checkout: &Path) -> Result<Command, String> {
    let mut config = git_command(governance, checkout)?;
    let keys = metadata(config.args(["config", "--list", "--name-only", "--null"]))?;
    let keys = std::str::from_utf8(&keys).map_err(|_| "Invalid Git filter keys")?;
    let mut command = git_command(governance, checkout)?;
    // Git for Windows can set built-in CRLF conversion in its installer config.
    // Keep system config disabled during Git operations and replay only this setting.
    if cfg!(windows)
        && !keys
            .split_terminator('\0')
            .any(|key| key.eq_ignore_ascii_case("core.autocrlf"))
        && let Some(value) = system_autocrlf(git_command(governance, checkout)?)?
    {
        command.args(["-c", &format!("core.autocrlf={value}")]);
    }
    let mut filters = 0;
    for key in keys.split_terminator('\0').collect::<BTreeSet<_>>() {
        if !key.starts_with("filter.") {
            continue;
        }
        let value = match key.rsplit('.').next() {
            Some("clean" | "process" | "smudge") => "",
            Some("required") => "false",
            _ => continue,
        };
        filters += 1;
        if filters > 32 || key.len() > 160 || key.chars().any(char::is_control) {
            return Err("Too many or invalid configured Git filters".into());
        }
        command.args(["-c", &format!("{key}={value}")]);
    }
    Ok(command)
}

pub(super) struct GitState {
    pub dirty: BTreeMap<String, String>,
    index: Vec<u8>,
}

pub(super) fn git_state(governance: &Path, checkout: &Path) -> Result<GitState, String> {
    let mut index = git_command(governance, checkout)?;
    let index = metadata(index.args(["ls-files", "--stage", "-z"]))?;
    if index
        .split(|byte| *byte == 0)
        .any(|entry| entry.starts_with(b"160000 "))
    {
        return Err("A Git submodule requires a separately approved worktree guard".into());
    }
    let mut command = no_filters(governance, checkout)?;
    let output = metadata(command.args([
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=all",
        "--no-renames",
        "--ignore-submodules=all",
    ]))?;
    Ok(GitState {
        dirty: parse_dirty(&output)?,
        index,
    })
}

pub(crate) fn bounded_git_info(root: &Path, args: &[&str]) -> Result<String, String> {
    let mut command = git_command(root, root)?;
    let bytes = metadata(command.args(args))?;
    String::from_utf8(bytes).map_err(|_| "Required Git identity is not UTF-8".into())
}

pub(crate) fn committed_blob(
    governance: &Path,
    checkout: &Path,
    oid: &str,
    byte_limit: usize,
) -> Result<Vec<u8>, String> {
    if !matches!(oid.len(), 40 | 64)
        || !oid
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || byte_limit == 0
        || byte_limit > FILE_BYTES as usize
    {
        return Err("Invalid committed blob identity or budget".into());
    }
    let mut command = git_command(governance, checkout)?;
    command.args(["cat-file", "blob", oid]);
    let outcome = run_process(&mut command, Duration::from_secs(5), byte_limit, None)?;
    if !outcome.exit.success() || outcome.timed_out || outcome.output.truncated {
        return Err("Committed blob is unavailable or exceeds the resource bound".into());
    }
    Ok(outcome.output.prefix)
}

pub(crate) fn committed_diff_check(
    governance: &Path,
    checkout: &Path,
    commits: (&str, &str),
    paths: &[&str],
    timeout: Duration,
    max_output_bytes: usize,
) -> Result<Option<bool>, String> {
    let mut command = git_command(governance, checkout)?;
    command.args([
        "diff",
        "--check",
        "--no-ext-diff",
        "--no-textconv",
        commits.0,
        commits.1,
        "--",
    ]);
    command.args(paths);
    let output = run_process(&mut command, timeout, max_output_bytes, None)?;
    if output.timed_out || output.output.truncated {
        return Ok(None);
    }
    Ok(Some(output.exit.success()))
}

pub(crate) fn guarded_git_clean(governance: &Path, checkout: &Path) -> Result<bool, String> {
    Ok(git_state(governance, checkout)?.dirty.is_empty())
}

pub(super) fn load_input(checkout: &Path, path: &str) -> Result<Option<Vec<u8>>, String> {
    if !safe_relative(path) || sensitive(path) {
        return Err("Source path is unsafe or protected".into());
    }
    let resolved = workspace::scoped_path(checkout, path)?;
    let meta = match fs::symlink_metadata(&resolved) {
        Ok(meta)
            if meta.is_file() && !crate::policy::is_link(&meta) && meta.len() <= FILE_BYTES =>
        {
            meta
        }
        Ok(_) => {
            return Err(format!(
                "Selected source is not a bounded regular file: {path}"
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(format!("Cannot access selected source: {path}")),
    };
    let mut bytes = Vec::with_capacity(meta.len() as usize);
    fs::File::open(&resolved)
        .and_then(|file| file.take(FILE_BYTES + 1).read_to_end(&mut bytes))
        .map_err(|_| format!("Cannot read selected source: {path}"))?;
    if bytes.len() as u64 > FILE_BYTES {
        return Err(format!(
            "Selected source exceeds the resource bound: {path}"
        ));
    }
    Ok(Some(bytes))
}

pub(super) fn capture(
    root: &Path,
    task: &Value,
    registry: &Value,
    workspace: Option<&Path>,
) -> Result<Value, String> {
    let descriptor = &task["descriptor"];
    let mut selected = BTreeMap::<&str, BTreeSet<&str>>::new();
    for key in ["read_scope", "write_scope", "verification_scope"] {
        for (repo, paths) in descriptor[key].as_object().into_iter().flatten() {
            let repo_paths = selected.entry(repo).or_default();
            for path in paths.as_array().into_iter().flatten() {
                repo_paths.insert(path.as_str().ok_or("Invalid scoped file")?);
            }
        }
    }
    if selected.len() > 16 || selected.values().map(BTreeSet::len).sum::<usize>() > 256 {
        return Err("Task snapshot exceeds the scope bound".into());
    }
    let mut repositories = Vec::new();
    let mut content_bytes = 0_u64;
    let mut content_files = 0_usize;
    for (repo, paths) in selected {
        let binding = workspace::resolve_task(root, workspace, repo, registry, task)?;
        let metadata = git_state(root, &binding.path)?;
        let mut staged = BTreeMap::<&str, Vec<&str>>::new();
        for entry in std::str::from_utf8(&metadata.index)
            .map_err(|_| "Git index paths are not UTF-8")?
            .split_terminator('\0')
        {
            let (identity, path) = entry.split_once('\t').ok_or("Invalid Git index metadata")?;
            if paths.contains(path) {
                staged.entry(path).or_default().push(identity);
            }
        }
        let mut inputs = Map::new();
        for path in paths {
            let mut fingerprint = match load_input(&binding.path, path)? {
                Some(bytes) => {
                    content_bytes += bytes.len() as u64;
                    content_files += 1;
                    if content_bytes > MAX_TOTAL_BYTES {
                        return Err("Task snapshot exceeds its content byte bound".into());
                    }
                    json!({
                        "sha256":docs::sha256(&bytes), "bytes":bytes.len(),
                        "boundary_signals":boundary_signals(&bytes)
                    })
                }
                None => json!({"missing":true}),
            };
            fingerprint["index"] = json!(staged.get(path));
            inputs.insert(path.to_owned(), fingerprint);
        }
        repositories.push(json!({
            "repo":repo,"identity":binding.identity,"commit":binding.commit,
            "dirty":metadata.dirty,"inputs":inputs
        }));
    }
    Ok(json!({
        "repositories":repositories, "content_files":content_files,
        "content_bytes":content_bytes
    }))
}
