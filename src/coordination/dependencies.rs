use super::{
    coordinate,
    model::{Dependency, Output, digest},
    revision,
};
use crate::{runner, tasks};
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};

fn historical_current(root: &Path, id: &str) -> Result<Value, String> {
    let record = tasks::load(root, id)?;
    tasks::current(root, &record)?;
    Ok(record)
}

fn rows(value: &Value) -> impl Iterator<Item = &Value> {
    value["repositories"].as_array().into_iter().flatten()
}

fn items(value: &Value) -> impl Iterator<Item = &Value> {
    value.as_array().into_iter().flatten()
}

pub(super) fn evidence_hash(task: &Value, source: &Value) -> Result<String, String> {
    let mut receipt = items(&task["evidence"])
        .last()
        .cloned()
        .ok_or("Verification receipt is missing")?;
    let signature = receipt
        .as_object_mut()
        .and_then(|record| record.remove("receipt_sha256"));
    let digest = digest(&receipt)?;
    if signature.as_ref().and_then(Value::as_str) != Some(digest.as_str())
        || receipt["result"] != "PASS"
        || receipt["producer"] != "cirnetrules-controlled-runner"
        || receipt["snapshot_sha256"] != source["sha256"]
        || receipt["observed_after_sha256"] != source["sha256"]
        || receipt["approval_epoch"] != task["approval_epoch"]
        || receipt["policy_sha256"] != task["policy_sha256"]
        || receipt["spec_sha256"] != task["spec_sha256"]
        || receipt["plan_sha256"] != task["plan_sha256"]
        || !receipt["steps"].as_array().is_some_and(|steps| {
            !steps.is_empty() && steps.iter().all(|step| step["status"] == "PASS")
        })
    {
        return Err("Verification receipt is stale or invalid".into());
    }
    check_dependency_step_inputs(
        task,
        receipt["steps"]
            .as_array()
            .ok_or("Invalid verification steps")?,
    )?;
    Ok(digest)
}

fn outputs(snapshot: &Value) -> Result<Vec<Output>, String> {
    let mut outputs = Vec::new();
    for row in rows(snapshot) {
        let repository = row["repo"].as_str().ok_or("Invalid producer repository")?;
        for entry in items(&row["changed_paths"]) {
            let path = entry.as_str().ok_or("Invalid producer output")?;
            outputs.push(Output {
                repository: repository.to_owned(),
                path: path.to_owned(),
                sha256: row["inputs"][path]["sha256"].as_str().map(str::to_owned),
            });
        }
    }
    Ok(outputs)
}

fn selected(
    task: &Value,
    id: &str,
    output: &[Output],
    consumer: Option<&Value>,
) -> Result<Vec<Output>, String> {
    let mut result = Vec::new();
    for link in items(&task["descriptor"]["dependency_inputs"]).filter(|link| link["task_id"] == id)
    {
        let producer_repo = link["producer_repo"]
            .as_str()
            .ok_or("Missing producer repository")?;
        let producer_path = link["producer_path"]
            .as_str()
            .ok_or("Missing producer output")?;
        let consumer_repo = link["consumer_repo"]
            .as_str()
            .ok_or("Missing consumer repository")?;
        let consumer_path = link["consumer_path"]
            .as_str()
            .ok_or("Missing consumer input")?;
        let expected = output
            .iter()
            .find(|entry| entry.repository == producer_repo && entry.path == producer_path)
            .and_then(|entry| entry.sha256.as_deref())
            .ok_or_else(|| format!("Dependency {id} did not seal the selected output"))?;
        let input = consumer
            .and_then(|snapshot| rows(snapshot).find(|row| row["repo"] == consumer_repo))
            .and_then(|row| row["inputs"][consumer_path]["sha256"].as_str());
        if input != Some(expected) {
            return Err(format!(
                "Dependency {id} selected input bytes do not match the accepted output"
            ));
        }
        result.push(Output {
            repository: consumer_repo.to_owned(),
            path: consumer_path.to_owned(),
            sha256: Some(expected.to_owned()),
        });
    }
    Ok(result)
}

struct Observation<'a> {
    root: &'a Path,
    work: Option<&'a Path>,
    registry: &'a Value,
    inspected: usize,
    active: BTreeSet<String>,
}

impl Observation<'_> {
    fn capture(
        &mut self,
        task: &Value,
        snapshot: Option<&Value>,
        children: bool,
    ) -> Result<Vec<Dependency>, String> {
        let mut result = Vec::new();
        let descriptor = &task["descriptor"];
        for reference in items(&descriptor["depends_on"]).chain(
            children
                .then(|| items(&descriptor["required_children"]))
                .into_iter()
                .flatten(),
        ) {
            let id = reference.as_str().ok_or("Invalid task dependency ID")?;
            self.inspected += 1;
            if self.inspected > 32 || !self.active.insert(id.to_ascii_lowercase()) {
                return Err(
                    "Dependency evidence traversal exceeds its budget or forms a cycle".into(),
                );
            }
            let producer = historical_current(self.root, id)
                .map_err(|_| format!("Dependency {id} has stale or unavailable authority"))?;
            if producer["state"] != "CLOSED"
                || (items(&descriptor["required_children"]).any(|value| value == id)
                    && producer["descriptor"]["parent"] != descriptor["id"])
            {
                return Err(format!(
                    "Dependency {id} is pending or has an invalid parent"
                ));
            }
            let source = runner::snapshot(self.root, &producer, self.registry, self.work)
                .map_err(|_| format!("Dependency {id} has unavailable source inputs"))?;
            if source["status"] != "PASS"
                || producer["acceptance"]["snapshot_sha256"] != source["sha256"]
            {
                return Err(format!("Dependency {id} accepted source has changed"));
            }
            let evidence_sha256 = evidence_hash(&producer, &source)
                .map_err(|_| format!("Dependency {id} has stale verification evidence"))?;
            let nested = self.capture(&producer, Some(&source), true)?;
            if coordinate(&producer)?.dependencies != nested {
                return Err(format!(
                    "Dependency {id} has stale predecessor or child evidence"
                ));
            }
            let outputs = outputs(&source)?;
            let selected_inputs = selected(task, id, &outputs, snapshot)?;
            result.push(Dependency {
                task_id: id.to_owned(),
                revision: revision(&producer)?,
                approval_epoch: producer["approval_epoch"]
                    .as_u64()
                    .ok_or("Invalid predecessor approval epoch")?,
                source_sha256: source["sha256"]
                    .as_str()
                    .ok_or("Invalid predecessor snapshot")?
                    .to_owned(),
                evidence_sha256,
                outputs,
                selected_inputs,
            });
            self.active.remove(&id.to_ascii_lowercase());
        }
        Ok(result)
    }
}

pub(super) fn capture(
    root: &Path,
    work: Option<&Path>,
    registry: &Value,
    task: &Value,
    snapshot: Option<&Value>,
    children: bool,
) -> Result<Vec<Dependency>, String> {
    let selected = if snapshot.is_none() && !task["descriptor"]["dependency_inputs"].is_null() {
        Some(runner::snapshot(root, task, registry, work)?)
    } else {
        None
    };
    Observation {
        root,
        work,
        registry,
        inspected: 0,
        active: BTreeSet::new(),
    }
    .capture(task, snapshot.or(selected.as_ref()), children)
}

pub(super) fn still_current(root: &Path, dependencies: &[Dependency]) -> Result<(), String> {
    let mut pending = dependencies.to_vec();
    let mut visited = BTreeSet::new();
    let mut inspected = 0;
    while let Some(observed) = pending.pop() {
        let id = &observed.task_id;
        inspected += 1;
        if inspected > 32 {
            return Err("Dependency commit traversal exceeds its budget".into());
        }
        let record = historical_current(root, id)
            .map_err(|_| format!("Dependency {id} changed while preparing task evidence"))?;
        if record["state"] != "CLOSED"
            || revision(&record)? != observed.revision
            || record["approval_epoch"] != observed.approval_epoch
        {
            return Err(format!(
                "Dependency {id} changed while preparing task evidence"
            ));
        }
        if visited.insert(id.to_ascii_lowercase()) {
            pending.extend(coordinate(&record)?.dependencies);
        }
    }
    Ok(())
}

pub(crate) fn validate_selection(descriptor: &Value) -> Result<(), String> {
    let mut selected = BTreeSet::new();
    for link in items(&descriptor["dependency_inputs"]) {
        let id = link["task_id"]
            .as_str()
            .ok_or("Missing dependency selection task")?;
        if !items(&descriptor["depends_on"]).any(|value| value == id) {
            return Err("Selected producer must be a direct task dependency".into());
        }
        let source = link["producer_path"]
            .as_str()
            .ok_or("Missing producer output")?;
        let repo = link["consumer_repo"]
            .as_str()
            .ok_or("Missing consumer repository")?;
        let path = link["consumer_path"]
            .as_str()
            .ok_or("Missing consumer input")?;
        tasks::relative(source)?;
        tasks::relative(path)?;
        if !["read_scope", "verification_scope"]
            .iter()
            .any(|scope| items(&descriptor[scope][repo]).any(|value| value == path))
            || !selected.insert((repo, path))
        {
            return Err(
                "Dependency input must be unique and included in the consumer's scope".into(),
            );
        }
    }
    Ok(())
}

pub(crate) fn check_dependency_step_inputs(task: &Value, results: &[Value]) -> Result<(), String> {
    for selected in items(&task["descriptor"]["dependency_inputs"]) {
        let observed = results.iter().any(|step| {
            step["status"] == "PASS"
                && step["plan"]["mode"] == "VERIFY"
                && matches!(
                    step["plan"]["program"].as_str(),
                    Some("check-json" | "check-utf8")
                )
                && step["plan"]["repository"] == selected["consumer_repo"]
                && items(&step["plan"]["paths"]).any(|path| path == &selected["consumer_path"])
        });
        if !observed {
            return Err(
                "Selected dependency bytes were not read by an approved verification step".into(),
            );
        }
    }
    Ok(())
}
