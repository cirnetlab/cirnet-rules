use super::{Scope, safe};
use crate::{governance, policy, service, tasks, workspace};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub(super) struct Context {
    pub root: PathBuf,
    pub workspace: PathBuf,
    pub registry: Value,
    pub head: String,
    identity: (String, String, String),
    digest: String,
}

pub(super) fn root(scope: &Scope) -> Result<PathBuf, String> {
    let explicit = scope
        .governance_root
        .as_ref()
        .map(|path| path_text(path))
        .transpose()?;
    service::verified_root(&governance::governance_root(explicit)?)
}

impl Context {
    pub fn open(scope: &Scope) -> Result<Self, String> {
        let root = root(scope)?;
        let digest = service::policy_hash(&root)?;
        let identity = service::store_identity(&root)?;
        let registry = policy::registry(&root)?;
        let workspace = workspace::workspace_path(&root, scope.workspace.as_deref())?;
        let context = Self {
            root,
            workspace,
            registry,
            head: identity.2.clone(),
            identity,
            digest,
        };
        context.fresh()?;
        Ok(context)
    }

    pub fn fresh(&self) -> Result<(), String> {
        if service::verified_root(&self.root)? != self.root
            || service::store_identity(&self.root)? != self.identity
            || service::policy_hash(&self.root)? != self.digest
            || workspace::workspace_path(&self.root, Some(&self.workspace))? != self.workspace
        {
            return Err("Authority or workspace changed. Run a new check or preview.".into());
        }
        Ok(())
    }

    pub fn arguments(&self) -> Result<BTreeMap<String, String>, String> {
        Ok(BTreeMap::from([(
            "workspace".into(),
            path_text(&self.workspace)?,
        )]))
    }

    pub fn repo(&self, repo: &str) -> Result<&Value, String> {
        workspace::repository(&self.registry, repo)
    }

    pub fn bindings(&self, repo: &str) -> Result<Vec<String>, String> {
        let mut args = self.arguments()?;
        args.insert("repo".into(), repo.into());
        let report = workspace::run("workspace-binding-list", &self.root, &args, &self.registry)?;
        Ok(report["bindings"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|row| row["binding_id"].as_str().map(str::to_owned))
            .collect())
    }

    pub fn binding(
        &self,
        scope: &Scope,
        repo: &str,
    ) -> Result<(String, workspace::Binding), String> {
        self.repo(repo)?;
        let task = scope
            .task
            .as_ref()
            .map(|task| tasks::load_current(&self.root, task))
            .transpose()?;
        let pin = task.as_ref().map(|task| {
            task["descriptor"]["bindings"][repo]
                .as_str()
                .unwrap_or("default")
        });
        let id = if let Some(id) = scope.binding.as_deref().or(pin) {
            id.to_owned()
        } else {
            match self.bindings(repo)?.as_slice() {
                [id] => id.clone(),
                [] => return Err("No confirmed binding. Choose a candidate and review its identity before binding.".into()),
                _ => return Err("Several bindings exist. Select one explicitly with --binding.".into()),
            }
        };
        let binding =
            workspace::resolve_named(&self.root, &self.workspace, repo, &self.registry, &id)?;
        if let Some(task) = task {
            let pinned = workspace::resolve_task(
                &self.root,
                Some(&self.workspace),
                repo,
                &self.registry,
                &task,
            )?;
            if pinned.identity != binding.identity || pinned.path != binding.path {
                return Err("Selected binding differs from the task binding".into());
            }
        }
        if let Some(target) = &scope.target {
            let target = self.workspace.join(target);
            let selected = workspace::workspace_path(&self.root, Some(&target))?;
            if selected != binding.path {
                return Err("The selected target differs from the confirmed binding".into());
            }
        }
        Ok((id, binding))
    }

    pub fn public(&self) -> Value {
        json!({
            "governance_head": self.head,
            "policy_sha256": self.digest,
            "workspace_resolved": true
        })
    }
}

pub(super) fn path_text(path: &Path) -> Result<String, String> {
    let value = path.to_str().ok_or("Path must be valid Unicode")?;
    if value.chars().any(char::is_control) || safe(value) == "[redacted]" {
        return Err("Path contains unsafe text".into());
    }
    Ok(value.into())
}
