use super::{archive, documents, files};
use crate::{authority, docs::sha256, policy, workspace};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const STATE: &str = ".cirnetrules/migration";
const PROPOSAL: &str = "proposal.json";
const RECEIPT: &str = "receipt.json";
const ROLLBACK_INTENT: &str = "rollback-intent.json";
const ROLLBACK_RECEIPT: &str = "rollback.json";
const PLAN_LIMIT: u64 = 32 * 1024 * 1024;
const RECEIPT_LIMIT: u64 = 1024 * 1024;
const PREVIEW_FILE_LIMIT: usize = 128;

#[derive(Serialize)]
struct Plan {
    schema: u8,
    target_identity: String,
    target_commit: String,
    authority_sha256: String,
    sources: Vec<Source>,
    candidates: Vec<Candidate>,
    summary: Value,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Source {
    schema: usize,
    receipt_sha256: String,
    snapshot_sha256: String,
    export_sha256: String,
}

#[derive(Serialize)]
struct Candidate {
    asset: Value,
    expected_path_state: String,
    source_receipt_sha256: String,
    document_sha256: Option<String>,
    draft_sha256: Option<String>,
    files: Vec<FileProposal>,
    review_issues: BTreeSet<String>,
}

impl Candidate {
    fn id(&self) -> &str {
        self.asset["id"].as_str().unwrap()
    }

    fn path(&self) -> &str {
        self.asset["path"].as_str().unwrap()
    }

    fn issue(&mut self, reason: &str) {
        self.review_issues.insert(reason.to_owned());
    }
}

#[derive(Serialize)]
struct FileProposal {
    path: String,
    content: String,
    sha256: String,
    expected_target: String,
}

impl FileProposal {
    fn new(path: String, content: String, root: &Path) -> Result<Self, String> {
        let sha256 = sha256(content.as_bytes());
        let expected_target = target_state(root, &path)?;
        Ok(Self {
            path,
            content,
            sha256,
            expected_target,
        })
    }
}

struct LegacyAsset {
    id: String,
    kind: String,
    slug: String,
    revision: i64,
    active: Option<i64>,
    published: Option<String>,
    draft: Option<String>,
    archived: bool,
    deleted: bool,
}

fn sql_error(_: rusqlite::Error) -> String {
    "Cannot read legacy governance proposal inputs".into()
}

fn target_state(root: &Path, relative: &str) -> Result<String, String> {
    let mut path = root.to_path_buf();
    let mut parts = relative.split('/').peekable();
    while let Some(part) = parts.next() {
        path.push(part);
        if files::case_alias(&path)? {
            return Ok("CASE_ALIAS".into());
        }
        match files::metadata(&path)? {
            Some(meta) if parts.peek().is_some() && !meta.is_dir() => {
                return Ok("FILE_COLLISION".into());
            }
            Some(meta) if parts.peek().is_none() => {
                return if meta.is_file() {
                    files::digest_file(&path)
                } else {
                    Ok("DIRECTORY_COLLISION".into())
                };
            }
            Some(_) => {}
            None => return Ok("MISSING".into()),
        }
    }
    Err("Invalid Git proposal path".into())
}

fn document_path(category: &str, slug: &str, locale: &str) -> String {
    if category == "skills" {
        format!("{locale}/skills/{slug}/SKILL.md")
    } else {
        format!("{locale}/{category}/{slug}.md")
    }
}

#[derive(Default)]
struct AssignmentMapping {
    mapped: BTreeMap<String, BTreeSet<String>>,
    unmapped: BTreeMap<String, u64>,
}

fn assignment_mapping(
    archive: &archive::Archive,
    bindings: &BTreeMap<String, workspace::Binding>,
) -> Result<AssignmentMapping, String> {
    let mut statement = archive
        .connection
        .prepare(
            "SELECT a.asset_id,b.root,b.git_dir,b.common_dir,b.anchor,b.status
         FROM governance_assignments a JOIN bindings b ON a.binding_id=b.id
         ORDER BY a.asset_id,b.id",
        )
        .map_err(sql_error)?;
    let mut rows = statement.query([]).map_err(sql_error)?;
    let mut mapping = AssignmentMapping::default();
    while let Some(row) = rows.next().map_err(sql_error)? {
        let id: String = row.get(0).map_err(sql_error)?;
        let root: String = row.get(1).map_err(sql_error)?;
        let git: String = row.get(2).map_err(sql_error)?;
        let common: String = row.get(3).map_err(sql_error)?;
        let anchor: String = row.get(4).map_err(sql_error)?;
        let status: String = row.get(5).map_err(sql_error)?;
        let mut resolved = None;
        if status == "ACTIVE"
            && matches!(anchor.len(), 40 | 64)
            && anchor
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            for (repo, binding) in bindings {
                if !same_path(&root, &binding.path.to_string_lossy()) {
                    continue;
                }
                let dirs = policy::git(
                    &binding.path,
                    &[
                        "rev-parse",
                        "--path-format=absolute",
                        "--absolute-git-dir",
                        "--git-common-dir",
                    ],
                )
                .unwrap_or_default();
                let mut dirs = dirs.lines();
                if dirs.next().is_some_and(|actual| same_path(&git, actual))
                    && dirs.next().is_some_and(|actual| same_path(&common, actual))
                    && dirs.next().is_none()
                    && policy::git(&binding.path, &["cat-file", "-t", &anchor])
                        .is_ok_and(|kind| kind.trim() == "commit")
                {
                    if resolved.is_some() {
                        resolved = None;
                        break;
                    }
                    resolved = Some(repo);
                }
            }
        }
        if let Some(repo) = resolved {
            mapping.mapped.entry(id).or_default().insert(repo.clone());
        } else {
            *mapping.unmapped.entry(id).or_default() += 1;
        }
    }
    Ok(mapping)
}

fn same_path(legacy: &str, current: &str) -> bool {
    if !Path::new(legacy).is_absolute() {
        return false;
    }
    #[cfg(windows)]
    {
        let legacy = legacy
            .strip_prefix(r"\\?\")
            .unwrap_or(legacy)
            .replace('/', "\\");
        let current = current
            .strip_prefix(r"\\?\")
            .unwrap_or(current)
            .replace('/', "\\");
        legacy.eq_ignore_ascii_case(&current)
    }
    #[cfg(not(windows))]
    {
        legacy == current
    }
}

fn collect(
    root: &Path,
    archive: &archive::Archive,
    bindings: &BTreeMap<String, workspace::Binding>,
    candidates: &mut Vec<Candidate>,
    totals: &mut BTreeMap<&'static str, u64>,
    budget: &mut u64,
) -> Result<(), String> {
    let AssignmentMapping { mapped, unmapped } = assignment_mapping(archive, bindings)?;
    *totals.entry("unmapped_assignments").or_default() += unmapped.values().sum::<u64>();
    *totals.entry("mapped_assignments").or_default() +=
        mapped.values().map(|items| items.len() as u64).sum::<u64>();
    let defaults: u64 = archive
        .connection
        .query_row("SELECT count(*) FROM governance_defaults", [], |row| {
            row.get(0)
        })
        .map_err(sql_error)?;
    *totals.entry("historical_default_profiles").or_default() += defaults;
    let mut statement = archive.connection.prepare(
        "SELECT a.id,a.kind,a.slug,a.revision,a.active_revision,r.document_json,a.draft_json,a.archived,a.deleted_at IS NOT NULL
         FROM governance_assets a LEFT JOIN governance_revisions r ON a.id=r.asset_id AND a.active_revision=r.revision
         ORDER BY a.id",
    ).map_err(sql_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(LegacyAsset {
                id: row.get(0)?,
                kind: row.get(1)?,
                slug: row.get(2)?,
                revision: row.get(3)?,
                active: row.get(4)?,
                published: row.get(5)?,
                draft: row.get(6)?,
                archived: row.get(7)?,
                deleted: row.get(8)?,
            })
        })
        .map_err(sql_error)?;
    for row in rows {
        let row = row.map_err(sql_error)?;
        if row.deleted || row.archived {
            *totals.entry("historical_assets").or_default() += 1;
            continue;
        }
        let Some(category) = documents::category(&row.kind) else {
            *totals.entry("unmapped_assets").or_default() += 1;
            continue;
        };
        if !documents::asset_id(&row.id) || !documents::valid_slug(&row.slug) || row.revision < 1 {
            *totals.entry("unmapped_assets").or_default() += 1;
            continue;
        }
        let raw = row.published.as_deref().or(row.draft.as_deref());
        let path = document_path(category, &row.slug, "en");
        let revision = row.active.unwrap_or(row.revision);
        let asset_ref = sha256(row.id.as_bytes());
        let mut candidate = Candidate {
            asset: json!({
                "id": row.id, "kind": row.kind, "path": path, "revision": revision,
                "status": "DRAFT", "depends_on": [],
                "repositories": mapped.get(&row.id).cloned().unwrap_or_default(),
                "provenance": {
                    "snapshot_sha256": archive.snapshot_sha256,
                    "legacy_asset_id": row.id,
                    "legacy_revision": revision,
                }
            }),
            expected_path_state: target_state(root, &path)?,
            source_receipt_sha256: archive.receipt_sha256.clone(),
            document_sha256: raw.map(|raw| sha256(raw.as_bytes())),
            draft_sha256: row.draft.as_deref().map(|raw| sha256(raw.as_bytes())),
            files: Vec::new(),
            review_issues: BTreeSet::new(),
        };
        if unmapped.contains_key(&row.id) {
            candidate.issue("UNMAPPED_ASSIGNMENT");
        }
        for issue in archive
            .issues
            .iter()
            .filter(|issue| issue.asset_ref == asset_ref)
        {
            if issue.revision == row.active
                || issue.code.starts_with("REVISION_")
                || issue.code == "MISSING_PRIOR_REVISION"
            {
                candidate.issue(&issue.code);
            }
        }
        match raw.map(documents::decode) {
            Some(Ok(document)) if !documents::contains_sensitive(&document) => {
                candidate.asset["depends_on"] = json!(document.dependencies);
                if matches!(document.format, documents::Format::Markdown) {
                    add_files(root, category, &row.slug, document, &mut candidate)?;
                } else {
                    candidate.issue("STRUCTURED_REVIEW_REQUIRED");
                }
            }
            Some(Ok(_)) => candidate.issue("SENSITIVE_DOCUMENT"),
            Some(Err(issue)) => candidate.issue(issue),
            None => candidate.issue("NO_DOCUMENT"),
        }
        if matches!(row.kind.as_str(), "hook" | "command") {
            candidate.issue("PROCEDURE_REVIEW_REQUIRED");
        }
        *budget += serde_json::to_vec(&candidate)
            .map_err(|_| "Cannot encode review candidate")?
            .len() as u64;
        if *budget > PLAN_LIMIT || candidates.len() >= 512 {
            return Err(
                "Legacy mapping exceeds its proposal budget; the archives are retained".into(),
            );
        }
        candidates.push(candidate);
    }
    Ok(())
}

fn add_files(
    root: &Path,
    category: &str,
    slug: &str,
    document: documents::Document,
    candidate: &mut Candidate,
) -> Result<(), String> {
    let en = document_path(category, slug, "en");
    let vi = document_path(category, slug, "vi");
    let pair_has_markers = document
        .en
        .lines()
        .any(|line| line.starts_with("<!-- doc-id: "))
        && document
            .vi
            .lines()
            .any(|line| line.starts_with("<!-- source-sha256: "));
    if !pair_has_markers && !document.vi.trim().is_empty() {
        candidate.issue("TRANSLATION_METADATA_REVIEW");
    }
    candidate
        .files
        .push(FileProposal::new(en, document.en, root)?);
    if !document.vi.is_empty() {
        candidate
            .files
            .push(FileProposal::new(vi, document.vi, root)?);
    }
    for file in document.files {
        let allowed = file.path.rsplit_once('.').is_some_and(|(_, extension)| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "md" | "txt" | "json" | "yaml" | "yml" | "csv"
            )
        });
        if allowed {
            candidate.files.push(FileProposal::new(
                format!("en/{category}/{slug}/{}", file.path),
                file.content,
                root,
            )?);
        } else {
            candidate.issue("ATTACHMENT_FORMAT_REVIEW");
        }
    }
    if candidate
        .files
        .iter()
        .any(|file| file.expected_target != "MISSING")
    {
        candidate.issue("DESTINATION_CONFLICT");
    }
    Ok(())
}

fn check_collisions(candidates: &mut [Candidate], registry: &Value) {
    let registered: Vec<_> = registry["assets"]["entries"]
        .as_array()
        .into_iter()
        .flatten()
        .collect();
    let known_ids: BTreeSet<_> = registered
        .iter()
        .filter_map(|item| item["id"].as_str())
        .map(str::to_ascii_lowercase)
        .collect();
    let known_paths: BTreeSet<_> = registered
        .iter()
        .filter_map(|item| item["path"].as_str())
        .map(str::to_ascii_lowercase)
        .collect();
    let mut ids: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    let mut paths: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, candidate) in candidates.iter_mut().enumerate() {
        let id = candidate.id().to_ascii_lowercase();
        let path = candidate.path().to_ascii_lowercase();
        if known_ids.contains(&id) || known_paths.contains(&path) {
            candidate.issue("AUTHORITY_COLLISION");
        }
        if candidate.expected_path_state != "MISSING" {
            candidate.issue("DESTINATION_CONFLICT");
        }
        ids.entry(id).or_default().push(index);
        paths.entry(path).or_default().push(index);
        for file in candidate.files.iter().skip(1) {
            paths
                .entry(file.path.to_ascii_lowercase())
                .or_default()
                .push(index);
        }
    }
    for group in ids
        .values()
        .chain(paths.values())
        .filter(|indices| indices.len() > 1)
    {
        let reason = if group.iter().all(|index| *index == group[0]) {
            "ATTACHMENT_COLLISION"
        } else {
            "MULTI_SOURCE_COLLISION"
        };
        for index in group {
            candidates[*index].issue(reason);
        }
    }
    for (path, indices) in &paths {
        for (index, _) in path.match_indices('/') {
            if let Some(others) = paths.get(&path[..index]) {
                for index in indices.iter().chain(others) {
                    candidates[*index].issue("PROPOSAL_PATH_COLLISION");
                }
            }
        }
    }
    let mut resolved: BTreeSet<String> = registered
        .iter()
        .filter(|item| item["status"] == "ACTIVE")
        .filter_map(|item| item["id"].as_str().map(str::to_owned))
        .collect();
    let proposed: BTreeSet<_> = candidates.iter().map(|item| item.id().to_owned()).collect();
    for candidate in &mut *candidates {
        if dependencies(candidate)
            .any(|dependency| !proposed.contains(dependency) && !resolved.contains(dependency))
        {
            candidate.issue("UNMAPPED_DEPENDENCY");
        }
    }
    loop {
        let before = resolved.len();
        for candidate in &*candidates {
            if !candidate.review_issues.contains("MULTI_SOURCE_COLLISION")
                && dependencies(candidate).all(|dependency| resolved.contains(dependency))
            {
                resolved.insert(candidate.id().to_owned());
            }
        }
        if before == resolved.len() {
            break;
        }
    }
    for candidate in candidates {
        if dependencies(candidate).any(|dependency| !resolved.contains(dependency)) {
            candidate.issue("DEPENDENCY_REVIEW_REQUIRED");
        }
    }
}

fn dependencies(candidate: &Candidate) -> impl Iterator<Item = &str> {
    candidate.asset["depends_on"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
}

fn prepare(root: &Path, paths: &[&Path], workspace_path: Option<&Path>) -> Result<Plan, String> {
    let registry = policy::registry(root)?;
    policy::validate(root, &registry)?;
    authority::validate(root, &registry)?;
    let target = workspace::resolve(root, None, "cirnet-rules", &registry)?;
    let mut authority_bytes = BTreeMap::new();
    for path in [
        "governance.yaml",
        "VERSION",
        "en/schemas/governance.schema.json",
        "en/schemas/cirnetrules-policy.schema.json",
    ] {
        authority_bytes.insert(
            path,
            sha256(&files::read(&root.join(path), 4 * 1024 * 1024)?),
        );
    }
    for asset in registry["assets"]["entries"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|asset| asset["status"] == "ACTIVE")
    {
        let path = asset["path"]
            .as_str()
            .ok_or("Missing authority asset path")?;
        authority_bytes.insert(
            path,
            sha256(&authority::read_authority(root, path, 2 * 1024 * 1024)?),
        );
    }
    let authority_sha256 =
        sha256(&serde_json::to_vec(&authority_bytes).map_err(|_| "Cannot encode authority state")?);
    let mut bindings = BTreeMap::new();
    if let Some(workspace_path) = workspace_path {
        let workspace_path = files::directory(workspace_path)?;
        for repository in registry["repositories"]
            .as_object()
            .into_iter()
            .flat_map(|entries| entries.keys())
        {
            if repository != "cirnet-rules"
                && let Ok(binding) =
                    workspace::resolve(root, Some(&workspace_path), repository, &registry)
            {
                bindings.insert(repository.to_owned(), binding);
            }
        }
    }
    let target_identity = target.identity.clone();
    let target_commit = target.commit.clone();
    bindings.insert("cirnet-rules".into(), target);
    let mut sources = Vec::new();
    let mut candidates = Vec::new();
    let mut totals = BTreeMap::new();
    let mut budget = 0;
    for path in paths {
        let (archive, _) = archive::open(path)?;
        if sources
            .iter()
            .any(|source: &Source| source.receipt_sha256 == archive.receipt_sha256)
        {
            return Err("Migration archive was selected more than once or has a path alias".into());
        }
        if archive.schema == archive::GOVERNANCE_SCHEMA {
            collect(
                root,
                &archive,
                &bindings,
                &mut candidates,
                &mut totals,
                &mut budget,
            )?;
        } else {
            *totals.entry("pre_governance_archives").or_default() += 1;
        }
        archive.ensure_unchanged()?;
        sources.push(Source {
            schema: archive.schema,
            receipt_sha256: archive.receipt_sha256,
            snapshot_sha256: archive.snapshot_sha256,
            export_sha256: archive.export_sha256,
        });
    }
    sources.sort_by(|a, b| a.receipt_sha256.cmp(&b.receipt_sha256));
    candidates.sort_by(|a, b| {
        (a.path(), a.id(), &a.source_receipt_sha256).cmp(&(
            b.path(),
            b.id(),
            &b.source_receipt_sha256,
        ))
    });
    check_collisions(&mut candidates, &registry);
    let mut issues = BTreeMap::new();
    for issue in candidates
        .iter()
        .flat_map(|candidate| candidate.review_issues.iter())
    {
        *issues.entry(issue).or_insert(0_u64) += 1;
    }
    let summary = json!({
        "archives": sources.len(), "review_candidates": candidates.len(),
        "pre_governance_archives": totals.get("pre_governance_archives").copied().unwrap_or_default(),
        "blocked_candidates": candidates.iter().filter(|candidate| !candidate.review_issues.is_empty()).count(),
        "historical_assets": totals.get("historical_assets").copied().unwrap_or_default(),
        "unmapped_assets": totals.get("unmapped_assets").copied().unwrap_or_default(),
        "historical_default_profiles": totals.get("historical_default_profiles").copied().unwrap_or_default(),
        "mapped_assignments": totals.get("mapped_assignments").copied().unwrap_or_default(),
        "unmapped_assignments": totals.get("unmapped_assignments").copied().unwrap_or_default(),
        "issues": issues,
    });
    Ok(Plan {
        schema: 1,
        target_identity,
        target_commit,
        authority_sha256,
        sources,
        candidates,
        summary,
    })
}

fn plan_bytes(plan: &Plan) -> Result<Vec<u8>, String> {
    let bytes = serde_json::to_vec(plan).map_err(|_| "Cannot encode migration proposal")?;
    if bytes.len() as u64 > PLAN_LIMIT {
        return Err("Migration proposal exceeds its byte budget".into());
    }
    Ok(bytes)
}

pub(super) fn preview(
    root: &Path,
    paths: &[&Path],
    workspace: Option<&Path>,
    output: Option<&Path>,
) -> Result<Value, String> {
    let root = files::directory(root)?;
    let plan = prepare(&root, paths, workspace)?;
    let bytes = plan_bytes(&plan)?;
    let digest = sha256(&bytes);
    if let Some(output) = output {
        let output = files::destination(&root, output)?;
        files::private_dir(&output)?;
        files::publish(&output.join(PROPOSAL), &bytes)?;
    }
    let mapping: Vec<_> = plan.candidates.iter().map(|candidate| {
        json!({
            "asset": candidate.asset, "expected_target": candidate.expected_path_state,
            "document_sha256": candidate.document_sha256, "draft_sha256": candidate.draft_sha256,
            "source_receipt_sha256": candidate.source_receipt_sha256,
            "planned_files": candidate.files.len(), "review_issues": candidate.review_issues,
        })
    }).collect();
    let files = plan
        .candidates
        .iter()
        .flat_map(|candidate| candidate.files.iter().map(|file| (candidate.id(), file)));
    let files_omitted = files.clone().count().saturating_sub(PREVIEW_FILE_LIMIT);
    let file_changes: Vec<_> = files
        .take(PREVIEW_FILE_LIMIT)
        .map(|(asset, file)| {
            json!({
                "asset_id": asset, "path": file.path, "expected_target": file.expected_target,
                "action": if file.expected_target == "MISSING" { "ADD" } else { "CONFLICT" },
                "proposed_sha256": file.sha256, "proposed_bytes": file.content.len(),
            })
        })
        .collect();
    Ok(json!({
        "status": "PASS", "state": "REVIEW_REQUIRED", "preview_sha256": digest,
        "target_commit": plan.target_commit, "authority_sha256": plan.authority_sha256,
        "sources": plan.sources, "summary": plan.summary, "mapping": mapping,
        "file_changes": file_changes, "file_changes_omitted": files_omitted,
        "local_proposal_written": output.is_some(),
    }))
}

fn valid_digest(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

fn state_dir(root: &Path, digest: &str, create: bool) -> Result<PathBuf, String> {
    if !valid_digest(digest) {
        return Err("Expected a lowercase SHA-256 migration preview digest".into());
    }
    let relative = Path::new(STATE).join(digest);
    files::require_ignored(root, &relative)?;
    let mut directory = root.to_path_buf();
    for part in relative.components() {
        directory.push(part.as_os_str());
        if files::case_alias(&directory)? {
            return Err("Migration state directory has a case alias".into());
        }
        match files::metadata(&directory)? {
            Some(meta) if meta.is_dir() => {}
            Some(_) => return Err("Migration state path must be an ordinary directory".into()),
            None if create => {
                if files::private_dir(&directory).is_err()
                    && !files::metadata(&directory)?.is_some_and(|meta| meta.is_dir())
                {
                    return Err("Cannot create local migration state".into());
                }
            }
            None => return Err("Migration proposal state is missing".into()),
        }
    }
    Ok(directory)
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema: u8,
    state: String,
    proposal_sha256: String,
    authority_sha256: String,
    target_identity: String,
    target_commit: String,
    sources: Vec<Source>,
    summary: Value,
    time_unix_seconds: u64,
}

fn now() -> Result<u64, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "Local time is unavailable")?
        .as_secs())
}

fn encode(value: &impl Serialize) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(value).map_err(|_| "Cannot encode migration receipt".into())
}

fn receipt(directory: &Path, digest: &str) -> Result<(Receipt, String), String> {
    let bytes = files::read(&directory.join(RECEIPT), RECEIPT_LIMIT)?;
    let receipt: Receipt =
        serde_json::from_slice(&bytes).map_err(|_| "Invalid migration proposal receipt")?;
    if receipt.schema != 1
        || receipt.state != "STAGED"
        || receipt.proposal_sha256 != digest
        || !valid_digest(&receipt.authority_sha256)
        || !valid_digest(&receipt.target_identity)
    {
        return Err("Migration receipt does not match the reviewed proposal".into());
    }
    Ok((receipt, sha256(&bytes)))
}

fn verify_receipted_proposal(path: &Path, digest: &str, receipt: &Receipt) -> Result<(), String> {
    let bytes = files::read(path, PLAN_LIMIT)?;
    if sha256(&bytes) != digest {
        return Err("Migration proposal digest does not match its receipt".into());
    }
    let proposal: Value =
        serde_json::from_slice(&bytes).map_err(|_| "Invalid staged migration proposal")?;
    if proposal["schema"] != 1
        || proposal["authority_sha256"] != receipt.authority_sha256
        || proposal["target_identity"] != receipt.target_identity
        || proposal["target_commit"] != receipt.target_commit
        || proposal["sources"] != json!(receipt.sources)
        || proposal["summary"] != receipt.summary
    {
        return Err("Migration receipt metadata differs from its staged proposal".into());
    }
    Ok(())
}

pub(super) fn apply(
    root: &Path,
    paths: &[&Path],
    workspace: Option<&Path>,
    digest: &str,
) -> Result<Value, String> {
    let root = files::directory(root)?;
    if sha256(&plan_bytes(&prepare(&root, paths, workspace)?)?) != digest {
        return Err("Migration preview no longer matches the source or destination".into());
    }
    let directory = state_dir(&root, digest, true)?;
    let _lock = files::lock(&directory.join(".lock"))?;
    let plan = prepare(&root, paths, workspace)?;
    let bytes = plan_bytes(&plan)?;
    if sha256(&bytes) != digest {
        return Err("Migration preview changed before staging".into());
    }
    if files::ordinary_file(&directory.join(ROLLBACK_INTENT))? {
        return Err(
            "The reviewed migration proposal was rolled back or has a pending rollback".into(),
        );
    }
    let proposal = directory.join(PROPOSAL);
    let prior_stage = files::ordinary_file(&proposal)?;
    if prior_stage && files::digest_file(&proposal)? != digest {
        return Err("Staged migration proposal has changed".into());
    }
    let prior_receipt = files::ordinary_file(&directory.join(RECEIPT))?;
    if prior_receipt && !prior_stage {
        return Err("Previously staged migration proposal is missing".into());
    }
    if !prior_stage {
        files::publish(&proposal, &bytes)?;
    }
    // Validate the current authority after publishing the local proposal.
    // An interruption here leaves a stage with no receipt, which apply resumes.
    let registry = policy::registry(&root)?;
    policy::validate(&root, &registry)?;
    authority::validate(&root, &registry)?;
    if !prior_receipt {
        let receipt = Receipt {
            schema: 1,
            state: "STAGED".into(),
            proposal_sha256: digest.into(),
            authority_sha256: plan.authority_sha256,
            target_identity: plan.target_identity,
            target_commit: plan.target_commit,
            sources: plan.sources,
            summary: plan.summary.clone(),
            time_unix_seconds: now()?,
        };
        files::publish(&directory.join(RECEIPT), &encode(&receipt)?)?;
    }
    let (receipt, receipt_sha256) = receipt(&directory, digest)?;
    verify_receipted_proposal(&proposal, digest, &receipt)?;
    let state = if prior_receipt {
        "UNCHANGED"
    } else if prior_stage {
        "RESUMED"
    } else {
        "STAGED"
    };
    Ok(json!({
        "status": "PASS", "state": state, "proposal_sha256": digest,
        "receipt_sha256": receipt_sha256, "summary": plan.summary,
    }))
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Rollback {
    schema: u8,
    proposal_sha256: String,
    apply_receipt_sha256: String,
}

fn verify_rollback(path: &Path, digest: &str, receipt_sha256: &str) -> Result<Vec<u8>, String> {
    let bytes = files::read(path, RECEIPT_LIMIT)?;
    let marker: Rollback =
        serde_json::from_slice(&bytes).map_err(|_| "Invalid migration rollback record")?;
    if marker.schema != 1
        || marker.proposal_sha256 != digest
        || marker.apply_receipt_sha256 != receipt_sha256
    {
        return Err("Migration rollback record does not match its apply receipt".into());
    }
    Ok(bytes)
}

pub(super) fn rollback(root: &Path, digest: &str) -> Result<Value, String> {
    let root = files::directory(root)?;
    let directory = state_dir(&root, digest, false)?;
    let _lock = files::lock(&directory.join(".lock"))?;
    let (receipt, receipt_sha256) = receipt(&directory, digest)?;
    let target = directory.join(PROPOSAL);
    let intent = directory.join(ROLLBACK_INTENT);
    let done = directory.join(ROLLBACK_RECEIPT);
    let exists = files::ordinary_file(&target)?;
    if exists {
        verify_receipted_proposal(&target, digest, &receipt)?;
    }
    if files::ordinary_file(&done)? {
        let bytes = verify_rollback(&done, digest, &receipt_sha256)?;
        if exists {
            return Err("A file appeared at the rolled-back migration proposal path".into());
        }
        return Ok(
            json!({"status":"PASS","state":"UNCHANGED","proposal_sha256":digest,"receipt_sha256":sha256(&bytes)}),
        );
    }
    if !files::ordinary_file(&intent)? {
        if !exists {
            return Err("Migration proposal disappeared before rollback".into());
        }
        let marker = Rollback {
            schema: 1,
            proposal_sha256: digest.into(),
            apply_receipt_sha256: receipt_sha256.clone(),
        };
        files::publish(&intent, &encode(&marker)?)?;
    }
    let bytes = verify_rollback(&intent, digest, &receipt_sha256)?;
    if exists {
        if files::digest_file(&target)? != digest {
            return Err("Migration proposal changed during rollback".into());
        }
        fs::remove_file(&target)
            .map_err(|_| "Cannot remove the exact staged migration proposal")?;
        files::sync_dir(&directory)?;
    }
    files::publish(&done, &bytes)?;
    Ok(
        json!({"status":"PASS","state":"ROLLED_BACK","proposal_sha256":digest,"receipt_sha256":sha256(&bytes)}),
    )
}

pub(super) fn audit(root: &Path, digest: &str) -> Result<Value, String> {
    let root = files::directory(root)?;
    let directory = state_dir(&root, digest, false)?;
    let (receipt, receipt_sha256) = receipt(&directory, digest)?;
    let proposal = directory.join(PROPOSAL);
    let rollback = directory.join(ROLLBACK_RECEIPT);
    let intent = directory.join(ROLLBACK_INTENT);
    let (status, state, rollback_sha256) = if files::ordinary_file(&rollback)? {
        if files::ordinary_file(&proposal)? {
            return Err("A file appeared at the rolled-back migration proposal path".into());
        }
        let bytes = verify_rollback(&rollback, digest, &receipt_sha256)?;
        if bytes != verify_rollback(&intent, digest, &receipt_sha256)? {
            return Err("Migration rollback receipt does not match its intent".into());
        }
        ("PASS", "ROLLED_BACK", Some(sha256(&bytes)))
    } else if files::ordinary_file(&intent)? {
        verify_rollback(&intent, digest, &receipt_sha256)?;
        ("BLOCKED", "ROLLBACK_PENDING", None)
    } else {
        verify_receipted_proposal(&proposal, digest, &receipt)?;
        ("PASS", "STAGED", None)
    };
    Ok(json!({
        "status": status, "state": state, "proposal_sha256": digest, "receipt_sha256": receipt_sha256,
        "rollback_receipt_sha256": rollback_sha256, "target_commit": receipt.target_commit,
        "sources": receipt.sources, "summary": receipt.summary,
        "time_unix_seconds": receipt.time_unix_seconds,
    }))
}

pub(super) fn adoption(
    root: &Path,
    digest: &str,
    approved_commit: &str,
    actor: &str,
) -> Result<Value, String> {
    if actor.is_empty()
        || actor.len() > 96
        || !actor
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_@.".contains(&byte))
        || workspace::contains_known_secrets(actor)
    {
        return Err("Adoption requires a bounded operator label".into());
    }
    let root = files::directory(root)?;
    let directory = state_dir(&root, digest, false)?;
    let _lock = files::lock(&directory.join(".lock"))?;
    if files::ordinary_file(&directory.join(ROLLBACK_INTENT))? {
        return Err("Migration proposal was rolled back or has a pending rollback".into());
    }
    let (receipt, receipt_sha256) = receipt(&directory, digest)?;
    let proposal = directory.join(PROPOSAL);
    verify_receipted_proposal(&proposal, digest, &receipt)?;
    let (commit, clean) = policy::checkout(&root)?;
    if !clean
        || commit != approved_commit
        || commit == receipt.target_commit
        || !policy::governance_origin_matches(&root)?
        || policy::git(
            &root,
            &[
                "merge-base",
                "--is-ancestor",
                &receipt.target_commit,
                &commit,
            ],
        )
        .is_err()
    {
        return Err("Adoption receipt requires the explicit clean reviewed authority commit descending from staging".into());
    }
    let registry = policy::registry(&root)?;
    policy::validate(&root, &registry)?;
    crate::docs::verify(&root)?;
    let data: Value = serde_json::from_slice(&files::read(&proposal, PLAN_LIMIT)?)
        .map_err(|_| "Invalid receipted migration proposal")?;
    let candidates = data["candidates"]
        .as_array()
        .ok_or("Missing migration candidates")?;
    let entries = registry["assets"]["entries"]
        .as_array()
        .ok_or("Missing authority asset catalog")?;
    let mut adopted = Vec::new();
    for candidate in candidates {
        let asset = &candidate["asset"];
        let entry = entries.iter().find(|entry| {
            ["id", "kind", "path", "provenance"]
                .iter()
                .all(|field| entry[*field] == asset[*field])
        });
        let Some(entry) = entry else {
            continue;
        };
        let Some(proposed_files) = candidate["files"]
            .as_array()
            .filter(|rows| !rows.is_empty())
        else {
            continue;
        };
        let mut matches = true;
        for file in proposed_files {
            let relative = file["path"]
                .as_str()
                .ok_or("Invalid receipted asset path")?;
            let path = policy::safe_path(&root, relative)?;
            matches &= files::ordinary_file(&path)?
                && files::digest_file(&path)? == file["sha256"]
                && policy::git(
                    &root,
                    &[
                        "--literal-pathspecs",
                        "ls-files",
                        "--error-unmatch",
                        "--",
                        relative,
                    ],
                )
                .is_ok();
        }
        if matches {
            adopted.push(json!({"id":asset["id"],"status":entry["status"]}));
        }
    }
    if adopted.is_empty() {
        return Ok(
            json!({"status":"BLOCKED","state":"PROPOSAL_STAGED","reason":"No reviewed proposal candidate matches the committed authority bytes and provenance"}),
        );
    }
    let authority = policy::read(&policy::safe_path(&root, "governance.yaml")?)?;
    let mut result = json!({
        "schema":1,"status":"PASS","state":if adopted.len() == candidates.len() {"ADOPTED_IN_GIT"} else {"PARTIALLY_ADOPTED"},
        "proposal_sha256":digest,"staging_receipt_sha256":receipt_sha256,
        "authority_commit":commit,"authority_sha256":sha256(authority.as_bytes()),
        "actor":actor,"adopted":adopted,"pending":candidates.len()-adopted.len(),
        "trust":"local operator assertion with verified committed bytes"
    });
    let destination = directory.join(format!("adoption-{commit}.json"));
    if files::ordinary_file(&destination)? {
        let bytes = files::read(&destination, RECEIPT_LIMIT)?;
        let mut previous: Value =
            serde_json::from_slice(&bytes).map_err(|_| "Invalid adoption receipt")?;
        let time = previous
            .as_object_mut()
            .and_then(|obj| obj.remove("time_unix_seconds"));
        if previous != result || time.as_ref().and_then(Value::as_u64).is_none() {
            return Err("Existing adoption receipt conflicts with this confirmation".into());
        }
        result["time_unix_seconds"] = time.unwrap();
        result["receipt_sha256"] = json!(sha256(&bytes));
        return Ok(result);
    }
    if policy::checkout(&root)? != (commit, true) {
        return Err("Authority checkout changed before receipt publication".into());
    }
    result["time_unix_seconds"] = json!(now()?);
    let bytes = encode(&result)?;
    files::publish(&destination, &bytes)?;
    result["receipt_sha256"] = json!(sha256(&bytes));
    Ok(result)
}
