mod execution;
mod snapshot;

pub use execution::{execute_step, run};
pub(crate) use snapshot::git_executable;

use crate::execution::process;
use crate::{docs, tasks};
use serde_json::{Value, json};
use snapshot::{capture, is_boundary, safe_relative, sensitive};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

fn blocked(reason: &str) -> Value {
    json!({"status":"BLOCKED","reason":reason})
}

fn scoped(descriptor: &Value, scope: &str, repo: &str, path: &str) -> bool {
    descriptor[scope][repo]
        .as_array()
        .is_some_and(|paths| paths.iter().any(|candidate| candidate == path))
}

fn rows(value: &Value) -> impl Iterator<Item = &Value> {
    value["repositories"].as_array().into_iter().flatten()
}

fn valid_actor(actor: &str) -> bool {
    !actor.is_empty()
        && actor.len() <= 96
        && actor
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_@.".contains(&c))
}

pub fn capture_baseline(
    root: &Path,
    task: &Value,
    registry: &Value,
    workspace: Option<&Path>,
) -> Result<Value, String> {
    let capture = capture(root, task, registry, workspace)?;
    for repo in rows(&capture) {
        let repo_id = repo["repo"].as_str().ok_or("Invalid captured repo")?;
        if let Some((path, _)) = repo["dirty"]
            .as_object()
            .into_iter()
            .flatten()
            .find(|(path, _)| scoped(&task["descriptor"], "write_scope", repo_id, path))
        {
            return Err(format!(
                "Existing unowned change overlaps the grant: {repo_id}/{path}"
            ));
        }
        if let Some((path, _)) =
            repo["inputs"]
                .as_object()
                .into_iter()
                .flatten()
                .find(|(path, value)| {
                    scoped(&task["descriptor"], "write_scope", repo_id, path)
                        && value["bytes"].is_number()
                        && value["index"].is_null()
                })
        {
            return Err(format!(
                "Existing untracked or ignored file overlaps the grant: {repo_id}/{path}"
            ));
        }
    }
    Ok(capture)
}

fn boundary(registry: &Value, path: &str) -> bool {
    is_boundary(path)
        || registry["control_plane"]["boundaries"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .any(|boundary| {
                if boundary.contains('/') {
                    path == boundary
                        || path
                            .strip_prefix(boundary)
                            .is_some_and(|suffix| suffix.starts_with('/'))
                } else {
                    path.split(['/', '.', '-', '_'])
                        .any(|part| part.eq_ignore_ascii_case(boundary))
                }
            })
}

pub fn snapshot(
    root: &Path,
    task: &Value,
    registry: &Value,
    workspace: Option<&Path>,
) -> Result<Value, String> {
    let capture = capture(root, task, registry, workspace)?;
    let descriptor = &task["descriptor"];
    let baseline = &task["baseline"];
    let docs_applied = matches!(
        task["state"].as_str(),
        Some("ACCEPTED" | "DOCS_SYNCED" | "CLOSED")
    );
    let docs_only = |repo, path| {
        scoped(descriptor, "documentation_scope", repo, path)
            && !scoped(descriptor, "verification_scope", repo, path)
            && !scoped(descriptor, "write_scope", repo, path)
    };
    let mut scope = Vec::new();
    let mut violations = Vec::new();
    let mut boundary_paths = Vec::new();
    let mut unrelated_preexisting = 0;
    for row in rows(&capture) {
        let repo = row["repo"].as_str().ok_or("Invalid captured repo")?;
        let initial = rows(baseline).find(|candidate| candidate["repo"] == repo);
        let mut changed = BTreeSet::new();
        if let Some(initial) = initial {
            if initial["identity"] != row["identity"] || initial["commit"] != row["commit"] {
                violations.push(json!({"repo":repo,"reason":"Worktree identity or commit changed after task start"}));
            }
            let a = initial["dirty"]
                .as_object()
                .ok_or("Invalid initial diff metadata")?;
            let b = row["dirty"].as_object().ok_or("Invalid diff metadata")?;
            for path in a.keys().chain(b.keys()).collect::<BTreeSet<_>>() {
                if a.get(path) != b.get(path) {
                    changed.insert(path.as_str());
                } else if !scoped(descriptor, "write_scope", repo, path) {
                    unrelated_preexisting += 1;
                }
            }
            let inputs = row["inputs"]
                .as_object()
                .ok_or("Invalid captured file inputs")?;
            for (path, fingerprint) in inputs {
                if initial["inputs"].get(path) != Some(fingerprint) {
                    changed.insert(path);
                }
            }
        } else if !baseline.is_null() {
            violations.push(json!({"repo":repo,"reason":"Task start did not bind this worktree"}));
        }
        let mut authorized_changes = Vec::new();
        for path in changed {
            if docs_applied && docs_only(repo, path) {
                continue;
            }
            if sensitive(path) {
                violations.push(json!({"repo":repo,"reason":"Protected path changed"}));
                continue;
            }
            let signal_changed = initial.is_some_and(|initial| {
                initial["inputs"][path]["boundary_signals"]
                    != row["inputs"][path]["boundary_signals"]
            });
            if boundary(registry, path) || signal_changed {
                boundary_paths.push(json!({"repo":repo,"path":path}));
                if matches!(
                    descriptor["risk"].as_str(),
                    Some("low" | "normal" | "LOW" | "NORMAL")
                ) {
                    violations.push(json!({"repo":repo,"path":path,"reason":"Boundary changed. Reassess task scope and risk."}));
                }
            }
            if scoped(descriptor, "write_scope", repo, path) {
                authorized_changes.push(path);
            } else {
                violations.push(json!({"repo":repo,"path":path,"reason":"Changed path is outside the task write grant"}));
            }
        }
        let inputs: BTreeMap<&str, &Value> = row["inputs"]
            .as_object()
            .ok_or("Invalid captured file inputs")?
            .iter()
            .filter_map(|(path, value)| (!docs_only(repo, path)).then_some((path.as_str(), value)))
            .collect();
        scope.push(json!({
            "repo":repo,"identity":row["identity"],"commit":row["commit"],
            "inputs":inputs,"changed_paths":authorized_changes
        }));
    }
    let evidence = json!({
        "repositories":scope,"approval_epoch":task["approval_epoch"],
        "descriptor":descriptor,"spec_sha256":task["spec_sha256"],"plan_sha256":task["plan_sha256"],
        "policy_sha256":task["policy_sha256"]
    });
    let bytes = serde_json::to_vec(&evidence).map_err(|_| "Unable to serialize scoped inputs")?;
    Ok(json!({
        "status":if violations.is_empty() {"PASS"} else {"BLOCKED"},
        "sha256":docs::sha256(&bytes),"repositories":scope,"violations":violations,
        "requires_review":!boundary_paths.is_empty() || matches!(descriptor["risk"].as_str(), Some("high" | "hardware")),
        "boundary_paths":boundary_paths,"unrelated_preexisting_metadata_only":unrelated_preexisting,
        "content_files":capture["content_files"],"content_bytes":capture["content_bytes"],
        "limits":"Explicit hashing covers scoped files. Git status may read other working files in the bound repositories internally. Ignored paths and unrelated changes that keep the same preexisting porcelain status are outside diff detection."
    }))
}

pub fn review_diff(
    root: &Path,
    workspace_root: Option<&Path>,
    registry: &Value,
    task: &Value,
    snapshot: &Value,
    mut remaining: usize,
) -> Result<Value, String> {
    let mut review = Vec::new();
    let budget = remaining;
    for row in rows(snapshot) {
        let repo = text(&row["repo"], "Missing review repository")?;
        let paths: Vec<_> = row["changed_paths"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|path| {
                safe_relative(path)
                    && !sensitive(path)
                    && scoped(&task["descriptor"], "read_scope", repo, path)
            })
            .collect();
        if paths.is_empty() {
            continue;
        }
        if remaining == 0 {
            review.push(json!({"repository":repo,"paths":paths,"status":"BLOCKED","reason":"Review diff budget exhausted"}));
            continue;
        }
        let binding = crate::workspace::resolve(root, workspace_root, repo, registry)?;
        if row["identity"] != binding.identity || row["commit"] != binding.commit {
            return Err("Review inputs moved to a different worktree".into());
        }
        let mut git = snapshot::no_filters(root, &binding.path)?;
        git.args([
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--no-color",
            "HEAD",
            "--",
        ])
        .args(&paths);
        let output =
            process::run_process(&mut git, std::time::Duration::from_secs(4), remaining, None)?;
        let diff = std::str::from_utf8(&output.output.prefix)
            .ok()
            .filter(|body| !crate::workspace::contains_known_secrets(body));
        if !output.exit.success() || output.timed_out || output.output.truncated {
            review.push(json!({"repository":repo,"paths":paths,"status":"WITHHELD","reason":"Unavailable or oversized diff; inspect locally"}));
            continue;
        }
        match diff {
            None => review.push(json!({"repository":repo,"paths":paths,"status":"WITHHELD","reason":"Sensitive or non-text diff; inspect locally"})),
            Some("") => review.push(json!({"repository":repo,"paths":paths,"status":"NOT_APPLICABLE","reason":"No tracked diff; inspect selected source and changed paths"})),
            Some(diff) => {
                remaining -= diff.len();
                review.push(json!({"repository":repo,"paths":paths,"status":"PASS","trust":"untrusted_repository_data","sha256":docs::sha256(diff.as_bytes()),"diff":diff}));
            }
        }
    }
    Ok(json!({"repositories":review,"emitted_bytes":budget-remaining}))
}

#[derive(Debug)]
struct Step<'a> {
    id: &'a str,
    repo: &'a str,
    program: &'a str,
    args: Vec<&'a str>,
    paths: Vec<&'a str>,
    mode: &'a str,
    timeout_ms: u64,
    log_bytes: usize,
}

fn text<'a>(value: &'a Value, reason: &str) -> Result<&'a str, String> {
    value
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| reason.into())
}

fn step<'a>(
    task: &'a Value,
    registry: &'a Value,
    id: &'a str,
    actor: &str,
    dry_run: bool,
) -> Result<Step<'a>, String> {
    if !valid_actor(actor) {
        return Err("A valid local operator identity is required".into());
    }
    let descriptor = &task["descriptor"];
    if !descriptor["commands"]
        .as_array()
        .is_some_and(|steps| steps.iter().any(|candidate| candidate == id))
    {
        return Err("Step is outside the approved task command set".into());
    }
    let selected = registry["control_plane"]["commands"]
        .get(id)
        .ok_or("Step is outside the command registry")?;
    let mode = text(&selected["mode"], "Step mode is missing")?;
    let allowed_state = match mode {
        "WORK" => {
            tasks::authorize(task, "run")?;
            if descriptor["owner"] != actor || descriptor["mode"] != "WORK" {
                return Err("Work step requires the task owner and WORK authorization".into());
            }
            task["state"] == "IN_PROGRESS"
        }
        "VERIFY" => {
            tasks::authorize(task, "verify")?;
            if descriptor["owner"] != actor && descriptor["authorization"]["owner"] != actor {
                return Err(
                    "Verify step requires the task owner or authorized verification actor".into(),
                );
            }
            matches!(task["state"].as_str(), Some("IMPLEMENTED" | "VERIFIED"))
        }
        _ => return Err("Unsupported step mode".into()),
    };
    if !(allowed_state || dry_run && task["state"] == "READY") {
        return Err("Task state does not permit the requested step".into());
    }
    let repo = text(&selected["repository"], "Step repository is missing")?;
    let program = text(&selected["program"], "Step executable is missing")?;
    let args: Vec<&str> = selected["args"]
        .as_array()
        .ok_or("Step argv is missing")?
        .iter()
        .map(|value| text(value, "Invalid step argument"))
        .collect::<Result<_, _>>()?;
    if !matches!(
        (program, args.as_slice()),
        ("git", ["diff", "--check"] | ["rev-parse", "HEAD"]) | ("check-json" | "check-utf8", [])
    ) {
        return Err("Executable and argv combination has no reviewed runner profile".into());
    }
    let scope = if mode == "VERIFY" {
        "verification_scope"
    } else {
        "read_scope"
    };
    let scope = descriptor[scope][repo]
        .as_array()
        .ok_or("Step checkout is outside its task scope")?;
    let configured = selected["paths"]
        .as_array()
        .ok_or("Step paths are missing")?;
    let paths = if configured.is_empty() {
        scope
    } else {
        configured
    };
    let paths: Vec<&str> = paths
        .iter()
        .map(|value| text(value, "Invalid step path"))
        .collect::<Result<_, _>>()?;
    if paths.len() > 64
        || (paths.is_empty() && args != ["rev-parse", "HEAD"])
        || paths.iter().any(|path| {
            !scope.iter().any(|allowed| allowed == path) || !safe_relative(path) || sensitive(path)
        })
    {
        return Err("Step paths exceed the explicit task scope or include a protected path".into());
    }
    let budgets = &registry["control_plane"]["budgets"];
    let budget = &descriptor["budget"];
    let cap = |step_key: &str, task_key: &str, maximum: u64| {
        [
            selected[step_key].as_u64(),
            budget[task_key].as_u64(),
            budgets[step_key].as_u64(),
            Some(maximum),
        ]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(1)
        .max(1)
    };
    Ok(Step {
        id,
        repo,
        program,
        args,
        paths,
        mode,
        timeout_ms: cap("timeout_ms", "timeout_ms", 30_000),
        log_bytes: cap("max_output_bytes", "log_bytes", 65_536) as usize,
    })
}

impl Step<'_> {
    fn plan(&self) -> Value {
        json!({
            "step":self.id,"repository":self.repo,"program":self.program,"argv":self.args,
            "paths":self.paths,"mode":self.mode,"timeout_ms":self.timeout_ms,"log_bytes":self.log_bytes,
            "enforcement":"Fixed, read-only runner profiles; no generic subprocess sandbox"
        })
    }
}

#[cfg(test)]
mod tests;
