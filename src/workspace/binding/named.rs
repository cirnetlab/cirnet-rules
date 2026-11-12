use super::*;

const DEFAULT: &str = "default";
const MAX_RECORDS: usize = 1024;
const SCOPES: [&str; 4] = [
    "read_scope",
    "write_scope",
    "verification_scope",
    "documentation_scope",
];

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BindingRecord {
    schema: u8,
    binding_id: String,
    repo_id: String,
    workspace_id: String,
    identity: String,
    physical_identity: String,
    git_admin_identity: String,
    git_common_identity: String,
    anchor: String,
    relative: String,
    branch: Option<String>,
    revision: u64,
    actor: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct TaskExecutionBinding {
    pub binding_id: String,
    pub revision: u64,
    pub identity: String,
    pub physical_identity: String,
    pub base_commit: String,
    pub workspace_id: String,
    pub branch: Option<String>,
    pub scope_sha256: String,
}

fn binding_id(id: &str) -> Result<(), String> {
    if !valid_id(id) || id.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return Err(
            "Invalid binding ID; use lowercase ASCII letters, digits, hyphen and underscore".into(),
        );
    }
    validate_relative(id).map_err(|_| "Invalid binding ID".into())
}

fn workspace_id(workspace: &Path) -> Result<String, String> {
    let text = workspace.to_str().ok_or("Non-UTF-8 local workspace path")?;
    Ok(crate::docs::sha256(text.as_bytes()))
}

fn named_dir(workspace: &Path, repo: &str) -> Result<String, String> {
    Ok(format!(
        ".cirnetrules/bindings/{}/named/{repo}",
        workspace_id(workspace)?
    ))
}

fn named_file(root: &Path, workspace: &Path, repo: &str, id: &str) -> Result<PathBuf, String> {
    binding_id(id)?;
    scoped_path(root, &format!("{}/{id}.json", named_dir(workspace, repo)?))
}

fn read_record(
    root: &Path,
    workspace: &Path,
    repo: &str,
    id: &str,
) -> Result<BindingRecord, String> {
    let row: BindingRecord = read_local(&named_file(root, workspace, repo, id)?)?;
    if row.schema != 1
        || row.revision != 1
        || row.binding_id != id
        || row.repo_id != repo
        || row.workspace_id != workspace_id(workspace)?
        || !valid_hash(&row.identity)
        || !valid_hash(&row.physical_identity)
        || !valid_hash(&row.git_admin_identity)
        || !valid_hash(&row.git_common_identity)
        || !git_id(&row.anchor)
        || row.branch.as_ref().is_some_and(|branch| {
            branch.is_empty() || branch.len() > 4096 || branch.chars().any(char::is_control)
        })
    {
        return Err("Invalid local named binding record".into());
    }
    validate_relative(&row.relative)?;
    validate_actor(&row.actor)?;
    Ok(row)
}

fn physical(git: &GitIdentity) -> Result<String, String> {
    let bytes = serde_json::to_vec(&(&git.admin, &git.common))
        .map_err(|_| "Cannot encode physical worktree identity")?;
    Ok(crate::docs::sha256(&bytes))
}

fn inspect_record(workspace: &Path, record: &BindingRecord) -> Result<GitIdentity, String> {
    let git = inspect_git(&scoped_path(workspace, &record.relative)?)?;
    if record.branch != git.branch
        || record.identity != fingerprint(&git, &record.anchor)?
        || record.physical_identity != physical(&git)?
        || record.git_admin_identity != workspace_id(&git.admin)?
        || record.git_common_identity != workspace_id(&git.common)?
        || !contains_commit(&git.path, &record.anchor)
    {
        return Err("Named worktree identity changed; review a new binding ID".into());
    }
    Ok(git)
}

fn repository_bindings(
    root: &Path,
    workspace: &Path,
    repo: &str,
) -> Result<Vec<BindingRecord>, String> {
    let dir = scoped_path(root, &named_dir(workspace, repo)?)?;
    let records = match fs::read_dir(dir) {
        Ok(records) => records,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => return Err("Cannot list local named binding records".into()),
    };
    let mut bindings = BTreeMap::new();
    for (index, entry) in records.enumerate() {
        if index >= MAX_RECORDS {
            return Err("Local named binding metadata exceeds scan limit".into());
        }
        let entry = entry.map_err(|_| "Cannot inspect local named binding")?;
        let name = entry.file_name();
        let Some(id) = name.to_str().and_then(|name| name.strip_suffix(".json")) else {
            continue;
        };
        bindings.insert(id.to_owned(), read_record(root, workspace, repo, id)?);
    }
    Ok(bindings.into_values().collect())
}

pub(super) fn check_repository_path(
    root: &Path,
    workspace: &Path,
    registry: &Value,
    repo: &str,
    git: &GitIdentity,
) -> Result<(), String> {
    let physical_identity = physical(git)?;
    let registered = registry["repositories"]
        .as_object()
        .ok_or("Missing repository registry")?;
    let mut scanned = 0;
    for other in registered.keys().filter(|name| name.as_str() != repo) {
        scanned += 1;
        if scanned > MAX_RECORDS {
            return Err("Local binding metadata exceeds scan limit".into());
        }
        if other == "cirnet-rules" {
            if git.path == root {
                return Err("This worktree is already bound to another repository ID".into());
            }
        } else {
            let file = scoped_path(root, &record_path(workspace, other)?)?;
            if file
                .try_exists()
                .map_err(|_| "Cannot inspect a default binding record")?
            {
                let row = read_default(&file, other)?;
                let path = scoped_path(workspace, &row.relative)?;
                if path.canonicalize().is_ok_and(|path| path == git.path) {
                    return Err("This worktree is already bound to another repository ID".into());
                }
            }
        }
        let records = repository_bindings(root, workspace, other)?;
        scanned += records.len();
        if scanned > MAX_RECORDS {
            return Err("Local binding metadata exceeds scan limit".into());
        }
        if records
            .iter()
            .any(|row| row.physical_identity == physical_identity)
        {
            return Err("This worktree is already bound to another repository ID".into());
        }
    }
    Ok(())
}

fn default_metadata(root: &Path, workspace: &Path, repo: &str) -> Result<Option<Value>, String> {
    if repo == "cirnet-rules" {
        let git = inspect_git(root)?;
        return Ok(Some(json!({
            "repo_id":repo,"binding_id":DEFAULT,"revision":1,"workspace_id":workspace_id(root)?,
            "identity":fingerprint(&git, "")?
        })));
    }
    let file = scoped_path(root, &record_path(workspace, repo)?)?;
    match file.try_exists() {
        Ok(false) => return Ok(None),
        Ok(true) => {}
        Err(_) => return Err("Cannot inspect the default binding record".into()),
    }
    let row = read_default(&file, repo)?;
    Ok(Some(json!({
        "repo_id":repo,"binding_id":DEFAULT,"revision":1,"workspace_id":workspace_id(workspace)?,
        "identity":row.identity,"anchor":row.anchor,"relative":row.relative,"actor":row.actor
    })))
}

fn add(
    root: &Path,
    workspace: &Path,
    registry: &Value,
    repo: &str,
    id: &str,
    relative: &str,
    actor: &str,
) -> Result<Value, String> {
    if id == DEFAULT {
        return Err("Use workspace bind to confirm or replace a default binding".into());
    }
    validate_actor(actor)?;
    let file = named_file(root, workspace, repo, id)?;
    let namespace = record_path(workspace, repo)?;
    let namespace = namespace.rsplit_once('/').unwrap().0;
    let lock_dir = local_directory(root, namespace)?;
    let _lock = LocalLock::acquire(lock_dir.join(".workspace.lock"))?;

    let git = inspect_git(&scoped_path(workspace, relative)?)?;
    let physical_identity = physical(&git)?;
    for other in repository_bindings(root, workspace, repo)? {
        if other.binding_id == id {
            if other.relative == relative && inspect_record(workspace, &other).is_ok() {
                return Ok(json!({"status":"PASS","state":"UNCHANGED","binding":other}));
            }
            return Err("Named binding IDs are immutable; review a different ID".into());
        }
        if other.physical_identity == physical_identity && inspect_record(workspace, &other).is_ok()
        {
            return Err("This physical worktree already has a named binding".into());
        }
    }
    let (default_git, _) = resolve_default(root, Some(workspace), repo, registry)?;
    if git.common != default_git.common || git.admin == default_git.admin {
        return Err("Select a distinct worktree of the confirmed default repository".into());
    }
    check_repository_path(root, workspace, registry, repo, &git)?;
    let anchor = registry["repositories"][repo]["snapshot"]["commit"]
        .as_str()
        .unwrap_or(&git.commit);
    if !contains_commit(&git.path, anchor) {
        return Err("Checkout does not contain the reviewed registry Git anchor".into());
    }
    let row = BindingRecord {
        schema: 1,
        binding_id: id.into(),
        repo_id: repo.into(),
        workspace_id: workspace_id(workspace)?,
        identity: fingerprint(&git, anchor)?,
        physical_identity,
        git_admin_identity: workspace_id(&git.admin)?,
        git_common_identity: workspace_id(&git.common)?,
        anchor: anchor.into(),
        relative: relative.into(),
        branch: git.branch,
        revision: 1,
        actor: actor.into(),
    };
    local_directory(root, &named_dir(workspace, repo)?)?;
    write_local(&file, &row)?;
    Ok(json!({"status":"PASS","state":"CONFIRMED_LOCAL","binding":row,"permissions_granted":false}))
}

pub(in crate::workspace) fn run(
    command: &str,
    root: &Path,
    options: &BTreeMap<String, String>,
    registry: &Value,
) -> Result<Value, String> {
    let workspace = workspace_path(root, options.get("workspace").map(Path::new))?;
    let option = |key: &str| {
        options
            .get(key)
            .map(String::as_str)
            .ok_or_else(|| format!("--{key} is required"))
    };
    if command == "workspace-binding-list" {
        let only = options.get("repo").map(String::as_str);
        if let Some(repo) = only {
            repository(registry, repo)?;
        }
        let mut bindings = Vec::new();
        for repo in registry["repositories"]
            .as_object()
            .ok_or("Missing repository registry")?
            .keys()
        {
            if only.is_some_and(|only| only != repo) || repository(registry, repo).is_err() {
                continue;
            }
            if let Some(default) = default_metadata(root, &workspace, repo)? {
                bindings.push(default);
            }
            for record in repository_bindings(root, &workspace, repo)? {
                bindings
                    .push(serde_json::to_value(record).map_err(|_| "Cannot encode named binding")?);
            }
            if bindings.len() > MAX_RECORDS {
                return Err("Local binding list exceeds the result bound".into());
            }
        }
        return Ok(json!({"status":"PASS","bindings":bindings}));
    }
    let repo = option("repo")?;
    repository(registry, repo)?;
    let id = option("binding")?;
    match command {
        "workspace-binding-add" => add(
            root,
            &workspace,
            registry,
            repo,
            id,
            option("path")?,
            option("actor")?,
        ),
        "workspace-binding-show" if id == DEFAULT => {
            let row = default_metadata(root, &workspace, repo)?
                .ok_or("Repository has no confirmed default binding")?;
            Ok(json!({"status":"PASS","binding":row}))
        }
        "workspace-binding-show" => {
            let row = read_record(root, &workspace, repo, id)?;
            Ok(json!({"status":"PASS","binding":row}))
        }
        "workspace-binding-validate" => {
            let checked = if id == DEFAULT {
                resolve_default(root, Some(&workspace), repo, registry)
            } else {
                read_record(root, &workspace, repo, id)
                    .and_then(|row| Ok((inspect_record(&workspace, &row)?, row.identity)))
            };
            Ok(match checked {
                Ok((git, identity)) => json!({
                    "status":"PASS","repository":repo,"binding_id":id,"binding":identity,
                    "physical_identity":physical(&git)?,"commit":git.commit,"branch":git.branch
                }),
                Err(reason) => {
                    json!({"status":"BLOCKED","repository":repo,"binding_id":id,"reason":reason})
                }
            })
        }
        _ => Err("Unknown binding command".into()),
    }
}

fn scoped_repositories(descriptor: &Value) -> BTreeSet<&str> {
    SCOPES
        .iter()
        .flat_map(|scope| {
            descriptor[scope]
                .as_object()
                .into_iter()
                .flat_map(|map| map.keys())
        })
        .map(String::as_str)
        .collect()
}

pub(crate) fn validate_task_bindings(descriptor: &Value) -> Result<(), String> {
    let Some(selections) = descriptor.get("bindings") else {
        return Ok(());
    };
    let selections = selections
        .as_object()
        .ok_or("Expected a repository binding map")?;
    if selections.len() > 16 {
        return Err("Task binding map exceeds its bound".into());
    }
    let scoped = scoped_repositories(descriptor);
    for (repo, id) in selections {
        if !valid_id(repo)
            || (descriptor["primary_repo"] != repo.as_str() && !scoped.contains(repo.as_str()))
        {
            return Err("Task binding selection is outside the task repositories".into());
        }
        binding_id(id.as_str().ok_or("Expected a task binding ID")?)?;
    }
    Ok(())
}

fn select(
    root: &Path,
    workspace: Option<&Path>,
    repo: &str,
    registry: &Value,
    descriptor: &Value,
) -> Result<(GitIdentity, TaskExecutionBinding), String> {
    repository(registry, repo)?;
    let id = descriptor["bindings"][repo].as_str().unwrap_or(DEFAULT);
    let (git, identity, namespace, revision) = if id == DEFAULT {
        let (git, identity) = resolve_default(root, workspace, repo, registry)?;
        let namespace = if repo == "cirnet-rules" {
            workspace_id(root)?
        } else {
            workspace_id(&workspace_path(root, workspace)?)?
        };
        (git, identity, namespace, 1)
    } else {
        let workspace = workspace_path(root, workspace)?;
        let row = read_record(root, &workspace, repo, id)?;
        let git = inspect_record(&workspace, &row)?;
        (git, row.identity, row.workspace_id, row.revision)
    };
    let approved = SCOPES.map(|scope| &descriptor[scope][repo]);
    let scope =
        serde_json::to_vec(&approved).map_err(|_| "Cannot encode approved binding scope")?;
    let frozen = TaskExecutionBinding {
        binding_id: id.to_owned(),
        revision,
        identity,
        physical_identity: physical(&git)?,
        base_commit: git.commit.clone(),
        workspace_id: namespace,
        branch: git.branch.clone(),
        scope_sha256: crate::docs::sha256(&scope),
    };
    Ok((git, frozen))
}

fn late_docs_binding(descriptor: &Value, repo: &str) -> bool {
    descriptor["primary_repo"] != repo
        && descriptor["documentation_scope"].get(repo).is_some()
        && SCOPES[..3]
            .iter()
            .all(|scope| descriptor[scope].get(repo).is_none())
        && descriptor["bindings"].get(repo).is_none()
}

fn has_default(root: &Path, workspace: Option<&Path>, repo: &str) -> Result<bool, String> {
    if repo == "cirnet-rules" {
        return Ok(true);
    }
    let workspace = workspace_path(root, workspace)?;
    let file = scoped_path(root, &record_path(&workspace, repo)?)?;
    file.try_exists()
        .map_err(|_| "Cannot inspect the default binding record".into())
}

pub(crate) fn freeze_task_bindings(
    root: &Path,
    workspace: Option<&Path>,
    registry: &Value,
    state: &Value,
) -> Result<BTreeMap<String, TaskExecutionBinding>, String> {
    let descriptor = &state["descriptor"];
    validate_task_bindings(descriptor)?;
    let mut repos = scoped_repositories(descriptor);
    if let Some(bindings) = descriptor["bindings"].as_object() {
        repos.extend(bindings.keys().map(String::as_str));
    }
    if let Some(primary) = descriptor["primary_repo"].as_str()
        && primary != "_workspace"
        && !repos.contains(primary)
        && has_default(root, workspace, primary)?
    {
        repos.insert(primary);
    }
    if repos.len() > 16 {
        return Err("Task binding freeze exceeds its repository bound".into());
    }
    let mut bindings = BTreeMap::new();
    for repo in repos {
        if late_docs_binding(descriptor, repo) && !has_default(root, workspace, repo)? {
            // Preserve the operator's ability to confirm a docs-only default
            // before explicitly accepting the task and its docs targets again.
            continue;
        }
        let (_, frozen) = select(root, workspace, repo, registry, descriptor)?;
        bindings.insert(repo.to_owned(), frozen);
    }
    Ok(bindings)
}

pub(crate) fn resolve_named(
    root: &Path,
    workspace: &Path,
    repo: &str,
    registry: &Value,
    id: &str,
) -> Result<Binding, String> {
    repository(registry, repo)?;
    if id == DEFAULT {
        return resolve(root, Some(workspace), repo, registry);
    }
    let record = read_record(root, workspace, repo, id)?;
    let git = inspect_record(workspace, &record)?;
    Ok(git.binding(record.identity))
}

pub(crate) fn resolve_task(
    root: &Path,
    workspace: Option<&Path>,
    repo: &str,
    registry: &Value,
    state: &Value,
) -> Result<Binding, String> {
    let descriptor = &state["descriptor"];
    validate_task_bindings(descriptor)?;
    let (git, current) = select(root, workspace, repo, registry, descriptor)?;
    if let Some(frozen) = state
        .get("execution_bindings")
        .filter(|value| !value.is_null())
    {
        if !frozen.is_object() {
            return Err("Invalid frozen task binding map".into());
        }
        let Some(selected) = frozen.get(repo) else {
            return if late_docs_binding(descriptor, repo) {
                Ok(git.binding(current.identity))
            } else {
                Err("Repository is outside the frozen task bindings".into())
            };
        };
        let selected: TaskExecutionBinding =
            serde_json::from_value(selected.clone()).map_err(|_| "Invalid frozen task binding")?;
        if current != selected {
            return Err("Task worktree binding, branch, base commit or approved scope changed; revise the task".into());
        }
    } else if let Some(previous) = state["baseline"]["repositories"]
        .as_array()
        .and_then(|rows| rows.iter().find(|row| row["repo"] == repo))
        && (previous["identity"] != current.identity || previous["commit"] != current.base_commit)
    {
        return Err("Legacy task worktree identity or base commit changed; revise the task".into());
    }
    Ok(git.binding(current.identity))
}
