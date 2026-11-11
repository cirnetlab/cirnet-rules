use crate::{
    adapter, agent_config, application_identity, audit, authority, docs, lifecycle, migration,
    policy, release, runner, tasks, workspace,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
    process::ExitCode,
};

pub const COMMANDS: &[&str] = &[
    "validate",
    "context",
    "check-translations",
    "check-versions",
    "check-release",
    "check-drift",
    "bootstrap",
    "doctor",
    "workspace-discover",
    "workspace-bind",
    "workspace-health",
    "route",
    "task-context",
    "context-prune",
    "task-create",
    "task-show",
    "task-resume",
    "task-checkpoint",
    "task-transition",
    "task-revise",
    "grant-evaluate",
    "run-step",
    "cancel-step",
    "diff-check",
    "task-verify",
    "review-context",
    "task-review",
    "task-accept",
    "task-close",
    "docs-preview",
    "docs-apply",
    "docs-check",
    "task-bootstrap",
    "release-check",
];

fn governance_root(explicit: Option<String>) -> Result<PathBuf, String> {
    if let Some(path) = explicit
        .map(PathBuf::from)
        .or_else(|| env::var_os(application_identity::GOVERNANCE_ENV).map(PathBuf::from))
    {
        return path
            .canonicalize()
            .map_err(|_| "Governance directory unavailable".into());
    }
    let current = env::current_dir().map_err(|error| error.to_string())?;
    for ancestor in current.ancestors().take(10) {
        for candidate in [ancestor.to_path_buf(), ancestor.join("cirnet-rules")] {
            if !candidate.join("governance.yaml").is_file() || !candidate.join("VERSION").is_file()
            {
                continue;
            }
            let candidate = candidate
                .canonicalize()
                .map_err(|_| "Governance directory unavailable")?;
            let top = policy::git(&candidate, &["rev-parse", "--show-toplevel"]);
            if top
                .ok()
                .and_then(|path| Path::new(path.trim()).canonicalize().ok())
                .as_ref()
                == Some(&candidate)
                && policy::git(
                    &candidate,
                    &[
                        "ls-files",
                        "--error-unmatch",
                        "--",
                        "governance.yaml",
                        "VERSION",
                        "en/AGENTS.md",
                    ],
                )
                .is_ok()
            {
                return Ok(candidate);
            }
        }
    }
    Err("CIRNETRULES governance checkout was not found in the local workspace".into())
}

fn execute(command: &str, mut args: BTreeMap<String, String>) -> Result<Value, String> {
    let root = governance_root(args.remove("governance"))?;
    if let Some(action) = command.strip_prefix("authority-") {
        return authority::run(&root, action, &args);
    }
    if let Some(action) = command.strip_prefix("agent-") {
        return agent_config::run(&root, action, &args);
    }
    if let Some(action) = command.strip_prefix("migration-") {
        if matches!(action, "apply" | "rollback") && args.remove("yes").as_deref() != Some("true") {
            return Err(format!("Migration {action} requires --yes"));
        }
        return migration::run(&root, action, &args);
    }
    if let Some(source) = command.strip_prefix("audit-") {
        let record = match source {
            "task" => audit::task(&root, one_option(&args, "task")?),
            "agent" => agent_config::audit(&root, one_option(&args, "receipt")?),
            "migration" => migration::audit(&root, one_option(&args, "receipt")?),
            _ => return Err("Unknown audit source".into()),
        }?;
        return Ok(json!({"schema":1,"status":record["status"],"source":source,"record":record}));
    }
    let mut registry = policy::registry(&root)?;
    if !matches!(
        command,
        "validate"
            | "context"
            | "check-translations"
            | "check-versions"
            | "check-release"
            | "check-drift"
            | "bootstrap"
            | "doctor"
    ) && tasks::legacy_operation_blocked(command, &root, &args)?
    {
        return Ok(json!({"status":"BLOCKED",
            "reason":"Existing legacy maintenance state requires reviewed migration before local task operations."}));
    }
    match command {
        "workspace-discover" | "workspace-bind" | "workspace-health" | "route" | "task-context"
        | "context-prune" => return workspace::run(command, &root, &args, &registry),
        "task-create" | "task-show" | "task-resume" | "task-checkpoint" | "task-transition"
        | "task-revise" => return tasks::run(command, &root, &args, &registry),
        "grant-evaluate" | "run-step" | "cancel-step" | "diff-check" => {
            return runner::run(command, &root, &args, &registry);
        }
        "task-verify" | "review-context" | "task-review" | "task-accept" | "task-close"
        | "docs-preview" | "docs-apply" | "docs-check" => {
            return lifecycle::run(command, &root, &args, &registry);
        }
        "task-bootstrap" => return task_bootstrap(&root, args, &registry),
        "release-check" => {
            if args
                .keys()
                .any(|key| !matches!(key.as_str(), "task" | "actor" | "workspace"))
            {
                return Err("Unexpected release-check option".into());
            }
            let task = tasks::load_current(&root, args.get("task").ok_or("Missing --task")?)?;
            let descriptor = &task["descriptor"];
            if descriptor["mode"] != "RELEASE"
                || !matches!(
                    task["state"].as_str(),
                    Some("READY" | "IN_PROGRESS" | "IMPLEMENTED" | "VERIFIED" | "ACCEPTED")
                )
                || descriptor["primary_repo"] != registry["authority"]["release_set"]["repository"]
                || descriptor["authorization"]["owner"].as_str()
                    != args.get("actor").map(String::as_str)
            {
                return Ok(
                    json!({"status":"BLOCKED","reason":"An explicit RELEASE task and release authority actor are required"}),
                );
            }
            tasks::authorize(&task, "release")?;
            tasks::authorize(&task, "read")?;
            if !release::release_scope_matches(&descriptor["read_scope"], &registry) {
                return Ok(
                    json!({"status":"BLOCKED","reason":"RELEASE scope must authorize each declared producer, manifest and build input in the selected profile"}),
                );
            }
            let workspace = args
                .get("workspace")
                .map(PathBuf::from)
                .or_else(|| root.parent().map(Path::to_path_buf))
                .ok_or("Product workspace unavailable")?;
            prepare_release_registry(&root, &mut registry)?;
            registry["_cirnetrules_release_governance"] = json!(root);
            return release::check_release(&workspace, &registry);
        }
        "check-translations" if args.contains_key("paths") => {
            if args.len() != 1 {
                return Err("Unexpected translation check option".into());
            }
            let paths: Vec<String> = args["paths"].split(',').map(str::to_owned).collect();
            return Ok(
                json!({"status":"PASS","paired_documents":docs::verify_paths(&root,&paths)?}),
            );
        }
        _ => {}
    }
    let permitted: &[&str] = match command {
        "validate" | "check-translations" | "check-versions" | "check-drift" | "check-release"
        | "doctor" => &["workspace"],
        "context" => &["workspace", "repo", "task", "risk"],
        "bootstrap" => &[
            "workspace",
            "repo",
            "task",
            "risk",
            "agent",
            "session",
            "action",
        ],
        _ => return Err("Unknown command".into()),
    };
    if let Some(key) = args.keys().find(|key| !permitted.contains(&key.as_str())) {
        return Err(format!("Unknown argument --{key}"));
    }
    let workspace = args.remove("workspace").map(PathBuf::from);
    let repository = args.remove("repo");
    let task = args.remove("task").unwrap_or_else(|| "audit".into());
    let risk = args.remove("risk").unwrap_or_else(|| "normal".into());
    let agent = args.remove("agent");
    let session = args.remove("session").map(PathBuf::from);
    let action = args.remove("action").unwrap_or_else(|| "dry-run".into());
    match command {
        "validate" => {
            policy::validate(&root, &registry)?;
            let tracked = policy::git(
                &root,
                &[
                    "ls-files",
                    "-z",
                    "--cached",
                    "--others",
                    "--exclude-standard",
                ],
            )?;
            if tracked.split('\0').any(|path| {
                path.ends_with(".rs")
                    && !["src/", "cli/src/", "cli/tests/"]
                        .iter()
                        .any(|prefix| path.starts_with(prefix))
                    && root.join(Path::new(path)).is_file()
            }) {
                return Err("Rust is limited to src/, cli/src/ and cli/tests/".into());
            }
            let paths = docs::verify(&root)?;
            Ok(json!({"status":"PASS","paired_documents":paths}))
        }
        "check-translations" => {
            Ok(json!({"status":"PASS","paired_documents":docs::verify(&root)?}))
        }
        "context" | "bootstrap" => {
            let (Some(workspace), Some(repository)) = (workspace, repository) else {
                return Ok(
                    json!({"status":"BLOCKED","reason":"--workspace and --repo are required"}),
                );
            };
            let context = policy::context(&root, &workspace, &registry, &repository, &task, &risk)?;
            if command == "context" {
                return Ok(json!({"status":"PASS","manifest":context}));
            }
            let (Some(agent), Some(session_root)) = (agent, session) else {
                return Ok(
                    json!({"status":"BLOCKED","reason":"--agent and --session are required"}),
                );
            };
            adapter::run(adapter::Request {
                action,
                agent,
                session_root,
                context,
            })
        }
        "check-versions" | "check-drift" | "check-release" => {
            let Some(workspace) = workspace else {
                return Ok(json!({"status":"BLOCKED","reason":"--workspace is required"}));
            };
            if !workspace.is_dir() {
                return Ok(json!({"status":"BLOCKED","reason":"Product workspace unavailable"}));
            }
            if command == "check-drift" {
                policy::validate(&root, &registry)?;
                docs::verify(&root)?;
            }
            if command == "check-release" {
                prepare_release_registry(&root, &mut registry)?;
            }
            match command {
                "check-versions" => release::check_versions(&workspace, &registry),
                "check-drift" => release::check_drift(&workspace, &registry),
                _ => release::check_release(&workspace, &registry),
            }
        }
        "doctor" => policy::doctor(&root, workspace.as_deref(), &registry),
        _ => Err("Unknown command".into()),
    }
}

fn one_option<'a>(args: &'a BTreeMap<String, String>, option: &str) -> Result<&'a str, String> {
    if args.len() != 1 {
        return Err(format!("Expected exactly --{option}"));
    }
    args.get(option)
        .map(String::as_str)
        .ok_or_else(|| format!("Missing --{option}"))
}

fn prepare_release_registry(root: &Path, registry: &mut Value) -> Result<(), String> {
    policy::validate(root, registry)?;
    let (commit, clean) = policy::checkout(root)?;
    let committed = format!("{commit}:governance.yaml");
    let matches = policy::git(root, &["cat-file", "blob", &committed]).is_ok_and(|contents| {
        policy::read(&root.join("governance.yaml"))
            .is_ok_and(|current| contents.replace("\r\n", "\n") == current)
    });
    let approved = registry["release_policy"]["approved_evidence"]["governance_commit"]
        .as_str()
        .is_some_and(|revision| {
            policy::git(root, &["merge-base", "--is-ancestor", revision, &commit]).is_ok()
        });
    registry["_approved_evidence_verified"] = json!(approved);
    registry["_governance_origin_verified"] =
        json!(policy::governance_origin_matches(root).unwrap_or(false));
    registry["_governance_commit"] = json!(commit);
    registry["_governance_clean"] = json!(clean && matches);
    Ok(())
}

fn task_bootstrap(
    root: &Path,
    mut args: BTreeMap<String, String>,
    registry: &Value,
) -> Result<Value, String> {
    let agent = args.remove("agent").ok_or("Missing --agent")?;
    let session_root = PathBuf::from(
        args.remove("session")
            .ok_or("Missing --session directory")?,
    );
    let action = args.remove("action").unwrap_or_else(|| "dry-run".into());
    let task = args.get("task").ok_or("Missing --task")?;
    let state = tasks::load_current(root, task)?;
    let descriptor = &state["descriptor"];
    let repo = descriptor["primary_repo"]
        .as_str()
        .ok_or("Missing primary repo")?;
    let binding = workspace::resolve(root, args.get("workspace").map(Path::new), repo, registry)?;
    let mut packet = workspace::run("task-context", root, &args, registry)?;
    if packet["status"] != "PASS" {
        return Ok(packet);
    }
    packet["scope_summary"] = json!({
        "read":descriptor["read_scope"],"write":descriptor["write_scope"],
        "verify":descriptor["verification_scope"],"documentation":descriptor["documentation_scope"],
        "harness_grants":"none", "state":state["state"]
    });
    let documents: Vec<_> = packet["sources"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|source| source["repository"] == "governance" && source["sha256"].is_string())
        .map(|source| json!({"path":source["path"],"sha256":source["sha256"]}))
        .collect();
    let (commit, clean) = policy::checkout(root)?;
    let profile = registry["repositories"][repo]["profile"]
        .as_str()
        .unwrap_or("governance");
    adapter::run(adapter::Request {
        action,
        agent,
        session_root,
        context: json!({
            "governanceRoot":root,"productRoot":binding.path,"repository":repo,"task":task,"risk":descriptor["risk"],
            "profile":profile,"planReference":descriptor["plan"],"auditReference":descriptor["spec"],
            "governanceCommit":commit,"governanceClean":clean,"targetCommit":binding.commit,
            "scope":"read-only","documents":documents,"taskPacket":packet
        }),
    })
}

fn args(
    command: &str,
    raw: impl IntoIterator<Item = String>,
) -> Result<BTreeMap<String, String>, String> {
    let mut raw = raw.into_iter();
    let mut parsed = BTreeMap::new();
    while let Some(arg) = raw.next() {
        let Some(key) = arg.strip_prefix("--") else {
            return Err("Expected a named option".into());
        };
        let (key, value) = if key == "dry-run" && command.starts_with("agent-") {
            ("dry-run", "true".to_owned())
        } else if key == "dry-run" {
            ("action", "dry-run".to_owned())
        } else if key == "yes"
            && (command.starts_with("agent-")
                || matches!(command, "migration-apply" | "migration-rollback"))
        {
            ("yes", "true".to_owned())
        } else {
            let value = raw
                .next()
                .filter(|v| !v.starts_with("--"))
                .ok_or("Missing option value")?;
            (key, value)
        };
        if parsed.insert(key.to_owned(), value).is_some() {
            return Err("Duplicate option".into());
        }
    }
    if parsed.contains_key("task") && parsed.contains_key("id") {
        return Err("Choose one task identifier option".into());
    }
    Ok(parsed)
}

pub fn maintenance_main() -> ExitCode {
    let mut argv = env::args().skip(1);
    let command = argv.next().unwrap_or_else(|| "--help".into());
    if command == "--help" {
        println!(
            "cirnet-governance <{}> [--governance PATH] [--workspace PATH]",
            COMMANDS.join("|")
        );
        println!("Context: --repo NAME --task NAME --risk normal|high|hardware");
        println!(
            "Bootstrap: --agent codex|claude|antigravity --session PATH [--dry-run|--action install|uninstall]"
        );
        return ExitCode::SUCCESS;
    }
    if command == "--version" {
        println!("cirnet-governance {}", application_identity::VERSION);
        return ExitCode::SUCCESS;
    }
    if command == "--application-info" {
        println!(
            "{}",
            json!({"product_generation":application_identity::PRODUCT_SYMBOL,"product_id":application_identity::PRODUCT_ID,
            "version":application_identity::VERSION,"release_name":application_identity::release_name()})
        );
        return ExitCode::SUCCESS;
    }
    let (code, result) = run(&command, argv);
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
    ExitCode::from(code)
}

pub fn run(command: &str, argv: impl IntoIterator<Item = String>) -> (u8, Value) {
    let result = if COMMANDS.contains(&command)
        || matches!(
            command,
            "migration-inspect"
                | "migration-export"
                | "migration-validate"
                | "migration-preview"
                | "migration-apply"
                | "migration-rollback"
                | "audit-task"
                | "authority-inspect"
                | "authority-context"
                | "agent-list"
                | "agent-export"
                | "agent-check"
                | "agent-sync"
                | "agent-rollback"
                | "audit-agent"
                | "audit-migration"
        ) {
        args(command, argv).and_then(|args| execute(command, args))
    } else {
        Err("Unknown command".into())
    };
    let report = result.unwrap_or_else(|reason| json!({"status":"FAIL","reason":reason}));
    let (status, code) = match report["status"].as_str() {
        Some("PASS") => ("PASS", 0),
        Some("BLOCKED") => ("BLOCKED", 2),
        Some("NOT_RUN" | "NOT_FOUND") => ("NOT_RUN", 2),
        _ => ("FAIL", 1),
    };
    (
        code,
        json!({"command":command,"status":status,"result":report}),
    )
}
