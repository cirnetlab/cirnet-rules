mod named;

use super::*;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    time::{SystemTime, UNIX_EPOCH},
};

pub(in crate::workspace) use named::run as run_named;
pub(crate) use named::{freeze_task_bindings, resolve_named, resolve_task, validate_task_bindings};

#[derive(Debug)]
pub struct Binding {
    pub path: PathBuf,
    pub identity: String,
    pub commit: String,
}

#[derive(Debug)]
struct GitIdentity {
    path: PathBuf,
    admin: PathBuf,
    common: PathBuf,
    commit: String,
    linked_worktree: bool,
    branch: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema: u8,
    relative: String,
    anchor: String,
    identity: String,
    actor: String,
}

pub fn resolve(
    root: &Path,
    workspace: Option<&Path>,
    repo: &str,
    registry: &Value,
) -> Result<Binding, String> {
    resolve_default(root, workspace, repo, registry).map(|(git, identity)| git.binding(identity))
}

fn resolve_default(
    root: &Path,
    workspace: Option<&Path>,
    repo: &str,
    registry: &Value,
) -> Result<(GitIdentity, String), String> {
    repository(registry, repo)?;
    if repo == "cirnet-rules" {
        let git = inspect_git(root)?;
        let identity = fingerprint(&git, "")?;
        return Ok((git, identity));
    }
    let workspace = workspace_path(root, workspace)?;
    let relative = record_path(&workspace, repo)?;
    let file = scoped_path(root, &relative)?;
    let row = read_default(&file, repo)?;
    let path = scoped_path(&workspace, &row.relative)?;
    let git = inspect_git(&path)?;
    if git.path != path
        || fingerprint(&git, &row.anchor)? != row.identity
        || !contains_commit(&path, &row.anchor)
    {
        return Err(format!(
            "Repository {repo} worktree identity changed; rebind explicitly"
        ));
    }
    Ok((git, row.identity))
}

fn read_default(file: &Path, repo: &str) -> Result<Record, String> {
    let row: Record = read_local(file)
        .map_err(|_| format!("Repository {repo} has no readable confirmed local binding"))?;
    if row.schema != 1 || !valid_hash(&row.identity) || !git_id(&row.anchor) {
        return Err(format!("Invalid local binding for {repo}"));
    }
    validate_relative(&row.relative)?;
    validate_actor(&row.actor)?;
    Ok(row)
}

fn contains_commit(path: &Path, anchor: &str) -> bool {
    git_id(anchor)
        && crate::policy::git(path, &["cat-file", "-t", anchor])
            .ok()
            .as_deref()
            .map(str::trim)
            == Some("commit")
}

impl GitIdentity {
    fn binding(self, identity: String) -> Binding {
        Binding {
            path: self.path,
            identity,
            commit: self.commit,
        }
    }
}

pub(super) fn bind(
    root: &Path,
    workspace: Option<&Path>,
    registry: &Value,
    repo: &str,
    relative: &str,
    actor: &str,
    review: (Option<&str>, Option<&str>),
) -> Result<Value, String> {
    let workspace = workspace_path(root, workspace)?;
    let (git, record, preview) =
        binding_proposal(root, &workspace, registry, repo, relative, actor, review.0)?;
    if review
        .1
        .is_some_and(|expected| preview["plan_sha256"] != expected)
    {
        return Err("Binding preview changed. Review the current worktree again.".into());
    }
    if preview["state"] == "UNCHANGED" {
        return Ok(json!({
            "status":"PASS","state":"UNCHANGED","repository":repo,"path":relative,
            "binding":preview["binding"],"linked_worktree":git.linked_worktree
        }));
    }
    let record_path = record_path(&workspace, repo)?;
    let parent = record_path.rsplit_once('/').unwrap().0;
    local_directory(root, parent)?;
    let file = scoped_path(root, &record_path)?;
    let _coordinator = crate::tasks::lock(root, "_coordination")?;
    let _lock = LocalLock::acquire(file.parent().unwrap().join(".workspace.lock"))?;
    let (_, current, fresh) =
        binding_proposal(root, &workspace, registry, repo, relative, actor, review.0)?;
    if preview["plan_sha256"] != fresh["plan_sha256"] {
        return Err("Binding changed while acquiring the write lock. Review again.".into());
    }
    write_local(&file, &current)?;
    Ok(json!({
        "status":"PASS","state":"CONFIRMED_LOCAL","repository":repo,"path":relative,
        "binding":record.identity,"linked_worktree":git.linked_worktree,
        "permissions_granted":false
    }))
}

pub(super) fn preview_bind(
    root: &Path,
    workspace: Option<&Path>,
    registry: &Value,
    repo: &str,
    relative: &str,
    actor: &str,
    expected: Option<&str>,
) -> Result<Value, String> {
    let workspace = workspace_path(root, workspace)?;
    binding_proposal(root, &workspace, registry, repo, relative, actor, expected)
        .map(|(_, _, preview)| preview)
}

fn binding_proposal(
    root: &Path,
    workspace: &Path,
    registry: &Value,
    repo: &str,
    relative: &str,
    actor: &str,
    expected: Option<&str>,
) -> Result<(GitIdentity, Record, Value), String> {
    repository(registry, repo)?;
    if repo == "cirnet-rules" {
        return Err("The invocation root already binds cirnet-rules".into());
    }
    validate_actor(actor)?;
    let path = scoped_path(workspace, relative)?;
    let git = inspect_git(&path)?;
    if git.path != path || git.path == root {
        return Err("Selected path is not an independent checkout root".into());
    }
    if let Some(anchor) = registry["repositories"][repo]["snapshot"]["commit"].as_str()
        && !contains_commit(&path, anchor)
    {
        return Err("Checkout does not contain the reviewed registry Git anchor".into());
    }
    let record_path = record_path(workspace, repo)?;
    let file = scoped_path(root, &record_path)?;
    let prior: Option<Record> = if file
        .try_exists()
        .map_err(|_| "Cannot inspect binding file")?
    {
        Some(read_local(&file)?)
    } else {
        None
    };
    let unchanged = prior
        .as_ref()
        .is_some_and(|prior| prior.relative == relative)
        && resolve(root, Some(workspace), repo, registry).is_ok();
    if !unchanged && expected != prior.as_ref().map(|row| row.identity.as_str()) {
        return Err(
            "Binding changed or exists; use --replace with its current identity after review"
                .into(),
        );
    }
    let identity = fingerprint(&git, &git.commit)?;
    named::check_repository_path(root, workspace, registry, repo, &git)?;
    let record = Record {
        schema: 1,
        relative: relative.to_owned(),
        anchor: git.commit.clone(),
        identity,
        actor: actor.to_owned(),
    };
    let mut preview = json!({
        "status":"PASS","state":if unchanged {"UNCHANGED"} else {"DRY_RUN"},"repository":repo,"path":relative,
        "binding":prior.as_ref().filter(|_| unchanged).map_or(&record.identity, |record| &record.identity),
        "previous_binding":prior.as_ref().map(|record| &record.identity),
        "head":git.commit,"actor":actor,"linked_worktree":git.linked_worktree,
        "permissions_granted":false
    });
    preview["plan_sha256"] = json!(crate::docs::sha256(
        &serde_json::to_vec(&preview).map_err(|_| "Cannot encode binding preview")?
    ));
    Ok((git, record, preview))
}

fn validate_actor(actor: &str) -> Result<(), String> {
    if actor.is_empty()
        || actor.len() > 96
        || !actor
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.@".contains(&b))
    {
        return Err("Invalid local binding actor label".into());
    }
    Ok(())
}

pub(super) fn discover(
    root: &Path,
    workspace: Option<&Path>,
    registry: &Value,
) -> Result<Value, String> {
    let workspace = workspace_path(root, workspace)?;
    let max = configuration(registry, "discovery_entries", 128, 1024)? as usize;
    let repositories: Vec<_> = registry["repositories"]
        .as_object()
        .into_iter()
        .flat_map(|repositories| repositories.iter())
        .map(|(id, definition)| {
            json!({
                "id":id,"classification":definition["classification"].as_str().unwrap_or("active"),
                "profile":definition["profile"]
            })
        })
        .collect();
    let mut examined = 0;
    let mut truncated = false;
    let mut candidates = BTreeMap::new();
    for child in fs::read_dir(&workspace).map_err(|_| "Cannot read workspace metadata")? {
        if examined == max {
            truncated = true;
            break;
        }
        examined += 1;
        let child = child.map_err(|_| "Cannot inspect workspace metadata")?;
        let Some(name) = child.file_name().into_string().ok() else {
            continue;
        };
        if !child
            .file_type()
            .map_err(|_| "Cannot inspect workspace entry")?
            .is_dir()
            || validate_relative(&name).is_err()
        {
            continue;
        }
        let Ok(marker) = fs::symlink_metadata(child.path().join(".git")) else {
            continue;
        };
        if !ordinary(&marker) || !(marker.is_file() || marker.is_dir()) {
            continue;
        }
        let definition = registry["repositories"].get(&name);
        candidates.insert(
            name.clone(),
            json!({
                "path":name,"linked_worktree_marker":marker.is_file(),
                "classification":definition.map(|row| row["classification"].as_str().unwrap_or("active")).unwrap_or("unregistered"),
                "repo_id_hint":definition.map(|_| name.as_str()),"trusted":false
            }),
        );
    }
    Ok(json!({
        "status":if truncated {"BLOCKED"} else {"PASS"},
        "repositories":repositories,"candidates":candidates.into_values().collect::<Vec<_>>(),
        "entries_examined":examined,"truncated":truncated,"max_depth":1,
        "inspection":"Local directory and Git marker metadata only; bind validates checkout identity",
        "release_checked":false,"network_used":false
    }))
}

pub(crate) fn workspace_path(root: &Path, workspace: Option<&Path>) -> Result<PathBuf, String> {
    let selected = workspace
        .or_else(|| root.parent())
        .ok_or("Local workspace root unavailable")?;
    local_drive(selected)?;
    ordinary_ancestors(selected)?;
    let canonical = selected
        .canonicalize()
        .map_err(|_| "Local workspace root unavailable")?;
    if !canonical.is_dir() {
        return Err("Local workspace is not a directory".into());
    }
    ordinary_ancestors(&canonical)?;
    Ok(canonical)
}

fn inspect_git(path: &Path) -> Result<GitIdentity, String> {
    ordinary_ancestors(path)?;
    let metadata =
        fs::symlink_metadata(path.join(".git")).map_err(|_| "No local Git checkout marker")?;
    if !ordinary(&metadata) || !(metadata.is_file() || metadata.is_dir()) {
        return Err("Invalid local Git checkout marker".into());
    }
    let marker = path.join(".git");
    let expected_admin = if metadata.is_file() {
        let declaration = marker_text(&marker)?;
        let target = declaration
            .strip_prefix("gitdir: ")
            .ok_or("Invalid Git worktree marker")?;
        metadata_directory(path, target)?
    } else {
        marker
            .canonicalize()
            .map_err(|_| "Git directory unavailable")?
    };
    let common_marker = expected_admin.join("commondir");
    let expected_common = if common_marker
        .try_exists()
        .map_err(|_| "Cannot inspect Git common directory")?
    {
        metadata_directory(&expected_admin, &marker_text(&common_marker)?)?
    } else {
        expected_admin.clone()
    };
    let values = crate::policy::git(
        path,
        &[
            "rev-parse",
            "--path-format=absolute",
            "--show-toplevel",
            "--git-dir",
            "--git-common-dir",
            "--verify",
            "HEAD^{commit}",
        ],
    )?;
    let mut values = values.lines();
    let mut local_path = || -> Result<PathBuf, String> {
        let declared = PathBuf::from(values.next().ok_or("Incomplete Git worktree identity")?);
        if !declared.is_absolute() {
            return Err("Git did not return an absolute local metadata path".into());
        }
        local_drive(&declared)?;
        ordinary_ancestors(&declared)?;
        declared
            .canonicalize()
            .map_err(|_| "Git local metadata path unavailable".into())
    };
    let actual = local_path()?;
    if actual != path {
        return Err("Git worktree root does not match selected checkout".into());
    }
    let admin = local_path()?;
    let common = local_path()?;
    if admin != expected_admin || common != expected_common {
        return Err("Git administrative directory does not match the reviewed local marker".into());
    }
    let commit = values.next().ok_or("Incomplete Git commit identity")?;
    if !git_id(commit) || values.next().is_some() {
        return Err("Git checkout has no valid commit".into());
    }
    if admin != common {
        let backlink = marker_text(&admin.join("gitdir"))?;
        let backlink = Path::new(&backlink);
        local_drive(backlink)?;
        let backlink = admin.join(backlink);
        ordinary_ancestors(&backlink)?;
        let matches = match (backlink.canonicalize(), marker.canonicalize()) {
            (Ok(actual), Ok(expected)) => actual == expected,
            _ => false,
        };
        if !matches {
            return Err("Git administrative directory points to a different worktree".into());
        }
    }
    let head = marker_text(&admin.join("HEAD"))?;
    let branch = if let Some(reference) = head.strip_prefix("ref: refs/heads/") {
        if reference.is_empty() {
            return Err("Git checkout has no valid branch reference".into());
        }
        Some(reference.to_owned())
    } else if head == commit {
        None
    } else {
        return Err("Git HEAD identity changed or is outside the local branch policy".into());
    };
    Ok(GitIdentity {
        path: actual,
        admin,
        common,
        commit: commit.to_owned(),
        linked_worktree: metadata.is_file(),
        branch,
    })
}

fn local_drive(path: &Path) -> Result<(), String> {
    use std::path::{Component, Prefix};
    if matches!(path.components().next(),
        Some(Component::Prefix(prefix))
            if !path.is_absolute() || !matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_))
    ) {
        return Err("Network or device paths are outside the local workspace scope".into());
    }
    Ok(())
}

fn marker_text(path: &Path) -> Result<String, String> {
    ordinary_ancestors(path)?;
    let metadata = fs::symlink_metadata(path).map_err(|_| "Cannot inspect Git marker")?;
    if !metadata.is_file() || metadata.len() > 4096 {
        return Err("Git marker exceeds metadata boundary".into());
    }
    let mut content = String::new();
    fs::File::open(path)
        .map_err(|_| "Cannot open Git marker")?
        .take(4097)
        .read_to_string(&mut content)
        .map_err(|_| "Cannot read UTF-8 Git marker")?;
    let content = content.trim_end_matches(['\r', '\n']);
    if content.is_empty() || content.len() > 4096 || content.chars().any(char::is_control) {
        return Err("Invalid Git marker contents".into());
    }
    Ok(content.to_owned())
}

fn metadata_directory(parent: &Path, path: &str) -> Result<PathBuf, String> {
    let path = Path::new(path);
    local_drive(path)?;
    let target = parent.join(path);
    ordinary_ancestors(&target)?;
    let canonical = target
        .canonicalize()
        .map_err(|_| "Git local metadata directory unavailable")?;
    if !canonical.is_dir() {
        return Err("Git metadata destination is not a directory".into());
    }
    Ok(canonical)
}

fn git_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

fn fingerprint(git: &GitIdentity, anchor: &str) -> Result<String, String> {
    let bytes = serde_json::to_vec(&(&git.path, &git.admin, &git.common, anchor))
        .map_err(|_| "Cannot encode local worktree identity")?;
    Ok(crate::docs::sha256(&bytes))
}

fn record_path(workspace: &Path, repo: &str) -> Result<String, String> {
    let workspace = workspace.to_str().ok_or("Non-UTF-8 local workspace path")?;
    let namespace = crate::docs::sha256(workspace.as_bytes());
    Ok(format!(".cirnetrules/bindings/{namespace}/{repo}.json"))
}

pub(super) fn valid_hash(value: &str) -> bool {
    value.len() == 64 && git_id(value)
}

pub(super) fn local_directory(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = scoped_path(root, relative)?;
    let mut current = root.to_path_buf();
    for part in relative.split('/') {
        current.push(part);
        match fs::create_dir(&current) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return Err("Cannot create control-plane local metadata directory".into()),
        }
        let metadata =
            fs::symlink_metadata(&current).map_err(|_| "Local metadata directory unavailable")?;
        if !ordinary(&metadata) || !metadata.is_dir() {
            return Err("Control-plane local path is not an ordinary directory".into());
        }
    }
    Ok(path)
}

pub(super) fn read_local<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    ordinary_ancestors(path)?;
    let metadata = fs::symlink_metadata(path).map_err(|_| "Local metadata is unavailable")?;
    if !metadata.is_file() || metadata.len() > 128 * 1024 {
        return Err("Local metadata exceeds its format boundary".into());
    }
    let mut data = Vec::new();
    fs::File::open(path)
        .map_err(|_| "Cannot open local metadata")?
        .take(128 * 1024 + 1)
        .read_to_end(&mut data)
        .map_err(|_| "Cannot read local metadata")?;
    if data.len() > 128 * 1024 {
        return Err("Local metadata changed beyond the read limit".into());
    }
    serde_json::from_slice(&data).map_err(|_| "Invalid local metadata JSON".into())
}

pub(super) fn write_local<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path.parent().ok_or("Local metadata path has no parent")?;
    ordinary_ancestors(parent)?;
    if path
        .try_exists()
        .map_err(|_| "Cannot inspect local metadata destination")?
    {
        ordinary_ancestors(path)?;
    }
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "Invalid local clock")?
        .as_nanos();
    let temporary = parent.join(format!(".cirnetrules-{}-{suffix}.tmp", std::process::id()));
    let data = serde_json::to_vec(value).map_err(|_| "Cannot encode local metadata")?;
    if data.len() > 128 * 1024 {
        return Err("Local metadata exceeds its write limit".into());
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|_| "Cannot create local metadata stage")?;
    file.write_all(&data)
        .and_then(|_| file.sync_all())
        .map_err(|_| "Cannot persist local metadata stage")?;
    drop(file);
    ordinary_ancestors(&temporary)?;
    if path
        .try_exists()
        .map_err(|_| "Cannot inspect local metadata destination")?
    {
        ordinary_ancestors(path)?;
    }
    fs::rename(temporary, path).map_err(|_| "Cannot replace local metadata atomically".into())
}

pub(super) struct LocalLock(PathBuf);

impl LocalLock {
    pub(super) fn acquire(path: PathBuf) -> Result<Self, String> {
        ordinary_ancestors(path.parent().ok_or("Invalid lock destination")?)?;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|_| "Local control-plane operation is busy; inspect any abandoned lock")?;
        Ok(Self(path))
    }
}

impl Drop for LocalLock {
    fn drop(&mut self) {
        if ordinary_ancestors(&self.0).is_ok() {
            let _ = fs::remove_file(&self.0);
        }
    }
}
