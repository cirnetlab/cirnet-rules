use crate::{docs, governance, policy, runner, workspace};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Application context shared by CLI commands and long-lived transports.
pub(crate) struct GovernanceService {
    root: PathBuf,
    workspace: PathBuf,
    identity: (String, String, String),
    policy_sha256: String,
}

fn store_identity(root: &Path) -> Result<(String, String, String), String> {
    let admin = runner::bounded_git_info(root, &["rev-parse", "--absolute-git-dir"])?;
    let common = runner::bounded_git_info(
        root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?;
    let head = runner::bounded_git_info(root, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    Ok((
        admin.trim().to_owned(),
        common.trim().to_owned(),
        head.trim().to_owned(),
    ))
}

fn policy_hash(root: &Path) -> Result<String, String> {
    let mut manifest = BTreeMap::new();
    for source in [
        "governance.yaml",
        "VERSION",
        "en/schemas/governance.schema.json",
        "en/schemas/cirnetrules-policy.schema.json",
    ] {
        let bytes = policy::read(&policy::safe_path(root, source)?)?;
        manifest.insert(source, docs::sha256(bytes.as_bytes()));
    }
    Ok(docs::sha256(
        &serde_json::to_vec(&manifest).map_err(|_| "Cannot encode service policy identity")?,
    ))
}

impl GovernanceService {
    pub(crate) fn open(root: &Path, workspace: &Path) -> Result<Self, String> {
        if !root.is_absolute() || !workspace.is_absolute() {
            return Err("Governance and workspace must be absolute directories".into());
        }
        let root = root
            .canonicalize()
            .map_err(|_| "Governance checkout is unavailable")?;
        let top = runner::bounded_git_info(&root, &["rev-parse", "--show-toplevel"])?;
        let origin = runner::bounded_git_info(
            &root,
            &[
                "config",
                "--local",
                "--no-includes",
                "--get-all",
                "remote.origin.url",
            ],
        )?;
        if Path::new(top.trim()).canonicalize().ok().as_ref() != Some(&root)
            || origin.trim().trim_end_matches(".git") != "https://github.com/cirnetlab/cirnet-rules"
        {
            return Err("Selected checkout does not match the governance repository".into());
        }
        let workspace = workspace::workspace_path(&root, Some(workspace))?;
        policy::registry(&root)?;
        Ok(Self {
            identity: store_identity(&root)?,
            policy_sha256: policy_hash(&root)?,
            root,
            workspace,
        })
    }

    pub(crate) fn call(
        &self,
        command: &str,
        mut args: BTreeMap<String, String>,
    ) -> Result<Value, String> {
        if args.keys().any(|key| {
            matches!(
                key.as_str(),
                "governance" | "governance-root" | "workspace" | "project"
            )
        }) {
            return Err("Tool arguments cannot select authority or workspace".into());
        }
        if store_identity(&self.root)? != self.identity
            || workspace::workspace_path(&self.root, Some(&self.workspace))? != self.workspace
            || policy_hash(&self.root)? != self.policy_sha256
        {
            return Err("Governance context changed. Restart the service.".into());
        }
        let work = self
            .workspace
            .to_str()
            .ok_or("Workspace path must be valid Unicode")?;
        if !matches!(command, "authority-inspect" | "audit-task" | "task-show") {
            args.insert("workspace".into(), work.into());
        }
        governance::execute_at(&self.root, command, args)
    }

    pub(crate) fn cli(
        root: PathBuf,
        command: &str,
        args: BTreeMap<String, String>,
    ) -> Result<Value, String> {
        if command.starts_with("coordination-") && args.contains_key("session") {
            return Err("Use an operator-prepared --session-file for coordination".into());
        }
        governance::execute_at(&root, command, args)
    }
}
