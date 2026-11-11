use super::{
    EventContext, append, apply_prepared, clock, coordinate, dependencies, ensure_quiescent,
    model::{
        Clock, Coordination, Lease, LeaseStatus, MAX_RECEIPTS, Report, RequestReceipt, Source,
        Submission, digest, is_sha256, public_label,
    },
    prepare_start, put, record_digest, revision, scope, session,
};
use crate::{docs, runner, tasks, workspace};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::Path,
};

const DEFAULT_TTL: u64 = 900;
const MAX_REPORT_BYTES: usize = 32_768;

fn required<'a>(args: &'a BTreeMap<String, String>, name: &str) -> Result<&'a str, String> {
    args.get(name)
        .map(String::as_str)
        .ok_or_else(|| format!("Missing --{name}"))
}

fn number(args: &BTreeMap<String, String>, name: &str) -> Result<u64, String> {
    required(args, name)?
        .parse()
        .map_err(|_| format!("Invalid --{name}"))
}

fn compare_revision(task: &Value, expected: u64) -> Result<(), String> {
    if revision(task)? != expected {
        return Err("Task revision conflict; reload and resume".into());
    }
    Ok(())
}

fn ttl(args: &BTreeMap<String, String>) -> Result<u64, String> {
    if !args.contains_key("ttl") {
        return Ok(DEFAULT_TTL);
    }
    let ttl = number(args, "ttl")?;
    if !(30..=3600).contains(&ttl) {
        return Err("Lease TTL must be 30–3600 seconds".into());
    }
    Ok(ttl)
}

fn expires(
    task: &Value,
    state: &Coordination,
    session_id: &str,
    now: &Clock,
    ttl: u64,
) -> Result<u64, String> {
    let session = state
        .sessions
        .get(session_id)
        .ok_or("Local session is unavailable")?;
    let end = now
        .unix_seconds
        .saturating_add(ttl)
        .min(session.expires_at)
        .min(
            task["descriptor"]["authorization"]["expires_at"]
                .as_u64()
                .unwrap_or(u64::MAX),
        );
    if end <= now.unix_seconds {
        return Err("Session or task authorization has expired".into());
    }
    Ok(end)
}

fn validate_report(raw: &str) -> Result<Report, String> {
    if raw.is_empty() || raw.len() > MAX_REPORT_BYTES {
        return Err("Agent report exceeds its byte budget".into());
    }
    let value = tasks::parse_json(raw).map_err(|_| "Invalid agent report JSON")?;
    tasks::reject_credentials(&value)?;
    let report: Report =
        serde_json::from_value(value).map_err(|_| "Invalid agent report schema")?;
    let bounded = |text: &str, limit: usize| {
        !text.trim().is_empty()
            && text.len() <= limit
            && !workspace::contains_known_secrets(text)
            && !text
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
    };
    if !bounded(&report.summary, 2048)
        || report.changed_paths.len() > 128
        || report.blockers.len() > 16
        || report.commands.len() > 32
        || report.artifacts.len() > 8
        || report.blockers.iter().any(|entry| !bounded(entry, 512))
        || report.commands.iter().any(|entry| {
            !public_label(&entry.command)
                || entry
                    .result
                    .as_ref()
                    .is_some_and(|result| !bounded(result, 128))
        })
    {
        return Err("Agent report exceeds its field bounds".into());
    }
    let artifacts: BTreeSet<_> = report.artifacts.iter().collect();
    if artifacts.len() != report.artifacts.len()
        || artifacts.iter().any(|value| !is_sha256(value))
        || report
            .commands
            .iter()
            .filter_map(|command| command.log_sha256.as_ref())
            .any(|digest| !artifacts.contains(digest))
    {
        return Err("Invalid or unlisted agent report artifact digest".into());
    }
    let mut seen = BTreeSet::new();
    for path in &report.changed_paths {
        if !public_label(&path.repo) || !seen.insert((&path.repo, &path.path)) {
            return Err("Invalid or repeated agent report path".into());
        }
        tasks::relative(&path.path)?;
    }
    Ok(report)
}

pub(super) fn validate_artifacts(root: &Path, report: &Report) -> Result<(), String> {
    const MAX_BYTES: u64 = 1024 * 1024;
    let mut remaining = 4 * MAX_BYTES;
    for digest in &report.artifacts {
        if !is_sha256(digest) {
            return Err("Invalid local artifact digest".into());
        }
        let path = workspace::scoped_path(root, &format!(".cirnetrules/artifacts/{digest}"))?;
        let limit = remaining.min(MAX_BYTES);
        let file = super::credential::open_read(&path, limit)
            .map_err(|_| "Required local report artifact is absent or exceeds its byte budget")?;
        let mut bytes = Vec::new();
        file.take(limit + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "Cannot read local report artifact")?;
        if bytes.len() as u64 > limit || docs::sha256(&bytes) != *digest {
            return Err(
                "Local report artifact bytes do not match the declared digest or budget".into(),
            );
        }
        remaining -= bytes.len() as u64;
    }
    Ok(())
}

fn build_submission(
    task: &Value,
    state: &Coordination,
    snapshot: &Value,
    report: Report,
    session_id: &str,
) -> Result<Submission, String> {
    if snapshot["status"] != "PASS" {
        return Err("Observed diff exceeds the current task grant".into());
    }
    let source = snapshot["repositories"]
        .as_array()
        .ok_or("Invalid observed source snapshot")?
        .iter()
        .map(|row| {
            Ok(Source {
                repository: row["repo"]
                    .as_str()
                    .ok_or("Invalid source repository")?
                    .to_owned(),
                identity: row["identity"]
                    .as_str()
                    .ok_or("Invalid source identity")?
                    .to_owned(),
                base_commit: row["commit"]
                    .as_str()
                    .ok_or("Invalid source commit")?
                    .to_owned(),
                observed_changes: serde_json::from_value(row["changed_paths"].clone())
                    .map_err(|_| "Invalid observed change paths")?,
            })
        })
        .collect::<Result<Vec<Source>, String>>()?;
    for path in &report.changed_paths {
        if !source
            .iter()
            .any(|row| row.repository == path.repo && row.observed_changes.contains(&path.path))
        {
            return Err("Agent report claimed a path absent from the observed diff".into());
        }
    }
    for command in &report.commands {
        if !task["descriptor"]["commands"]
            .as_array()
            .is_some_and(|commands| commands.iter().any(|id| id == &command.command))
        {
            return Err("Agent report refers to a command absent from its grant".into());
        }
    }
    let mut submission = Submission {
        submission_id: String::new(),
        task_revision: revision(task)?,
        approval_epoch: task["approval_epoch"]
            .as_u64()
            .ok_or("Invalid task approval epoch")?,
        generation: state.generation,
        session_id: session_id.to_owned(),
        source_sha256: snapshot["sha256"]
            .as_str()
            .ok_or("Invalid observed source digest")?
            .to_owned(),
        authority_sha256: scope(task)?,
        binding_sha256: digest(&task["execution_bindings"])?,
        dependencies_sha256: digest(&state.dependencies)?,
        source,
        report,
    };
    submission.submission_id = digest(&submission)?;
    Ok(submission)
}

fn replay(
    state: &Coordination,
    request_id: &str,
    action: &str,
    args_hash: &str,
    caller: &str,
) -> Result<Option<Value>, String> {
    let Some(receipt) = state.receipts.get(request_id) else {
        return Ok(None);
    };
    if receipt.session_id != caller
        || receipt.args_sha256 != args_hash
        || receipt.operation != action
    {
        return Err("Request ID conflict; arguments or caller scope changed".into());
    }
    if digest(&receipt.response)? != receipt.response_sha256
        || receipt.response["revision"] != receipt.revision
    {
        return Err("Committed request receipt failed integrity validation".into());
    }
    Ok(Some(receipt.response.clone()))
}

struct Mutation<'a> {
    root: &'a Path,
    id: &'a str,
    request_id: Option<&'a str>,
    args_hash: &'a str,
    caller: &'a str,
    actor: Option<&'a str>,
    now: &'a Clock,
    before: String,
}

impl Mutation<'_> {
    fn commit(
        self,
        task: &mut Value,
        mut state: Coordination,
        action: &str,
        mut response: Value,
    ) -> Result<Value, String> {
        let rev = revision(task)?
            .checked_add(1)
            .ok_or("Task revision exhausted")?;
        task["revision"] = json!(rev);
        response["revision"] = json!(rev);
        response["state"] = task["state"].clone();
        response["task"] = json!(self.id);
        response["lease"] = json!(state.lease.as_ref().map(|lease| lease.summary(self.now)));
        if let Some(request) = self.request_id {
            if state.receipts.len() >= MAX_RECEIPTS {
                return Err("Task request receipt budget exhausted".into());
            }
            state.receipts.insert(
                request.to_owned(),
                RequestReceipt {
                    operation: action.to_owned(),
                    args_sha256: self.args_hash.to_owned(),
                    session_id: self.caller.to_owned(),
                    revision: rev,
                    response_sha256: digest(&response)?,
                    response: response.clone(),
                },
            );
        }
        put(task, state)?;
        append(
            task,
            EventContext {
                operation: if response["reason"] == "Task lease expired; recovery is required" {
                    "expire"
                } else {
                    action
                },
                actor: self.actor,
                session_id: self.actor.is_none().then_some(self.caller),
                before: Some(self.before),
                outcome: response["status"].as_str().unwrap_or("BLOCKED"),
                evidence_kind: if action == "report" {
                    "AGENT_ASSERTION"
                } else if self.actor.is_none() {
                    "LOCAL_CAPABILITY"
                } else {
                    "LOCAL_ACTOR_ASSERTION"
                },
                time: self.now.unix_seconds,
            },
        )?;
        tasks::persist(self.root, self.id, task)?;
        Ok(response)
    }
}

fn status(root: &Path, task: &Value) -> Result<Value, String> {
    let now = clock()?;
    let state = coordinate(task)?;
    let id = task["descriptor"]["id"]
        .as_str()
        .ok_or("Invalid task identity")?;
    Ok(json!({
        "status":"PASS","task":id,"state":task["state"],"revision":task["revision"],
        "approval_epoch":task["approval_epoch"],"task_schema":task["task_schema"],
        "authority":if tasks::load_current(root,id).is_ok() {"CURRENT"} else {"REVIEW_REQUIRED"},
        "lease":state.lease.as_ref().map(|lease| lease.summary(&now)),
        "depends_on":task["descriptor"]["depends_on"],"required_children":task["descriptor"]["required_children"],
        "blocker_count":task["blockers"].as_array().map_or(0, Vec::len),
        "submission_id":state.submission.as_ref().map(|record| &record.submission_id),
        "report_blocker_count":state.submission.as_ref().map_or(0, |record| record.report.blockers.len()),
        "evidence_kind":state.submission.as_ref().map(|_| "AGENT_ASSERTION")
    }))
}

fn check_active(
    task: &Value,
    state: &Coordination,
    caller: &str,
    generation: u64,
    now: &Clock,
) -> Result<bool, String> {
    let lease = state
        .lease
        .as_ref()
        .ok_or("Task has no active coordinated lease")?;
    if lease.session_id != caller
        || lease.generation != generation
        || lease.status != LeaseStatus::Active
    {
        return Err("Task lease generation or owner has changed".into());
    }
    if lease.scope_sha256 != scope(task)?
        || lease.binding_sha256 != digest(&task["execution_bindings"])?
    {
        return Err("Task lease authority or binding changed".into());
    }
    Ok(lease.status_at(now) == LeaseStatus::RecoveryRequired)
}

pub(crate) fn validate_bindings(
    root: &Path,
    work: Option<&Path>,
    registry: &Value,
    task: &Value,
) -> Result<(), String> {
    for (repo, _) in task["execution_bindings"].as_object().into_iter().flatten() {
        workspace::resolve_task(root, work, repo, registry, task)?;
    }
    Ok(())
}

fn recover(
    root: &Path,
    work: Option<&Path>,
    registry: &Value,
    task: &Value,
    now: &Clock,
) -> Result<bool, String> {
    let state = coordinate(task)?;
    let lease = state.lease.as_ref().ok_or("Task has no lease to recover")?;
    if lease.status_at(now) != LeaseStatus::RecoveryRequired && tasks::current(root, task).is_ok() {
        return Err("Task lease does not require recovery".into());
    }
    if let Some(submission) = state.submission {
        let current = runner::snapshot(root, task, registry, work)?;
        if current["status"] != "PASS" || current["sha256"] != submission.source_sha256 {
            return Err("Sealed source changed; retain the workspace for operator review".into());
        }
        if !matches!(task["state"].as_str(), Some("CANCELLED" | "SUPERSEDED")) {
            return Ok(true);
        }
    }
    let current = runner::capture_baseline(root, task, registry, work)?;
    let reservations = super::reservations(task)?;
    for row in current["repositories"].as_array().into_iter().flatten() {
        let repo = row["repo"].as_str().ok_or("Invalid recovery repository")?;
        if task["execution_bindings"][repo]["physical_identity"]
            .as_str()
            .is_some_and(|identity| reservations.contains(identity))
            && !row["dirty"]
                .as_object()
                .is_some_and(|changes| changes.is_empty())
        {
            return Err(format!(
                "Worktree {repo} is dirty; retain it for operator review"
            ));
        }
    }
    Ok(false)
}

enum Prepared {
    None,
    Session(String),
    Claim(super::Prepared),
    Renew(Vec<super::model::Dependency>),
    Report(Value, Report, Vec<super::model::Dependency>),
    Release(bool),
    Recover(bool),
}

pub(super) fn run(
    root: &Path,
    work: Option<&Path>,
    registry: &Value,
    action: &str,
    args: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let flags: &[&str] = match action {
        "status" | "dependency-check" => &[],
        "session-prepare" => &["actor", "revision", "request-id"],
        "claim" => &["revision", "ttl"],
        "renew" => &["revision", "generation", "ttl"],
        "release" => &["revision", "generation"],
        "report" => &["revision", "generation", "report"],
        "recover" => &["revision", "generation", "request-id", "actor", "stopped"],
        "migrate" => &["revision", "actor", "confirm"],
        _ => return Err("Unknown task coordination operation".into()),
    };
    let session_action = matches!(action, "claim" | "renew" | "release" | "report");
    if args.keys().any(|key| {
        !matches!(key.as_str(), "task" | "workspace")
            && !flags.contains(&key.as_str())
            && !(session_action
                && matches!(key.as_str(), "request-id" | "session" | "session-file"))
    }) {
        return Err("Unexpected task coordination argument".into());
    }
    let id = required(args, "task")?;
    let mut task = tasks::load(root, id)?;
    if action == "status" {
        return status(root, &task);
    }
    if action == "dependency-check" {
        tasks::current(root, &task)?;
        let deps = dependencies::capture(root, work, registry, &task, None, true)?;
        let pinned = coordinate(&task)?.dependencies;
        if !pinned.is_empty() && deps.get(..pinned.len()) != Some(pinned.as_slice()) {
            return Err("Pinned task dependency evidence is stale".into());
        }
        return Ok(
            json!({"status":"PASS","task":id,"revision":task["revision"],"dependencies":deps}),
        );
    }
    if action == "migrate" {
        return super::migrate(root, registry, id, args);
    }
    if task["task_schema"] != 2 {
        return Err("Task schema requires explicit migration before coordination".into());
    }
    let now = clock()?;
    let token = match (args.get("session"), args.get("session-file")) {
        (Some(token), None) => Some(token.clone()),
        (None, Some(file)) => Some(session::read_token(root, file)?),
        (None, None) if !session_action => None,
        _ => return Err("A single prepared local session capability is required".into()),
    };
    let actor = if session_action {
        None
    } else {
        Some(required(args, "actor")?)
    };
    let caller = if let Some(token) = &token {
        session::authenticate(root, work, &task, token, &now, false)?
    } else {
        let actor = actor.ok_or("Missing local authorization principal")?;
        tasks::actor_is(&task, actor, true)?;
        format!("local:{actor}")
    };
    let request_id = if session_action || action == "recover" {
        Some(required(args, "request-id")?)
    } else {
        args.get("request-id").map(String::as_str)
    };
    if request_id.is_some_and(|request| !public_label(request)) {
        return Err("Invalid business request ID".into());
    }
    let mut canonical_args = args
        .iter()
        .filter(|(key, _)| !matches!(key.as_str(), "session" | "session-file" | "workspace"))
        .map(|(key, value)| (key.as_str(), json!(value)))
        .collect::<BTreeMap<_, _>>();
    let report = if action == "report" {
        let raw = required(args, "report")?;
        let report = validate_report(raw)?;
        canonical_args.insert("report", json!(report));
        Some(report)
    } else {
        None
    };
    for flag in ["revision", "generation"] {
        if args.contains_key(flag) {
            canonical_args.insert(flag, json!(number(args, flag)?));
        }
    }
    if matches!(action, "claim" | "renew") {
        canonical_args.insert("ttl", json!(ttl(args)?));
    }
    let args_hash = digest(&canonical_args)?;
    let state = coordinate(&task)?;
    if let Some(request) = request_id
        && let Some(response) = replay(&state, request, action, &args_hash, &caller)?
    {
        return Ok(response);
    }
    let expected = if matches!(action, "renew" | "release") && !args.contains_key("revision") {
        None
    } else {
        Some(number(args, "revision")?)
    };
    if let Some(expected) = expected {
        compare_revision(&task, expected)?;
    }
    let generation = if matches!(action, "renew" | "release" | "report" | "recover") {
        Some(number(args, "generation")?)
    } else {
        None
    };
    let expired = if session_action && action != "claim" {
        check_active(&task, &state, &caller, generation.unwrap(), &now)?
    } else {
        false
    };
    let prepared = match action {
        "session-prepare" => {
            let bindings = workspace::freeze_task_bindings(root, work, registry, &task)?;
            Prepared::Session(digest(&bindings)?)
        }
        "claim" if task["state"] == "READY" => {
            tasks::current(root, &task)?;
            session::authenticate(root, work, &task, token.as_deref().unwrap(), &now, true)?;
            Prepared::Claim(prepare_start(root, work, registry, &task)?)
        }
        "report" if !expired => {
            let report = report.ok_or("Missing agent report")?;
            validate_artifacts(root, &report)?;
            let snapshot = runner::snapshot(root, &task, registry, work)?;
            let deps = dependencies::capture(root, work, registry, &task, Some(&snapshot), false)?;
            Prepared::Report(snapshot, report, deps)
        }
        "renew" if !expired => Prepared::Renew(dependencies::capture(
            root, work, registry, &task, None, false,
        )?),
        "release" if !expired => Prepared::Release(if let Some(submission) = &state.submission {
            let source = runner::snapshot(root, &task, registry, work)?;
            source["status"] == "PASS" && source["sha256"] == submission.source_sha256
        } else {
            false
        }),
        "recover" => {
            if required(args, "stopped")? != "yes" || state.generation != generation.unwrap() {
                return Err("Recovery requires current generation and explicit stopped=yes".into());
            }
            Prepared::Recover(recover(root, work, registry, &task, &now)?)
        }
        _ => Prepared::None,
    };
    let prepared_revision = revision(&task)?;
    let _coordinator = tasks::lock(root, "_coordination")?;
    let _task = tasks::lock(root, &id.to_ascii_lowercase())?;
    task = tasks::load(root, id)?;
    let now = clock()?;
    let mut state = coordinate(&task)?;
    if let Some(token) = &token {
        session::authenticate(root, work, &task, token, &now, false)?;
    } else {
        tasks::actor_is(
            &task,
            actor.ok_or("Missing local authorization principal")?,
            true,
        )?;
    }
    if let Some(request) = request_id
        && let Some(response) = replay(&state, request, action, &args_hash, &caller)?
    {
        return Ok(response);
    }
    compare_revision(&task, prepared_revision)?;
    let mutation = Mutation {
        root,
        id,
        request_id,
        args_hash: &args_hash,
        caller: &caller,
        actor,
        now: &now,
        before: record_digest(&task)?,
    };
    if session_action
        && action != "claim"
        && check_active(&task, &state, &caller, generation.unwrap(), &now)?
    {
        state.lease.as_mut().unwrap().status = LeaseStatus::RecoveryRequired;
        if matches!(task["state"].as_str(), Some("IN_PROGRESS" | "IMPLEMENTED")) {
            task["state"] = json!("BLOCKED");
        }
        return mutation.commit(
            &mut task,
            state,
            action,
            json!({
                "status":"BLOCKED","reason":"Task lease expired; recovery is required"
            }),
        );
    }
    if action != "recover" {
        tasks::current(root, &task)?;
    }
    if let Some(token) = &token {
        session::authenticate(root, work, &task, token, &now, true)?;
        tasks::authorize(&task, "write")?;
    }
    let mut response = json!({"status":"PASS"});
    match action {
        "session-prepare" => {
            if state.events.len() >= super::model::MAX_EVENTS
                || (request_id.is_some() && state.receipts.len() >= MAX_RECEIPTS)
            {
                return Err("Task session preparation exceeds its record budget".into());
            }
            let Prepared::Session(binding_sha256) = prepared else {
                return Err("Missing session binding observation".into());
            };
            let (_, output) = session::prepare(
                root,
                work,
                &task,
                actor.unwrap(),
                &now,
                binding_sha256,
                &mut state,
            )?;
            response
                .as_object_mut()
                .unwrap()
                .extend(output.as_object().unwrap().clone());
        }
        "claim" => {
            if task["state"] != "READY"
                || state
                    .lease
                    .as_ref()
                    .is_some_and(|lease| lease.status != LeaseStatus::Released)
            {
                return Err("Claim requires a ready, unreserved task".into());
            }
            let Prepared::Claim(start) = prepared else {
                return Err("No prepared task baseline".into());
            };
            ensure_quiescent(root, id)?;
            apply_prepared(root, &mut task, Some(start))?;
            tasks::overlapping(root, &task)?;
            validate_bindings(root, work, registry, &task)?;
            state = coordinate(&task)?;
            let session = state
                .sessions
                .get(&caller)
                .ok_or("Missing authenticated task session")?;
            let binding_sha256 = digest(&task["execution_bindings"])?;
            if session.binding_sha256 != binding_sha256 {
                return Err("Prepared session binding changed; issue a new local session".into());
            }
            state.generation = state
                .generation
                .checked_add(1)
                .ok_or("Task lease generation exhausted")?;
            state.lease = Some(Lease {
                task_id: id.to_owned(),
                session_id: caller.clone(),
                generation: state.generation,
                role: session.role.clone(),
                mode: session.mode.clone(),
                acquired_at: now.unix_seconds,
                renewed_at: now.clone(),
                expires_at: expires(&task, &state, &caller, &now, ttl(args)?)?,
                status: LeaseStatus::Active,
                scope_sha256: scope(&task)?,
                binding_sha256,
            });
            task["state"] = json!("IN_PROGRESS");
        }
        "renew" => {
            validate_bindings(root, work, registry, &task)?;
            let Prepared::Renew(observed) = prepared else {
                return Err("No renewal dependency observation".into());
            };
            if observed != state.dependencies {
                return Err("Pinned task dependencies changed".into());
            }
            dependencies::still_current(root, &observed)?;
            let expires_at = expires(&task, &state, &caller, &now, ttl(args)?)?;
            let lease = state.lease.as_mut().unwrap();
            lease.renewed_at = now.clone();
            lease.expires_at = expires_at;
        }
        "release" => {
            ensure_quiescent(root, id)?;
            validate_bindings(root, work, registry, &task)?;
            let lease = state.lease.as_mut().unwrap();
            let Prepared::Release(unchanged) = prepared else {
                return Err("No release observation".into());
            };
            if unchanged {
                lease.status = LeaseStatus::Sealed;
            } else {
                if state.submission.is_some() {
                    response = json!({"status":"BLOCKED","reason":"Sealed source changed; operator recovery is required"});
                }
                lease.status = LeaseStatus::RecoveryRequired;
                task["state"] = json!("BLOCKED");
            }
        }
        "report" => {
            let Prepared::Report(snapshot, report, observed) = prepared else {
                return Err("No observed report snapshot".into());
            };
            if task["state"] != "IN_PROGRESS" || state.submission.is_some() {
                return Err(
                    "Task already has an immutable submission or is not in progress".into(),
                );
            }
            ensure_quiescent(root, id)?;
            validate_bindings(root, work, registry, &task)?;
            if observed != state.dependencies {
                return Err("Pinned task dependencies changed".into());
            }
            dependencies::still_current(root, &observed)?;
            let submission = match build_submission(&task, &state, &snapshot, report, &caller) {
                Ok(submission) => submission,
                Err(reason) => {
                    return mutation.commit(
                        &mut task,
                        state,
                        action,
                        json!({"status":"BLOCKED","reason":reason}),
                    );
                }
            };
            response["submission_id"] = json!(submission.submission_id);
            response["source_sha256"] = json!(submission.source_sha256);
            response["evidence_kind"] = json!("AGENT_ASSERTION");
            state.submission = Some(submission);
            task["state"] = json!("IMPLEMENTED");
        }
        "recover" => {
            ensure_quiescent(root, id)?;
            let Prepared::Recover(sealed) = prepared else {
                return Err("No recovery observation".into());
            };
            state
                .lease
                .as_mut()
                .ok_or("Task has no lease to recover")?
                .status = if sealed {
                LeaseStatus::Sealed
            } else {
                LeaseStatus::Released
            };
            if !matches!(task["state"].as_str(), Some("CANCELLED" | "SUPERSEDED")) {
                task["state"] = json!(if sealed { "IMPLEMENTED" } else { "BLOCKED" });
            }
        }
        _ => unreachable!(),
    }
    mutation.commit(&mut task, state, action, response)
}
