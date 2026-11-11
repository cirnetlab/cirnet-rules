pub(crate) mod mcp_config;
mod paths;
mod render;
mod sync;
#[cfg(test)]
mod tests;

use crate::{
    authority::{self, AssetKind, Resolved, TaskProvenance},
    docs, policy,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

const GENERATOR: &str = concat!(env!("CARGO_PKG_VERSION"), "/agent-export-1");
const AGENTS: &[&str] = &["antigravity", "claude", "codex"];
const MANIFEST: &str = "manifest.json";

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SourceRecord {
    id: String,
    kind: AssetKind,
    path: String,
    revision: u64,
    dependencies: Vec<String>,
    sha256: String,
    trust: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OutputRecord {
    path: String,
    sha256: String,
    managed_by: String,
    owners: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: u8,
    generator: String,
    repository: String,
    task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_provenance: Option<TaskProvenance>,
    profile: Option<String>,
    binding: Option<String>,
    source_commit: String,
    source_dirty: bool,
    authority_sha256: String,
    agents: Vec<String>,
    capabilities: BTreeMap<String, Value>,
    sources: Vec<SourceRecord>,
    outputs: Vec<OutputRecord>,
}

impl Manifest {
    fn new(
        resolved: &Resolved,
        agents: &[String],
        capabilities: BTreeMap<String, Value>,
        sources: Vec<SourceRecord>,
        files: &BTreeMap<String, Output>,
    ) -> Self {
        Self {
            schema: 1,
            generator: GENERATOR.into(),
            repository: resolved.repository.clone(),
            task: resolved.task.clone(),
            task_provenance: resolved.task_provenance.clone(),
            profile: resolved.profile.clone(),
            binding: resolved
                .binding
                .as_ref()
                .map(|bound| bound.identity.clone()),
            source_commit: resolved.commit.clone(),
            source_dirty: resolved.dirty,
            authority_sha256: resolved.digest.clone(),
            agents: agents.to_vec(),
            capabilities,
            sources,
            outputs: files
                .iter()
                .map(|(path, file)| OutputRecord {
                    path: path.clone(),
                    sha256: docs::sha256(&file.bytes),
                    managed_by: "cirnetrules".into(),
                    owners: file.owners.clone(),
                })
                .collect(),
        }
    }
}

struct Output {
    bytes: Vec<u8>,
    owners: Vec<String>,
}

struct Bundle {
    manifest: Manifest,
    files: BTreeMap<String, Output>,
}

impl Bundle {
    fn manifest_bytes(&self) -> Result<Vec<u8>, String> {
        let mut bytes = serde_json::to_vec_pretty(&self.manifest)
            .map_err(|_| "Cannot encode agent manifest")?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    fn summary(&self, state: &str) -> Result<Value, String> {
        Ok(json!({
            "status":"PASS", "state":state, "static_validation":"STATIC_EXPORT_VALIDATED",
            "native_discovery":"NOT_RUN",
            "manifest_sha256":docs::sha256(&self.manifest_bytes()?),
            "manifest":self.manifest,
        }))
    }
}

fn targets(csv: &str) -> Result<Vec<String>, String> {
    let mut agents = BTreeSet::new();
    for agent in csv.split(',') {
        if !AGENTS.contains(&agent) || !agents.insert(agent.to_owned()) {
            return Err("Use a comma-separated list of distinct supported agents".into());
        }
    }
    Ok(agents.into_iter().collect())
}

fn registry_ownership<'a>(registry: &'a Value, repo: &str) -> Option<&'a str> {
    registry["repositories"][repo]["ownership"].as_str()
}

fn generate(
    root: &Path,
    work: Option<&Path>,
    registry: &Value,
    repo: &str,
    task: Option<&str>,
    profile: Option<&str>,
    agents: &[String],
) -> Result<Bundle, String> {
    if repo == "_workspace" {
        return Err("Select a concrete registered repository for agent export".into());
    }
    let resolved = authority::resolve_for_export(root, work, registry, repo, task, profile)?;
    render::bundle(&resolved, agents, registry_ownership(registry, repo))
}

fn verify_payload(path: &Path, manifest: &[u8], bundle: &Bundle) -> Result<(), String> {
    if bundle.manifest_bytes()? != manifest {
        return Err(
            "Bundle authority, selector, renderer or manifest changed. Export a fresh bundle."
                .into(),
        );
    }
    let mut files: BTreeSet<_> = bundle.files.keys().map(String::as_str).collect();
    files.insert(MANIFEST);
    let mut directories = BTreeSet::new();
    for path in &files {
        let mut current = *path;
        while let Some((parent, _)) = current.rsplit_once('/') {
            directories.insert(parent);
            current = parent;
        }
    }
    let mut pending = vec![(String::new(), path.to_path_buf())];
    while let Some((parent, folder)) = pending.pop() {
        for item in fs::read_dir(folder).map_err(|_| "Cannot list agent bundle")? {
            let item = item.map_err(|_| "Cannot inspect bundle entry")?;
            let name = item
                .file_name()
                .into_string()
                .map_err(|_| "Non-UTF-8 bundle name")?;
            let relative = if parent.is_empty() {
                name
            } else {
                format!("{parent}/{name}")
            };
            let metadata =
                fs::symlink_metadata(item.path()).map_err(|_| "Cannot inspect bundle entry")?;
            if policy::is_link(&metadata) {
                return Err("Agent bundle contains a symbolic link or reparse point".into());
            }
            if metadata.is_dir() && directories.remove(relative.as_str()) {
                pending.push((relative, item.path()));
            } else if metadata.is_file() && files.remove(relative.as_str()) {
                let expected = if relative == MANIFEST {
                    manifest
                } else {
                    &bundle.files[&relative].bytes
                };
                if paths::read(&item.path())? != expected {
                    return Err("Agent bundle payload changed".into());
                }
            } else {
                return Err("Agent bundle contains an unexpected file or directory".into());
            }
        }
    }
    if !files.is_empty() || !directories.is_empty() {
        return Err("Agent bundle payload is incomplete".into());
    }
    Ok(())
}

fn checked(
    root: &Path,
    work: Option<&Path>,
    registry: &Value,
    input: &Path,
) -> Result<Bundle, String> {
    let path = paths::directory(input)?;
    let manifest = paths::read(&path.join(MANIFEST))?;
    let metadata: Manifest =
        serde_json::from_slice(&manifest).map_err(|_| "Invalid agent bundle manifest")?;
    let agents = targets(&metadata.agents.join(","))?;
    if metadata.schema != 1 || metadata.generator != GENERATOR || metadata.agents != agents {
        return Err("Agent bundle uses an unsupported manifest or renderer version".into());
    }
    let bundle = generate(
        root,
        work,
        registry,
        &metadata.repository,
        metadata.task.as_deref(),
        metadata.profile.as_deref(),
        &agents,
    )?;
    verify_payload(&path, &manifest, &bundle)?;
    Ok(bundle)
}

fn export(input: &Path, bundle: &Bundle) -> Result<Value, String> {
    let path = paths::export_path(input)?;
    let manifest = bundle.manifest_bytes()?;
    if path.exists() {
        verify_payload(&paths::directory(&path)?, &manifest, bundle)?;
        return bundle.summary("UNCHANGED");
    }
    fs::create_dir(&path).map_err(|_| "Cannot create fresh agent bundle directory")?;
    for (name, file) in &bundle.files {
        if let Some((parent, _)) = name.rsplit_once('/') {
            paths::directories(&path, parent)?;
        }
        paths::write_new(&path.join(name), &file.bytes)?;
    }
    paths::write_new(&path.join(MANIFEST), &manifest)?;
    verify_payload(&path, &manifest, bundle)?;
    bundle.summary("EXPORTED")
}

pub(crate) fn run(
    root: &Path,
    action: &str,
    args: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let allowed: &[&str] = match action {
        "list" => &[],
        "export" => &["repo", "agents", "output", "profile", "task", "workspace"],
        "check" => &["bundle", "workspace"],
        "sync" => &[
            "bundle",
            "target",
            "workspace",
            "dry-run",
            "yes",
            "expect",
            "task",
            "actor",
        ],
        "rollback" => &[
            "target",
            "workspace",
            "receipt",
            "dry-run",
            "yes",
            "expect",
            "task",
            "actor",
        ],
        _ => return Err("Unknown agent configuration operation".into()),
    };
    if args.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err("Unsupported agent configuration option".into());
    }
    if action == "list" {
        return Ok(json!({
            "status":"PASS",
            "agents":AGENTS.iter().map(|agent| (agent, render::capability(agent))).collect::<BTreeMap<_,_>>(),
            "native_discovery":"NOT_RUN"
        }));
    }
    let option = |key: &str| args.get(key).map(String::as_str);
    let require = |key: &str| option(key).ok_or_else(|| format!("--{key} is required"));
    let work = option("workspace").map(Path::new);
    let root = paths::directory(root)?;
    let registry = policy::registry(&root)?;
    match action {
        "export" => {
            let agents = targets(require("agents")?)?;
            let bundle = generate(
                &root,
                work,
                &registry,
                require("repo")?,
                option("task"),
                option("profile"),
                &agents,
            )?;
            export(Path::new(require("output")?), &bundle)
        }
        "check" => {
            checked(&root, work, &registry, Path::new(require("bundle")?))?.summary("CHECKED")
        }
        "sync" => sync::sync(&root, work, &registry, args),
        "rollback" => sync::rollback(&root, work, &registry, args),
        _ => unreachable!(),
    }
}

pub(crate) fn audit(root: &Path, id: &str) -> Result<Value, String> {
    sync::audit(root, id)
}
