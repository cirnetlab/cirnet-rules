mod patch;
mod registration;

pub(crate) use registration::audit;

#[cfg(test)]
pub(crate) use registration::test_project_registration;

use super::paths;
use crate::{docs, service::GovernanceService};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{self, Read},
    path::{Path, PathBuf},
};

const MANIFEST: &str = "mcp-launch-manifest.json";
const BINARY_LIMIT: u64 = 256 * 1024 * 1024;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: u32,
    binary: PathBuf,
    binary_sha256: String,
    governance: PathBuf,
    workspace: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    authority_sha256: Option<String>,
    files: BTreeMap<String, String>,
}

fn executable(input: &Path) -> Result<(PathBuf, String), String> {
    if !input.is_absolute() {
        return Err("MCP launch export requires an absolute executable path".into());
    }
    let name = input
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("Invalid executable name")?;
    if name != format!("cnr{}", std::env::consts::EXE_SUFFIX) {
        return Err("Select the built cnr executable".into());
    }
    let parent = paths::directory(input.parent().ok_or("Missing executable directory")?)?;
    let metadata =
        fs::symlink_metadata(parent.join(name)).map_err(|_| "CNR executable unavailable")?;
    if crate::policy::is_link(&metadata) || !metadata.is_file() || metadata.len() > BINARY_LIMIT {
        return Err("CNR launch executable must be a bounded ordinary file".into());
    }
    let executable = parent.join(name);
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
        .open(&executable)
        .map_err(|_| "Cannot open launch executable")?;
    let opened = file
        .metadata()
        .map_err(|_| "Cannot inspect the opened launch executable")?;
    if crate::policy::is_link(&opened) || !opened.is_file() || opened.len() != metadata.len() {
        return Err("Launch executable changed while being opened".into());
    }
    let mut reader = file.take(BINARY_LIMIT + 1);
    let mut hash = Sha256::new();
    let bytes = io::copy(&mut reader, &mut hash).map_err(|_| "Cannot hash launch executable")?;
    if bytes == 0 || bytes > BINARY_LIMIT {
        return Err("Invalid executable length".into());
    }
    Ok((executable, format!("{:x}", hash.finalize())))
}

pub(crate) fn current_executable() -> Result<(PathBuf, String), String> {
    executable(&std::env::current_exe().map_err(|_| "Cannot locate the running CNR executable")?)
}

fn native_path(path: &Path) -> Result<&str, String> {
    let text = path.to_str().ok_or("MCP paths must be valid Unicode")?;
    if text.chars().any(char::is_control)
        || text.contains("${")
        || crate::workspace::contains_known_secrets(text)
    {
        return Err("MCP launch path contains unsafe text".into());
    }
    #[cfg(windows)]
    let text = text.strip_prefix(r"\\?\").unwrap_or(text);
    Ok(text)
}

fn render(
    governance: &Path,
    workspace: &Path,
    binary: &Path,
) -> Result<(Manifest, BTreeMap<String, Vec<u8>>), String> {
    let governance = paths::directory(governance)?;
    let workspace = paths::directory(workspace)?;
    let authority_sha256 = GovernanceService::open(&governance, &workspace)?.snapshot_sha256()?;
    let (binary, binary_sha256) = executable(binary)?;
    let command = native_path(&binary)?;
    let args = [
        "mcp",
        "serve",
        "--governance-root",
        native_path(&governance)?,
        "--workspace",
        native_path(&workspace)?,
        "--access",
        "read-only",
    ];
    let mut files = BTreeMap::<String, Vec<u8>>::new();
    let codex = json!({"mcp_servers":{"cirnetrules":{"command":command,"args":args}}});
    files.insert(
        "codex.toml".into(),
        toml::to_string_pretty(&codex)
            .map_err(|_| "Cannot render Codex TOML")?
            .into_bytes(),
    );
    for (name, value) in [
        (
            "claude.mcp.json",
            json!({"mcpServers":{"cirnetrules":{"type":"stdio","command":command,"args":args}}}),
        ),
        (
            "antigravity.mcp_config.json",
            json!({"mcpServers":{"cirnetrules":{"command":command,"args":args}}}),
        ),
    ] {
        let mut bytes = serde_json::to_vec_pretty(&value).map_err(|_| "Cannot render MCP JSON")?;
        bytes.push(b'\n');
        files.insert(name.into(), bytes);
    }
    let manifest = Manifest {
        schema: 2,
        binary,
        binary_sha256,
        governance,
        workspace,
        authority_sha256: Some(authority_sha256),
        files: files
            .iter()
            .map(|(name, data)| (name.clone(), docs::sha256(data)))
            .collect(),
    };
    Ok((manifest, files))
}

fn bundle(root: &Path, path: &Path) -> Result<(Manifest, BTreeMap<String, Vec<u8>>), String> {
    let directory = paths::directory(path)?;
    let stored: Manifest = serde_json::from_slice(&paths::read(&directory.join(MANIFEST))?)
        .map_err(|_| "Invalid MCP launch manifest")?;
    if stored.governance != paths::directory(root)? {
        return Err("MCP configuration targets a different governance checkout".into());
    }
    let (mut current, files) = render(&stored.governance, &stored.workspace, &stored.binary)?;
    if stored.schema == 1 && stored.authority_sha256.is_none() {
        current.schema = 1;
        current.authority_sha256 = None;
    }
    if stored != current {
        return Err(
            "MCP executable, workspace or authority changed. Export a fresh bundle.".into(),
        );
    }
    verify(&directory, &current, &files)?;
    Ok((current, files))
}

pub(crate) fn check_probe_bundle(
    root: &Path,
    workspace: &Path,
    path: &Path,
) -> Result<Value, String> {
    let (manifest, _) = bundle(root, path)?;
    let binary = current_executable()?;
    if manifest.workspace != paths::directory(workspace)?
        || (manifest.binary, manifest.binary_sha256) != binary
        || manifest.authority_sha256.is_none()
    {
        return Err("The selected launch bundle uses a different executable or workspace, or has no pinned authority. Export a fresh bundle from the running CNR.".into());
    }
    Ok(json!({
        "status":"PASS","static_config":"STATIC_CONFIG_VALIDATED","native_client":"NOT_RUN",
        "binary_sha256":binary.1,"access":"read-only"
    }))
}

fn verify(
    directory: &Path,
    manifest: &Manifest,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<(), String> {
    let manifest_bytes =
        serde_json::to_vec(manifest).map_err(|_| "Cannot encode MCP launch manifest")?;
    if paths::read(&directory.join(MANIFEST))? != manifest_bytes {
        return Err("MCP launch manifest changed".into());
    }
    if fs::read_dir(directory)
        .map_err(|_| "Cannot list MCP configuration bundle")?
        .take(files.len() + 2)
        .count()
        != files.len() + 1
    {
        return Err("MCP configuration bundle contains unexpected entries".into());
    }
    for (name, expected) in files {
        if paths::read(&directory.join(name))? != *expected {
            return Err("MCP configuration content changed".into());
        }
    }
    Ok(())
}

pub(crate) fn run(
    root: &Path,
    action: &str,
    args: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let allowed: &[&str] = match action {
        "export" => &["output", "binary", "workspace", "expect"],
        "preview-export" => &["output", "binary", "workspace"],
        "check" => &["bundle"],
        "probe" => &["workspace", "repo", "task"],
        "install" => &[
            "agent",
            "scope",
            "repo",
            "bundle",
            "target",
            "workspace",
            "binding",
            "dry-run",
            "yes",
            "expect",
            "task",
            "actor",
            "surface",
            "trust-reviewed",
        ],
        "rollback" => &[
            "agent",
            "scope",
            "repo",
            "target",
            "workspace",
            "binding",
            "dry-run",
            "yes",
            "expect",
            "task",
            "actor",
            "receipt",
            "surface",
        ],
        "inspect" => &[
            "agent",
            "scope",
            "repo",
            "bundle",
            "target",
            "workspace",
            "binding",
            "task",
            "surface",
        ],
        _ => return Err("Unknown MCP launch configuration action".into()),
    };
    if args.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err("Unexpected MCP launch configuration option".into());
    }
    if matches!(action, "install" | "rollback" | "inspect") {
        return registration::run(root, action, args);
    }
    let input = |key| {
        args.get(key)
            .map(Path::new)
            .ok_or_else(|| format!("Missing --{key}"))
    };
    if action == "probe" {
        return Ok(
            match crate::mcp::probe(
                root,
                input("workspace")?,
                args.get("repo").map(String::as_str),
                args.get("task").map(String::as_str),
            ) {
                Ok(report) => report,
                Err(error) => json!({
                    "status": match error.code {
                        "CANCELLED" => "CANCELLED",
                        "DOMAIN_BLOCKED" | "AUTHORITY_MISMATCH" | "CONFIG_DRIFT" | "EXECUTABLE_NOT_FOUND" => "BLOCKED",
                        _ => "FAIL",
                    },
                    "code":error.code,"reason":error.message,"native_client":"NOT_RUN","mcp_protocol":"NOT_RUN"
                }),
            },
        );
    }
    let (manifest, state) = if matches!(action, "export" | "preview-export") {
        let directory = paths::export_path(input("output")?)?;
        let (manifest, files) = render(root, input("workspace")?, input("binary")?)?;
        let manifest_bytes =
            serde_json::to_vec(&manifest).map_err(|_| "Cannot encode launch manifest")?;
        let plan_sha256 = docs::sha256(
            &serde_json::to_vec(&(&directory, docs::sha256(&manifest_bytes)))
                .map_err(|_| "Cannot encode launch export plan")?,
        );
        if action == "preview-export" {
            if directory.exists() {
                verify(&directory, &manifest, &files)?;
            }
            return Ok(json!({
                "status":"PASS","state":if directory.exists() {"UNCHANGED"} else {"PREVIEW"},
                "plan_sha256":plan_sha256,"files":manifest.files,"native_client":"NOT_RUN"
            }));
        }
        if args
            .get("expect")
            .is_some_and(|expected| expected != &plan_sha256)
        {
            return Err("MCP launch export preview changed. Review a new preview.".into());
        }
        let state = if directory.exists() {
            "UNCHANGED"
        } else {
            fs::create_dir(&directory).map_err(|_| "Cannot create MCP configuration bundle")?;
            for (name, data) in &files {
                paths::write_new(&directory.join(name), data)?;
            }
            paths::write_new(&directory.join(MANIFEST), &manifest_bytes)?;
            "EXPORTED"
        };
        verify(&directory, &manifest, &files)?;
        (manifest, state)
    } else {
        let (manifest, _) = bundle(root, input("bundle")?)?;
        (manifest, "CHECKED")
    };
    let bytes = serde_json::to_vec(&manifest).map_err(|_| "Cannot encode launch manifest")?;
    Ok(json!({
        "status":"PASS","state":state,"static_config":"STATIC_CONFIG_VALIDATED",
        "native_client":"NOT_RUN","mcp_protocol":"NOT_RUN","access":"read-only",
        "bundle_sha256":docs::sha256(&bytes),"files":manifest.files,
        "authority_pinned":manifest.authority_sha256.is_some()
    }))
}
