use super::{Manifest, bundle, current_executable, patch, render};
use crate::{
    agent_config::{paths, sync::Operation},
    docs, policy,
    service::GovernanceService,
    tasks, workspace,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
};

const STORE: &str = ".cirnetrules/agent-config/mcp-registration";

#[derive(Clone, Copy)]
struct Adapter {
    file: &'static str,
    template: &'static str,
    guide: &'static str,
}

fn adapter(agent: &str) -> Result<Adapter, String> {
    Ok(match agent {
        "codex" => Adapter {
            file: ".codex/config.toml",
            template: "codex.toml",
            guide: "In the trusted Codex project configuration, review and merge the cirnetrules table. User scope and CODEX_HOME must be selected in the native client.",
        },
        "claude" => Adapter {
            file: ".mcp.json",
            template: "claude.mcp.json",
            guide: "Merge the cirnetrules project MCP entry and approve it in Claude Code. Confirm user scope using Claude Code's MCP manager.",
        },
        "antigravity" => Adapter {
            file: ".agents/mcp_config.json",
            template: "antigravity.mcp_config.json",
            guide: "Open the installed Antigravity surface's MCP manager and review its raw config. For App or an unverified surface, import the cirnetrules entry there.",
        },
        _ => return Err("Select codex, claude or antigravity".into()),
    })
}

fn canonical_parents(mut path: &Path) -> Result<PathBuf, String> {
    let mut missing = Vec::new();
    while missing.len() < 64 {
        if let Ok(mut resolved) = path.canonicalize() {
            for name in missing.iter().rev() {
                resolved.push(name);
            }
            return Ok(resolved);
        }
        let Some((parent, name)) = path.parent().zip(path.file_name()) else {
            break;
        };
        missing.push(name);
        path = parent;
    }
    Err("Cannot safely resolve the native configuration path".into())
}

fn user_scope_guard(
    binding: &workspace::Binding,
    adapter: Adapter,
    agent: &str,
) -> Result<(), String> {
    let file = canonical_parents(&binding.path.join(adapter.file))?;
    let (overrides, default) = match agent {
        "codex" => (&["CODEX_HOME"][..], ".codex"),
        "claude" => (&["CLAUDE_CONFIG_DIR", "XDG_CONFIG_HOME"][..], ".claude"),
        "antigravity" => (&["XDG_CONFIG_HOME"][..], ".gemini/config"),
        _ => return Err("Unknown MCP agent".into()),
    };
    let configured = overrides.iter().filter_map(env::var_os).map(PathBuf::from);
    let defaults = ["USERPROFILE", "HOME"]
        .into_iter()
        .filter_map(env::var_os)
        .map(|home| PathBuf::from(home).join(default));
    for directory in configured.chain(defaults) {
        if !directory.is_absolute() {
            return Err(
                "A relative native configuration path requires manual scope review.".into(),
            );
        }
        let directory = paths::explicit(&directory)?;
        if paths::contains(&canonical_parents(&directory)?, &file) {
            return Err("The project MCP destination overlaps a native user configuration directory. Use the Agent's MCP manager.".into());
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    schema: u8,
    agent: String,
    repository: String,
    surface: Option<String>,
    binding: String,
    target_commit: String,
    authority: String,
    launch: String,
    task: Option<String>,
    actor: Option<String>,
    revision: Option<u64>,
    before: paths::Observation,
    after: String,
    insertion: patch::Insertion,
}

struct Request<'a> {
    root: &'a Path,
    repository: &'a str,
    agent: &'a str,
    adapter: Adapter,
    binding: workspace::Binding,
    operation: Operation<'a>,
    args: &'a BTreeMap<String, String>,
}

fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value).map_err(|_| "Cannot encode MCP registration record".into())
}

fn hash(value: &impl Serialize) -> Result<String, String> {
    Ok(docs::sha256(&encode(value)?))
}

fn receipt_path(id: &str) -> Result<String, String> {
    if id.len() != 64
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("Select an MCP registration receipt SHA-256".into());
    }
    Ok(format!("{STORE}/{id}"))
}

fn stage_path(file: &str, id: &str, operation: &str) -> String {
    let stage = format!(".cnr-mcp-{operation}-{id}");
    file.rsplit_once('/')
        .map_or_else(|| stage.clone(), |(parent, _)| format!("{parent}/{stage}"))
}

fn retry_stage(root: &Path, target: &Path, stage: &str, intent: &str) -> Result<(), String> {
    let staged = paths::observe(target, stage)?;
    if let Some(intended) = read::<paths::Observation>(root, intent)? {
        if staged.sha256.is_some() {
            if staged != intended {
                return Err("An interrupted MCP operation has an unrecognized stage. Inspect its receipt before retrying.".into());
            }
            paths::replace(target, stage, &staged, None)?;
        }
        let record = paths::observe(root, intent)?;
        if read::<paths::Observation>(root, intent)?.as_ref() != Some(&intended) {
            return Err("MCP registration journal changed while recovering".into());
        }
        paths::replace(root, intent, &record, None)?;
    } else if staged.sha256.is_some() {
        return Err(
            "An MCP stage exists without a recorded intent. Inspect the files before retrying."
                .into(),
        );
    }
    Ok(())
}

fn read<T: DeserializeOwned + Serialize>(root: &Path, relative: &str) -> Result<Option<T>, String> {
    if paths::observe(root, relative)?.sha256.is_none() {
        return Ok(None);
    }
    let bytes = paths::read(&root.join(relative))?;
    let data = serde_json::from_slice(&bytes).map_err(|_| "Invalid MCP registration record")?;
    if encode(&data)? != bytes {
        return Err("MCP registration record has unexpected data".into());
    }
    Ok(Some(data))
}

fn before(
    binding: &workspace::Binding,
    path: &str,
) -> Result<(paths::Observation, Option<Vec<u8>>), String> {
    let observed = paths::observe(&binding.path, path)?;
    let bytes = observed
        .sha256
        .as_ref()
        .map(|digest| {
            let bytes = paths::read(&binding.path.join(path))?;
            if docs::sha256(&bytes) != *digest {
                return Err("Project MCP config changed during preview".to_owned());
            }
            Ok(bytes)
        })
        .transpose()?;
    Ok((observed, bytes))
}

fn report(agent: &str, adapter: Adapter, status: &str, state: &str, reason: &str) -> Value {
    json!({
        "status":status,"registration":state,"reason":reason,"agent":agent,
        "file":adapter.file,"template":adapter.template,"next_action":adapter.guide,"server":"cirnetrules",
        "scope":"project","native_client":"NOT_RUN"
    })
}

pub(crate) fn audit(root: &Path, id: &str) -> Result<Option<Value>, String> {
    let directory = receipt_path(id)?;
    let root = paths::directory(root)?;
    let Some(plan) = read::<Plan>(&root, &format!("{directory}/plan.json"))? else {
        return Ok(None);
    };
    if hash(&plan)? != id || plan.schema != 1 {
        return Err("Invalid MCP registration receipt identity".into());
    }
    let selected = adapter(&plan.agent)?;
    Ok(Some(json!({
        "status":"PASS","receipt":id,"kind":"MCP_PROJECT_REGISTRATION",
        "agent":plan.agent,"repository":plan.repository,"binding":plan.binding,"file":selected.file,
        "task":plan.task,"task_revision":plan.revision,"actor":plan.actor,
        "install_intent_recorded":read::<paths::Observation>(&root, &format!("{directory}/intent.json"))?.is_some(),
        "rollback_intent_recorded":read::<paths::Observation>(&root, &format!("{directory}/rollback-intent.json"))?.is_some(),
        "before_sha256":plan.before.sha256,"after_sha256":plan.after,
        "native_client":"NOT_RUN"
    })))
}

impl<'a> Request<'a> {
    fn new(
        root: &'a Path,
        workspace: &'a Path,
        registry: &'a Value,
        repository: &'a str,
        agent: &'a str,
        args: &'a BTreeMap<String, String>,
    ) -> Result<Self, String> {
        let operation = Operation::new(root, Some(workspace), registry, args)?;
        let binding = match args.get("binding") {
            Some(id) => {
                let chosen = workspace::resolve_named(root, workspace, repository, registry, id)?;
                if operation.target != chosen.path {
                    return Err("Selected binding does not match the explicit MCP target".into());
                }
                if (args.contains_key("task") || operation.apply)
                    && operation.bind(repository, None)?.identity != chosen.identity
                {
                    return Err("Task binding differs from the selected MCP worktree".into());
                }
                chosen
            }
            None => operation.bind(repository, None)?,
        };
        let adapter = adapter(agent)?;
        user_scope_guard(&binding, adapter, agent)?;
        Ok(Self {
            root,
            repository,
            agent,
            adapter,
            binding,
            operation,
            args,
        })
    }

    fn revision(&self) -> Result<Option<u64>, String> {
        self.args
            .get("task")
            .map(|task| {
                tasks::load_current(self.root, task)?["revision"]
                    .as_u64()
                    .ok_or("Invalid MCP task revision".into())
            })
            .transpose()
    }

    fn grant(&self, revision: Option<u64>) -> Result<(), String> {
        if Some(
            self.operation
                .grant(self.repository, &self.binding, self.adapter.file)?,
        ) != revision
        {
            return Err("MCP task revision changed. Review a fresh preview.".into());
        }
        Ok(())
    }

    fn current_receipt(&self, plan: &Plan) -> bool {
        plan.schema == 1
            && plan.agent == self.agent
            && plan.repository == self.repository
            && plan.binding == self.binding.identity
    }

    fn install(
        &self,
        authority: &str,
        launch: &str,
        template: &[u8],
        check_snapshot: impl Fn() -> Result<(), String>,
    ) -> Result<Value, String> {
        let revision = self.revision()?;
        let (observation, bytes) = before(&self.binding, self.adapter.file)?;
        let insertion = match patch::plan(self.agent, bytes.as_deref(), template) {
            Ok(Some(insertion)) => insertion,
            Ok(None) => {
                let mut outcome = report(
                    self.agent,
                    self.adapter,
                    "PASS",
                    "UNCHANGED",
                    "Matching project entry is present.",
                );
                outcome["evidence"] = json!("CONFIG_REGISTERED");
                if self.operation.apply {
                    self.require_trust()?;
                    self.grant(revision)?;
                    let id = self
                        .args
                        .get("expect")
                        .ok_or("Missing reviewed registration digest")?;
                    let dir = receipt_path(id)?;
                    if let (Some(plan), Some(installed)) = (
                        read::<Plan>(self.root, &format!("{dir}/plan.json"))?,
                        read::<paths::Observation>(self.root, &format!("{dir}/intent.json"))?,
                    ) && hash(&plan)? == *id
                        && self.current_receipt(&plan)
                        && plan.revision == revision
                        && plan.authority == authority
                        && plan.task.as_ref() == self.args.get("task")
                        && plan.actor.as_ref() == self.args.get("actor")
                        && plan.surface.as_ref() == self.args.get("surface")
                        && plan.target_commit == self.binding.commit
                        && plan.launch == launch
                        && installed == observation
                    {
                        check_snapshot()?;
                        outcome["receipt"] = json!(id);
                        return Ok(outcome);
                    }
                    return Err("The preview changed. Review the matching project entry with a new dry run.".into());
                }
                return Ok(outcome);
            }
            Err(reason) => {
                return Ok(report(
                    self.agent,
                    self.adapter,
                    "BLOCKED",
                    "CONFLICT",
                    &reason,
                ));
            }
        };
        let after = insertion.apply(bytes.as_deref().unwrap_or_default())?;
        let plan = Plan {
            schema: 1,
            agent: self.agent.into(),
            repository: self.repository.into(),
            surface: self.args.get("surface").cloned(),
            binding: self.binding.identity.clone(),
            target_commit: self.binding.commit.clone(),
            authority: authority.into(),
            launch: launch.into(),
            task: self.args.get("task").cloned(),
            actor: self.args.get("actor").cloned(),
            revision,
            before: observation,
            after: docs::sha256(&after),
            insertion,
        };
        let id = hash(&plan)?;
        let state = if plan.before.sha256.is_some() {
            "UPDATE"
        } else {
            "CREATE"
        };
        let mut outcome = report(
            self.agent,
            self.adapter,
            "PASS",
            state,
            "Review the cirnetrules project entry and grant for this file.",
        );
        outcome["plan_sha256"] = json!(id);
        outcome["binding_identity"] = json!(self.binding.identity);
        outcome["task_revision"] = json!(revision);
        outcome["entry"] = patch::preview(self.agent, template)?;
        if !self.operation.apply {
            return Ok(outcome);
        }
        self.require_trust()?;
        self.operation.review(&id)?;
        self.grant(revision)?;
        let _coordinator = tasks::lock(self.root, "_coordination")?;
        let _lock = self.operation.lock(&self.binding.identity)?;
        self.grant(revision)?;
        check_snapshot()?;
        user_scope_guard(&self.binding, self.adapter, self.agent)?;
        if paths::observe(&self.binding.path, self.adapter.file)? != plan.before {
            return Err("Project MCP config changed after preview".into());
        }
        let dir = receipt_path(&id)?;
        paths::directories(self.root, &dir)?;
        let plan_path = format!("{dir}/plan.json");
        if let Some(prior) = read::<Plan>(self.root, &plan_path)? {
            if hash(&prior)? != id {
                return Err("MCP registration receipt does not match the reviewed plan".into());
            }
        } else {
            paths::write_private(&self.root.join(plan_path), &encode(&plan)?)?;
        }
        let intent = format!("{dir}/intent.json");
        let stage = stage_path(self.adapter.file, &id, "install");
        retry_stage(self.root, &self.binding.path, &stage, &intent)?;
        paths::replace_private_journaled(
            &self.binding.path,
            self.adapter.file,
            &stage,
            &plan.before,
            Some(&after),
            |intended| {
                self.grant(revision)?;
                check_snapshot()?;
                paths::write_private(&self.root.join(&intent), &encode(&intended)?)
            },
        )?;
        let written = paths::observe(&self.binding.path, self.adapter.file)?;
        if written.sha256.as_deref() != Some(&plan.after)
            || Some(written) != read(self.root, &intent)?
        {
            return Err("MCP registration result changed before verification".into());
        }
        outcome["registration"] = json!("CONFIG_REGISTERED");
        outcome["evidence"] = json!("CONFIG_REGISTERED");
        outcome["receipt"] = json!(id);
        outcome["reason"] = json!(
            "The project entry was registered. Observe the native connection in the installed client."
        );
        Ok(outcome)
    }

    fn require_trust(&self) -> Result<(), String> {
        if self.args.get("trust-reviewed").map(String::as_str) != Some("true") {
            return Err("Confirm the installed client's project trust separately, then use --trust-reviewed.".into());
        }
        Ok(())
    }

    fn rollback(
        &self,
        plan: Plan,
        id: &str,
        authority: &str,
        check_snapshot: impl Fn() -> Result<(), String>,
    ) -> Result<Value, String> {
        if !self.current_receipt(&plan) || hash(&plan)? != id {
            return Err("MCP receipt does not match the selected agent or binding".into());
        }
        let revision = self.revision()?;
        let dir = receipt_path(id)?;
        let (observation, bytes) = before(&self.binding, self.adapter.file)?;
        let installed: Option<paths::Observation> = read(self.root, &format!("{dir}/intent.json"))?;
        let rollback_intent = format!("{dir}/rollback-intent.json");
        let rolled_back: Option<paths::Observation> = read(self.root, &rollback_intent)?;
        if rolled_back.as_ref() == Some(&observation) || observation == plan.before {
            return Ok(report(
                self.agent,
                self.adapter,
                "PASS",
                "UNCHANGED",
                "The receipt has no installed entry remaining.",
            ));
        }
        if installed.as_ref() != Some(&observation)
            || observation.sha256.as_deref() != Some(&plan.after)
        {
            return Ok(report(
                self.agent,
                self.adapter,
                "BLOCKED",
                "CONFLICT",
                "The project config changed after install. Review changes before manual recovery.",
            ));
        }
        let restored = plan.insertion.reverse(
            bytes
                .as_deref()
                .ok_or("Missing installed project MCP config")?,
        )?;
        let restore = if let Some(expected) = &plan.before.sha256 {
            if docs::sha256(&restored) != *expected {
                return Err("MCP rollback does not reproduce the reviewed preimage".into());
            }
            Some(restored.as_slice())
        } else {
            if !restored.is_empty() {
                return Err("MCP rollback cannot remove a file with user content".into());
            }
            None
        };
        let preview = hash(&(
            id,
            &observation,
            revision,
            &self.binding.commit,
            authority,
            self.args.get("task"),
            self.args.get("actor"),
        ))?;
        let mut outcome = report(
            self.agent,
            self.adapter,
            "PASS",
            "ROLLBACK_PREVIEW",
            "Review removal of the exact entry recorded in the receipt.",
        );
        outcome["plan_sha256"] = json!(preview);
        outcome["receipt"] = json!(id);
        if self.operation.apply {
            self.operation.review(&preview)?;
            self.grant(revision)?;
            let _coordinator = tasks::lock(self.root, "_coordination")?;
            let _lock = self.operation.lock(&self.binding.identity)?;
            self.grant(revision)?;
            check_snapshot()?;
            user_scope_guard(&self.binding, self.adapter, self.agent)?;
            if paths::observe(&self.binding.path, self.adapter.file)? != observation {
                return Err("Project MCP config changed after rollback preview".into());
            }
            let stage = stage_path(self.adapter.file, id, "rollback");
            retry_stage(self.root, &self.binding.path, &stage, &rollback_intent)?;
            paths::replace_private_journaled(
                &self.binding.path,
                self.adapter.file,
                &stage,
                &observation,
                restore,
                |intended| {
                    self.grant(revision)?;
                    check_snapshot()?;
                    paths::write_private(&self.root.join(&rollback_intent), &encode(&intended)?)
                },
            )?;
            if Some(paths::observe(&self.binding.path, self.adapter.file)?)
                != read(self.root, &rollback_intent)?
            {
                return Err("MCP rollback result changed before verification".into());
            }
            outcome["registration"] = json!("ROLLED_BACK");
            outcome["reason"] = json!("The recorded MCP entry was removed.");
        }
        Ok(outcome)
    }
}

pub(super) fn run(
    root: &Path,
    action: &str,
    args: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let agent = args.get("agent").ok_or("Select an explicit MCP agent")?;
    let selected = adapter(agent)?;
    let scope = args.get("scope").map_or("project", String::as_str);
    let surface = args.get("surface").map(String::as_str);
    if !matches!(scope, "project" | "user")
        || surface.is_some_and(|value| !matches!(value, "app" | "desktop" | "cli" | "ide"))
    {
        return Err("Select project or user scope and an installed Agent surface".into());
    }
    if scope == "user"
        || (agent == "antigravity"
            && !matches!(surface, Some("cli" | "ide"))
            && action != "rollback")
    {
        let mut outcome = report(
            agent,
            selected,
            "NOT_RUN",
            "UNSUPPORTED",
            "Automatic registration is unavailable for this scope or surface. Use the reviewed template in the installed client.",
        );
        outcome["scope"] = json!(scope);
        return Ok(outcome);
    }
    let root = paths::directory(root)?;
    let registry = policy::registry(&root)?;
    let work = workspace::workspace_path(&root, args.get("workspace").map(Path::new))?;
    let authority = GovernanceService::open(&root, &work)?;
    let receipt = if action == "rollback" {
        let id = args
            .get("receipt")
            .ok_or("Select a reviewed MCP registration receipt")?;
        let dir = receipt_path(id)?;
        let plan = read::<Plan>(&root, &format!("{dir}/plan.json"))?
            .ok_or("Unknown MCP registration receipt")?;
        Some((plan, id.as_str()))
    } else {
        None
    };
    let repository = args
        .get("repo")
        .cloned()
        .or_else(|| receipt.as_ref().map(|(plan, _)| plan.repository.clone()))
        .ok_or("Select a repository for project MCP configuration")?;
    let mut normalized = args.clone();
    if action == "inspect" {
        normalized.insert("dry-run".into(), "true".into());
    }
    normalized.insert(
        "workspace".into(),
        work.to_str()
            .ok_or("MCP workspace must be valid Unicode")?
            .into(),
    );
    let request = Request::new(&root, &work, &registry, &repository, agent, &normalized)?;
    if let Some((plan, id)) = receipt {
        return request.rollback(plan, id, &authority.snapshot_sha256()?, || {
            authority.ensure_current()
        });
    }
    let binary = current_executable()?;
    let load = || -> Result<(Manifest, BTreeMap<String, Vec<u8>>), String> {
        let (manifest, files) = match args.get("bundle") {
            Some(path) => bundle(&root, Path::new(path))?,
            None if action == "inspect" => render(&root, &work, &binary.0)?,
            None => return Err("Select a verified MCP template bundle".into()),
        };
        if manifest.workspace != work
            || (&manifest.binary, &manifest.binary_sha256) != (&binary.0, &binary.1)
            || manifest.authority_sha256.as_deref() != Some(&authority.snapshot_sha256()?)
        {
            return Err("MCP launch identity changed. Re-export using the running CNR executable and selected workspace.".into());
        }
        Ok((manifest, files))
    };
    let (manifest, files) = load()?;
    let template = &files[selected.template];
    if action == "inspect" {
        let (_, original) = before(&request.binding, selected.file)?;
        return Ok(match patch::plan(agent, original.as_deref(), template) {
            Ok(None) => report(
                agent,
                selected,
                "PASS",
                "CONFIG_REGISTERED",
                "The exact read-only launch entry is present in the bound project.",
            ),
            Ok(Some(_)) => report(
                agent,
                selected,
                "NOT_RUN",
                "NOT_RUN",
                "The CIRNETRULES entry is missing from the bound project configuration.",
            ),
            Err(reason) => report(agent, selected, "BLOCKED", "CONFLICT", &reason),
        });
    }
    let launch = hash(&manifest)?;
    request.install(&authority.snapshot_sha256()?, &launch, template, || {
        authority.ensure_current()?;
        let (current, _) = load()?;
        if hash(&current)? != launch {
            return Err("The MCP launch manifest changed after preview".into());
        }
        Ok(())
    })
}

#[cfg(test)]
pub(crate) fn test_project_registration(
    root: &Path,
    product: &Path,
    registry: &Value,
    agent: &str,
    relative: &str,
) {
    let work = root.parent().unwrap();
    let root = paths::directory(root).unwrap();
    let template: &[u8] = match agent {
        "codex" => b"[mcp_servers.cirnetrules]\ncommand = \"cnr\"\nargs = [\"mcp\"]\n",
        "claude" => {
            br#"{"mcpServers":{"cirnetrules":{"command":"cnr","args":["mcp"],"type":"stdio"}}}"#
        }
        "antigravity" => br#"{"mcpServers":{"cirnetrules":{"command":"cnr","args":["mcp"]}}}"#,
        _ => unreachable!(),
    };
    let mut options = BTreeMap::from([
        ("task".into(), "config".into()),
        ("actor".into(), "worker".into()),
        ("target".into(), product.to_str().unwrap().into()),
        ("dry-run".into(), "true".into()),
    ]);
    let original = std::fs::read(product.join(relative)).unwrap();
    let install = |args: &BTreeMap<String, String>| {
        Request::new(&root, work, registry, "product", agent, args)?.install(
            "fixture-authority",
            "fixture-launch",
            template,
            || Ok(()),
        )
    };
    let preview = install(&options).unwrap();
    assert_eq!(preview["registration"], "UPDATE");
    assert!(!preview.to_string().contains("fixture-secret"));
    assert_eq!(std::fs::read(product.join(relative)).unwrap(), original);
    options.remove("dry-run");
    options.insert("yes".into(), "true".into());
    let reviewed_digest = preview["plan_sha256"].as_str().unwrap();
    let reviewed_receipt = root.join(receipt_path(reviewed_digest).unwrap());
    options.insert("expect".into(), reviewed_digest.into());
    assert_eq!(
        install(&options).unwrap_err(),
        "Confirm the installed client's project trust separately, then use --trust-reviewed."
    );
    assert_eq!(std::fs::read(product.join(relative)).unwrap(), original);
    assert!(!reviewed_receipt.exists());

    options.insert("trust-reviewed".into(), "true".into());
    let mut user_scope = options.clone();
    user_scope.insert("agent".into(), agent.into());
    user_scope.insert("repo".into(), "product".into());
    user_scope.insert("scope".into(), "user".into());
    let handoff = super::run(&root, "install", &user_scope).unwrap();
    assert_eq!(handoff["status"], "NOT_RUN");
    assert_eq!(handoff["registration"], "UNSUPPORTED");
    assert_eq!(handoff["scope"], "user");
    assert_eq!(handoff["native_client"], "NOT_RUN");
    assert_eq!(std::fs::read(product.join(relative)).unwrap(), original);
    assert!(!reviewed_receipt.exists());

    let request = Request::new(&root, work, registry, "product", agent, &options).unwrap();
    for (authority, launch) in [
        ("fixture-authority", "rebuilt-launch"),
        ("changed-authority", "fixture-launch"),
    ] {
        assert_eq!(
            request
                .install(authority, launch, template, || Ok(()))
                .unwrap_err(),
            "Agent configuration preview changed. Review a new --dry-run."
        );
        assert_eq!(std::fs::read(product.join(relative)).unwrap(), original);
        assert!(!reviewed_receipt.exists());
    }

    let applied = install(&options).unwrap();
    assert_eq!(applied["registration"], "CONFIG_REGISTERED");
    let id = applied["receipt"].as_str().unwrap();
    let dir = receipt_path(id).unwrap();
    let receipt_path = format!("{dir}/plan.json");
    let read_plan = || read::<Plan>(&root, &receipt_path).unwrap().unwrap();
    assert!(
        !std::fs::read_to_string(root.join(&receipt_path))
            .unwrap()
            .contains("fixture-secret")
    );
    assert_eq!(install(&options).unwrap()["receipt"], id);
    let rollback = |args: &BTreeMap<String, String>| {
        Request::new(&root, work, registry, "product", agent, args)?.rollback(
            read_plan(),
            id,
            "fixture-authority",
            || Ok(()),
        )
    };
    options.remove("yes");
    options.remove("expect");
    options.insert("dry-run".into(), "true".into());
    let written = std::fs::read(product.join(relative)).unwrap();
    let mut changed = written.clone();
    changed.extend_from_slice(b" ");
    std::fs::write(product.join(relative), &changed).unwrap();
    assert_eq!(rollback(&options).unwrap()["registration"], "CONFLICT");
    assert_eq!(std::fs::read(product.join(relative)).unwrap(), changed);
    std::fs::write(product.join(relative), written).unwrap();
    let review = rollback(&options).unwrap();
    assert_eq!(review["registration"], "ROLLBACK_PREVIEW");
    options.remove("dry-run");
    options.insert("yes".into(), "true".into());
    options.insert(
        "expect".into(),
        review["plan_sha256"].as_str().unwrap().into(),
    );
    assert_eq!(rollback(&options).unwrap()["registration"], "ROLLED_BACK");
}
