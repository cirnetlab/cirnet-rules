use super::paths;
use crate::{docs, service::GovernanceService};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
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
    let mut reader = File::open(&executable)
        .map_err(|_| "Cannot open launch executable")?
        .take(BINARY_LIMIT + 1);
    let mut hash = Sha256::new();
    let bytes = io::copy(&mut reader, &mut hash).map_err(|_| "Cannot hash launch executable")?;
    if bytes == 0 || bytes > BINARY_LIMIT {
        return Err("Invalid executable length".into());
    }
    Ok((executable, format!("{:x}", hash.finalize())))
}

fn native_path(path: &Path) -> Result<&str, String> {
    let text = path.to_str().ok_or("MCP paths must be valid Unicode")?;
    if text.chars().any(char::is_control) || crate::workspace::contains_known_secrets(text) {
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
    GovernanceService::open(&governance, &workspace)?;
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
        schema: 1,
        binary,
        binary_sha256,
        governance,
        workspace,
        files: files
            .iter()
            .map(|(name, data)| (name.clone(), docs::sha256(data)))
            .collect(),
    };
    Ok((manifest, files))
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
        "export" => &["output", "binary", "workspace"],
        "check" => &["bundle"],
        _ => return Err("Unknown MCP launch configuration action".into()),
    };
    if args.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err("Unexpected MCP launch configuration option".into());
    }
    let input = |key| {
        args.get(key)
            .map(Path::new)
            .ok_or_else(|| format!("Missing --{key}"))
    };
    let (directory, manifest, files, state) = if action == "export" {
        let directory = paths::export_path(input("output")?)?;
        let (manifest, files) = render(root, input("workspace")?, input("binary")?)?;
        let state = if directory.exists() {
            "UNCHANGED"
        } else {
            fs::create_dir(&directory).map_err(|_| "Cannot create MCP configuration bundle")?;
            for (name, data) in &files {
                paths::write_new(&directory.join(name), data)?;
            }
            let manifest_bytes =
                serde_json::to_vec(&manifest).map_err(|_| "Cannot encode launch manifest")?;
            paths::write_new(&directory.join(MANIFEST), &manifest_bytes)?;
            "EXPORTED"
        };
        (directory, manifest, files, state)
    } else {
        let directory = paths::directory(input("bundle")?)?;
        let stored: Manifest = serde_json::from_slice(&paths::read(&directory.join(MANIFEST))?)
            .map_err(|_| "Invalid MCP launch manifest")?;
        if stored.governance != root {
            return Err("MCP configuration targets a different governance checkout".into());
        }
        let (manifest, files) = render(&stored.governance, &stored.workspace, &stored.binary)?;
        (directory, manifest, files, "CHECKED")
    };
    verify(&directory, &manifest, &files)?;
    let bytes = serde_json::to_vec(&manifest).map_err(|_| "Cannot encode launch manifest")?;
    Ok(json!({
        "status":"PASS","state":state,"static_config":"STATIC_CONFIG_VALIDATED",
        "native_client":"NOT_RUN","mcp_protocol":"NOT_RUN","access":"read-only",
        "bundle_sha256":docs::sha256(&bytes),"files":manifest.files
    }))
}
