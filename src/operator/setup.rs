use super::{
    AGENTS, Scope,
    context::{Context, path_text},
    safe,
};
use crate::{agent_config, docs, runner, tasks, workspace};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SetupAction {
    Bind,
    ExportInstructions,
    SyncStrict,
    SyncManaged,
    ExportMcp,
    RegisterMcp,
    RollbackSync,
    RollbackMcp,
    ProposeTask,
}

#[derive(Clone, Debug, Default)]
pub struct SetupInput {
    pub scope: Scope,
    pub actor: Option<String>,
    pub profile: Option<String>,
    pub receipt: Option<String>,
    pub replace_binding: Option<String>,
    pub proposal: Option<String>,
    pub spec: Option<String>,
    pub plan: Option<String>,
}

pub struct SetupPreview {
    pub action: SetupAction,
    pub report: Value,
    input: SetupInput,
    context: Context,
    digest: String,
}

pub fn inspect_setup(scope: &Scope) -> Result<Value, String> {
    let context = Context::open(scope)?;
    let discovery = workspace::run(
        "workspace-discover",
        &context.root,
        &context.arguments()?,
        &context.registry,
    )?;
    let repositories: Vec<_> = context.registry["repositories"]
        .as_object()
        .into_iter()
        .flat_map(|map| map.keys().filter(|repo| context.repo(repo).is_ok()))
        .collect();
    let binding = scope.repo.as_deref().map(|repo| {
        context.binding(scope, repo).and_then(|(id, binding)| Ok(json!({
            "id":id,"target":path_text(&binding.path)?,"identity":binding.identity,"commit":binding.commit
        })))
    });
    let bindings = scope
        .repo
        .as_deref()
        .map(|repo| context.bindings(repo))
        .transpose()?
        .unwrap_or_default();
    let task = scope.task.as_deref().map(|id| {
        tasks::load_current(&context.root, id)
            .map(|record| json!({"id":id,"state":record["state"],"revision":record["revision"]}))
    });
    context.fresh()?;
    Ok(json!({
        "schema_version":1, "status":"PASS",
        "governance_root":path_text(&context.root)?, "governance_head":context.head,
        "workspace":path_text(&context.workspace)?, "repositories":repositories,
        "repo":scope.repo, "agent":scope.agent, "agents":AGENTS,
        "binding":binding.as_ref().and_then(|result| result.as_ref().ok()),
        "binding_reason":binding.as_ref().and_then(|result| result.as_ref().err()).map(|reason| safe(reason)),
        "bindings":bindings,
        "candidates":discovery["candidates"].as_array().into_iter().flatten().map(|row| json!({
            "path":row["path"].as_str().map(safe),"repo_hint":row["repo_id_hint"].as_str().map(safe),"trusted":false
        })).collect::<Vec<_>>(),
        "task":task.as_ref().and_then(|result| result.as_ref().ok()),
        "task_reason":task.as_ref().and_then(|result| result.as_ref().err()).map(|reason| safe(reason)),
        "native_client":"NOT_RUN"
    }))
}

pub fn preview_setup(action: SetupAction, input: &SetupInput) -> Result<SetupPreview, String> {
    let context = Context::open(&input.scope)?;
    let mut report = execute(&context, input, action, None, false)?;
    context.fresh()?;
    if matches!(
        action,
        SetupAction::SyncStrict | SetupAction::SyncManaged | SetupAction::RegisterMcp
    ) && report["status"] == "PASS"
        && report["state"] != "UNCHANGED"
        && report["registration"] != "UNCHANGED"
    {
        match sync_authority(&context, input, action, &report) {
            Ok(()) => {
                report["authorization"] = json!("VALIDATED");
            }
            Err(reason) => {
                report["status"] = json!("BLOCKED");
                report["authorization"] = json!("APPROVAL_REQUIRED");
                report["reason"] = json!(safe(&reason));
            }
        }
    }
    let digest = report["plan_sha256"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    Ok(SetupPreview {
        action,
        report,
        input: input.clone(),
        context,
        digest,
    })
}

pub fn perform_setup(preview: SetupPreview, trust_reviewed: bool) -> Result<Value, String> {
    if preview.report["status"] != "PASS" || preview.digest.is_empty() {
        return Err(
            "A fresh, passing preview is required. Review the next action before applying.".into(),
        );
    }
    preview.context.fresh()?;
    execute(
        &preview.context,
        &preview.input,
        preview.action,
        Some(&preview.digest),
        trust_reviewed,
    )
}

fn execute(
    context: &Context,
    input: &SetupInput,
    action: SetupAction,
    expect: Option<&str>,
    trust_reviewed: bool,
) -> Result<Value, String> {
    use SetupAction::*;
    let scope = &input.scope;
    let repo = scope.repo.as_deref().ok_or("Select a repository first.")?;
    context.repo(repo)?;
    let agent = scope
        .agent
        .as_deref()
        .filter(|id| AGENTS.contains(id))
        .ok_or("Select one Agent first.")?;
    let mut args = context.arguments()?;

    match action {
        Bind => {
            if scope.binding.as_deref().is_some_and(|id| id != "default") {
                return Err("Use the default binding for new guided bindings or select an existing named binding.".into());
            }
            let target = scope
                .target
                .as_deref()
                .ok_or("Enter the candidate worktree path.")?;
            let target = if target.is_absolute() {
                let canonical = workspace::workspace_path(&context.root, Some(target))?;
                path_text(
                    canonical
                        .strip_prefix(&context.workspace)
                        .map_err(|_| "Worktree is outside the selected workspace.")?,
                )?
            } else {
                path_text(target)?
            };
            let relative = target.replace('\\', "/");
            let actor = input
                .actor
                .as_deref()
                .ok_or("Enter the operator label for binding.")?;
            args.extend([
                ("repo".into(), repo.into()),
                ("path".into(), relative),
                ("actor".into(), actor.into()),
            ]);
            if let Some(replace) = &input.replace_binding {
                args.insert("replace".into(), replace.clone());
            } else {
                let mut lookup = context.arguments()?;
                lookup.extend([
                    ("repo".into(), repo.into()),
                    ("binding".into(), "default".into()),
                ]);
                if let Ok(previous) = workspace::run(
                    "workspace-binding-show",
                    &context.root,
                    &lookup,
                    &context.registry,
                ) && let Some(identity) = previous["binding"]["identity"].as_str()
                {
                    args.insert("replace".into(), identity.into());
                }
            }
            let command = if let Some(expect) = expect {
                args.insert("expect".into(), expect.into());
                "workspace-bind-reviewed"
            } else {
                "workspace-bind-preview"
            };
            workspace::run(command, &context.root, &args, &context.registry)
        }
        ExportInstructions => {
            let (id, _) = context.binding(scope, repo)?;
            let output = scope.bundle.as_deref().ok_or(
                "Enter the instruction bundle directory outside Git and Agent configuration.",
            )?;
            args.extend([
                ("repo".into(), repo.into()),
                ("agents".into(), agent.into()),
                ("binding".into(), id),
                ("output".into(), path_text(output)?),
            ]);
            if let Some(task) = &scope.task {
                args.insert("task".into(), task.clone());
            }
            if let Some(profile) = &input.profile {
                args.insert("profile".into(), profile.clone());
            }
            let command = if let Some(expect) = expect {
                args.insert("expect".into(), expect.into());
                "export"
            } else {
                "preview-export"
            };
            agent_config::run(&context.root, command, &args)
        }
        SyncStrict | SyncManaged | RollbackSync => {
            let (id, binding) = context.binding(scope, repo)?;
            args.extend([
                ("target".into(), path_text(&binding.path)?),
                ("binding".into(), id.clone()),
            ]);
            set_optional_authority(input, &mut args);
            set_apply_mode(&mut args, expect);
            if action == RollbackSync {
                args.insert(
                    "receipt".into(),
                    input.receipt.clone().ok_or("Enter the sync receipt ID.")?,
                );
                return agent_config::run(&context.root, "rollback", &args);
            }
            let bundle = scope
                .bundle
                .as_deref()
                .ok_or("Select the instruction bundle first.")?;
            args.insert("bundle".into(), path_text(bundle)?);
            args.insert(
                "entry-mode".into(),
                if action == SyncManaged {
                    "managed-block"
                } else {
                    "strict"
                }
                .into(),
            );
            args.insert("keep-unselected".into(), "true".into());
            let checked = agent_config::run(
                &context.root,
                "check",
                &BTreeMap::from([
                    ("bundle".into(), path_text(bundle)?),
                    ("workspace".into(), path_text(&context.workspace)?),
                    ("binding".into(), id),
                ]),
            )?;
            if checked["manifest"]["repository"] != repo
                || !checked["manifest"]["agents"]
                    .as_array()
                    .is_some_and(|agents| agents.iter().any(|id| id == agent))
            {
                return Err("The bundle belongs to a different repository or Agent. Select a matching bundle.".into());
            }
            agent_config::run(&context.root, "sync", &args)
        }
        ExportMcp => {
            let output = scope
                .mcp_bundle
                .as_deref()
                .ok_or("Enter the MCP bundle directory outside Git and Agent configuration.")?;
            let binary =
                std::env::current_exe().map_err(|_| "Current CNR executable is unavailable.")?;
            args.extend([
                ("output".into(), path_text(output)?),
                ("binary".into(), path_text(&binary)?),
            ]);
            let command = if let Some(expect) = expect {
                args.insert("expect".into(), expect.into());
                "export"
            } else {
                "preview-export"
            };
            agent_config::mcp_config::run(&context.root, command, &args)
        }
        RegisterMcp | RollbackMcp => {
            let (id, binding) = context.binding(scope, repo)?;
            args.extend([
                ("target".into(), path_text(&binding.path)?),
                ("binding".into(), id),
                ("repo".into(), repo.into()),
                ("agent".into(), agent.into()),
                ("scope".into(), "project".into()),
            ]);
            if let Some(surface) = &scope.surface {
                args.insert("surface".into(), surface.clone());
            }
            set_optional_authority(input, &mut args);
            set_apply_mode(&mut args, expect);
            if trust_reviewed {
                args.insert("trust-reviewed".into(), "true".into());
            }
            if action == RollbackMcp {
                args.insert(
                    "receipt".into(),
                    input
                        .receipt
                        .clone()
                        .ok_or("Enter the MCP registration receipt ID.")?,
                );
                return agent_config::mcp_config::run(&context.root, "rollback", &args);
            }
            let bundle = scope
                .mcp_bundle
                .as_deref()
                .ok_or("Select the MCP launch bundle first.")?;
            args.insert("bundle".into(), path_text(bundle)?);
            agent_config::mcp_config::run(&context.root, "install", &args)
        }
        ProposeTask => propose(context, input, repo, agent, expect),
    }
}

fn set_optional_authority(input: &SetupInput, args: &mut BTreeMap<String, String>) {
    if let Some(task) = &input.scope.task {
        args.insert("task".into(), task.clone());
    }
    if let Some(actor) = &input.actor {
        args.insert("actor".into(), actor.clone());
    }
}

fn set_apply_mode(args: &mut BTreeMap<String, String>, expect: Option<&str>) {
    if let Some(expect) = expect {
        args.extend([
            ("yes".into(), "true".into()),
            ("expect".into(), expect.into()),
        ]);
    } else {
        args.insert("dry-run".into(), "true".into());
    }
}

fn sync_authority(
    context: &Context,
    input: &SetupInput,
    action: SetupAction,
    report: &Value,
) -> Result<(), String> {
    let repo = input.scope.repo.as_deref().ok_or("Select a repository.")?;
    let task = input
        .scope
        .task
        .as_deref()
        .ok_or("Choose a current approved task, or preview a task proposal.")?;
    let actor = input
        .actor
        .as_deref()
        .ok_or("Enter the declared operator label for this grant.")?;
    let (_, binding) = context.binding(&input.scope, repo)?;
    let paths: Vec<_> = if action == SetupAction::RegisterMcp {
        report["file"].as_str().into_iter().collect()
    } else {
        report["files"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|row| row["action"] != "UNCHANGED")
            .filter_map(|row| row["path"].as_str())
            .collect()
    };
    for path in paths {
        let mut args = context.arguments()?;
        args.extend([
            ("task".into(), task.into()),
            ("actor".into(), actor.into()),
            ("repo".into(), repo.into()),
            ("action".into(), "write".into()),
            ("path".into(), path.into()),
        ]);
        let outcome = runner::run("grant-evaluate", &context.root, &args, &context.registry)?;
        if outcome["status"] != "PASS" || outcome["worktree_identity"] != binding.identity {
            return Err(format!(
                "Write grant blocked for {path}. Review the task's exact scope, revision and actor."
            ));
        }
    }
    Ok(())
}

fn propose(
    context: &Context,
    input: &SetupInput,
    repo: &str,
    agent: &str,
    expect: Option<&str>,
) -> Result<Value, String> {
    let mut scope = input.scope.clone();
    scope.task = None;
    let (binding, _) = context.binding(&scope, repo)?;
    let id = input
        .proposal
        .as_deref()
        .ok_or("Enter a new proposal ID.")?;
    let actor = input
        .actor
        .as_deref()
        .ok_or("Enter the proposed authorization owner and operator label.")?;
    let spec = input
        .spec
        .as_deref()
        .ok_or("Enter the reviewed central English spec path for this repository.")?;
    let plan = input
        .plan
        .as_deref()
        .ok_or("Enter the reviewed central English plan path for this repository.")?;
    let bundle = scope
        .bundle
        .as_deref()
        .ok_or("Select an exported bundle to enumerate exact proposed write paths.")?;
    let mut args = context.arguments()?;
    args.insert("bundle".into(), path_text(bundle)?);
    args.insert("binding".into(), binding.clone());
    let checked = agent_config::run(&context.root, "check", &args)?;
    let manifest = &checked["manifest"];
    if manifest["repository"] != repo
        || !manifest["agents"]
            .as_array()
            .is_some_and(|agents| agents.iter().any(|id| id == agent))
    {
        return Err("Select an instruction bundle for this repository and Agent.".into());
    }
    let mut paths: Vec<_> = manifest["outputs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| row["path"].as_str())
        .collect();
    let mcp_path = match (agent, scope.surface.as_deref()) {
        ("codex", _) => Some(".codex/config.toml"),
        ("claude", _) => Some(".mcp.json"),
        ("antigravity", Some("cli" | "ide")) => Some(".agents/mcp_config.json"),
        _ => None,
    };
    paths.extend(mcp_path);
    paths.sort_unstable();
    paths.dedup();
    let descriptor = json!({
        "id":id,"request":"Set up the selected CIRNETRULES Agent in a confirmed worktree.",
        "objective":format!("Review and synchronize {agent} instructions and scoped MCP launch configuration."),
        "non_goals":["Native user settings, workspace trust and independent client verification."],
        "primary_repo":repo,"hints":[],"read_scope":{(repo):paths},"write_scope":{(repo):paths},
        "verification_scope":{},"documentation_scope":{},"bindings":{(repo):binding},
        "risk":"normal","role":"developer","mode":"WORK","owner":actor,
        "authorization":{"basis":"Pending independent owner approval.","owner":actor,"actions":[]},
        "spec":spec,"plan":plan,"acceptance":["Review exact sync and MCP registration plans; verify resulting bytes."],
        "commands":[],"docs_targets":[]
    });
    let candidate = tasks::preview_proposal(&context.root, &descriptor, &context.registry)?;
    let digest = docs::sha256(&serde_json::to_vec(&json!({
        "descriptor":descriptor,"spec_sha256":candidate["spec_sha256"],"plan_sha256":candidate["plan_sha256"],
        "policy_sha256":candidate["policy_sha256"],"bundle_sha256":checked["manifest_sha256"],
        "governance_head":context.head
    })).map_err(|_| "Cannot encode task proposal.")?);
    if let Some(expect) = expect {
        if expect != digest {
            return Err("Task proposal changed. Review a fresh preview.".into());
        }
        return tasks::create_proposal(&context.root, descriptor, &context.registry);
    }
    Ok(
        json!({"status":"PASS","state":"DRY_RUN","plan_sha256":digest,"descriptor":descriptor,
        "next":"Ask the authorization owner to review and approve exact scope before using this task."}),
    )
}
