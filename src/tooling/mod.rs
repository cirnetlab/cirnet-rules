//! CNR's native provider catalog and project-owned installation store.
mod catalog;
mod providers;
mod store;

use crate::environment::Project;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct Options {
    pub dry_run: bool,
    pub offline: bool,
    pub yes: bool,
    pub scope: String,
    pub kind: Option<String>,
    pub installed: bool,
    pub available: bool,
    pub locked: bool,
    pub expected_selection: Option<BTreeMap<String, String>>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            dry_run: false,
            offline: false,
            yes: false,
            scope: "project".into(),
            kind: None,
            installed: false,
            available: false,
            locked: false,
            expected_selection: None,
        }
    }
}

pub fn find_native(name: &str, project: Option<&Path>) -> Option<PathBuf> {
    providers::find(name, project)
}

pub(crate) fn known_tool(id: &str) -> bool {
    catalog::TOOLS.iter().any(|tool| tool.id == id)
}

/// Hold while consuming a managed executable to prevent cooperative removal.
pub fn lock_use(root: &Path) -> Result<Option<fs::File>, String> {
    store::shared(root)
}

fn exact_version(version: &str) -> Result<&str, String> {
    let version = version.strip_prefix('v').unwrap_or(version);
    let parts: Vec<_> = version.split('.').collect();
    if parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.len() <= 8
                && (part.len() == 1 || !part.starts_with('0'))
                && part.bytes().all(|byte| byte.is_ascii_digit())
        })
    {
        Ok(version)
    } else {
        Err("Use an exact release version (major.minor.patch). Aliases and ranges do not authorize installation.".into())
    }
}

fn split_spec(spec: &str) -> (&str, Option<&str>) {
    spec.split_once('@')
        .map_or((spec, None), |(tool, version)| (tool, Some(version)))
}

pub fn resolved(root: &Path, tool: &str, version: &str) -> Result<Option<PathBuf>, String> {
    let tool = catalog::get(tool, None)?;
    if !tool.supported() {
        return Ok(None);
    }
    let id = store::Identity::tool(tool.id, exact_version(version)?);
    match store::state(root, &id)? {
        store::State::Absent => Ok(None),
        store::State::Ready(path) => Ok(Some(path)),
        _ => Err("Managed install is incomplete. Inspect it and use cnr remove to recover.".into()),
    }
}

fn blocked(reason: &str, plan: Value) -> Value {
    json!({"status":"BLOCKED", "reason":reason, "plan":plan})
}

fn project(root: Option<&Path>) -> Result<Project, String> {
    Project::open(root.ok_or("Choose a project with --project.")?)
}

fn select_project_scope(options: &Options) -> Result<bool, String> {
    match options.scope.as_str() {
        "project" => Ok(true),
        "user" | "system" => Ok(false),
        _ => Err("Invalid CNR scope".into()),
    }
}

fn inventory(root: Option<&Path>, words: &[&str], options: &Options) -> Result<Value, String> {
    let kind = catalog::kind_filter(options.kind.as_deref())?;
    let query = match words {
        ["search", query] if !query.trim().is_empty() && query.len() <= 256 => {
            Some(query.to_ascii_lowercase())
        }
        ["search", ..] => return Err("Usage: cnr search <query>".into()),
        _ => None,
    };
    let (tool_filter, version_filter) = match words {
        ["info", spec] => {
            let (name, version) = split_spec(spec);
            (
                Some(catalog::get(name, kind)?.id),
                version.map(exact_version).transpose()?,
            )
        }
        ["info", ..] => return Err("Usage: cnr info <tool>[@version]".into()),
        ["list" | "doctor"] | ["search", _] => (None, None),
        _ => return Err("Invalid inventory arguments".into()),
    };
    let mut tools = Vec::new();
    for tool in catalog::TOOLS {
        if tool_filter.is_some_and(|id| id != tool.id)
            || !catalog::matches(tool, query.as_deref().unwrap_or_default(), kind)
        {
            continue;
        }
        let mut row = tool.evidence(
            root,
            query.is_none() && !options.available || options.installed,
        );
        let mut managed = if let Some(root) = root {
            versions(root, tool)?
        } else {
            Vec::new()
        };
        if let Some(version) = version_filter {
            managed.retain(|value| value["version"] == version);
            let matches = row["native_version"].as_str().is_some_and(|observed| {
                observed
                    .split_whitespace()
                    .any(|token| exact_version(token).ok() == Some(version))
            });
            if !matches && row["native_path"].is_string() {
                row["native_path"] = Value::Null;
                row["native_version"] = Value::Null;
                row["status"] = json!("VERSION_NOT_FOUND");
            }
        }
        let native = row["native_path"].is_string();
        let installable = tool.supported() && row["provider_path"].is_string();
        if options.available && !installable && !native
            || options.installed
                && !native
                && !managed.iter().any(|item| item["status"] == "INSTALLED")
        {
            continue;
        }
        row["managed"] = json!(managed);
        tools.push(row);
    }
    Ok(json!({
        "status":"PASS",
        "state": if tools.is_empty() { "NO_MATCHES" } else { "INVENTORY" },
        "platform": env::consts::OS,
        "architecture": env::consts::ARCH,
        "project": root,
        "tools": tools,
        "available_source": "local catalog and provider detection",
        "suggestions": if words == ["doctor"] {
            vec!["Use cnr env plan to compare project pins.", "Install only a reviewed exact version into the project store.",
            "For Qt/MSVC/Windows SDK, verify the vendor installation and developer environment."]
        } else { Vec::new() },
    }))
}

fn versions(root: &Path, tool: &catalog::Tool) -> Result<Vec<Value>, String> {
    let relative = Path::new(".cnr/tools").join(tool.id);
    let Some(base) = store::directory(root, &relative, false)? else {
        return Ok(Vec::new());
    };
    let mut versions = Vec::new();
    for target in store::entries(&base)? {
        let Some(name) = target.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if exact_version(name).ok() != Some(name) {
            continue;
        }
        let id = store::Identity::tool(tool.id, name);
        let (status, path) = match store::state(root, &id) {
            Ok(store::State::Ready(path)) => ("INSTALLED", Some(path)),
            Ok(store::State::Interrupted) => ("RECOVERY_REQUIRED", None),
            _ => ("UNVERIFIED", None),
        };
        versions.push(json!({"version":name, "status":status, "path":path}));
    }
    Ok(versions)
}

fn provider_plan(
    tool: &catalog::Tool,
    version: &str,
    action: &str,
    options: &Options,
    root: &Path,
) -> Value {
    let id = store::Identity::tool(tool.id, version);
    json!({
        "tool": tool.id,
        "type": tool.kind,
        "version": version,
        "action": action,
        "provider": tool.manager.unwrap_or("native"),
        "scope": options.scope,
        "path": id.path(root),
        "download_required": tool.supported() && action != "remove",
        "source": tool.upstream,
        "effects": "Project store only. No profile, parent shell, user or system configuration changes.",
        "rollback": "The native provider is not transactional. A failed reservation remains identifiable for explicit cleanup."
    })
}

fn ensure_cache(root: &Path) -> Result<PathBuf, String> {
    let id = store::Identity::cache();
    let path = match store::state(root, &id)? {
        store::State::Absent => store::reserve(root, &id)?,
        store::State::OwnedCache => id.path(root),
        _ => return Err("Invalid cache ownership state".into()),
    };
    store::directory(&path, Path::new("uv"), true)?.ok_or("Provider cache unavailable".into())
}

fn validate_selection(
    project: &Project,
    tool: &catalog::Tool,
    version: &str,
    options: &Options,
) -> Result<(), String> {
    if !options.locked && options.expected_selection.is_none() {
        return Ok(());
    }
    let selected = if options.locked {
        project.selected_locked()?
    } else {
        project.selected_for_sync()?
    };
    if options
        .expected_selection
        .as_ref()
        .is_some_and(|expected| *expected != selected)
        || selected
            .get(tool.id)
            .and_then(|pin| exact_version(pin).ok())
            != Some(version)
    {
        return Err("Project selection changed or the requested version differs from its pin. Review the environment plan again.".into());
    }
    Ok(())
}

fn install_one(
    project: &Project,
    tool: &catalog::Tool,
    version: &str,
    action: &str,
    options: &Options,
) -> Result<Value, String> {
    let root = project.root();
    let mut plan = provider_plan(tool, version, action, options, root);
    if !select_project_scope(options)? {
        return Ok(blocked(
            "User and system mutations require separate authority. Only project scope is implemented.",
            plan,
        ));
    }
    if let Err(reason) = validate_selection(project, tool, version, options) {
        return Ok(blocked(&reason, plan));
    }
    if !tool.supported() {
        if let Some((path, Some(observed))) = providers::discover(tool, Some(root))
            && observed
                .split_whitespace()
                .any(|token| exact_version(token).ok() == Some(version))
        {
            plan["path"] = json!(path);
            return Ok(
                json!({"status":"PASS","state":"ALREADY_AVAILABLE","plan":plan,"path":path,"observed":observed}),
            );
        }
        return Ok(blocked(
            "This tool is detect-only. Use the platform vendor's supported installer.",
            plan,
        ));
    }
    let id = store::Identity::tool(tool.id, version);
    match store::state(root, &id)? {
        store::State::Ready(path) => {
            plan["download_required"] = json!(false);
            return Ok(
                json!({"status":"PASS","state":"ALREADY_INSTALLED","plan":plan,"path":path}),
            );
        }
        store::State::Interrupted => {
            return Ok(blocked(
                "An interrupted installation is present. Remove it explicitly before retrying.",
                plan,
            ));
        }
        _ => {}
    }
    if options.offline {
        return Ok(blocked(
            "This provider cannot guarantee an offline download. Existing installations remain usable.",
            plan,
        ));
    }
    let Some(binary) = tool.manager.and_then(|name| find_native(name, Some(root))) else {
        return Ok(blocked(
            "Required native provider was not found on the external PATH.",
            plan,
        ));
    };
    if options.dry_run {
        return Ok(json!({"status":"PASS","state":"PLANNED","plan":plan}));
    }
    if !options.yes {
        return Ok(blocked(
            "Review the plan with --dry-run, then pass --yes to authorize project installation.",
            plan,
        ));
    }

    let _lock = store::exclusive(root)?;
    let _environment = project.lock()?;
    if let Err(reason) = validate_selection(project, tool, version, options) {
        return Ok(blocked(&reason, plan));
    }
    // Recheck ownership and provider availability after locking.
    let Some(confirmed) = tool.manager.and_then(|name| find_native(name, Some(root))) else {
        return Ok(blocked("Native provider is no longer available.", plan));
    };
    if confirmed != binary {
        return Ok(blocked(
            "Native provider changed after planning. Review the plan again.",
            plan,
        ));
    }
    match store::state(root, &id)? {
        store::State::Ready(path) => {
            plan["download_required"] = json!(false);
            return Ok(
                json!({"status":"PASS","state":"ALREADY_INSTALLED","plan":plan,"path":path}),
            );
        }
        store::State::Absent => {}
        _ => {
            return Ok(blocked(
                "Installation state changed. Inspect it before retrying.",
                plan,
            ));
        }
    }
    let cache = (tool.id == "python")
        .then(|| ensure_cache(root))
        .transpose()?;
    let target = store::reserve(root, &id)?;
    store::audit(root, Some(&id), action, "STARTED")?;
    let installed = providers::install(tool, version, &binary, &target, root, cache.as_deref())
        .and_then(|path| store::publish(root, &id, &path));
    match installed {
        Ok(path) => {
            store::audit(root, Some(&id), action, "PASS")?;
            Ok(json!({"status":"PASS","state":"INSTALLED","plan":plan,"path":path}))
        }
        Err(reason) => {
            store::audit(root, Some(&id), action, "RECOVERY_REQUIRED")?;
            Ok(json!({"status":"FAIL","state":"RECOVERY_REQUIRED","plan":plan,"reason":reason}))
        }
    }
}

fn remove_tool(
    project: &Project,
    tool: &catalog::Tool,
    version: &str,
    options: &Options,
) -> Result<Value, String> {
    let root = project.root();
    let plan = provider_plan(tool, version, "remove", options, root);
    if !select_project_scope(options)? {
        return Ok(blocked(
            "Only project-owned installations may be removed.",
            plan,
        ));
    }
    if !tool.supported() {
        return Ok(blocked(
            "This tool is detect-only. There is no managed installation to remove.",
            plan,
        ));
    }
    let id = store::Identity::tool(tool.id, version);
    if !store::owned(root, &id)? {
        return Ok(blocked("No owned version matches this request.", plan));
    }
    let referenced = || -> Result<bool, String> {
        Ok(project.references(tool.id, version)?
            || (tool.id == "python" && store::references_python(root, version)?))
    };
    if referenced()? {
        return Ok(blocked(
            "A project pin, native manifest or virtual environment still references this version.",
            plan,
        ));
    }
    if options.dry_run {
        return Ok(json!({"status":"PASS","state":"PLANNED","plan":plan}));
    }
    if !options.yes {
        return Ok(blocked(
            "Review the plan with --dry-run, then pass --yes to authorize removal.",
            plan,
        ));
    }
    let _lock = store::exclusive(root)?;
    let _environment = project.lock()?;
    if referenced()? {
        return Ok(blocked(
            "A reference was added after planning. Update the reference before removing this version.",
            plan,
        ));
    }
    store::audit(root, Some(&id), "remove", "STARTED")?;
    let removed = store::remove(root, &id)?;
    store::audit(root, Some(&id), "remove", "PASS")?;
    Ok(json!({"status":"PASS","state":if removed {"REMOVED"} else {"ABSENT"},"plan":plan}))
}

fn update_plan(
    project: Option<&Project>,
    tool: Option<&catalog::Tool>,
    options: &Options,
    apply_all: bool,
) -> Result<Value, String> {
    let selected = project
        .map(Project::selected)
        .transpose()?
        .unwrap_or_default();
    let mut results = Vec::new();
    if let Some(project) = project {
        // Always preflight all selected runtime installs before applying any of them.
        let preview = Options {
            dry_run: true,
            yes: false,
            ..options.clone()
        };
        let mut targets = Vec::new();
        for (name, version) in selected {
            let Ok(candidate) = catalog::get(&name, options.kind.as_deref()) else {
                continue;
            };
            if !candidate.supported() || tool.is_some_and(|filter| filter.id != candidate.id) {
                continue;
            }
            let check = install_one(
                project,
                candidate,
                exact_version(&version)?,
                "update",
                &preview,
            )?;
            results.push(check);
            targets.push((candidate, version));
        }
        if apply_all
            && !options.dry_run
            && options.yes
            && results.iter().any(|item| item["status"] != "PASS")
        {
            return Ok(
                json!({"status":"BLOCKED", "state":"PLANNED", "items":results, "reason":"At least one pinned tool cannot be updated."}),
            );
        }
        if apply_all
            && !targets.is_empty()
            && !options.dry_run
            && options.yes
            && results.iter().all(|item| item["status"] == "PASS")
        {
            results.clear();
            for (candidate, version) in targets {
                let item = install_one(
                    project,
                    candidate,
                    exact_version(&version)?,
                    "update",
                    options,
                )?;
                let failed = item["status"] != "PASS";
                results.push(item);
                if failed {
                    break;
                }
            }
            let failed = results.iter().any(|item| item["status"] != "PASS");
            return Ok(
                json!({"status": if failed {"FAIL"} else {"PASS"}, "state":if failed {"PARTIAL"} else {"APPLIED"},"items":results}),
            );
        }
    }
    Ok(json!({
        "status":"PASS",
        "state":"PLANNED",
        "items":results,
        "reason":"Review explicit target versions. Use cnr update TOOL@VERSION or cnr update --all --yes for pinned versions."
    }))
}

fn tools_command(root: Option<&Path>, words: &[&str], options: &Options) -> Result<Value, String> {
    let kind = catalog::kind_filter(options.kind.as_deref())?;
    if words == ["update"] {
        return update_plan(
            root.map(Project::open).transpose()?.as_ref(),
            None,
            options,
            false,
        );
    }
    if matches!(words, ["update", "--all" | "all"]) {
        return update_plan(Some(&project(root)?), None, options, true);
    }
    let [action, spec] = words else {
        return Err("Choose exactly one tool and version.".into());
    };
    let project = project(root)?;
    let (name, version) = split_spec(spec);
    let tool = catalog::get(name, kind)?;
    if *action == "update" && version.is_none() {
        return update_plan(Some(&project), Some(tool), options, false);
    }
    let chosen;
    let version = match version {
        Some(version) => exact_version(version)?,
        None if *action == "remove" => {
            let found = versions(project.root(), tool)?;
            let owned = found
                .iter()
                .filter(|item| {
                    matches!(
                        item["status"].as_str(),
                        Some("INSTALLED" | "RECOVERY_REQUIRED")
                    )
                })
                .collect::<Vec<_>>();
            if owned.len() != 1 {
                return Ok(blocked(
                    "Specify one managed version to remove.",
                    json!({"candidates":found}),
                ));
            }
            chosen = owned[0]["version"]
                .as_str()
                .expect("managed version")
                .to_owned();
            &chosen
        }
        None => {
            chosen = project
                .selected()?
                .remove(tool.id)
                .ok_or("Supply TOOL@VERSION or pin an exact version in this project.")?;
            exact_version(&chosen)?
        }
    };
    if *action == "remove" {
        remove_tool(&project, tool, version, options)
    } else {
        install_one(&project, tool, version, action, options)
    }
}

fn venv_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > 40
        || !name.as_bytes()[0].is_ascii_alphanumeric()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || matches!(name, "con" | "prn" | "aux" | "nul")
        || (name.len() == 4
            && matches!(&name[..3], "com" | "lpt")
            && name.as_bytes()[3].is_ascii_digit())
    {
        return Err(
            "Virtual environment names use 1–40 lowercase ASCII letters, digits or hyphens.".into(),
        );
    }
    Ok(())
}

pub fn venv_python(root: &Path, name: &str) -> Result<PathBuf, String> {
    venv_name(name)?;
    match store::state(root, &store::Identity::venv(name))? {
        store::State::Ready(path) => Ok(path),
        _ => Err("This virtual environment has no verified interpreter.".into()),
    }
}

fn venv_list(root: &Path) -> Result<Value, String> {
    let mut rows = Vec::new();
    if let Some(dir) = store::directory(root, Path::new(".cnr/venvs"), false)? {
        for target in store::entries(&dir)? {
            let Some(name) = target.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if venv_name(name).is_err() {
                continue;
            }
            let id = store::Identity::venv(name);
            let (status, executable) = match store::state(root, &id) {
                Ok(store::State::Ready(path)) => ("READY", Some(path)),
                Ok(store::State::Interrupted) => ("RECOVERY_REQUIRED", None),
                _ => ("UNVERIFIED", None),
            };
            rows.push(json!({"name":name,"status":status,"path":target,"interpreter":executable}));
        }
    }
    Ok(json!({"status":"PASS","environments":rows}))
}

fn venv_create(
    project: &Project,
    id: &store::Identity,
    version: Option<&str>,
    options: &Options,
    mut plan: Value,
) -> Result<Value, String> {
    let root = project.root();
    let selected = project.selected()?;
    let version = version
        .or_else(|| selected.get("python").map(String::as_str))
        .map(exact_version)
        .transpose()?;
    let python = version
        .map(|version| resolved(root, "python", version))
        .transpose()?
        .flatten()
        .or_else(|| {
            find_native("python3", Some(root)).or_else(|| find_native("python", Some(root)))
        });
    plan["python_version"] = json!(version);
    plan["base_interpreter"] = json!(python);
    let Some(python) = python else {
        return Ok(blocked(
            "No verified local Python interpreter is available. Install a project Python explicitly first.",
            plan,
        ));
    };
    let Some(uv) = find_native("uv", Some(root)) else {
        return Ok(blocked("uv is required on the external PATH.", plan));
    };
    if !matches!(store::state(root, id)?, store::State::Absent) {
        return Ok(blocked(
            "A resource already has this name. Inspect it before creating a new environment.",
            plan,
        ));
    }
    if options.dry_run {
        return Ok(json!({"status":"PASS","state":"PLANNED","plan":plan}));
    }
    if !options.yes {
        return Ok(blocked(
            "Review the plan with --dry-run, then pass --yes to create this project virtual environment.",
            plan,
        ));
    }
    let _lock = store::exclusive(root)?;
    if let Some(version) = version {
        if python.starts_with(root) && resolved(root, "python", version)?.as_ref() != Some(&python)
        {
            return Ok(blocked("The managed Python changed after planning.", plan));
        }
        providers::version_matches(
            &python,
            "python",
            version,
            python.parent().ok_or("Invalid Python location")?,
            root,
        )?;
    }
    if !matches!(store::state(root, id)?, store::State::Absent) {
        return Ok(blocked(
            "A resource already has this name. Inspect it before creating a new environment.",
            plan,
        ));
    }
    let cache = ensure_cache(root)?;
    let target = store::reserve_with_python(root, id, version, Some(&python))?;
    store::audit(root, Some(id), "venv.create", "STARTED")?;
    match providers::venv(&uv, &python, &target, root, &cache)
        .and_then(|path| store::publish(root, id, &path))
    {
        Ok(path) => {
            store::audit(root, Some(id), "venv.create", "PASS")?;
            Ok(json!({"status":"PASS","state":"CREATED","plan":plan,"interpreter":path}))
        }
        Err(reason) => {
            store::audit(root, Some(id), "venv.create", "RECOVERY_REQUIRED")?;
            Ok(json!({"status":"FAIL","state":"RECOVERY_REQUIRED","plan":plan,"reason":reason}))
        }
    }
}

fn venv_command(root: Option<&Path>, words: &[&str], options: &Options) -> Result<Value, String> {
    let project = project(root)?;
    let root = project.root();
    if matches!(words, ["venv"] | ["venv", "list"]) {
        return venv_list(root);
    }
    let ["venv", action, name, rest @ ..] = words else {
        return Err("Usage: cnr venv <list|info|create|run|remove> [NAME]".into());
    };
    venv_name(name)?;
    let id = store::Identity::venv(name);
    let plan = json!({"name":name,"action":action,"scope":options.scope,"path":id.path(root),"provider":"uv","download_required":false,
        "effects":"Uses a separate project interpreter process. Does not alter the parent shell."});
    if !select_project_scope(options)? {
        return Ok(blocked(
            "Virtual environments are managed only in project scope.",
            plan,
        ));
    }
    match (*action, rest) {
        ("info", []) => {
            let state = store::state(root, &id)?;
            let (state, executable) = match state {
                store::State::Ready(path) => ("READY", Some(path)),
                store::State::Absent => ("ABSENT", None),
                _ => ("RECOVERY_REQUIRED", None),
            };
            Ok(json!({"status":"PASS","state":state,"plan":plan,"interpreter":executable}))
        }
        ("create", [] | [_]) => venv_create(&project, &id, rest.first().copied(), options, plan),
        ("remove", []) => {
            if !store::owned(root, &id)? {
                return Ok(blocked("No owned virtual environment matches.", plan));
            }
            if options.dry_run {
                return Ok(json!({"status":"PASS","state":"PLANNED","plan":plan}));
            }
            if !options.yes {
                return Ok(blocked(
                    "Review the plan and pass --yes to authorize virtual environment removal.",
                    plan,
                ));
            }
            let _lock = store::exclusive(root)?;
            store::audit(root, Some(&id), "venv.remove", "STARTED")?;
            let removed = store::remove(root, &id)?;
            store::audit(root, Some(&id), "venv.remove", "PASS")?;
            Ok(json!({"status":"PASS","state":if removed {"REMOVED"} else {"ABSENT"},"plan":plan}))
        }
        ("run", ["--", args @ ..]) if !args.is_empty() => {
            if options.offline {
                return Ok(blocked(
                    "CNR does not enforce offline isolation for arbitrary Python code.",
                    plan,
                ));
            }
            if options.dry_run {
                let python = venv_python(root, name)?;
                return Ok(
                    json!({"status":"PASS","state":"PLANNED","plan":plan,"interpreter":python}),
                );
            }
            if !options.yes {
                return Ok(blocked(
                    "Pass --yes to authorize Python code execution in this trusted project.",
                    plan,
                ));
            }
            let _guard = lock_use(root)?;
            let python = venv_python(root, name)?;
            let mut command = providers::command(&python, root, Some(root), None);
            command
                .env("VIRTUAL_ENV", id.path(root).join("env"))
                .args(args);
            let result = providers::run(&mut command, 120_000)?;
            crate::execution::write_output(&result.output);
            crate::execution::write_output(&result.error);
            let status = if result.cancelled {
                "CANCELLED"
            } else if result.timed_out {
                "BLOCKED"
            } else if result.success() {
                "PASS"
            } else {
                "FAIL"
            };
            store::audit(root, Some(&id), "venv.run", status)?;
            let child_exit_code = result
                .exit_code
                .filter(|_| !result.cancelled && !result.timed_out);
            Ok(
                json!({"status":status,"child_exit_code":child_exit_code,"elapsed_ms":result.elapsed_ms,
                "cancelled":result.cancelled,"timed_out":result.timed_out,"output_truncated":result.output_truncated}),
            )
        }
        _ => Err(
            "Usage: cnr venv <list|info NAME|create NAME [VERSION]|run NAME -- ARGS|remove NAME>"
                .into(),
        ),
    }
}

fn cache_command(root: Option<&Path>, words: &[&str], options: &Options) -> Result<Value, String> {
    let project = project(root)?;
    let root = project.root();
    let id = store::Identity::cache();
    let status = store::state(root, &id)?;
    let plan = json!({"action":"cache.clean","scope":options.scope,"path":id.path(root),
        "effects":"Retire CNR-owned uv cache and clean retired provider resources only."});
    match words {
        ["cache"] | ["cache", "list" | "info"] => Ok(
            json!({"status":"PASS","exists":matches!(status, store::State::OwnedCache),
            "plan":plan,"offline_install_supported":false}),
        ),
        ["cache", "clean"] => {
            if !select_project_scope(options)? {
                return Ok(blocked(
                    "Only the project-owned cache may be cleared.",
                    plan,
                ));
            }
            if options.dry_run {
                return Ok(json!({"status":"PASS","state":"PLANNED","plan":plan}));
            }
            if !options.yes {
                return Ok(blocked(
                    "Review the plan and pass --yes to authorize project cache cleanup.",
                    plan,
                ));
            }
            let _lock = store::exclusive(root)?;
            let removed = store::remove(root, &id)?;
            let retired = store::clean_trash(root)?;
            store::audit(root, Some(&id), "cache.clean", "PASS")?;
            Ok(
                json!({"status":"PASS","state":"CLEAN","cache_removed":removed,"retired_cleaned":retired,"plan":plan}),
            )
        }
        _ => Err("Usage: cnr cache <info|clean>".into()),
    }
}

pub fn dispatch(root: Option<&Path>, words: &[String], options: &Options) -> Result<Value, String> {
    let words: Vec<_> = words.iter().map(String::as_str).collect();
    let project_root = root.map(Project::open).transpose()?;
    let root = project_root.as_ref().map(Project::root);
    match words.as_slice() {
        ["list" | "search" | "info" | "doctor", ..] => inventory(root, &words, options),
        ["install" | "remove" | "update", ..] => tools_command(root, &words, options),
        ["venv", ..] => venv_command(root, &words, options),
        ["cache", ..] => cache_command(root, &words, options),
        ["audit"] | ["audit", "list"] => {
            let project = project(root)?;
            Ok(json!({"status":"PASS","events":store::audit_list(project.root())?}))
        }
        _ => Err("Unknown CNR tooling command".into()),
    }
}

#[cfg(test)]
mod tests;
