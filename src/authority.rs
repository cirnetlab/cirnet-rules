mod catalog;
#[cfg(test)]
mod tests;

pub(crate) use catalog::AssetKind;
use catalog::{Asset, Catalog, eligible_repository, visit};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use crate::{docs, policy, tasks, workspace};

pub(crate) const FILE_LIMIT: &str = "Selected sources exceed the file budget";
pub(crate) const BYTE_LIMIT: &str = "Context byte limit exceeded";

#[derive(Debug, Serialize)]
pub(crate) struct Source {
    pub id: String,
    pub kind: AssetKind,
    pub path: String,
    pub revision: u64,
    pub dependencies: Vec<String>,
    pub sha256: String,
    pub trust: &'static str,
    #[serde(skip)]
    pub bytes: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub(crate) struct AssetDecision {
    pub id: String,
    pub kind: AssetKind,
    pub path: String,
    pub reason: &'static str,
}

impl AssetDecision {
    fn new(asset: &Asset, reason: &'static str) -> Self {
        Self {
            id: asset.id.clone(),
            kind: asset.kind,
            path: asset.path.clone(),
            reason,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Resolved {
    pub repository: String,
    pub task: Option<String>,
    pub task_provenance: Option<TaskProvenance>,
    pub profile: Option<String>,
    pub digest: String,
    pub commit: String,
    pub dirty: bool,
    pub binding: Option<workspace::Binding>,
    pub included: Vec<Source>,
    pub manual: Vec<Source>,
    pub deferred: Vec<AssetDecision>,
    pub excluded: Vec<AssetDecision>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TaskProvenance {
    pub spec_sha256: String,
    pub plan_sha256: String,
    pub descriptor_sha256: String,
    pub approval_epoch: u64,
}

impl TaskProvenance {
    fn from_current(task: &Value) -> Result<Self, String> {
        let hash = |field: &str| {
            task[field]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("Missing current task provenance field: {field}"))
        };
        Ok(Self {
            spec_sha256: hash("spec_sha256")?,
            plan_sha256: hash("plan_sha256")?,
            descriptor_sha256: hash("descriptor_sha256")?,
            approval_epoch: task["approval_epoch"]
                .as_u64()
                .filter(|epoch| *epoch > 0)
                .ok_or("Missing current task approval epoch")?,
        })
    }
}

#[derive(Serialize)]
pub(crate) struct Snapshot {
    pub profile: Option<String>,
    pub digest: String,
    pub included: Vec<Source>,
    pub deferred: Vec<AssetDecision>,
    pub excluded: Vec<AssetDecision>,
}

struct Selection<'a> {
    repository: &'a str,
    profile: Option<&'a Asset>,
    included: Vec<&'a Asset>,
    deferred: Vec<AssetDecision>,
    excluded: Vec<AssetDecision>,
}

fn select<'a>(
    catalog: &'a Catalog,
    registry: &Value,
    repository: &'a str,
    task_kind: &str,
    skill: Option<&str>,
    profile: Option<&str>,
    risk: &str,
) -> Result<Selection<'a>, String> {
    if !eligible_repository(registry, repository) {
        return Err("Repository is outside active governance scope".into());
    }
    let task = &registry["context"]["tasks"][task_kind];
    let profile = match profile {
        Some(profile) => Some(profile_asset(catalog, profile)?),
        None => task["profile"]
            .as_str()
            .map(|path| catalog.path(path))
            .transpose()?,
    };
    let mut roots = vec![
        catalog.path(
            registry["control_plane"]["entrypoint"]
                .as_str()
                .unwrap_or("en/AGENTS.md"),
        )?,
    ];
    for path in registry["context"]["common"]
        .as_array()
        .into_iter()
        .flatten()
    {
        roots.push(catalog.path(path.as_str().ok_or("Invalid common authority source")?)?);
    }
    roots.extend(profile);
    if let Some(path) = skill.or_else(|| task["skill"].as_str()) {
        roots.push(catalog.path(path)?);
    }
    if matches!(risk, "high" | "hardware") {
        roots.push(catalog.path("en/profiles/reviewer.md")?);
    }
    if risk == "hardware" {
        roots.push(catalog.path("en/skills/hardware-evidence/SKILL.md")?);
    }
    let by_id = catalog.by_id();
    let mut selected = BTreeSet::new();
    let mut included = Vec::new();
    for asset in roots {
        if !asset.active() || !asset.assigned(repository) || asset.kind.manual() {
            return Err(format!(
                "Authority asset {} is ineligible for {repository}",
                asset.id
            ));
        }
        visit(
            asset,
            &by_id,
            &mut BTreeSet::new(),
            &mut selected,
            &mut included,
        )?;
    }
    let mut deferred = Vec::new();
    let mut excluded = Vec::new();
    for asset in by_id.values() {
        if selected.contains(asset.id.as_str()) {
            continue;
        }
        if !asset.active() {
            excluded.push(AssetDecision::new(asset, "asset is inactive"));
        } else if !asset.assigned(repository) {
            excluded.push(AssetDecision::new(
                asset,
                "repository is outside asset assignment",
            ));
        } else if asset.kind.manual() {
            deferred.push(AssetDecision::new(asset, "manual procedure only"));
        } else {
            deferred.push(AssetDecision::new(
                asset,
                "outside the selected profile or task",
            ));
        }
    }
    Ok(Selection {
        repository,
        profile,
        included,
        deferred,
        excluded,
    })
}

fn profile_asset<'a>(catalog: &'a Catalog, selector: &str) -> Result<&'a Asset, String> {
    if let Some(asset) = catalog
        .entries
        .iter()
        .find(|asset| asset.kind == AssetKind::Profile && asset.id == selector)
    {
        return Ok(asset);
    }
    let mut matches = catalog.entries.iter().filter(|asset| {
        asset.kind == AssetKind::Profile
            && asset
                .path
                .rsplit('/')
                .next()
                .and_then(|name| name.strip_suffix(".md"))
                == Some(selector)
    });
    let asset = matches.next().ok_or("Unknown governance profile")?;
    if matches.next().is_some() {
        return Err("Ambiguous governance profile. Select the exact asset ID".into());
    }
    Ok(asset)
}

fn task_kind(mode: &str) -> Result<&'static str, String> {
    match mode {
        "WORK" => Ok("implementation"),
        "VERIFY" | "ACCEPT" => Ok("verification"),
        "AUDIT" => Ok("audit"),
        "RELEASE" => Ok("release-readiness"),
        _ => Err("Unknown governance task mode".into()),
    }
}

fn select_task<'a>(
    catalog: &'a Catalog,
    registry: &Value,
    descriptor: &'a Value,
) -> Result<Selection<'a>, String> {
    let repository = descriptor["primary_repo"]
        .as_str()
        .ok_or("Missing task repository")?;
    let mode = descriptor["mode"].as_str().ok_or("Missing task mode")?;
    let risk = descriptor["risk"]
        .as_str()
        .ok_or("Missing task risk classification")?;
    let skill = registry["control_plane"]["skills"][mode].as_str();
    select(
        catalog,
        registry,
        repository,
        task_kind(mode)?,
        skill,
        None,
        risk,
    )
}

fn tracked_sources(root: &Path) -> Result<BTreeSet<String>, String> {
    if !policy::git(root, &["rev-parse", "--show-prefix"])?
        .trim()
        .is_empty()
    {
        return Err("Authority must be resolved at its Git worktree root".into());
    }
    let index = policy::git(
        root,
        &["ls-files", "--stage", "-z", "--", "en", "governance.yaml"],
    )?;
    let mut paths = BTreeSet::new();
    for entry in index.split_terminator('\0') {
        let (meta, path) = entry
            .split_once('\t')
            .ok_or("Invalid Git authority index entry")?;
        if (meta.starts_with("100644 ") || meta.starts_with("100755 ")) && meta.ends_with(" 0") {
            paths.insert(path.to_owned());
        }
    }
    if !paths.contains("governance.yaml") {
        return Err("The governance registry must be a tracked ordinary file".into());
    }
    Ok(paths)
}

fn require_tracked(tracked: &BTreeSet<String>, path: &str) -> Result<(), String> {
    if !tracked.contains(path) {
        return Err(format!(
            "Authority source is outside the resolved Git index: {path}"
        ));
    }
    Ok(())
}

pub(crate) fn read_authority(root: &Path, path: &str, limit: u64) -> Result<Vec<u8>, String> {
    let file = policy::safe_path(root, path)?;
    let metadata =
        fs::symlink_metadata(&file).map_err(|_| format!("Authority source unavailable: {path}"))?;
    if !metadata.is_file() || policy::is_link(&metadata) {
        return Err(format!("Expected an ordinary authority file: {path}"));
    }
    if metadata.len() > limit {
        return Err(BYTE_LIMIT.into());
    }
    let mut bytes = Vec::new();
    fs::File::open(file)
        .map_err(|_| format!("Authority source unavailable: {path}"))?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| format!("Cannot read authority source: {path}"))?;
    if bytes.len() as u64 > limit {
        return Err(BYTE_LIMIT.into());
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| format!("Invalid UTF-8 authority source: {path}"))?;
    if text.starts_with('\u{feff}')
        || text.contains('\0')
        || text.replace("\r\n", "").contains('\r')
        || workspace::contains_known_secrets(text)
    {
        return Err(format!(
            "Authority source failed text or credential validation: {path}"
        ));
    }
    Ok(bytes)
}

fn read_registry(root: &Path) -> Result<(Value, String), String> {
    let bytes = read_authority(root, "governance.yaml", 4 * 1024 * 1024)?;
    let registry: serde_yaml::Value =
        serde_yaml::from_slice(&bytes).map_err(|_| "Invalid governance YAML or duplicate key")?;
    let registry =
        serde_json::to_value(registry).map_err(|_| "Registry must be JSON-compatible YAML")?;
    Ok((registry, docs::sha256(&bytes)))
}

fn registry_digest(root: &Path, registry: &Value) -> Result<String, String> {
    let (current, digest) = read_registry(root)?;
    if &current != registry {
        return Err("Governance registry changed while resolving authority".into());
    }
    Ok(digest)
}

fn default_limits(registry: &Value) -> Result<(u64, u64), String> {
    Ok((
        workspace::configuration(registry, "context_bytes", 32_768, 2 * 1024 * 1024)?,
        workspace::configuration(registry, "context_files", 32, 128)?,
    ))
}

fn snapshot(
    root: &Path,
    selection: Selection<'_>,
    registry_sha256: &str,
    limits: (u64, u64),
) -> Result<Snapshot, String> {
    let sources = load_sources(root, selection.included, limits, "governance_instruction")?;
    let profile = selection.profile.map(|asset| asset.id.clone());
    let identity = serde_json::to_vec(&json!({
        "authority_schema":1,"registry_sha256":registry_sha256,"repository":selection.repository,
        "profile":profile,"sources":sources
    }))
    .map_err(|_| "Cannot encode authority snapshot")?;
    Ok(Snapshot {
        profile,
        digest: docs::sha256(&identity),
        included: sources,
        deferred: selection.deferred,
        excluded: selection.excluded,
    })
}

fn load_sources(
    root: &Path,
    assets: Vec<&Asset>,
    limits: (u64, u64),
    trust: &'static str,
) -> Result<Vec<Source>, String> {
    let (mut bytes_left, file_limit) = limits;
    if assets.len() as u64 > file_limit {
        return Err(FILE_LIMIT.into());
    }
    if assets.is_empty() {
        return Ok(Vec::new());
    }
    let tracked = tracked_sources(root)?;
    let mut sources = Vec::new();
    for asset in assets {
        require_tracked(&tracked, &asset.path)?;
        let bytes = read_authority(root, &asset.path, bytes_left)?;
        bytes_left -= bytes.len() as u64;
        sources.push(Source {
            id: asset.id.clone(),
            kind: asset.kind,
            path: asset.path.clone(),
            revision: asset.revision,
            dependencies: asset.depends_on.clone(),
            sha256: docs::sha256(&bytes),
            trust,
            bytes,
        });
    }
    Ok(sources)
}

fn confirm_sources<'a>(
    root: &Path,
    sources: impl Iterator<Item = &'a Source>,
) -> Result<(), String> {
    let tracked = tracked_sources(root)?;
    for source in sources {
        require_tracked(&tracked, &source.path)?;
        if !read_authority(root, &source.path, source.bytes.len() as u64)
            .is_ok_and(|current| current == source.bytes)
        {
            return Err(format!(
                "Governance source changed while resolving authority: {}",
                source.path
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate(root: &Path, registry: &Value) -> Result<(), String> {
    let catalog = Catalog::load(registry)?;
    let tracked = tracked_sources(root)?;
    for asset in catalog.entries.iter().filter(|asset| asset.active()) {
        require_tracked(&tracked, &asset.path)?;
        read_authority(root, &asset.path, 2 * 1024 * 1024)?;
    }
    Ok(())
}

pub(crate) fn validate_catalog(registry: &Value) -> Result<(), String> {
    Catalog::load(registry).map(|_| ())
}

pub(crate) fn task_digest(
    root: &Path,
    descriptor: &Value,
    validated_registry: Option<&Value>,
) -> Result<String, String> {
    let (registry, digest) = read_registry(root)?;
    #[cfg(test)]
    if registry.get("assets").is_none() {
        return Ok(docs::sha256(
            format!("legacy-registry\0{digest}").as_bytes(),
        ));
    }
    if validated_registry.is_some_and(|expected| expected != &registry) {
        return Err("Governance registry changed after validating the task descriptor".into());
    }
    let catalog = Catalog::load(&registry)?;
    Ok(snapshot(
        root,
        select_task(&catalog, &registry, descriptor)?,
        &digest,
        default_limits(&registry)?,
    )?
    .digest)
}

pub(crate) fn task_snapshot(
    root: &Path,
    registry: &Value,
    descriptor: &Value,
    limits: (u64, u64),
) -> Result<Snapshot, String> {
    let catalog = Catalog::load(registry)?;
    snapshot(
        root,
        select_task(&catalog, registry, descriptor)?,
        &registry_digest(root, registry)?,
        limits,
    )
}

pub(crate) fn legacy_context(
    root: &Path,
    registry: &Value,
    repository: &str,
    task: &str,
    risk: &str,
) -> Result<Vec<Source>, String> {
    let catalog = Catalog::load(registry)?;
    let selection = select(&catalog, registry, repository, task, None, None, risk)?;
    Ok(snapshot(
        root,
        selection,
        &registry_digest(root, registry)?,
        default_limits(registry)?,
    )?
    .included)
}

pub(crate) fn resolve(
    root: &Path,
    workspace: Option<&Path>,
    registry: &Value,
    repository: &str,
    task: Option<&str>,
    profile: Option<&str>,
) -> Result<Resolved, String> {
    resolve_with_manual(root, workspace, registry, repository, task, profile, false)
}

pub(crate) fn resolve_for_export(
    root: &Path,
    workspace: Option<&Path>,
    registry: &Value,
    repository: &str,
    task: Option<&str>,
    profile: Option<&str>,
) -> Result<Resolved, String> {
    resolve_with_manual(root, workspace, registry, repository, task, profile, true)
}

fn resolve_with_manual(
    root: &Path,
    workspace: Option<&Path>,
    registry: &Value,
    repository: &str,
    task: Option<&str>,
    profile: Option<&str>,
    preserve_manual: bool,
) -> Result<Resolved, String> {
    if repository == "_workspace" && task.is_none() {
        return Err("Workspace context requires a parent task".into());
    }
    let workspace = workspace
        .map(|path| workspace::workspace_path(root, Some(path)))
        .transpose()?;
    let catalog = Catalog::load(registry)?;
    let limits = default_limits(registry)?;
    let registry_sha256 = registry_digest(root, registry)?;
    let (commit, clean) = policy::checkout(root)?;
    let state = task.map(|id| tasks::load_current(root, id)).transpose()?;
    let selection = if let Some(state) = &state {
        tasks::authorize(state, "read")?;
        if state["descriptor"]["primary_repo"] != repository {
            return Err("Task authority does not apply to the selected repository".into());
        }
        let selection = select_task(&catalog, registry, &state["descriptor"])?;
        if let Some(profile) = profile
            && selection.profile.map(|asset| &asset.id)
                != Some(&profile_asset(&catalog, profile)?.id)
        {
            return Err("The requested profile conflicts with the task's current authority".into());
        }
        selection
    } else {
        select(
            &catalog,
            registry,
            repository,
            "implementation",
            None,
            profile,
            "normal",
        )?
    };
    let task_provenance = state
        .as_ref()
        .map(TaskProvenance::from_current)
        .transpose()?;
    let mut manual_assets = Vec::new();
    if preserve_manual {
        let by_id = catalog.by_id();
        let mut complete = selection
            .included
            .iter()
            .map(|asset| asset.id.as_str())
            .collect();
        for asset in by_id
            .values()
            .filter(|asset| asset.kind.manual() && asset.active() && asset.assigned(repository))
        {
            visit(
                asset,
                &by_id,
                &mut BTreeSet::new(),
                &mut complete,
                &mut manual_assets,
            )?;
        }
    }
    let snapshot = snapshot(root, selection, &registry_sha256, limits)?;
    if state
        .as_ref()
        .is_some_and(|state| state["policy_sha256"] != snapshot.digest)
    {
        return Err(
            "Task authority changed while resolving context. Request explicit revision".into(),
        );
    }
    let manual_limits = (
        limits.0
            - snapshot
                .included
                .iter()
                .map(|source| source.bytes.len() as u64)
                .sum::<u64>(),
        limits.1 - snapshot.included.len() as u64,
    );
    let manual = load_sources(root, manual_assets, manual_limits, "governance_reference")?;
    let digest = if manual.is_empty() {
        snapshot.digest
    } else {
        let input =
            serde_json::to_vec(&json!({"authority_sha256":snapshot.digest,"manual":manual}))
                .map_err(|_| "Cannot encode manual export references")?;
        docs::sha256(&input)
    };
    let binding = if repository == "_workspace" {
        None
    } else if let Some(state) = &state {
        Some(workspace::resolve_task(
            root,
            workspace.as_deref(),
            repository,
            registry,
            state,
        )?)
    } else {
        workspace::resolve(root, workspace.as_deref(), repository, registry).ok()
    };
    confirm_sources(root, snapshot.included.iter().chain(&manual))?;
    let (current_commit, current_clean) = policy::checkout(root)?;
    if commit != current_commit
        || clean != current_clean
        || registry_sha256 != registry_digest(root, registry)?
    {
        return Err("Governance worktree changed while resolving authority".into());
    }
    if let Some((id, expected)) = task.zip(task_provenance.as_ref()) {
        let latest = tasks::load_current(root, id)?;
        tasks::authorize(&latest, "read")?;
        if TaskProvenance::from_current(&latest)? != *expected {
            return Err("Task approval changed while resolving authority. Resolve again.".into());
        }
    }
    if let Some(expected) = binding.as_ref()
        && let Some(state) = state.as_ref()
    {
        let current =
            workspace::resolve_task(root, workspace.as_deref(), repository, registry, state)?;
        if current.identity != expected.identity || current.commit != expected.commit {
            return Err("Task binding changed while resolving authority".into());
        }
    }
    Ok(Resolved {
        repository: repository.into(),
        task: task.map(str::to_owned),
        task_provenance,
        profile: snapshot.profile,
        digest,
        commit,
        dirty: !clean,
        binding,
        included: snapshot.included,
        manual,
        deferred: snapshot.deferred,
        excluded: snapshot.excluded,
    })
}

pub(crate) fn run(
    root: &Path,
    action: &str,
    args: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let registry = policy::registry(root)?;
    match action {
        "inspect" => {
            if !args.is_empty() {
                return Err("Unexpected governance inspect option".into());
            }
            let catalog = Catalog::load(&registry)?;
            validate(root, &registry)?;
            let (commit, clean) = policy::checkout(root)?;
            Ok(
                json!({"status":"PASS","schema":1,"commit":commit,"dirty":!clean,
                "registry_revision":registry["governance"]["registry_revision"],"assets":catalog.entries}),
            )
        }
        "context" => {
            if args
                .keys()
                .any(|key| !matches!(key.as_str(), "repo" | "task" | "profile" | "workspace"))
            {
                return Err("Unexpected governance context option".into());
            }
            let task = args.get("task").map(String::as_str);
            let repository = match args.get("repo") {
                Some(repository) => repository.clone(),
                None if task.is_some() => {
                    let task = tasks::load(root, task.unwrap())?;
                    task["descriptor"]["primary_repo"]
                        .as_str()
                        .ok_or("Missing task repository")?
                        .to_owned()
                }
                None => return Err("Governance context requires --repo or --task".into()),
            };
            let workspace = args.get("workspace").map(PathBuf::from);
            let resolved = resolve(
                root,
                workspace.as_deref(),
                &registry,
                &repository,
                task,
                args.get("profile").map(String::as_str),
            )?;
            let limits = default_limits(&registry)?;
            let included = resolved
                .included
                .iter()
                .map(|source| {
                    let mut metadata = serde_json::to_value(source)
                        .map_err(|_| "Cannot encode authority metadata")?;
                    if let Some(rule) = registry["rules"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .find(|rule| rule["id"] == source.id && rule["path"] == source.path)
                    {
                        metadata["enforcement_class"] = rule["severity"].clone();
                    }
                    Ok::<_, String>(metadata)
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(
                json!({"status":"PASS","schema":1,"repository":resolved.repository,"task":resolved.task,
                "task_provenance":resolved.task_provenance,
                "profile":resolved.profile,"authority_sha256":resolved.digest,"commit":resolved.commit,"dirty":resolved.dirty,
                "binding":resolved.binding.map(|binding| json!({"identity":binding.identity,"commit":binding.commit})),
                "included":included,"deferred":resolved.deferred,"excluded":resolved.excluded,
                "limits":{"context_bytes":limits.0,"context_files":limits.1},
                "capabilities":{"manual_procedures":"REFERENCE_ONLY","native_profiles":"UNSUPPORTED","native_discovery":"NOT_RUN"}}),
            )
        }
        _ => Err("Unknown governance authority command".into()),
    }
}
