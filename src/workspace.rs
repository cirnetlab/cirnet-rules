mod binding;
mod context_packet;

#[cfg(test)]
mod tests;

use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

pub use binding::{Binding, resolve};
pub(crate) use binding::{
    freeze_task_bindings, resolve_named, resolve_task, validate_task_bindings, workspace_path,
};
pub(crate) use context_packet::contains_known_secrets;

pub fn run(
    command: &str,
    root: &Path,
    options: &BTreeMap<String, String>,
    registry: &Value,
) -> Result<Value, String> {
    let allowed: &[&str] = match command {
        "workspace-discover" => &["workspace"],
        "workspace-bind" => &["workspace", "repo", "path", "actor", "replace"],
        "workspace-bind-preview" | "workspace-bind-reviewed" => {
            &["workspace", "repo", "path", "actor", "replace", "expect"]
        }
        "workspace-health" | "workspace-binding-list" => &["workspace", "repo"],
        "workspace-binding-add" => &["workspace", "repo", "binding", "path", "actor"],
        "workspace-binding-show" | "workspace-binding-validate" => {
            &["workspace", "repo", "binding"]
        }
        "route" => &["workspace", "id", "task"],
        "task-context" => &["workspace", "id", "task", "session"],
        "context-prune" => &[],
        _ => return Err("Unknown workspace command".into()),
    };
    if let Some(key) = options.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(format!("Unexpected option --{key} for {command}"));
    }
    let workspace = options.get("workspace").map(Path::new);
    let option = |key: &str| {
        options
            .get(key)
            .map(String::as_str)
            .ok_or_else(|| format!("--{key} is required"))
    };
    match command {
        "workspace-discover" => binding::discover(root, workspace, registry),
        "workspace-bind" | "workspace-bind-reviewed" => binding::bind(
            root,
            workspace,
            registry,
            option("repo")?,
            option("path")?,
            option("actor")?,
            (
                options.get("replace").map(String::as_str),
                if command == "workspace-bind-reviewed" {
                    Some(option("expect")?)
                } else {
                    None
                },
            ),
        ),
        "workspace-bind-preview" => binding::preview_bind(
            root,
            workspace,
            registry,
            option("repo")?,
            option("path")?,
            option("actor")?,
            options.get("replace").map(String::as_str),
        ),
        "workspace-binding-list"
        | "workspace-binding-add"
        | "workspace-binding-show"
        | "workspace-binding-validate" => binding::run_named(command, root, options, registry),
        "workspace-health" => {
            let repo = option("repo")?;
            match resolve(root, workspace, repo, registry) {
                Ok(binding) => Ok(json!({
                    "status":"PASS","repository":repo,"binding":binding.identity,
                    "commit":binding.commit,"release_checked":false
                })),
                Err(reason) => Ok(json!({"status":"BLOCKED","repository":repo,"reason":reason})),
            }
        }
        "route" | "task-context" => {
            if options.contains_key("id") && options.contains_key("task") {
                return Err("Choose one task identifier option".into());
            }
            let id = options
                .get("id")
                .or_else(|| options.get("task"))
                .ok_or("--task is required")?;
            if command == "route" {
                let state = match crate::tasks::load_current(root, id) {
                    Ok(state) => state,
                    Err(reason) => return Ok(json!({"status":"BLOCKED","reason":reason})),
                };
                if let Err(reason) = crate::tasks::authorize(&state, "read") {
                    return Ok(json!({"status":"BLOCKED","reason":reason}));
                }
                let route = make_route(&state, registry)?;
                let descriptor = &state["descriptor"];
                let main = descriptor["primary_repo"].as_str().unwrap();
                let hints = paths(&descriptor["hints"])?;
                let modules = selected_modules(registry, main, &hints)?;
                let impacts: Vec<_> = modules
                    .iter()
                    .flat_map(|module| module["impact"].as_array().into_iter().flatten())
                    .filter(|impact| {
                        impact["repository"]
                            .as_str()
                            .is_some_and(|repo| route.contains_key(repo))
                    })
                    .collect();
                let excluded: Vec<_> = registry["repositories"]
                    .as_object()
                    .into_iter()
                    .flat_map(|map| map.keys())
                    .filter(|repo| !route.contains_key(*repo))
                    .collect();
                let readiness: BTreeMap<_, _> = route
                    .keys()
                    .map(|repo| {
                        let selected = resolve_task(root, workspace, repo, registry, &state);
                        let row = match selected {
                            Ok(binding) => json!({"status":"PASS","binding":binding.identity}),
                            Err(reason) => json!({"status":"BLOCKED","reason":reason}),
                        };
                        (repo, row)
                    })
                    .collect();
                let complete = readiness.values().all(|value| value["status"] == "PASS");
                return Ok(json!({
                    "status":if complete {"PASS"} else {"BLOCKED"},
                    "task":id,"mode":descriptor["mode"],"primary":main,
                    "ownership":registry["repositories"][main]["ownership"],
                    "profile":registry["repositories"][main]["profile"],
                    "reason":"Exact read scope intersected with module ownership and direct configured impacts",
                    "modules":modules.iter().map(|row| &row["id"]).collect::<Vec<_>>(),
                    "selected":route,"direct_impacts":impacts,"omitted_repositories":excluded,
                    "bindings":readiness,"release_checked":false
                }));
            }
            let state = match crate::tasks::load_current(root, id) {
                Ok(state) => state,
                Err(reason) => return Ok(json!({"status":"BLOCKED","reason":reason})),
            };
            if let Err(reason) = crate::tasks::authorize(&state, "read") {
                return Ok(json!({"status":"BLOCKED","reason":reason}));
            }
            context_packet::context(
                root,
                workspace,
                registry,
                &state,
                options.get("session").map(String::as_str),
            )
        }
        _ => context_packet::prune(root, registry),
    }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id.as_bytes()[0].is_ascii_alphanumeric()
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
}

fn ordinary(metadata: &fs::Metadata) -> bool {
    !crate::policy::is_link(metadata)
}

fn ordinary_ancestors(path: &Path) -> Result<(), String> {
    for ancestor in path.ancestors() {
        if ancestor.as_os_str().is_empty() || ancestor.parent().is_none() {
            continue;
        }
        if !ordinary(&fs::symlink_metadata(ancestor).map_err(|_| "Unusable filesystem ancestry")?) {
            return Err("Symbolic link or reparse point is outside scope".into());
        }
    }
    Ok(())
}

fn validate_relative(relative: &str) -> Result<(), String> {
    if relative.len() > 512
        || relative.contains(['\\', ':', '*', '?', '"', '<', '>', '|'])
        || relative.chars().any(char::is_control)
    {
        return Err("Unsafe relative path".into());
    }
    for part in relative.split('/') {
        let stem = part
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        if part.is_empty()
            || part.len() > 200
            || matches!(part, "." | "..")
            || part.ends_with(['.', ' '])
            || part.starts_with(' ')
            || part.eq_ignore_ascii_case(".git")
            || matches!(
                stem.as_str(),
                "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
            )
            || stem
                .strip_prefix("COM")
                .or_else(|| stem.strip_prefix("LPT"))
                .is_some_and(|suffix| {
                    matches!(
                        suffix,
                        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                    )
                })
        {
            return Err("Unsafe relative path component".into());
        }
    }
    Ok(())
}

/// Validate each existing path component without following a link. Missing
/// suffixes stay lexical so the caller can enforce its own creation policy.
pub fn scoped_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    validate_relative(relative)?;
    let mut current = root.to_path_buf();
    for part in relative.split('/') {
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if ordinary(&metadata) => {}
            Ok(_) => return Err("Symbolic link or reparse point is outside scope".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("Cannot inspect scoped path".into()),
        }
    }
    Ok(current)
}

pub(crate) fn repository<'a>(registry: &'a Value, repo: &str) -> Result<&'a Value, String> {
    if !valid_id(repo) {
        return Err("Invalid repository ID".into());
    }
    let definition = registry["repositories"]
        .get(repo)
        .ok_or("Repository is outside the registry")?;
    if !matches!(
        definition["classification"].as_str().unwrap_or("active"),
        "active" | "supporting"
    ) {
        return Err("Repository classification does not permit active tasks".into());
    }
    Ok(definition)
}

fn paths(value: &Value) -> Result<BTreeSet<String>, String> {
    let rows = value.as_array().ok_or("Expected relative path list")?;
    if rows.len() > 128 {
        return Err("Scoped path list exceeds the safety limit".into());
    }
    rows.iter()
        .map(|path| {
            let path = path.as_str().ok_or("Expected relative path string")?;
            validate_relative(path)?;
            Ok(path.to_owned())
        })
        .collect()
}

fn selected_modules<'a>(
    registry: &'a Value,
    primary: &str,
    hints: &BTreeSet<String>,
) -> Result<Vec<&'a Value>, String> {
    let Some(rules) = registry["control_plane"]["routes"].get(primary) else {
        return Ok(Vec::new());
    };
    let rules = rules.as_array().ok_or("Invalid routing rules")?;
    let mut result = Vec::new();
    for rule in rules {
        let prefixes = paths(&rule["paths"])?;
        if hints
            .iter()
            .any(|file| prefixes.iter().any(|prefix| contains(file, prefix)))
        {
            result.push(rule);
        }
    }
    Ok(result)
}

fn contains(path: &str, prefix: &str) -> bool {
    path == prefix
        || path
            .strip_prefix(prefix)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn make_route(
    state: &Value,
    registry: &Value,
) -> Result<BTreeMap<String, BTreeSet<String>>, String> {
    let descriptor = &state["descriptor"];
    let primary = descriptor["primary_repo"]
        .as_str()
        .ok_or("Missing primary repository")?;
    let hints = paths(&descriptor["hints"])?;
    let read = descriptor["read_scope"]
        .as_object()
        .ok_or("Missing read scope")?;
    if primary == "_workspace" {
        if !hints.is_empty() {
            return Err("Initiative hints must be assigned to child task repositories".into());
        }
        return read
            .iter()
            .map(|(repo, allowed)| {
                repository(registry, repo)?;
                Ok((repo.clone(), paths(allowed)?))
            })
            .collect();
    }
    repository(registry, primary)?;
    let mut evidence_hints = hints.clone();
    let review = matches!(descriptor["mode"].as_str(), Some("VERIFY" | "ACCEPT"))
        || matches!(
            state["state"].as_str(),
            Some("IMPLEMENTED" | "VERIFIED" | "ACCEPTED")
        );
    if review && let Some(changed) = state["checkpoint"]["changed_paths"].get(primary) {
        let allowed = read
            .get(primary)
            .map(paths)
            .transpose()?
            .unwrap_or_default();
        evidence_hints.extend(
            paths(changed)?
                .into_iter()
                .filter(|path| allowed.contains(path)),
        );
    }
    let selected = selected_modules(registry, primary, &evidence_hints)?;
    let mut route = BTreeMap::<String, BTreeSet<String>>::new();
    let primary_allowed = read
        .get(primary)
        .map(paths)
        .transpose()?
        .unwrap_or_default();
    if !hints.is_subset(&primary_allowed) {
        return Err("A routing hint is outside the primary repository read scope".into());
    }
    let explicit = matches!(descriptor["mode"].as_str(), Some("RELEASE" | "AUDIT"));
    let mut direct = if hints.is_empty() || explicit {
        primary_allowed.clone()
    } else {
        evidence_hints
    };
    for module in selected {
        let prefixes = paths(&module["paths"])?;
        direct.extend(
            primary_allowed
                .iter()
                .filter(|file| prefixes.iter().any(|prefix| contains(file, prefix)))
                .cloned(),
        );
        if let Some(impacts) = module["impact"].as_array() {
            for impact in impacts {
                let repo = impact["repository"]
                    .as_str()
                    .ok_or("Invalid impact repository")?;
                if let Some(allowed) = read.get(repo) {
                    repository(registry, repo)?;
                    let prefixes = paths(&impact["paths"])?;
                    let matches: BTreeSet<_> = paths(allowed)?
                        .into_iter()
                        .filter(|file| prefixes.iter().any(|prefix| contains(file, prefix)))
                        .collect();
                    if !matches.is_empty() {
                        route.entry(repo.to_owned()).or_default().extend(matches);
                    }
                }
            }
        }
    }
    if explicit {
        for (repo, allowed) in read {
            repository(registry, repo)?;
            route
                .entry(repo.to_owned())
                .or_default()
                .extend(paths(allowed)?);
        }
    }
    if review && let Some(changes) = state["checkpoint"]["changed_paths"].as_object() {
        for (repo, changed) in changes {
            let Some(allowed) = read.get(repo) else {
                continue;
            };
            let allowed = paths(allowed)?;
            let selected: BTreeSet<_> = paths(changed)?
                .into_iter()
                .filter(|path| allowed.contains(path))
                .collect();
            if !selected.is_empty() {
                repository(registry, repo)?;
                route.entry(repo.to_owned()).or_default().extend(selected);
            }
        }
    }
    if review && let Some(verification) = descriptor["verification_scope"].as_object() {
        for (repo, tests) in verification {
            let Some(allowed) = read.get(repo) else {
                continue;
            };
            let allowed = paths(allowed)?;
            let tests: BTreeSet<_> = paths(tests)?.intersection(&allowed).cloned().collect();
            if !tests.is_empty() {
                repository(registry, repo)?;
                route.entry(repo.to_owned()).or_default().extend(tests);
            }
        }
    }
    route.entry(primary.to_owned()).or_default().extend(direct);
    Ok(route)
}

pub(crate) fn configuration(
    registry: &Value,
    key: &str,
    fallback: u64,
    max: u64,
) -> Result<u64, String> {
    match registry["control_plane"]["budgets"].get(key) {
        None => Ok(fallback),
        Some(value) => value
            .as_u64()
            .filter(|n| *n > 0 && *n <= max)
            .ok_or_else(|| format!("Invalid configured resource limit {key}")),
    }
}

fn budget(
    state: &Value,
    registry: &Value,
    key: &str,
    fallback: u64,
    max: u64,
) -> Result<u64, String> {
    let cap = configuration(registry, key, fallback, max)?;
    match state["descriptor"]["budget"].get(key) {
        None => Ok(cap),
        Some(value) => value
            .as_u64()
            .filter(|n| *n > 0 && *n <= cap)
            .ok_or_else(|| format!("Task budget {key} exceeds configured limits")),
    }
}
