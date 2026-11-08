mod adapter;
mod docs;
mod policy;
mod release;

use serde_json::{Value, json};
use std::{collections::BTreeMap, env, path::PathBuf, process::ExitCode};

const COMMANDS: &[&str] = &[
    "validate",
    "context",
    "check-translations",
    "check-versions",
    "check-release",
    "check-drift",
    "bootstrap",
    "doctor",
];

fn execute(command: &str, mut args: BTreeMap<String, String>) -> Result<Value, String> {
    let root = match args.remove("governance") {
        Some(path) => PathBuf::from(path),
        None => env::current_dir().map_err(|error| error.to_string())?,
    };
    let root = root
        .canonicalize()
        .map_err(|_| "Governance directory unavailable")?;
    let workspace = args.remove("workspace").map(PathBuf::from);
    let repository = args.remove("repo");
    let task = args.remove("task").unwrap_or_else(|| "audit".into());
    let risk = args.remove("risk").unwrap_or_else(|| "normal".into());
    let agent = args.remove("agent");
    let session = args.remove("session").map(PathBuf::from);
    let action = args.remove("action").unwrap_or_else(|| "dry-run".into());
    if let Some(key) = args.keys().next() {
        return Err(format!("Unknown argument --{key}"));
    }
    let mut registry = policy::registry(&root)?;
    match command {
        "validate" => {
            policy::validate(&root, &registry)?;
            let paths = docs::verify(&root)?;
            Ok(json!({"status":"PASS","paired_documents":paths}))
        }
        "check-translations" => {
            Ok(json!({"status":"PASS","paired_documents":docs::verify(&root)?}))
        }
        "context" | "bootstrap" => {
            policy::validate(&root, &registry)?;
            docs::verify(&root)?;
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
                policy::validate(&root, &registry)?;
                let (commit, clean) = policy::checkout(&root)?;
                let committed_registry = format!("{commit}:governance.yaml");
                let registry_matches =
                    policy::git(&root, &["cat-file", "blob", &committed_registry]).is_ok_and(
                        |contents| {
                            policy::read(&root.join("governance.yaml"))
                                .is_ok_and(|current| contents.replace("\r\n", "\n") == current)
                        },
                    );
                let approved_evidence =
                    registry["release_policy"]["approved_evidence"]["governance_commit"]
                        .as_str()
                        .is_some_and(|revision| {
                            policy::git(&root, &["merge-base", "--is-ancestor", revision, &commit])
                                .is_ok()
                        });
                registry["_approved_evidence_verified"] = json!(approved_evidence);
                registry["_governance_origin_verified"] =
                    json!(policy::governance_origin_matches(&root).unwrap_or(false));
                registry["_governance_commit"] = json!(commit);
                registry["_governance_clean"] = json!(clean && registry_matches);
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

fn args(raw: impl IntoIterator<Item = String>) -> Result<BTreeMap<String, String>, String> {
    let mut raw = raw.into_iter();
    let mut parsed = BTreeMap::new();
    while let Some(arg) = raw.next() {
        let Some(key) = arg.strip_prefix("--") else {
            return Err("Expected a named option".into());
        };
        let (key, value) = if key == "dry-run" {
            ("action", "dry-run".to_owned())
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
    Ok(parsed)
}

fn main() -> ExitCode {
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
        println!("cirnet-governance {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    let result = if COMMANDS.contains(&command.as_str()) {
        args(argv).and_then(|args| execute(&command, args))
    } else {
        Err("Unknown command".into())
    };
    let report = result.unwrap_or_else(|reason| json!({"status":"FAIL","reason":reason}));
    let status = report["status"]
        .as_str()
        .filter(|status| matches!(*status, "PASS" | "BLOCKED" | "NOT_RUN" | "FAIL"))
        .unwrap_or("FAIL");
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({"command":command,"status":status,"result":report}))
            .unwrap()
    );
    ExitCode::from(match status {
        "PASS" => 0,
        "BLOCKED" | "NOT_RUN" => 2,
        _ => 1,
    })
}
