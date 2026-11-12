use super::{
    AGENTS, CheckReport, Effect, Finding, NextAction, Scope, Status, TARGETS, context::Context,
    elapsed, finish, label, safe,
};
use crate::{
    agent_config, audit, authority, coordination, docs, environment, lifecycle, policy, runner,
    tasks, workspace,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    time::{Instant, SystemTime},
};

const ASSETS: &[(&str, &str)] = &[
    ("policies", "policy"),
    ("rules", "rule"),
    ("skills", "skill"),
    ("profiles", "profile"),
    ("references", "reference"),
    ("hooks", "hook"),
];
const TASK_CHECKS: &[&str] = &["task", "diff", "verification", "audit"];

pub fn check(
    target: &str,
    scope: &Scope,
    strict: bool,
    cancelled: impl Fn() -> bool,
) -> CheckReport {
    let started = Instant::now();
    let wall = SystemTime::now();
    let requested = json!({
        "target": label(Some(target)), "repo": label(scope.repo.as_deref()),
        "binding": label(scope.binding.as_deref()), "task": label(scope.task.as_deref()),
        "agent": label(scope.agent.as_deref()), "strict": strict
    });
    let mut run = CheckRun {
        target,
        scope,
        findings: Vec::new(),
        cancelled: &cancelled,
    };
    if !TARGETS.contains(&target)
        || scope
            .agent
            .as_deref()
            .is_some_and(|id| id != "all" && !AGENTS.contains(&id))
    {
        run.row(
            "input",
            Status::Error,
            true,
            "INVALID_SELECTOR",
            "Select a supported check and Agent.",
            None,
        );
        return finish(started, wall, requested, Value::Null, run.findings, strict);
    }

    let mut resolved = Value::Null;
    match Context::open(scope) {
        Ok(context) => {
            resolved = context.public();
            run.inspect(&context, &mut resolved);
            if let Err(reason) = context.fresh() {
                for row in run
                    .findings
                    .iter_mut()
                    .filter(|row| row.status == Status::Pass)
                {
                    row.status = Status::Blocked;
                    row.code = "STALE_CONTEXT".into();
                    row.message = safe(&reason);
                    row.evidence_level = None;
                }
                run.row(
                    "governance",
                    Status::Blocked,
                    true,
                    "STALE_CONTEXT",
                    &reason,
                    None,
                );
            }
        }
        Err(reason) => {
            if scope.workspace.is_some() {
                let default_scope = Scope {
                    workspace: None,
                    ..scope.clone()
                };
                if let Ok(context) = Context::open(&default_scope) {
                    run.inspect_authority(&context);
                    run.assets(&context, scope.repo.as_deref());
                    run.row(
                        "workspace",
                        Status::Blocked,
                        true,
                        "WORKSPACE_UNAVAILABLE",
                        &reason,
                        Some((
                            "select-workspace",
                            "Select a local directory containing the intended worktrees.",
                            Effect::ReadOnly,
                        )),
                    );
                    for target in TARGETS.iter().skip(1).filter(|candidate| {
                        run_selects(target, candidate)
                            && !matches!(**candidate, "governance" | "registry")
                            && !ASSETS.iter().any(|(name, _)| name == *candidate)
                    }) {
                        let row = run.row(
                            target,
                            Status::Blocked,
                            false,
                            "PREREQUISITE",
                            "The selected workspace could not be verified.",
                            None,
                        );
                        row.blocked_by.push("workspace".into());
                    }
                    resolved = context.public();
                    resolved["workspace_resolved"] = json!(false);
                    run.stop();
                    return finish(started, wall, requested, resolved, run.findings, strict);
                }
            }
            let missing = super::context::root(scope).is_err();
            let component = if missing { "governance" } else { "registry" };
            run.row(
                component,
                if missing {
                    Status::Blocked
                } else {
                    Status::Error
                },
                true,
                if missing {
                    "AUTHORITY_UNAVAILABLE"
                } else {
                    "AUTHORITY_INPUT_INVALID"
                },
                &reason,
                Some((
                    "open-authority",
                    "Select the governance checkout or review governance.yaml and its schemas.",
                    Effect::ReadOnly,
                )),
            );
            for target in TARGETS
                .iter()
                .skip(1)
                .filter(|candidate| run_selects(target, candidate) && **candidate != component)
            {
                let row = run.row(
                    target,
                    Status::Blocked,
                    false,
                    "PREREQUISITE",
                    "Requires the governance checkout, registry and workspace.",
                    None,
                );
                row.blocked_by.push(component.into());
            }
            if run_selects(target, "environment") {
                run.row(
                    "host",
                    Status::Pass,
                    false,
                    "HOST_IDENTIFIED",
                    "CNR process platform identified.",
                    None,
                )
                .observed = Some(json!({"os":std::env::consts::OS,"arch":std::env::consts::ARCH}));
            }
        }
    }
    run.stop();
    finish(started, wall, requested, resolved, run.findings, strict)
}

fn run_selects(selected: &str, candidate: &str) -> bool {
    selected == "all" || selected == candidate
}

struct CheckRun<'a> {
    target: &'a str,
    scope: &'a Scope,
    findings: Vec<Finding>,
    cancelled: &'a dyn Fn() -> bool,
}

impl CheckRun<'_> {
    fn includes(&self, target: &str) -> bool {
        run_selects(self.target, target)
    }

    fn stop(&mut self) -> bool {
        if !(self.cancelled)() {
            return false;
        }
        if !self
            .findings
            .iter()
            .any(|row| row.status == Status::Cancelled)
        {
            self.row(
                "run",
                Status::Cancelled,
                true,
                "CHECK_CANCELLED",
                "Operator cancelled. The report contains partial coverage.",
                None,
            );
        }
        true
    }

    fn row(
        &mut self,
        component: &str,
        status: Status,
        required: bool,
        code: &str,
        message: &str,
        action: Option<(&str, &str, Effect)>,
    ) -> &mut Finding {
        let repository = label(self.scope.repo.as_deref());
        let binding_id = label(self.scope.binding.as_deref());
        let task_id = label(self.scope.task.as_deref());
        let actions = action
            .into_iter()
            .map(|(id, name, effect)| NextAction {
                id: id.into(),
                label: name.into(),
                operation: id.into(),
                typed_arguments: BTreeMap::new(),
                effect,
                requires_confirmation: !matches!(effect, Effect::ReadOnly),
                required_grant: matches!(effect, Effect::RepositoryWrite)
                    .then(|| "task:write".into()),
                plan_digest: None,
                stale_when: vec![
                    "governance identity".into(),
                    "binding or task revision".into(),
                ],
            })
            .collect();
        self.findings.push(Finding {
            id: format!("{component}-{}", self.findings.len() + 1),
            component: component.into(),
            repository,
            binding_id,
            task_id,
            agent_id: None,
            status,
            required,
            evidence_level: None,
            code: code.into(),
            message: safe(message),
            expected: None,
            observed: None,
            sources: Vec::new(),
            blocked_by: Vec::new(),
            duration_ms: 0,
            actions,
        });
        self.findings.last_mut().expect("finding just appended")
    }

    fn result(
        &mut self,
        component: &str,
        result: Result<Value, String>,
        required: bool,
        pass_message: &str,
        action: (&str, &str, Effect),
        started: Instant,
    ) -> &mut Finding {
        let (status, reason) = match &result {
            Ok(value) => (
                match value["status"].as_str() {
                    Some("PASS") => Status::Pass,
                    Some("WARNING") => Status::Warning,
                    Some("BLOCKED") => Status::Blocked,
                    Some("NOT_RUN") => Status::NotRun,
                    Some("UNSUPPORTED") => Status::Unsupported,
                    Some("SKIPPED") => Status::Skipped,
                    Some("CANCELLED") => Status::Cancelled,
                    _ => Status::Error,
                },
                value["reason"].as_str().unwrap_or_else(|| {
                    if value["status"] == "PASS" {
                        pass_message
                    } else {
                        "Inspect the selected inputs and the next action."
                    }
                }),
            ),
            Err(reason) => (Status::Blocked, reason.as_str()),
        };
        let row = self.row(
            component,
            status,
            required,
            if status == Status::Pass {
                "VERIFIED"
            } else {
                "REVIEW_REQUIRED"
            },
            reason,
            (status != Status::Pass).then_some(action),
        );
        row.duration_ms = elapsed(started);
        row
    }

    fn inspect_authority(&mut self, context: &Context) {
        if self.includes("registry") {
            let row = self.row(
                "registry",
                Status::Pass,
                true,
                "REGISTRY_VALIDATED",
                "Registry schema, versions, IDs and asset graph validated.",
                None,
            );
            row.sources.push("governance.yaml".into());
            row.evidence_level = Some("STATIC_VALIDATED".into());
            row.observed = Some(
                json!({"revision":context.registry["governance"]["registry_revision"],"sha256":context.public()["policy_sha256"]}),
            );
        }
        if self.includes("governance") && !self.stop() {
            let started = Instant::now();
            let result = policy::validate(&context.root, &context.registry)
                .and_then(|()| docs::verify(&context.root))
                .map(|count| json!({"status":"PASS","paired_documents":count}));
            let failed = result.is_err();
            let pairs = result
                .as_ref()
                .ok()
                .map(|value| json!({"paired_documents":value["paired_documents"]}));
            let row = self.result(
                "governance",
                result,
                true,
                "Canonical assets, registry policy and bilingual documents validated.",
                (
                    "review-governance",
                    "Review the governance assets or translations.",
                    Effect::ReadOnly,
                ),
                started,
            );
            if failed {
                row.status = Status::Error;
                row.code = "GOVERNANCE_VALIDATION_FAILED".into();
            } else {
                row.evidence_level = Some("STATIC_VALIDATED".into());
                row.observed = pairs;
            }
        }
    }

    fn inspect(&mut self, context: &Context, resolved: &mut Value) {
        let scope = self.scope;
        self.inspect_authority(context);
        if self.stop() {
            return;
        }

        let task_record = scope
            .task
            .as_ref()
            .map(|id| tasks::load_current(&context.root, id));
        let selected = scope.repo.as_deref().or_else(|| {
            task_record
                .as_ref()
                .and_then(|record| record.as_ref().ok())
                .and_then(|record| record["descriptor"]["primary_repo"].as_str())
        });
        if let Some(repo) = selected {
            if let Err(reason) = context.repo(repo) {
                self.row(
                    "repository",
                    Status::Error,
                    true,
                    "UNKNOWN_REPOSITORY",
                    &reason,
                    None,
                );
                return;
            }
            resolved["repository"] = json!(label(Some(repo)));
            if task_record
                .as_ref()
                .and_then(|record| record.as_ref().ok())
                .is_some_and(|record| record["descriptor"]["primary_repo"] != repo)
            {
                self.row(
                    "task",
                    Status::Error,
                    true,
                    "TASK_REPOSITORY_MISMATCH",
                    "The selected task belongs to a different repository.",
                    None,
                );
                return;
            }
        } else if scope.binding.is_some() {
            self.row(
                "bindings",
                Status::Error,
                true,
                "REPOSITORY_REQUIRED",
                "A named binding requires --repo or a task.",
                None,
            );
        }

        let binding_needed = [
            "bindings",
            "routing",
            "task",
            "environment",
            "agent-config",
            "agent",
            "mcp",
        ]
        .iter()
        .any(|target| self.includes(target));
        let binding = if binding_needed {
            selected.map(|repo| context.binding(scope, repo))
        } else {
            None
        };
        if let Some(Ok((id, binding))) = &binding {
            resolved["binding"] =
                json!({"id":id,"identity":binding.identity,"commit":binding.commit});
        }
        if let Some(Ok(record)) = &task_record {
            resolved["task"] =
                json!({"revision":record["revision"],"approval_epoch":record["approval_epoch"]});
        }

        self.repositories(context, selected, binding.as_ref());
        self.assets(context, selected);
        self.task_checks(context, selected, task_record.as_ref(), binding.as_ref());
        self.routing(context, selected, binding.as_ref());
        self.environment(binding.as_ref());
        self.agents(context, selected, binding.as_ref());

        if let Some((id, Ok(task))) = scope.task.as_deref().zip(task_record.as_ref())
            && !self.stop()
            && !tasks::load_current(&context.root, id).is_ok_and(|current| {
                current["revision"] == task["revision"]
                    && current["approval_epoch"] == task["approval_epoch"]
            })
        {
            for row in self
                .findings
                .iter_mut()
                .filter(|row| row.task_id.as_deref() == Some(id) && row.status == Status::Pass)
            {
                row.status = Status::Blocked;
                row.evidence_level = None;
                row.code = "STALE_TASK".into();
                row.message = "The task changed during inspection.".into();
            }
            self.row(
                "task",
                Status::Blocked,
                true,
                "STALE_TASK",
                "The task changed during inspection. Run the check again.",
                None,
            );
        }

        if let (Some(repo), Some(Ok((id, before)))) = (selected, binding.as_ref())
            && !self.stop()
            && !context
                .binding(scope, repo)
                .is_ok_and(|(current_id, current)| {
                    current_id == *id
                        && current.identity == before.identity
                        && current.commit == before.commit
                        && current.path == before.path
                })
        {
            for row in self
                .findings
                .iter_mut()
                .filter(|row| row.repository.as_deref() == Some(repo) && row.status == Status::Pass)
            {
                row.status = Status::Blocked;
                row.evidence_level = None;
                row.code = "STALE_BINDING".into();
                row.message = "The worktree changed during inspection.".into();
            }
            self.row(
                "bindings",
                Status::Blocked,
                true,
                "STALE_BINDING",
                "The worktree changed during inspection. Run the check again.",
                None,
            );
        }
    }

    fn repositories(
        &mut self,
        context: &Context,
        selected: Option<&str>,
        binding: Option<&BindingResult>,
    ) {
        if self.stop() || (!self.includes("repositories") && !self.includes("bindings")) {
            return;
        }
        let discovery = context.arguments().and_then(|args| {
            workspace::run(
                "workspace-discover",
                &context.root,
                &args,
                &context.registry,
            )
        });
        if self.includes("repositories") {
            let candidates = discovery
                .as_ref()
                .ok()
                .map(|report| report["candidates"].clone());
            let row = self.result(
                "repositories",
                discovery.clone(),
                true,
                "Workspace candidate discovery completed. Candidates are untrusted until bound.",
                (
                    "discover",
                    "Review the workspace and candidate worktrees.",
                    Effect::ReadOnly,
                ),
                Instant::now(),
            );
            row.observed = Some(
                json!({"candidates":candidates.as_ref().and_then(Value::as_array).map(Vec::len)}),
            );
        }
        let repositories = context.registry["repositories"]
            .as_object()
            .expect("validated repository registry");
        for repo in repositories.keys().filter(|repo| {
            selected.map_or_else(
                || repositories[*repo]["classification"] == "active",
                |selected| *repo == selected,
            )
        }) {
            if self.stop() {
                return;
            }
            if self.includes("repositories") {
                let candidate = discovery
                    .as_ref()
                    .ok()
                    .and_then(|report| report["candidates"].as_array())
                    .is_some_and(|rows| {
                        rows.iter().any(|row| row["repo_id_hint"] == repo.as_str())
                    });
                let state = if candidate {
                    Status::Pass
                } else {
                    Status::NotRun
                };
                let row = self.row(
                    "repositories",
                    state,
                    false,
                    if candidate {
                        "CANDIDATE_FOUND"
                    } else {
                        "CANDIDATE_NOT_FOUND"
                    },
                    if candidate {
                        "A candidate Git marker was found. Binding validates identity."
                    } else {
                        "No same-name checkout candidate was discovered."
                    },
                    None,
                );
                row.repository = Some(repo.clone());
                row.observed = Some(json!({"candidate_found":candidate,"trusted":false}));
            }
            if !self.includes("bindings") {
                continue;
            }
            let automatic;
            let outcome = if let Some(binding) = binding {
                binding
            } else {
                automatic = context.binding(&Scope::default(), repo);
                &automatic
            };
            let options = context.bindings(repo).unwrap_or_default();
            let row = self.result(
                "bindings",
                binding_value(outcome),
                selected.is_some() || self.target == "bindings",
                "Confirmed worktree identity and anchor validated.",
                (
                    "select-binding",
                    "Select an existing binding or review a new binding in Agent setup.",
                    Effect::ReadOnly,
                ),
                Instant::now(),
            );
            row.repository = Some(repo.clone());
            row.observed = Some(
                json!({"bindings":options.iter().map(|id| label(Some(id))).collect::<Vec<_>>() }),
            );
            if let Ok((id, binding)) = outcome {
                row.binding_id = label(Some(id));
                row.observed.as_mut().expect("binding observation")["identity"] =
                    json!(binding.identity);
            }
        }
    }

    fn assets(&mut self, context: &Context, selected: Option<&str>) {
        if self.stop() || !ASSETS.iter().any(|(target, _)| self.includes(target)) {
            return;
        }
        let result = authority::resolve_for_export(
            &context.root,
            None,
            &context.registry,
            selected.unwrap_or("cirnet-rules"),
            self.scope.task.as_deref(),
            None,
        );
        for (component, kind) in ASSETS
            .iter()
            .filter(|(target, _)| run_selects(self.target, target))
        {
            let started = Instant::now();
            let inspected = result
                .as_ref()
                .map(|resolved| {
                    let assets: Vec<_> = resolved
                        .included
                        .iter()
                        .chain(&resolved.manual)
                        .filter_map(|source| {
                            let mut value = serde_json::to_value(source).ok()?;
                            if value["kind"] != *kind {
                                return None;
                            }
                            value.as_object_mut()?.retain(|key, _| {
                                matches!(key.as_str(), "id" | "path" | "revision" | "sha256")
                            });
                            Some(value)
                        })
                        .collect();
                    json!({"status":"PASS","assets":assets})
                })
                .map_err(Clone::clone);
            let assets = inspected.as_ref().ok().map(|value| value["assets"].clone());
            let row = self.result(component, inspected, true, match *component {
                "hooks" => "Selected hook sources validated. Exported hooks are inert.",
                "profiles" => "Selected profile sources validated for context use.",
                "skills" => "Selected skill sources validated. Native discovery requires separate evidence.",
                _ => "Selected authority sources and dependency hashes validated.",
            }, ("review-assets", "Review authority selection and declared source files.", Effect::ReadOnly), started);
            if let Some(assets) = assets {
                row.evidence_level = Some("STATIC_VALIDATED".into());
                row.observed = Some(json!({"assets":assets}));
            }
        }
    }

    fn task_checks(
        &mut self,
        context: &Context,
        repo: Option<&str>,
        task: Option<&Result<Value, String>>,
        binding: Option<&BindingResult>,
    ) {
        if self.stop() || !TASK_CHECKS.iter().any(|target| self.includes(target)) {
            return;
        }
        let need_snapshot = self.includes("diff") || self.includes("verification");
        let snapshot = match task {
            Some(Ok(record)) if need_snapshot && !record["baseline"].is_null() => {
                Some(tasks::authorize(record, "read").and_then(|()| {
                    runner::snapshot(
                        &context.root,
                        record,
                        &context.registry,
                        Some(&context.workspace),
                    )
                }))
            }
            _ => None,
        };
        for &component in TASK_CHECKS {
            if !self.includes(component) || self.stop() {
                continue;
            }
            let started = Instant::now();
            let required = self.target == component || task.is_some();
            let Some((task_id, task)) = self.scope.task.as_deref().zip(task) else {
                self.row(
                    component,
                    Status::NotRun,
                    required,
                    "TASK_NOT_SELECTED",
                    "No task selected.",
                    Some((
                        "select-task",
                        "Select an existing task with --task.",
                        Effect::ReadOnly,
                    )),
                );
                continue;
            };
            let task = match task {
                Ok(task) => task,
                Err(reason) => {
                    let row = self.row(
                        component,
                        Status::Blocked,
                        required,
                        "TASK_UNAVAILABLE",
                        reason,
                        None,
                    );
                    if component != "task" {
                        row.blocked_by.push("task".into());
                    }
                    continue;
                }
            };
            let outcome = match component {
                "task" => tasks::ready(&context.root, task, &context.registry)
                    .and_then(|()| coordination::check_run(&context.root, task, false))
                    .and_then(|()| binding.map_or(Ok(()), |binding| binding.as_ref().map(|_| ()).map_err(Clone::clone)))
                    .map(|()| json!({"status":"PASS","state":task["state"],"revision":task["revision"]})),
                "diff" => snapshot.clone().unwrap_or_else(|| Ok(json!({"status":"NOT_RUN","reason":"The task has no approved start snapshot."}))),
                "verification" => match snapshot.as_ref() {
                    Some(Ok(snapshot)) if lifecycle::fresh(task, snapshot) => Ok(json!({"status":"PASS"})),
                    Some(Err(reason)) => Err(reason.clone()),
                    _ => Ok(json!({"status":if task["evidence"].as_array().is_some_and(|rows| !rows.is_empty()) {"BLOCKED"} else {"NOT_RUN"},
                        "reason":"A fresh passing receipt for the current approved diff and inputs is unavailable."})),
                },
                "audit" => audit::task(&context.root, task_id),
                _ => unreachable!(),
            };
            let observed = outcome.as_ref().ok().map(|value| match component {
                "task" => json!({"state":value["state"],"revision":value["revision"],"approval_epoch":task["approval_epoch"]}),
                "audit" => json!({"events":value["events"].as_array().map(Vec::len),"authority":value["authority"]}),
                "diff" => json!({"violations":value["violations"].as_array().map(Vec::len),"snapshot_sha256":value["sha256"]}),
                _ => Value::Null,
            });
            let row = self.result(component, outcome, required, match component {
                "task" => "Task authority, binding, grant, dependencies and coordinated ownership checked.",
                "diff" => "Current changes match the approved scope and start snapshot.",
                "verification" => "A fresh passing receipt matches the approved task and current inputs.",
                _ => "Receipt and event integrity checked. Local actors are assertions.",
            }, ("review-task", "Review the task, binding and recorded evidence.", Effect::ReadOnly), started);
            row.repository = label(repo);
            row.observed = observed;
        }
    }

    fn routing(&mut self, context: &Context, repo: Option<&str>, binding: Option<&BindingResult>) {
        if !self.includes("routing") || self.stop() {
            return;
        }
        let started = Instant::now();
        if let Some(id) = self.scope.task.as_ref() {
            let outcome = context.arguments().and_then(|mut args| {
                args.insert("task".into(), id.clone());
                workspace::run("route", &context.root, &args, &context.registry)
            });
            self.result(
                "routing",
                outcome,
                true,
                "Scoped task route and its worktree bindings checked.",
                (
                    "review-route",
                    "Review task scope and registered module routes.",
                    Effect::ReadOnly,
                ),
                started,
            );
        } else if let Some((repo, entry)) =
            repo.and_then(|repo| context.repo(repo).ok().map(|entry| (repo, entry)))
        {
            let detailed = context.registry["control_plane"]["routes"][repo]
                .as_array()
                .is_some_and(|rows| !rows.is_empty());
            self.row("routing", Status::Warning, false, "ROUTE_COVERAGE", if detailed {
                "Repository ownership and module routes are declared. Select a task for exact scoped routing."
            } else {
                "Repository ownership is declared. Detailed task routing was not established."
            }, Some(("select-task", "Select a task to check exact scoped routing.", Effect::ReadOnly)))
                .observed = Some(json!({"classification":entry["classification"],"dependencies":entry["depends_on"].as_array().map(Vec::len),"binding_resolved":matches!(binding, Some(Ok(_)))}));
        } else {
            self.row(
                "routing",
                Status::NotRun,
                self.target == "routing",
                "SCOPE_NOT_SELECTED",
                "Select a repository or task to inspect routing.",
                None,
            );
        }
    }

    fn environment(&mut self, binding: Option<&BindingResult>) {
        if !self.includes("environment") || self.stop() {
            return;
        }
        let started = Instant::now();
        let Some(binding) = binding else {
            self.row(
                "environment",
                Status::NotRun,
                self.target == "environment",
                "REPOSITORY_NOT_SELECTED",
                "No bound repository was selected for native pin and lock inspection.",
                None,
            );
            return;
        };
        let result = binding
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|(_, binding)| {
                let project = environment::Project::open(&binding.path)?;
                environment::dispatch(&project, &["env".into(), "plan".into()], false, true)
            });
        let metadata = result.as_ref().ok().map(|value| json!({"lock":value["lock"],"unresolved_count":value["unresolved"].as_array().map(Vec::len),
            "host":std::env::consts::OS}));
        let drift = metadata.as_ref().is_some_and(|value| {
            value["lock"] == "STALE" || value["unresolved_count"].as_u64().unwrap_or(0) > 0
        });
        let row = self.result(
            "environment",
            result,
            true,
            "Native requirements and environment lock inspected.",
            (
                "review-environment",
                "Review the environment plan and pinned versions.",
                Effect::ReadOnly,
            ),
            started,
        );
        if drift {
            row.status = Status::Warning;
            row.code = "ENVIRONMENT_PIN_DRIFT".into();
            row.message =
                "Native version pins are unresolved or the environment lock is stale.".into();
        }
        row.observed = metadata;
    }

    fn agents(&mut self, context: &Context, repo: Option<&str>, binding: Option<&BindingResult>) {
        if self.stop()
            || !["agent-config", "agent", "mcp"]
                .iter()
                .any(|id| self.includes(id))
        {
            return;
        }
        let scope = self.scope;
        if self.target == "agent" && scope.agent.is_none() {
            self.row(
                "agent-selection",
                Status::NotRun,
                true,
                "AGENT_NOT_SELECTED",
                "Select one Agent or request all three with --agent all.",
                None,
            );
        }
        let agents = match scope.agent.as_deref() {
            Some(agent) if agent != "all" => vec![agent],
            Some("all") => AGENTS.to_vec(),
            _ => AGENTS.to_vec(),
        };
        let bundle = scope.bundle.as_deref().map(|path| {
            context.arguments().and_then(|mut args| {
                args.insert("bundle".into(), super::context::path_text(path)?);
                if let Some(Ok((id, _))) = binding {
                    args.insert("binding".into(), id.clone());
                }
                agent_config::run(&context.root, "check", &args)
            })
        });
        let sync = bundle
            .as_ref()
            .and_then(|bundle| bundle.as_ref().ok())
            .zip(binding.and_then(|value| value.as_ref().ok()))
            .filter(|(bundle, _)| repo.is_some_and(|repo| bundle["manifest"]["repository"] == repo))
            .map(|(_, (id, binding))| {
                context.arguments().and_then(|mut args| {
                    let bundle = scope.bundle.as_deref().ok_or("Agent bundle is required")?;
                    args.insert("bundle".into(), super::context::path_text(bundle)?);
                    args.insert("target".into(), super::context::path_text(&binding.path)?);
                    args.insert("binding".into(), id.clone());
                    args.insert("entry-mode".into(), "managed-block".into());
                    args.insert("dry-run".into(), "true".into());
                    if let Some(task) = &scope.task {
                        args.insert("task".into(), task.clone());
                    }
                    agent_config::run(&context.root, "sync", &args)
                })
            });
        let includes_config =
            self.includes("agent-config") || (self.includes("agent") && scope.bundle.is_some());
        let includes_mcp = self.includes("mcp");
        if includes_mcp {
            self.mcp_protocol(context, repo);
        }
        for agent in agents {
            if self.stop() {
                return;
            }
            if includes_config {
                let required = self.target == "agent-config" || scope.bundle.is_some();
                let action = (
                    "choose-bundle",
                    "Select an exported instruction bundle and confirmed target in Agent setup.",
                    Effect::ReadOnly,
                );
                let row = match bundle.as_ref() {
                    None => self.row(
                        "agent-config",
                        Status::NotRun,
                        required,
                        "BUNDLE_NOT_SELECTED",
                        "No instruction bundle selected.",
                        Some(action),
                    ),
                    Some(Err(reason)) => self.row(
                        "agent-config",
                        Status::Blocked,
                        required,
                        "BUNDLE_INVALID_OR_STALE",
                        reason,
                        Some(action),
                    ),
                    Some(Ok(bundle))
                        if !bundle["manifest"]["agents"]
                            .as_array()
                            .is_some_and(|agents| agents.iter().any(|id| id == agent)) =>
                    {
                        self.row(
                            "agent-config",
                            Status::Skipped,
                            scope.agent.as_deref() == Some(agent),
                            "AGENT_NOT_IN_BUNDLE",
                            "The selected bundle does not include this Agent.",
                            Some(action),
                        )
                    }
                    Some(Ok(bundle)) => {
                        let row = self.row("agent-config", Status::Pass, required, "STATIC_CONFIG_VALIDATED", "Bundle manifest, current authority and selected file digests validated.", None);
                        row.evidence_level = Some("STATIC_VALIDATED".into());
                        row.observed = Some(
                            json!({"manifest_sha256":bundle["manifest_sha256"],"selected_outputs":bundle["manifest"]["outputs"].as_array().map(Vec::len)}),
                        );
                        row
                    }
                };
                row.agent_id = Some(agent.into());
                row.repository = label(repo);
                if row.status == Status::Pass {
                    let row = if let Some(sync) = &sync {
                        let clean = sync.as_ref().ok().is_some_and(|plan| {
                            plan["status"] == "PASS" && plan["state"] == "UNCHANGED"
                        });
                        let status = if clean { Status::Pass } else { Status::Blocked };
                        let reason = sync.as_ref().err().map(String::as_str).unwrap_or(if clean {
                            "Target bytes match the reviewed sync plan."
                        } else {
                            "Target differs or has an ownership conflict. Review the sync plan."
                        });
                        let row = self.row(
                            "agent-target",
                            status,
                            required,
                            if clean {
                                "TARGET_CURRENT"
                            } else {
                                "SYNC_REVIEW_REQUIRED"
                            },
                            reason,
                            (!clean).then_some((
                                "preview-sync",
                                "Review the per-file sync plan.",
                                Effect::ReadOnly,
                            )),
                        );
                        if clean {
                            row.evidence_level = Some("STATIC_VALIDATED".into());
                        }
                        row
                    } else {
                        self.row(
                            "agent-target",
                            Status::Blocked,
                            required,
                            "TARGET_UNRESOLVED",
                            "Select the bundle's repository and exact binding.",
                            Some(action),
                        )
                    };
                    row.agent_id = Some(agent.into());
                    row.repository = label(repo);
                }
            }
            if self.includes("agent") || includes_mcp {
                let row = self.row(
                    "native-client",
                    Status::NotRun,
                    false,
                    "NATIVE_CLIENT_NOT_OBSERVED",
                    "No Agent-specific native discovery or MCP session was observed.",
                    Some((
                        "native-handoff",
                        "Verify in the selected native Agent at the confirmed target.",
                        Effect::ReadOnly,
                    )),
                );
                row.agent_id = Some(agent.into());
                row.repository = label(repo);
            }
            if self.includes("agent") {
                let row = self.row("agent", Status::NotRun, false, "CLIENT_VERSION_NOT_OBSERVED",
                    "PATH and platform metadata were inspected. Native client version and effective trust require operator observation.", None);
                row.agent_id = Some(agent.into());
                row.repository = label(repo);
                row.observed = Some(client_metadata(agent));
            }
            if self.includes("agent") || includes_mcp {
                self.registration(context, repo, binding, agent);
            }
        }
    }

    fn mcp_protocol(&mut self, context: &Context, repo: Option<&str>) {
        if self.stop() {
            return;
        }
        let started = Instant::now();
        let required = self.target == "mcp";
        let action = (
            "probe-mcp",
            "Review the executable and authority, then rerun the MCP check.",
            Effect::ReadOnly,
        );
        if let Some(bundle) = self.scope.mcp_bundle.as_deref() {
            let checked = agent_config::mcp_config::check_probe_bundle(
                &context.root,
                &context.workspace,
                bundle,
            );
            let row = self.result(
                "mcp-launch-config",
                checked,
                true,
                "MCP launch templates and executable digest validated.",
                action,
                started,
            );
            if row.status != Status::Pass {
                self.row(
                    "mcp-protocol",
                    Status::Blocked,
                    required,
                    "CONFIG_DRIFT",
                    "Review the selected launch bundle before probing.",
                    Some(action),
                )
                .blocked_by
                .push("mcp-launch-config".into());
                return;
            }
            row.evidence_level = Some("STATIC_VALIDATED".into());
        } else {
            self.row(
                "mcp-launch-config",
                Status::NotRun,
                false,
                "MCP_BUNDLE_NOT_SELECTED",
                "No exported launch bundle selected; protocol uses the current CNR executable.",
                None,
            );
        }
        let result = crate::mcp::probe_with_cancel(
            &context.root,
            &context.workspace,
            repo,
            self.scope.task.as_deref(),
            self.cancelled,
        );
        let row = match result {
            Ok(observed) => {
                let row = self.row(
                    "mcp-protocol",
                    Status::Pass,
                    required,
                    "MCP_PROTOCOL_VERIFIED",
                    "Current CNR server completed the read-only STDIO protocol probe.",
                    None,
                );
                row.evidence_level = Some("MCP_PROTOCOL_VERIFIED".into());
                row.observed = Some(observed);
                row
            }
            Err(failure) => {
                let status = match failure.code {
                    "CANCELLED" => Status::Cancelled,
                    "PROTOCOL_ERROR" | "TOOL_SCHEMA_MISMATCH" | "TIMEOUT" => Status::Error,
                    _ if required => Status::Blocked,
                    _ => Status::Warning,
                };
                self.row(
                    "mcp-protocol",
                    status,
                    required,
                    failure.code,
                    failure.message,
                    Some(action),
                )
            }
        };
        row.duration_ms = elapsed(started);
        row.repository = label(repo);
    }

    fn registration(
        &mut self,
        context: &Context,
        repo: Option<&str>,
        binding: Option<&BindingResult>,
        agent: &str,
    ) {
        let started = Instant::now();
        let action = (
            "preview-mcp-registration",
            "Review the scoped entry or manual import in Agent setup.",
            Effect::ReadOnly,
        );
        let row = if let Some((repo, (id, binding))) =
            repo.zip(binding.and_then(|binding| binding.as_ref().ok()))
        {
            let result = context.arguments().and_then(|mut args| {
                args.extend([
                    ("repo".into(), repo.into()),
                    ("binding".into(), id.clone()),
                    ("target".into(), super::context::path_text(&binding.path)?),
                    ("agent".into(), agent.into()),
                    ("scope".into(), "project".into()),
                ]);
                if let Some(task) = &self.scope.task {
                    args.insert("task".into(), task.clone());
                }
                if let Some(surface) = &self.scope.surface {
                    args.insert("surface".into(), surface.clone());
                }
                if let Some(bundle) = &self.scope.mcp_bundle {
                    args.insert("bundle".into(), super::context::path_text(bundle)?);
                }
                agent_config::mcp_config::run(&context.root, "inspect", &args)
            });
            let unsupported = result
                .as_ref()
                .is_ok_and(|report| report["registration"] == "UNSUPPORTED");
            let row = self.result(
                "mcp-registration",
                result,
                self.target == "agent" && self.scope.agent.is_some(),
                "The scoped CNR MCP entry matches its expected launch settings.",
                action,
                started,
            );
            if unsupported {
                row.status = Status::Unsupported;
            }
            row
        } else {
            let blocked = binding.is_some_and(Result::is_err);
            self.row(
                "mcp-registration",
                if blocked {
                    Status::Blocked
                } else {
                    Status::NotRun
                },
                self.target == "agent" && self.scope.agent.is_some(),
                "TARGET_UNRESOLVED",
                "Select a confirmed repository binding to inspect project registration.",
                Some(action),
            )
        };
        if row.status == Status::Pass {
            row.evidence_level = Some("CONFIG_REGISTERED".into());
        }
        row.agent_id = Some(agent.into());
        row.repository = label(repo);
    }
}

type BindingResult = Result<(String, workspace::Binding), String>;

fn binding_value(binding: &BindingResult) -> Result<Value, String> {
    binding
        .as_ref()
        .map(|_| json!({"status":"PASS"}))
        .map_err(Clone::clone)
}

fn client_metadata(agent: &str) -> Value {
    let program = format!("{agent}{}", std::env::consts::EXE_SUFFIX);
    let paths = std::env::var_os("PATH");
    let paths = paths
        .as_deref()
        .into_iter()
        .flat_map(std::env::split_paths)
        .filter(|path| path.is_absolute())
        .take(64);
    let mut binary = false;
    let mut shim = false;
    for path in paths {
        binary |= std::fs::symlink_metadata(path.join(&program))
            .is_ok_and(|file| file.is_file() && !policy::is_link(&file));
        if cfg!(windows) {
            shim |= ["cmd", "bat", "ps1"]
                .iter()
                .any(|extension| path.join(format!("{agent}.{extension}")).is_file());
        }
    }
    let custom_config = match agent {
        "codex" => std::env::var_os("CODEX_HOME").is_some(),
        "claude" => std::env::var_os("CLAUDE_CONFIG_DIR").is_some(),
        _ => false,
    };
    json!({"host":std::env::consts::OS,"wsl_environment":std::env::var_os("WSL_DISTRO_NAME").is_some(),
        "path_binary_candidate":binary,"path_shim_candidate":shim,"custom_config_environment":custom_config})
}
