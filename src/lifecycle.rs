use crate::{docs, policy, runner, tasks, workspace};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn run(
    command: &str,
    root: &Path,
    options: &BTreeMap<String, String>,
    registry: &Value,
) -> Result<Value, String> {
    let permitted: &[&str] = match command {
        "task-verify" | "task-accept" | "task-close" => &["revision", "actor"],
        "task-review" => &["revision", "actor", "decision", "basis"],
        "docs-preview" => &["proposal"],
        "docs-apply" => &["proposal", "revision", "actor"],
        "docs-check" | "review-context" => &[],
        _ => return Err("Unknown lifecycle action".into()),
    };
    if options.keys().any(|key| {
        !matches!(key.as_str(), "task" | "id" | "workspace") && !permitted.contains(&key.as_str())
    }) {
        return Err("Unexpected lifecycle option".into());
    }
    let id = options
        .get("task")
        .or_else(|| options.get("id"))
        .ok_or("Missing --task")?;
    let work = options.get("workspace").map(Path::new);
    let task = tasks::load_current(root, id)?;
    tasks::authorize(&task, "read")?;
    if command == "review-context" {
        let snapshot = runner::snapshot(root, &task, registry, work)?;
        let mut packet = workspace::run("task-context", root, options, registry)?;
        if packet["status"] != "PASS" {
            return Ok(packet);
        }
        let limit = packet["metrics"]["limits"]["context_bytes"]
            .as_u64()
            .unwrap_or(0);
        let used = packet["metrics"]["read_bytes"].as_u64().unwrap_or(0);
        let mut review = json!({
            "acceptance":task["descriptor"]["acceptance"], "changed":snapshot["repositories"],
            "evidence":task["evidence"].as_array().and_then(|records| records.last()),
            "limitations":"read-only handoff; actor authentication is external"
        });
        let metadata_bytes = serde_json::to_vec(&review)
            .map_err(|_| "Cannot encode review context")?
            .len() as u64;
        if used.saturating_add(metadata_bytes) > limit {
            return Ok(blocked(
                "Review metadata exceeds the remaining context budget",
            ));
        }
        let diff_budget = (limit - used - metadata_bytes).min(
            registry["control_plane"]["budgets"]["max_output_bytes"]
                .as_u64()
                .unwrap_or(4096),
        ) as usize;
        let diff = runner::review_diff(root, work, registry, &task, &snapshot, diff_budget)?;
        let current = runner::snapshot(root, &task, registry, work)?;
        if current["sha256"] != snapshot["sha256"]
            || current["violations"] != snapshot["violations"]
        {
            return Ok(blocked("Review inputs changed while preparing the handoff"));
        }
        packet["metrics"]["review_diff_bytes"] = diff["emitted_bytes"].clone();
        review["diff"] = diff;
        packet["review"] = review;
        return Ok(packet);
    }
    if command == "docs-check" {
        return docs_check(root, registry, work, &task);
    }
    if command == "docs-preview" {
        let proposal = load_proposal(root, options)?;
        return sync(root, registry, work, &task, &proposal, false).map(|(report, _)| report);
    }
    let revision = options
        .get("revision")
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or("Missing or invalid --revision")?;
    if task["revision"] != revision {
        return Err("Stale task revision".into());
    }
    let actor = options
        .get("actor")
        .filter(|value| !value.trim().is_empty())
        .ok_or("Missing --actor")?;
    match command {
        "task-verify" => verify(root, registry, work, id, revision, &task, actor),
        "task-review" | "task-accept" | "task-close" => {
            authorizes(
                &task,
                actor,
                if command == "task-review" {
                    "review"
                } else {
                    "accept"
                },
                true,
            )?;
            let snapshot = runner::snapshot(root, &task, registry, work)?;
            if !fresh(&task, &snapshot) {
                return Ok(blocked("Verification is absent, failed, or stale"));
            }
            if command == "task-accept" && elevated(&task, &snapshot) {
                return Ok(blocked(
                    "Independent authenticated review is required for this boundary; local actor labels are assertions",
                ));
            }
            let mut bindings = BTreeMap::new();
            if command == "task-accept" {
                for target in task["descriptor"]["docs_targets"]
                    .as_array()
                    .into_iter()
                    .flatten()
                {
                    let repo = target["repo"].as_str().ok_or("Invalid docs repository")?;
                    if let Ok(binding) = workspace::resolve(root, work, repo, registry) {
                        bindings.insert(repo, binding.identity);
                    }
                }
            }
            let result = tasks::update(root, id, revision, |next| match command {
                "task-review" => record_review(next, actor, options, &snapshot),
                "task-accept" => accept(next, actor, &snapshot, &bindings),
                _ => {
                    authorizes(next, actor, "accept", true)?;
                    tasks::ensure_children_closed(root, next)?;
                    let report = docs_check(root, registry, work, next)?;
                    if report["status"] != "PASS" {
                        return Err("Documentation is pending or changed since sync".into());
                    }
                    if !matches!(next["state"].as_str(), Some("ACCEPTED" | "DOCS_SYNCED")) {
                        return Err("Only an accepted task can close".into());
                    }
                    next["state"] = json!("CLOSED");
                    Ok(())
                }
            })?;
            Ok(
                json!({"status":"PASS","task":id,"revision":result["revision"],"state":result["state"]}),
            )
        }
        "docs-apply" => {
            authorizes(&task, actor, "docs", false)?;
            let snapshot = runner::snapshot(root, &task, registry, work)?;
            if !fresh(&task, &snapshot) {
                return Ok(blocked("Accepted verification became stale"));
            }
            let proposal = load_proposal(root, options)?;
            let mut report = Value::Null;
            let result = tasks::update(root, id, revision, |next| {
                let (value, progress) = sync(root, registry, work, next, &proposal, true)?;
                next["docs"] = progress;
                report = value;
                if report["status"] == "PASS" {
                    match docs_check(root, registry, work, next) {
                        Ok(check) if check["status"] == "PASS" => {
                            next["state"] = json!("DOCS_SYNCED")
                        }
                        _ => {
                            report["status"] = json!("BLOCKED");
                            report["reason"] = json!(
                                "One or more applied documents require conflict or translation review"
                            );
                            report["locally_applied"] = json!(false);
                            next["state"] = json!("ACCEPTED");
                        }
                    }
                }
                Ok(())
            })?;
            report["revision"] = result["revision"].clone();
            report["state"] = result["state"].clone();
            Ok(report)
        }
        _ => unreachable!(),
    }
}

fn blocked(reason: &str) -> Value {
    json!({"status":"BLOCKED","reason":reason})
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn authorizes(task: &Value, actor: &str, action: &str, authorizer: bool) -> Result<(), String> {
    let descriptor = &task["descriptor"];
    let authorization = &descriptor["authorization"];
    let principal = if authorizer {
        &authorization["owner"]
    } else {
        &descriptor["owner"]
    };
    if principal != actor
        || !authorization["actions"]
            .as_array()
            .is_some_and(|values| values.iter().any(|v| v == action))
        || authorization["expires_at"]
            .as_u64()
            .is_some_and(|expires| expires <= now())
    {
        return Err(format!(
            "Task grant does not authorize {action} for this actor"
        ));
    }
    Ok(())
}

fn elevated(task: &Value, snapshot: &Value) -> bool {
    matches!(
        task["descriptor"]["risk"].as_str(),
        Some("high" | "hardware")
    ) || snapshot["requires_review"] == true
        || snapshot["boundary_paths"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
}

fn fresh(task: &Value, snapshot: &Value) -> bool {
    let evidence = task["evidence"].as_array().and_then(|values| values.last());
    evidence.is_some_and(|value| {
        let mut receipt = value.clone();
        let hash = receipt
            .as_object_mut()
            .and_then(|object| object.remove("receipt_sha256"));
        let integrity = serde_json::to_vec(&receipt)
            .ok()
            .is_some_and(|bytes| hash == Some(json!(docs::sha256(&bytes))));
        integrity
            && value["result"] == "PASS"
            && value["snapshot_sha256"].is_string()
            && value["snapshot_sha256"] == snapshot["sha256"]
            && value["spec_sha256"] == task["spec_sha256"]
            && value["policy_sha256"] == task["policy_sha256"]
            && value["plan_sha256"] == task["plan_sha256"]
            && value["approval_epoch"] == task["approval_epoch"]
    }) && snapshot["violations"].as_array().is_some_and(Vec::is_empty)
}

fn verify(
    root: &Path,
    registry: &Value,
    work: Option<&Path>,
    id: &str,
    revision: u64,
    task: &Value,
    actor: &str,
) -> Result<Value, String> {
    if !matches!(task["state"].as_str(), Some("IMPLEMENTED" | "VERIFIED")) {
        return Ok(blocked("Verification requires IMPLEMENTED or VERIFIED"));
    }
    let implementer = task["descriptor"]["owner"] == actor;
    authorizes(task, actor, "verify", !implementer)?;
    let snapshot = runner::snapshot(root, task, registry, work)?;
    if !snapshot["violations"].as_array().is_some_and(Vec::is_empty) {
        return Ok(
            json!({"status":"BLOCKED","reason":"Diff exceeds the current grant","violations":snapshot["violations"]}),
        );
    }
    let steps: Vec<&str> = task["descriptor"]["commands"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|step| registry["control_plane"]["commands"][step]["mode"] == "VERIFY")
        .collect();
    if steps.is_empty() {
        return Ok(blocked("No approved VERIFY command for this task"));
    }
    let mut results = Vec::new();
    for step in steps {
        let result = runner::execute_step(root, task, registry, work, step, actor)?;
        let pass = result["status"] == "PASS";
        results.push(result);
        if !pass {
            break;
        }
    }
    let after = runner::snapshot(root, task, registry, work)?;
    let status = if snapshot["sha256"] != after["sha256"]
        || !after["violations"].as_array().is_some_and(Vec::is_empty)
    {
        "BLOCKED"
    } else if results.iter().all(|result| result["status"] == "PASS") {
        "PASS"
    } else if results.iter().any(|result| result["status"] == "FAIL") {
        "FAIL"
    } else {
        "BLOCKED"
    };
    let mut receipt = json!({
        "result":status, "snapshot_sha256":snapshot["sha256"], "observed_after_sha256":after["sha256"],
        "spec_sha256":task["spec_sha256"], "policy_sha256":task["policy_sha256"],
        "plan_sha256":task["plan_sha256"], "approval_epoch":task["approval_epoch"], "task_revision":revision,
        "actor":actor, "verification_kind":if implementer {"self_check"} else {"tool_verification"},
        "producer":"cgacp-controlled-runner", "time_unix_seconds":now(),
        "platform":format!("{}-{}",std::env::consts::OS,std::env::consts::ARCH),
        "governance_version":env!("CARGO_PKG_VERSION"), "steps":results,
        "requires_review":elevated(task,&after), "limitations":"local integrity; actor identity and sandbox enforcement are external"
    });
    receipt["receipt_sha256"] = json!(docs::sha256(
        &serde_json::to_vec(&receipt).map_err(|e| e.to_string())?
    ));
    let state = tasks::update(root, id, revision, |next| {
        let evidence = next["evidence"]
            .as_array_mut()
            .ok_or("Invalid evidence store")?;
        if evidence.len() >= 32 {
            return Err("Evidence limit reached; checkpoint or reopen task".into());
        }
        evidence.push(receipt.clone());
        next["review"] = Value::Null;
        next["state"] = json!(if status == "PASS" {
            "VERIFIED"
        } else {
            "IMPLEMENTED"
        });
        Ok(())
    })?;
    Ok(
        json!({"status":status,"state":state["state"],"revision":state["revision"],"evidence":receipt}),
    )
}

fn record_review(
    task: &mut Value,
    actor: &str,
    options: &BTreeMap<String, String>,
    snapshot: &Value,
) -> Result<(), String> {
    authorizes(task, actor, "review", true)?;
    if task["state"] != "VERIFIED" || task["descriptor"]["owner"] == actor {
        return Err("Review requires a separate reviewer and verified inputs".into());
    }
    let decision = options
        .get("decision")
        .filter(|v| matches!(v.as_str(), "approve" | "reject"))
        .ok_or("Expected review decision approve or reject")?;
    let basis = options
        .get("basis")
        .filter(|v| !v.trim().is_empty() && v.len() <= 256)
        .ok_or("Review requires a bounded review reference")?;
    task["review"] = json!({"actor":actor,"decision":decision,"basis":basis,
        "snapshot_sha256":snapshot["sha256"],"identity":"locally asserted","time_unix_seconds":now()});
    Ok(())
}

fn accept(
    task: &mut Value,
    actor: &str,
    snapshot: &Value,
    bindings: &BTreeMap<&str, String>,
) -> Result<(), String> {
    authorizes(task, actor, "accept", true)?;
    if task["descriptor"]["owner"] == actor {
        return Err("The implementer cannot accept their own task".into());
    }
    if !matches!(task["state"].as_str(), Some("VERIFIED" | "ACCEPTED"))
        || task["review"]["decision"] == "reject"
    {
        return Err("Acceptance requires a verified task without rejected review".into());
    }
    for (repo, identity) in task["acceptance"]["docs_bindings"]
        .as_object()
        .into_iter()
        .flatten()
    {
        if bindings
            .get(repo.as_str())
            .is_none_or(|bound| identity != bound.as_str())
        {
            return Err("An accepted docs binding changed; revise the grant explicitly".into());
        }
    }
    task["acceptance"] = json!({"actor":actor,"snapshot_sha256":snapshot["sha256"],"docs_bindings":bindings,"time_unix_seconds":now()});
    task["state"] = json!("ACCEPTED");
    Ok(())
}

fn load_proposal(root: &Path, options: &BTreeMap<String, String>) -> Result<Value, String> {
    let name = options
        .get("proposal")
        .ok_or("Missing --proposal (governance-relative JSON)")?;
    tasks::relative(name.strip_prefix(".cgacp/inputs/").unwrap_or(name))?;
    if !name.ends_with(".json") {
        return Err("Expected a local JSON docs proposal".into());
    }
    let path = policy::safe_path(root, name)?;
    serde_json::from_str(&policy::read(&path)?).map_err(|_| "Invalid docs proposal JSON".into())
}

fn key(value: &Value) -> Result<String, String> {
    let repo = value["repo"].as_str().ok_or("Missing docs repo")?;
    let path = value["path"].as_str().ok_or("Missing docs path")?;
    let locale = value["locale"].as_str().ok_or("Missing docs locale")?;
    Ok(format!("{repo}:{path}:{locale}"))
}

fn target_path(base: &Path, repo: &str, path: &str, locale: &str) -> Result<PathBuf, String> {
    if !matches!(locale, "en" | "vi") || !path.ends_with(".md") {
        return Err("Only registered bilingual Markdown paths can be synced".into());
    }
    let allowed = if repo == "cirnet-rules" {
        ["references", "routing", "tools", "commands", "adapters"]
            .iter()
            .any(|group| path.starts_with(&format!("{locale}/{group}/")))
    } else {
        path.starts_with(&format!("docs/{locale}/"))
    };
    if !allowed {
        return Err("Documentation path is outside publishable content scope".into());
    }
    policy::safe_path(base, path)
}

fn render(current: &str, section: Option<&str>, content: &str) -> Result<String, String> {
    if content.contains('\r') || content.contains('\0') || content.len() > 128 * 1024 {
        return Err("Invalid or oversized docs replacement".into());
    }
    if let Some(section) = section {
        let headings = markdown_headings(current);
        let mut matches = headings
            .iter()
            .filter(|(range, _)| current[range.clone()].trim_end() == section);
        let Some((range, level)) = matches.next() else {
            return Err("Target section is missing".into());
        };
        if matches.next().is_some() {
            return Err("Target section is ambiguous".into());
        }
        if markdown_headings(content)
            .iter()
            .any(|(_, next)| next <= level)
        {
            return Err("Replacement leaves the selected section".into());
        }
        let end = headings
            .iter()
            .find(|(next, next_level)| next.start >= range.end && next_level <= level)
            .map_or(current.len(), |(next, _)| next.start);
        let prefix = current[..range.end].trim_end_matches('\n');
        let content = content.trim_end_matches('\n');
        let inserted = format!("{prefix}\n\n{content}\n\n");
        let output = format!("{inserted}{}", &current[end..]);
        if end < current.len()
            && !markdown_headings(&output)
                .iter()
                .any(|(heading, _)| heading.start == inserted.len())
        {
            return Err("Replacement would hide the next Markdown section".into());
        }
        Ok(output)
    } else {
        Ok(content.to_owned())
    }
}

fn markdown_headings(body: &str) -> Vec<(std::ops::Range<usize>, pulldown_cmark::HeadingLevel)> {
    use pulldown_cmark::{Event, Parser, Tag};
    Parser::new(body)
        .into_offset_iter()
        .filter_map(|(event, mut range)| {
            if let Event::Start(Tag::Heading { level, .. }) = event {
                range.start = body[..range.start]
                    .rfind('\n')
                    .map_or(0, |position| position + 1);
                Some((range, level))
            } else {
                None
            }
        })
        .collect()
}

fn sync(
    root: &Path,
    registry: &Value,
    work: Option<&Path>,
    task: &Value,
    proposal: &Value,
    apply: bool,
) -> Result<(Value, Value), String> {
    if !matches!(task["state"].as_str(), Some("ACCEPTED" | "DOCS_SYNCED")) {
        return Err("Docs sync requires acceptance".into());
    }
    let targets = task["descriptor"]["docs_targets"]
        .as_array()
        .ok_or("Invalid docs targets")?;
    let proposals = proposal["targets"]
        .as_array()
        .ok_or("Docs proposal needs a targets array")?;
    if targets.is_empty() || proposals.is_empty() || proposals.len() > targets.len() {
        return Err("Proposal must contain one or more registered destinations".into());
    }
    let mut progress = task["docs"].clone();
    if !progress.is_object() {
        progress = json!({});
    }
    let mut report = Vec::new();
    let mut seen = BTreeSet::new();
    // Validate the whole routing envelope before mutating any destination.
    for item in proposals {
        let identity = key(item)?;
        if !seen.insert(identity.clone())
            || !targets.iter().any(|target| {
                key(target).ok().as_deref() == Some(&identity)
                    && target["section"] == item["section"]
            })
        {
            return Err("Duplicate or unapproved docs proposal target".into());
        }
        let repo = item["repo"].as_str().unwrap();
        let path = item["path"].as_str().unwrap();
        if !task["descriptor"]["documentation_scope"][repo]
            .as_array()
            .is_some_and(|paths| paths.iter().any(|candidate| candidate == path))
            || item["content"].as_str().is_none()
            || !item["base_sha256"].as_str().is_some_and(|value| {
                value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
            })
        {
            return Err("Invalid docs content, base digest or scope".into());
        }
    }
    for item in proposals {
        let identity = key(item)?;
        let repo = item["repo"].as_str().unwrap();
        let path = item["path"].as_str().unwrap();
        let locale = item["locale"].as_str().unwrap();
        let content = item["content"].as_str().unwrap();
        let base_hash = item["base_sha256"].as_str().unwrap();
        let result = (|| -> Result<Value,String> {
            let binding = workspace::resolve(root, work, repo, registry)?;
            if task["acceptance"]["docs_bindings"][repo] != binding.identity {
                return Err("Documentation worktree is outside the accepted binding".into());
            }
            let file = target_path(&binding.path,repo,path,locale)?;
            let current = policy::read(&file)?;
            let current_hash = docs::sha256(current.as_bytes());
            let rendered = render(&current,item["section"].as_str(),content)?;
            let sha256 = docs::sha256(rendered.as_bytes());
            if sha256 == current_hash {
                return Ok(json!({"state":"LOCAL_APPLIED","sha256":sha256,"binding":binding.identity,"unchanged":true}));
            }
            if current_hash != base_hash {
                return Err("Document base changed; review and rebase proposal".into());
            }
            if !apply {
                return Ok(json!({"state":"PENDING","patch":{"path":path,"section":item["section"],
                    "base_sha256":base_hash,"replacement":content},"sha256":sha256}));
            }
            let rechecked = workspace::resolve(root,work,repo,registry)?;
            if binding.identity != rechecked.identity || target_path(&rechecked.path,repo,path,locale)? != file {
                return Err("Documentation worktree binding changed".into());
            }
            replace_document(&file,&current_hash,&rendered)?;
            Ok(json!({"state":"LOCAL_APPLIED","sha256":sha256,"binding":binding.identity,"unchanged":false}))
        })().unwrap_or_else(|reason| json!({"state":"PENDING","reason":reason}));
        if apply {
            progress[&identity] = result.clone();
        }
        report.push(json!({"target":identity,"result":result}));
    }
    let complete = apply
        && targets.iter().all(|target| {
            key(target).is_ok_and(|identity| progress[identity]["state"] == "LOCAL_APPLIED")
        });
    let status = if complete
        || !apply
            && report
                .iter()
                .all(|row| row["result"].get("reason").is_none())
    {
        "PASS"
    } else {
        "BLOCKED"
    };
    Ok((
        json!({"status":status,"targets":report,"locally_applied":complete,"published":false}),
        progress,
    ))
}

fn replace_document(path: &Path, current_hash: &str, content: &str) -> Result<(), String> {
    let parent = path.parent().ok_or("Missing document parent")?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let temp = parent.join(format!(".cgacp-doc-{}-{nonce}.tmp", std::process::id()));
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|e| e.to_string())?;
    output
        .write_all(content.as_bytes())
        .and_then(|_| output.sync_all())
        .map_err(|e| e.to_string())?;
    output
        .set_permissions(fs::metadata(path).map_err(|e| e.to_string())?.permissions())
        .map_err(|e| e.to_string())?;
    drop(output);
    if docs::sha256(policy::read(path)?.as_bytes()) != current_hash {
        return Err("Document changed during staging; inspect the staged file".into());
    }
    fs::rename(&temp, path)
        .map_err(|_| "Could not replace document; inspect the staged file".into())
}

fn docs_check(
    root: &Path,
    registry: &Value,
    work: Option<&Path>,
    task: &Value,
) -> Result<Value, String> {
    if !matches!(
        task["state"].as_str(),
        Some("ACCEPTED" | "DOCS_SYNCED" | "CLOSED")
    ) {
        return Ok(blocked("Documentation check requires acceptance"));
    }
    if task["descriptor"]["docs_not_required"]
        .as_str()
        .is_some_and(|v| !v.trim().is_empty())
    {
        return Ok(json!({"status":"PASS","decision":"docs_not_required"}));
    }
    let targets = task["descriptor"]["docs_targets"]
        .as_array()
        .ok_or("Invalid docs targets")?;
    let mut governance_targets = Vec::new();
    for target in targets {
        let identity = key(target)?;
        let repo = target["repo"].as_str().unwrap();
        let path = target["path"].as_str().unwrap();
        let binding = workspace::resolve(root, work, repo, registry)?;
        if task["acceptance"]["docs_bindings"][repo] != binding.identity {
            return Ok(blocked(
                "Documentation worktree is outside the accepted binding",
            ));
        }
        let current = policy::read(&target_path(
            &binding.path,
            repo,
            path,
            target["locale"].as_str().unwrap(),
        )?)?;
        let saved = &task["docs"][identity];
        if saved["state"] != "LOCAL_APPLIED"
            || saved["binding"] != binding.identity
            || saved["sha256"] != docs::sha256(current.as_bytes())
        {
            return Ok(blocked(
                "One or more documentation targets are pending or stale",
            ));
        }
        if repo == "cirnet-rules" {
            governance_targets.push(path.to_owned());
        }
    }
    if !governance_targets.is_empty() {
        docs::verify_paths(root, &governance_targets)?;
    }
    if targets.is_empty() {
        return Ok(blocked("No docs decision or destination recorded"));
    }
    Ok(json!({"status":"PASS","verified_destinations":targets.len(),"published":false}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_patch_is_idempotent_and_preserves_neighbouring_sections() {
        let old = "# Guide\n\n## Behavior\n\nold\n\n## Limits\n\nkeep\n";
        let result = render(old, Some("## Behavior"), "new").unwrap();
        assert_eq!(
            result,
            "# Guide\n\n## Behavior\n\nnew\n\n## Limits\n\nkeep\n"
        );
        assert_eq!(render(&result, Some("## Behavior"), "new").unwrap(), result);
        assert!(render(old, Some("## Behavior"), "## Escape").is_err());
        assert!(render(old, Some("## Behavior"), "   # Escape").is_err());
        assert!(render(old, Some("## Behavior"), "Escape\n======").is_err());
        assert!(render(old, Some("## Behavior"), "```\nhides the next section").is_err());
        let indented = old.replace("## Limits", "   ## Limits");
        assert!(
            render(&indented, Some("## Behavior"), "new")
                .unwrap()
                .contains("   ## Limits")
        );
        assert!(render(&format!("{old}\n## Behavior\n"), Some("## Behavior"), "new").is_err());
    }

    #[test]
    fn a_local_label_cannot_substitute_for_another_authorizer() {
        let task = json!({"descriptor":{"owner":"implementer","authorization":{"owner":"maintainer","actions":["accept"]}}});
        assert!(authorizes(&task, "implementer", "accept", true).is_err());
        assert!(authorizes(&task, "maintainer", "accept", true).is_ok());
        assert!(authorizes(&task, "maintainer", "docs", true).is_err());
    }
}
