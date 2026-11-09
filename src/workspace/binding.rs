use super::*;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    time::{SystemTime, UNIX_EPOCH},
};

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
    repository(registry, repo)?;
    if repo == "cirnet-rules" {
        let git = inspect_git(root)?;
        // The governance root is the operator-selected invocation root. Its
        // physical identity does not change merely because local source is dirty.
        let identity = fingerprint(&git, "")?;
        return Ok(Binding {
            path: git.path,
            identity,
            commit: git.commit,
        });
    }
    let workspace = workspace_path(root, workspace)?;
    let relative = record_path(&workspace, repo)?;
    let file = scoped_path(root, &relative)?;
    let row: Record = read_local(&file)
        .map_err(|_| format!("Repository {repo} has no readable confirmed local binding"))?;
    if row.schema != 1 || !valid_hash(&row.identity) || !git_id(&row.anchor) {
        return Err(format!("Invalid local binding for {repo}"));
    }
    let path = scoped_path(&workspace, &row.relative)?;
    let git = inspect_git(&path)?;
    if git.path != path
        || fingerprint(&git, &row.anchor)? != row.identity
        || crate::policy::git(&path, &["cat-file", "-t", &row.anchor])
            .ok()
            .as_deref()
            .map(str::trim)
            != Some("commit")
    {
        return Err(format!(
            "Repository {repo} worktree identity changed; rebind explicitly"
        ));
    }
    Ok(Binding {
        path: git.path,
        identity: row.identity,
        commit: git.commit,
    })
}

pub(super) fn bind(
    root: &Path,
    workspace: Option<&Path>,
    registry: &Value,
    repo: &str,
    relative: &str,
    actor: &str,
    expected: Option<&str>,
) -> Result<Value, String> {
    repository(registry, repo)?;
    if repo == "cirnet-rules" {
        return Err("The invocation root already binds cirnet-rules".into());
    }
    if actor.is_empty()
        || actor.len() > 96
        || !actor
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.@".contains(&b))
    {
        return Err("Invalid local binding actor label".into());
    }
    let workspace = workspace_path(root, workspace)?;
    let path = scoped_path(&workspace, relative)?;
    let git = inspect_git(&path)?;
    if git.path != path || git.path == root {
        return Err("Selected path is not an independent checkout root".into());
    }
    if let Some(anchor) = registry["repositories"][repo]["snapshot"]["commit"].as_str()
        && (!git_id(anchor)
            || crate::policy::git(&path, &["cat-file", "-t", anchor])
                .ok()
                .as_deref()
                .map(str::trim)
                != Some("commit"))
    {
        return Err("Checkout does not contain the reviewed registry Git anchor".into());
    }
    let record_path = record_path(&workspace, repo)?;
    let parent = record_path.rsplit_once('/').unwrap().0;
    local_directory(root, parent)?;
    let file = scoped_path(root, &record_path)?;
    let _lock = LocalLock::acquire(file.parent().unwrap().join(".workspace.lock"))?;
    let prior: Option<Record> = if file
        .try_exists()
        .map_err(|_| "Cannot inspect binding file")?
    {
        Some(read_local(&file)?)
    } else {
        None
    };
    if let Some(prior) = &prior
        && prior.relative == relative
        && resolve(root, Some(&workspace), repo, registry).is_ok()
    {
        return Ok(json!({
            "status":"PASS","state":"UNCHANGED","repository":repo,"path":relative,
            "binding":prior.identity,"linked_worktree":git.linked_worktree
        }));
    }
    if expected != prior.as_ref().map(|row| row.identity.as_str()) {
        return Err(
            "Binding changed or exists; use --replace with its current identity after review"
                .into(),
        );
    }
    let identity = fingerprint(&git, &git.commit)?;
    // Two different repository IDs must not be allowed to route to the
    // same exact active worktree in a workspace namespace.
    let records = file.parent().unwrap();
    let registered = registry["repositories"]
        .as_object()
        .ok_or("Missing repository registry")?;
    for (index, item) in fs::read_dir(records)
        .map_err(|_| "Cannot list local binding records")?
        .enumerate()
    {
        if index >= 1024 {
            return Err("Local binding metadata exceeds scan limit".into());
        }
        let item = item.map_err(|_| "Cannot inspect local binding record")?;
        if item.path() == file || item.path().extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        if item
            .path()
            .file_stem()
            .and_then(|x| x.to_str())
            .is_some_and(|name| registered.contains_key(name))
        {
            let other: Record = read_local(&item.path())?;
            if scoped_path(&workspace, &other.relative)?
                .canonicalize()
                .ok()
                .as_ref()
                == Some(&path)
            {
                return Err("This worktree is already bound to another repository ID".into());
            }
        }
    }
    let record = Record {
        schema: 1,
        relative: relative.to_owned(),
        anchor: git.commit,
        identity,
        actor: actor.to_owned(),
    };
    write_local(&file, &record)?;
    Ok(json!({
        "status":"PASS","state":"CONFIRMED_LOCAL","repository":repo,"path":relative,
        "binding":record.identity,"linked_worktree":git.linked_worktree,
        "permissions_granted":false
    }))
}

pub(super) fn discover(
    root: &Path,
    workspace: Option<&Path>,
    registry: &Value,
) -> Result<Value, String> {
    let workspace = workspace_path(root, workspace)?;
    let max = configuration(registry, "discovery_entries", 128, 1024)? as usize;
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
        "candidates":candidates.into_values().collect::<Vec<_>>(),
        "entries_examined":examined,"truncated":truncated,"max_depth":1,
        "inspection":"Local directory and Git marker metadata only; bind validates checkout identity",
        "release_checked":false,"network_used":false
    }))
}

fn workspace_path(root: &Path, workspace: Option<&Path>) -> Result<PathBuf, String> {
    let selected = workspace
        .or_else(|| root.parent())
        .ok_or("Local workspace root unavailable")?;
    local_drive(selected)?;
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
    Ok(GitIdentity {
        path: actual,
        admin,
        common,
        commit: commit.to_owned(),
        linked_worktree: metadata.is_file(),
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
