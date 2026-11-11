use crate::{docs, tasks, workspace};
use serde_json::{Value, json};
use std::path::Path;

pub(crate) fn task(root: &Path, id: &str) -> Result<Value, String> {
    let task = tasks::load(root, id)?;
    let authority = if tasks::load_current(root, id).is_ok() {
        "CURRENT"
    } else {
        "REVIEW_REQUIRED"
    };
    let records = task["evidence"]
        .as_array()
        .ok_or("Invalid task evidence store")?;
    let repository = public_label(&task["descriptor"]["primary_repo"]);
    let events: Vec<_> = records
        .iter()
        .map(|record| event(record, id, repository))
        .collect();
    let status = if events.iter().any(|event| event["integrity"] != "PASS") {
        "FAIL"
    } else if events.is_empty() {
        "NOT_RUN"
    } else {
        "PASS"
    };
    Ok(json!({
        "schema":1, "status":status, "task":id, "state":task["state"], "authority":authority,
        "events":events, "limitations":"Historical verification records; digest integrity and local actor assertions only."
    }))
}

fn public_label(value: &Value) -> Option<&str> {
    value.as_str().filter(|label| {
        !label.is_empty()
            && label.len() <= 96
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-_@.".contains(&byte))
            && !workspace::contains_known_secrets(label)
    })
}

fn sha256(value: &Value) -> Option<&str> {
    value.as_str().filter(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn event(record: &Value, id: &str, repository: Option<&str>) -> Value {
    let mut unsigned = record.clone();
    let hash = unsigned
        .as_object_mut()
        .and_then(|value| value.remove("receipt_sha256"));
    let integrity = serde_json::to_vec(&unsigned)
        .ok()
        .is_some_and(|bytes| hash == Some(json!(docs::sha256(&bytes))));
    if !integrity {
        return json!({"operation":"task-verify","task":id,"integrity":"FAIL"});
    }
    json!({
        "operation":"task-verify", "task":id, "repository":repository,
        "revision":record["task_revision"].as_u64(), "authority_sha256":sha256(&record["policy_sha256"]),
        "before_sha256":sha256(&record["snapshot_sha256"]), "after_sha256":sha256(&record["observed_after_sha256"]),
        "actor":public_label(&record["actor"]), "result":public_label(&record["result"]),
        "time_unix_seconds":record["time_unix_seconds"].as_u64(),
        "evidence_source":public_label(&record["producer"]), "evidence_kind":public_label(&record["verification_kind"]),
        "receipt_sha256":hash, "integrity":"PASS"
    })
}

#[cfg(test)]
#[test]
fn task_audit_does_not_emit_raw_or_corrupted_evidence() {
    let mut record =
        json!({"actor":"ghp_fixture_not_a_real_key","steps":[{"output":"private fixture bytes"}]});
    assert_eq!(event(&record, "TASK", None)["integrity"], "FAIL");
    record["receipt_sha256"] = json!(docs::sha256(&serde_json::to_vec(&record).unwrap()));
    let visible = event(&record, "TASK", None);
    assert_eq!(visible["integrity"], "PASS");
    assert!(visible["actor"].is_null());
    assert!(!visible.to_string().contains("private fixture bytes"));
}
