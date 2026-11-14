mod credential;
mod dependencies;
mod leases;
mod migration;
mod model;
mod session;

use crate::{runner, tasks, workspace};
use model::{
    AuditEvent, Clock, Coordination, Dependency, LeaseStatus, MAX_EVENTS, StoreIdentity, digest,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::ErrorKind,
    path::Path,
};

pub(crate) use credential::create as create_private_file;
pub(crate) use dependencies::{check_dependency_step_inputs, validate_selection};
pub(crate) use leases::validate_bindings;
use migration::migrate;
pub(crate) use session::{
    check as check_session, contains_session_capability, read_token as session_token,
};

pub(crate) fn empty_record() -> Result<Value, String> {
    serde_json::to_value(Coordination::default())
        .map_err(|_| "Cannot initialize task coordination record".into())
}

fn coordinate(task: &Value) -> Result<Coordination, String> {
    if task.get("task_schema").is_none() && task.get("coordination").is_none() {
        return Ok(Coordination::default());
    }
    if task["task_schema"] != 2 {
        return Err("Unsupported task schema; inspect or migrate explicitly".into());
    }
    let state: Coordination = serde_json::from_value(task["coordination"].clone())
        .map_err(|_| "Invalid task coordination record")?;
    if state.events.len() > MAX_EVENTS
        || state.receipts.len() > model::MAX_RECEIPTS
        || state.sessions.len() > model::MAX_SESSIONS
        || state.lease.as_ref().is_some_and(|lease| {
            lease.generation == 0
                || lease.generation != state.generation
                || task["descriptor"]["id"] != lease.task_id
        })
    {
        return Err("Invalid task coordination bounds or lease identity".into());
    }
    Ok(state)
}

fn put(task: &mut Value, state: Coordination) -> Result<(), String> {
    task["coordination"] =
        serde_json::to_value(state).map_err(|_| "Cannot encode task coordination")?;
    Ok(())
}

fn identity(root: &Path) -> Result<StoreIdentity, String> {
    let authority = workspace::workspace_path(root, Some(root))?;
    Ok(StoreIdentity {
        schema: 1,
        authority_sha256: digest(&authority)?,
        reader_version: 2,
        writer_version: 2,
    })
}

pub(crate) fn store_identity(root: &Path) -> Result<Value, String> {
    serde_json::to_value(identity(root)?).map_err(|_| "Cannot encode task store identity".into())
}

pub(crate) fn validate_record(root: &Path, task: &Value) -> Result<(), String> {
    if task["task_schema"] == 2 {
        let recorded: StoreIdentity = serde_json::from_value(task["store_identity"].clone())
            .map_err(|_| "Invalid task store identity")?;
        if recorded != identity(root)? {
            return Err(
                "Task store belongs to another authority root or uses an unsupported schema".into(),
            );
        }
    } else if task.get("store_identity").is_some() {
        return Err("Task record store identity requires a supported task schema".into());
    }
    coordinate(task).map(|_| ())
}

fn revision(task: &Value) -> Result<u64, String> {
    task["revision"]
        .as_u64()
        .ok_or_else(|| "Invalid task revision".into())
}

fn clock() -> Result<Clock, String> {
    Clock::read()
}

fn scope(task: &Value) -> Result<String, String> {
    digest(&[
        &task["descriptor_sha256"],
        &task["spec_sha256"],
        &task["plan_sha256"],
        &task["policy_sha256"],
        &task["approval_epoch"],
    ])
}

pub(crate) fn record_digest(task: &Value) -> Result<String, String> {
    let mut state = task.clone();
    if let Some(coordination) = state["coordination"].as_object_mut() {
        coordination.remove("events");
    }
    digest(&state)
}

struct EventContext<'a> {
    operation: &'a str,
    actor: Option<&'a str>,
    session_id: Option<&'a str>,
    before: Option<String>,
    outcome: &'a str,
    evidence_kind: &'a str,
    time: u64,
}

fn append(task: &mut Value, details: EventContext<'_>) -> Result<(), String> {
    if task["task_schema"] != 2 {
        return Ok(());
    }
    let mut state = coordinate(task)?;
    if state.events.len() >= MAX_EVENTS {
        return Err("Task audit event budget exhausted".into());
    }
    let mut event = AuditEvent {
        sequence: state.events.len() as u64 + 1,
        operation: details.operation.to_owned(),
        task_id: task["descriptor"]["id"]
            .as_str()
            .ok_or("Invalid task ID")?
            .to_owned(),
        revision: revision(task)?,
        actor: details
            .actor
            .filter(|value| model::public_label(value))
            .map(str::to_owned),
        session_id: details.session_id.map(str::to_owned),
        before_sha256: details.before,
        after_sha256: record_digest(task)?,
        previous_event_sha256: state.events.last().map(|entry| entry.event_sha256.clone()),
        outcome: details.outcome.to_owned(),
        evidence_kind: details.evidence_kind.to_owned(),
        timestamp: details.time,
        event_sha256: String::new(),
    };
    event.event_sha256 = event.digest()?;
    state.events.push(event);
    put(task, state)
}

pub(crate) fn record_event(
    task: &mut Value,
    operation: &str,
    actor: Option<&str>,
    before: Option<String>,
) -> Result<(), String> {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| "Local audit clock is unavailable")?
        .as_secs();
    let (outcome, evidence_kind) = match operation {
        "task-verify" => (
            task["evidence"]
                .as_array()
                .and_then(|items| items.last())
                .and_then(|last| last["result"].as_str())
                .filter(|value| matches!(*value, "PASS" | "FAIL" | "BLOCKED"))
                .unwrap_or("BLOCKED")
                .to_owned(),
            "CONTROLLED_RUNNER",
        ),
        "docs-apply" => (
            if task["state"] == "DOCS_SYNCED" {
                "PASS"
            } else {
                "BLOCKED"
            }
            .to_owned(),
            "LOCAL_DOCUMENTATION_SYNC",
        ),
        _ => ("PASS".to_owned(), "LOCAL_ACTOR_ASSERTION"),
    };
    append(
        task,
        EventContext {
            operation,
            actor,
            session_id: None,
            before,
            outcome: &outcome,
            evidence_kind,
            time,
        },
    )
}

pub(crate) fn audit_events(task: &Value) -> Result<Vec<Value>, String> {
    let state = coordinate(task)?;
    Ok(state
        .events
        .iter()
        .enumerate()
        .map(|(index, event)| {
            event.projection(
                index
                    .checked_sub(1)
                    .and_then(|prior| state.events.get(prior)),
                index as u64 + 1,
            )
        })
        .collect())
}

pub(crate) fn guard_update(task: &mut Value, operation: &str) -> Result<(), String> {
    let mut state = coordinate(task)?;
    let Some(lease) = state.lease.as_mut() else {
        return Ok(());
    };
    match lease.status {
        LeaseStatus::Active if matches!(operation, "block" | "cancel" | "supersede") => {
            lease.status = LeaseStatus::RecoveryRequired;
            put(task, state)
        }
        LeaseStatus::Active | LeaseStatus::RecoveryRequired => {
            Err("Task is reserved by a coordinated lease; release or recover it first".into())
        }
        LeaseStatus::Sealed | LeaseStatus::Released => Ok(()),
    }
}

pub(crate) fn reset(old: &Value, replacement: &mut Value) -> Result<(), String> {
    let mut state = coordinate(old)?;
    if let Some(submission) = state.submission.take() {
        if state.submissions.len() >= 32 {
            return Err("Immutable submission history budget exhausted".into());
        }
        state.submissions.push(submission);
    }
    state.lease = None;
    state.dependencies.clear();
    put(replacement, state)
}

pub(crate) fn reserves(task: &Value) -> Result<bool, String> {
    let state = coordinate(task)?;
    Ok(state.lease.is_some_and(|lease| {
        matches!(
            lease.status,
            LeaseStatus::Active | LeaseStatus::RecoveryRequired
        )
    }) || matches!(
        task["state"].as_str(),
        Some("IN_PROGRESS" | "IMPLEMENTED" | "VERIFIED" | "ACCEPTED" | "DOCS_SYNCED")
    ))
}

pub(crate) fn reservations(task: &Value) -> Result<BTreeSet<&str>, String> {
    let mut selected = BTreeSet::new();
    for scope in ["write_scope", "documentation_scope"] {
        for (repo, paths) in task["descriptor"][scope].as_object().into_iter().flatten() {
            if paths.as_array().is_some_and(|items| !items.is_empty())
                && let Some(binding) =
                    task["execution_bindings"][repo]["physical_identity"].as_str()
            {
                if binding.len() != 64 || !binding.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    return Err("Invalid physical worktree reservation identity".into());
                }
                selected.insert(binding);
            }
        }
    }
    Ok(selected)
}

pub(crate) struct Prepared {
    bindings: Option<Value>,
    baseline: Option<Value>,
    dependencies: Vec<Dependency>,
}

fn prepare_start(
    root: &Path,
    work: Option<&Path>,
    registry: &Value,
    task: &Value,
) -> Result<Prepared, String> {
    tasks::ready(root, task, registry)?;
    let bindings =
        serde_json::to_value(workspace::freeze_task_bindings(root, work, registry, task)?)
            .map_err(|_| "Cannot freeze task execution bindings")?;
    let mut frozen = task.clone();
    frozen["execution_bindings"] = bindings.clone();
    let baseline = runner::capture_baseline(root, &frozen, registry, work)?;
    let snapshot = runner::snapshot(root, &frozen, registry, work)?;
    let dependencies =
        dependencies::capture(root, work, registry, &frozen, Some(&snapshot), false)?;
    Ok(Prepared {
        bindings: Some(bindings),
        baseline: Some(baseline),
        dependencies,
    })
}

pub(crate) fn prepare_legacy_transition(
    root: &Path,
    work: Option<&Path>,
    registry: &Value,
    id: &str,
    expected: u64,
    next: &str,
) -> Result<Option<Prepared>, String> {
    if !matches!(next, "READY" | "IN_PROGRESS") {
        return Ok(None);
    }
    let task = tasks::load_current(root, id)?;
    if revision(&task)? != expected {
        return Err("Task revision conflict; reload and resume".into());
    }
    if task["task_schema"] != 2 {
        return Err("Task schema requires explicit migration before a new execution".into());
    }
    if next == "IN_PROGRESS" && task["state"] == "READY" {
        return prepare_start(root, work, registry, &task).map(Some);
    }
    if next == "READY" && task["state"] == "PROPOSED" {
        let dependencies = dependencies::capture(root, work, registry, &task, None, false)?;
        return Ok(Some(Prepared {
            bindings: None,
            baseline: None,
            dependencies,
        }));
    }
    Ok(None)
}

pub(crate) fn apply_prepared(
    root: &Path,
    task: &mut Value,
    prepared: Option<Prepared>,
) -> Result<(), String> {
    let prepared = prepared.ok_or("Task transition has no prepared inputs")?;
    let mut state = coordinate(task)?;
    dependencies::still_current(root, &prepared.dependencies)?;
    if let Some(bindings) = prepared.bindings {
        if state.dependencies != prepared.dependencies {
            return Err("Task dependencies changed after readiness; revise the grant".into());
        }
        task["execution_bindings"] = bindings;
        task["baseline"] = prepared.baseline.ok_or("Task start snapshot is missing")?;
    }
    state.dependencies = prepared.dependencies;
    put(task, state)
}

pub(crate) fn ensure_quiescent(root: &Path, id: &str) -> Result<(), String> {
    let marker = workspace::scoped_path(root, &format!(".cirnetrules/runs/{id}/active.json"))?;
    match fs::symlink_metadata(marker) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        _ => Err(
            "Managed run is active or abandoned; inspect it before changing task ownership".into(),
        ),
    }
}

pub(crate) fn check_run(root: &Path, task: &Value, work: bool) -> Result<(), String> {
    let id = task["descriptor"]["id"]
        .as_str()
        .ok_or("Invalid task identity")?;
    let current = tasks::load_current(root, id)?;
    if current["revision"] != task["revision"] {
        return Err("Task changed before managed execution".into());
    }
    if let Some(lease) = coordinate(&current)?.lease
        && (matches!(
            lease.status,
            LeaseStatus::Active | LeaseStatus::RecoveryRequired
        ) || work)
    {
        return Err("Coordinated task ownership prevents this managed operation".into());
    }
    Ok(())
}

pub(crate) fn check_documentation_writer(
    root: &Path,
    work: Option<&Path>,
    registry: &Value,
    task: &Value,
) -> Result<(), String> {
    let mut checked = task.clone();
    let bindings = workspace::freeze_task_bindings(root, work, registry, task)?;
    for repo in task["descriptor"]["documentation_scope"]
        .as_object()
        .into_iter()
        .flatten()
        .map(|(repo, _)| repo)
    {
        if checked["execution_bindings"][repo].is_null()
            && let Some(binding) = bindings.get(repo)
        {
            checked["execution_bindings"][repo] = serde_json::to_value(binding)
                .map_err(|_| "Cannot identify documentation worktree")?;
        }
    }
    tasks::overlapping(root, &checked)
}

pub(crate) fn check_evidence(
    root: &Path,
    work: Option<&Path>,
    registry: &Value,
    task: &Value,
    snapshot: &Value,
) -> Result<(), String> {
    let state = coordinate(task)?;
    let deps = dependencies::capture(root, work, registry, task, Some(snapshot), true)?;
    let direct = task["descriptor"]["depends_on"]
        .as_array()
        .map_or(0, Vec::len);
    if state.dependencies.get(..direct) != deps.get(..direct) {
        return Err("Pinned task dependencies changed after readiness; revise the grant".into());
    }
    if state.dependencies.len() > direct && state.dependencies != deps {
        return Err("Pinned child evidence changed; revise the integration task".into());
    }
    if state
        .lease
        .as_ref()
        .is_some_and(|lease| lease.status != LeaseStatus::Sealed)
    {
        return Err("Release or recover the coordinated lease before verification".into());
    }
    if state.lease.is_some() && state.submission.is_none() {
        return Err("Coordinated implementation has no sealed submission".into());
    }
    if let Some(submission) = state.submission {
        leases::validate_artifacts(root, &submission.report)?;
        if !submission.report.blockers.is_empty() {
            return Err(
                "Submission records unresolved worker blockers; review and revise the task".into(),
            );
        }
        let direct_digest = digest(&deps[..direct].to_vec())?;
        if submission.source_sha256 != snapshot["sha256"]
            || submission.approval_epoch != task["approval_epoch"]
            || submission.authority_sha256 != scope(task)?
            || submission.binding_sha256 != digest(&task["execution_bindings"])?
            || submission.dependencies_sha256 != direct_digest
        {
            return Err(
                "Sealed implementation source, binding, authority or dependencies changed".into(),
            );
        }
    }
    Ok(())
}

pub(crate) fn commit_evidence_dependencies(
    root: &Path,
    work: Option<&Path>,
    registry: &Value,
    task: &mut Value,
    snapshot: &Value,
) -> Result<(), String> {
    let mut state = coordinate(task)?;
    let deps = dependencies::capture(root, work, registry, task, Some(snapshot), true)?;
    let direct = task["descriptor"]["depends_on"]
        .as_array()
        .map_or(0, Vec::len);
    if state.dependencies.get(..direct) != deps.get(..direct)
        || (state.dependencies.len() > direct && state.dependencies != deps)
    {
        return Err("Task dependency or child evidence changed before commit".into());
    }
    dependencies::still_current(root, &deps)?;
    if task["task_schema"] == 2 {
        state.dependencies = deps;
        put(task, state)?;
    }
    Ok(())
}

pub(crate) fn run(
    root: &Path,
    work: Option<&Path>,
    registry: &Value,
    action: &str,
    args: &BTreeMap<String, String>,
) -> Result<Value, String> {
    leases::run(root, work, registry, action, args)
}

#[cfg(test)]
mod tests;
