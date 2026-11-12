pub(crate) mod mcp_config;
pub(crate) mod paths;
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    binding_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    binding_commit: Option<String>,
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
            binding_id: None,
            binding_commit: None,
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

struct ExportSelection<'a> {
    repo: &'a str,
    task: Option<&'a str>,
    profile: Option<&'a str>,
    binding_id: Option<&'a str>,
}

fn generate(
    root: &Path,
    work: Option<&Path>,
    registry: &Value,
    selection: ExportSelection<'_>,
    agents: &[String],
) -> Result<Bundle, String> {
    let ExportSelection {
        repo,
        task,
        profile,
        binding_id,
    } = selection;
    if repo == "_workspace" {
        return Err("Select a concrete registered repository for agent export".into());
    }
    let resolved = if binding_id.is_some() {
        authority::resolve_for_export_binding(
            root, work, registry, repo, task, profile, binding_id,
        )?
    } else {
        authority::resolve_for_export(root, work, registry, repo, task, profile)?
    };
    let mut bundle = render::bundle(&resolved, agents, registry_ownership(registry, repo))?;
    if let Some(id) = binding_id {
        bundle.manifest.binding_id = Some(id.into());
        bundle.manifest.binding_commit = resolved.binding.as_ref().map(|row| row.commit.clone());
    }
    Ok(bundle)
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
    binding_id: Option<&str>,
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
        ExportSelection {
            repo: &metadata.repository,
            task: metadata.task.as_deref(),
            profile: metadata.profile.as_deref(),
            binding_id: metadata.binding_id.as_deref(),
        },
        &agents,
    )?;
    if let Some(id) = binding_id {
        if metadata
            .binding_id
            .as_deref()
            .is_some_and(|declared| declared != id)
        {
            return Err("Selected binding differs from the reviewed bundle binding".into());
        }
        let workspace = crate::workspace::workspace_path(root, work)?;
        let selected =
            crate::workspace::resolve_named(root, &workspace, &metadata.repository, registry, id)?;
        if metadata.binding.as_deref() != Some(&selected.identity) {
            return Err("Selected binding differs from the verified bundle identity".into());
        }
    }
    verify_payload(&path, &manifest, &bundle)?;
    Ok(bundle)
}

fn export(
    input: &Path,
    bundle: &Bundle,
    preview: bool,
    expected: Option<&str>,
) -> Result<Value, String> {
    let location = paths::export_location(input)?;
    let path = &location.path;
    let manifest = bundle.manifest_bytes()?;
    let identity = serde_json::to_vec(&(&location, docs::sha256(&manifest)))
        .map_err(|_| "Cannot encode reviewed export destination")?;
    let plan_sha256 = docs::sha256(&identity);
    if expected.is_some_and(|expected| expected != plan_sha256) {
        return Err("Agent export preview changed. Review a fresh --dry-run.".into());
    }
    if location.existing.is_some() {
        verify_payload(&paths::directory(path)?, &manifest, bundle)?;
    }
    if preview {
        let mut report = bundle.summary(if location.existing.is_some() {
            "UNCHANGED"
        } else {
            "DRY_RUN"
        })?;
        report["destination"] = json!(path);
        report["plan_sha256"] = json!(plan_sha256);
        report["files"] = json!(
            bundle
                .manifest
                .outputs
                .iter()
                .map(|row| json!({
                    "path":row.path,
                    "sha256":row.sha256,
                    "action":if location.existing.is_some() { "UNCHANGED" } else { "CREATE" }
                }))
                .chain(std::iter::once(json!({
                    "path":MANIFEST,
                    "sha256":docs::sha256(&manifest),
                    "action":if location.existing.is_some() { "UNCHANGED" } else { "CREATE" }
                })))
                .collect::<Vec<_>>()
        );
        return Ok(report);
    }
    if location.existing.is_some() {
        return bundle.summary("UNCHANGED");
    }
    if paths::export_location(input)? != location {
        return Err("Agent export destination changed after review".into());
    }
    fs::create_dir(path).map_err(|_| "Cannot create fresh agent bundle directory")?;
    let created = paths::export_location(input)?;
    if created.parent_identity != location.parent_identity || created.existing.is_none() {
        return Err("Agent export destination changed during creation".into());
    }
    for (name, file) in &bundle.files {
        if paths::export_location(input)? != created {
            return Err("Agent export destination changed during export".into());
        }
        if let Some((parent, _)) = name.rsplit_once('/') {
            paths::directories(path, parent)?;
        }
        paths::write_new(&path.join(name), &file.bytes)?;
    }
    if paths::export_location(input)? != created {
        return Err("Agent export destination changed before publishing its manifest".into());
    }
    paths::write_new(&path.join(MANIFEST), &manifest)?;
    verify_payload(path, &manifest, bundle)?;
    bundle.summary("EXPORTED")
}

pub(crate) fn run(
    root: &Path,
    action: &str,
    args: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let allowed: &[&str] = match action {
        "list" => &[],
        "export" => &[
            "repo",
            "agents",
            "output",
            "profile",
            "task",
            "workspace",
            "binding",
            "expect",
        ],
        "preview-export" => &[
            "repo",
            "agents",
            "output",
            "profile",
            "task",
            "workspace",
            "binding",
        ],
        "check" => &["bundle", "workspace", "binding"],
        "sync" => &[
            "bundle",
            "target",
            "entry-mode",
            "keep-unselected",
            "binding",
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
            "binding",
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
        "export" | "preview-export" => {
            let agents = targets(require("agents")?)?;
            let bundle = generate(
                &root,
                work,
                &registry,
                ExportSelection {
                    repo: require("repo")?,
                    task: option("task"),
                    profile: option("profile"),
                    binding_id: option("binding"),
                },
                &agents,
            )?;
            export(
                Path::new(require("output")?),
                &bundle,
                action == "preview-export",
                option("expect"),
            )
        }
        "check" => checked(
            &root,
            work,
            &registry,
            Path::new(require("bundle")?),
            option("binding"),
        )?
        .summary("CHECKED"),
        "sync" => sync::sync(&root, work, &registry, args),
        "rollback" => sync::rollback(&root, work, &registry, args),
        _ => unreachable!(),
    }
}

pub(crate) fn audit(root: &Path, id: &str) -> Result<Value, String> {
    if let Some(receipt) = mcp_config::audit(root, id)? {
        return Ok(receipt);
    }
    sync::audit(root, id)
}
