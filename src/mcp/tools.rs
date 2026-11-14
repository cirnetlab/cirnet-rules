use crate::{docs, service::GovernanceService, workspace};
use rmcp::model::{JsonObject, Tool, ToolAnnotations};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc};

const LABEL: &str = "^[A-Za-z0-9][A-Za-z0-9_-]{0,79}$";
const SHA256: &str = "^[0-9a-f]{64}$";
const CURSOR: &str = "^[0-9a-f]{64}\\.[1-9][0-9]{0,5}$";
const PAGE_SIZE: u64 = 20;

pub(super) struct Route {
    pub(super) tool: Tool,
    validator: jsonschema::Validator,
    command: &'static str,
    write: bool,
}

fn input(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})
}

fn output_schema(command: &str) -> Arc<JsonObject> {
    let objects = json!({"type":"array","items":{"type":"object"}});
    let revision = json!({"type":"integer","minimum":1});
    let lease = json!({"type":["object","null"]});
    let properties = match command {
        "workspace-discover" => {
            json!({"repositories":objects,"candidates":objects,"next_cursor":{"type":["string","null"]}})
        }
        "authority-context" => {
            json!({"repository":{"type":"string"},"authority_sha256":{"pattern":SHA256},"included":objects,"excluded":objects,"deferred":objects})
        }
        "task-context" => {
            let scope_map = json!({
                "type":"object","additionalProperties":{"type":"array","items":{"type":"string"}}
            });
            let string_array = json!({"type":"array","items":{"type":"string"}});
            json!({
                "scope":{"const":"read-only context"},
                "spec_revision":revision,
                "primary_repo":{"type":"string"},
                "descriptor_sha256":{"type":"string","pattern":SHA256},
                "approval_epoch":{"type":["integer","null"],"minimum":1},
                "task_scope":input(json!({
                    "read_scope":scope_map,"write_scope":scope_map,
                    "verification_scope":scope_map,"documentation_scope":scope_map
                }), &["read_scope","write_scope","verification_scope","documentation_scope"]),
                "authorization_summary":input(json!({
                    "actions":string_array,"expires_at":{"type":["integer","null"],"minimum":1},
                    "status":{"enum":["WRITE_NOT_AUTHORIZED","WRITE_REQUIRES_ACTIVE_TASK","WRITE_REQUIRES_GRANT_EVALUATION"]}
                }), &["actions","expires_at","status"]),
                "execution_bindings":{"type":"object","additionalProperties":input(json!({
                    "binding_id":{"type":"string"},"identity":{"type":"string","pattern":SHA256},
                    "commit":{"type":"string"},"frozen":{"type":"boolean"}
                }), &["binding_id","identity","commit","frozen"])},
                "acceptance":string_array,
                "allowed_command_ids":string_array,
                "sources":objects,
                "worktrees":{"type":"object"},
                "authority_snapshot":{"type":["object","null"]}
            })
        }
        "diff-check" => {
            json!({"sha256":{"pattern":SHA256},"repositories":objects,"violations":{"type":"array"}})
        }
        "audit-task" => json!({"events":objects,"next_cursor":{"type":["string","null"]}}),
        _ => json!({"revision":revision,"state":{"type":"string"},"lease":lease}),
    };
    let required: Vec<_> = properties
        .as_object()
        .expect("output data properties")
        .keys()
        .collect();
    let output = json!({
        "type":"object","additionalProperties":false,"required":["status"],
        "properties":{
            "status":{"enum":["PASS","FAIL","BLOCKED","NOT_RUN"]},
            "data":{"type":"object","description":"The CIRNETRULES domain result"},
            "reason":{"type":"string","description":"Redacted tool error, when applicable"}
        },
        "if":{"properties":{"status":{"const":"PASS"}}},
        "then":{"required":["data"],"properties":{"data":{"required":required,"properties":properties}}}
    });
    Arc::new(output.as_object().expect("output schema object").clone())
}
pub(super) fn routes(coordinated: bool) -> Result<Vec<Route>, String> {
    let label = json!({"type":"string","pattern":LABEL,"maxLength":80});
    let task_id = json!({"type":"string","pattern":LABEL,"maxLength":64});
    let digest = json!({"type":"string","pattern":SHA256});
    let number = json!({"type":"integer","minimum":1,"maximum":9_007_199_254_740_991_u64});
    let limit = json!({"type":"integer","minimum":1,"maximum":40,"default":PAGE_SIZE});
    let cursor = json!({"type":"string","pattern":CURSOR,"maxLength":72});
    let mut routes = Vec::new();
    let mut add = |name: &'static str,
                   description: &'static str,
                   command: &'static str,
                   write: bool,
                   schema: Value|
     -> Result<(), String> {
        let validator =
            jsonschema::validator_for(&schema).map_err(|_| "Invalid MCP input schema")?;
        let tool = Tool::new(
            name,
            description,
            schema.as_object().expect("input schema object").clone(),
        )
        .with_raw_output_schema(output_schema(command))
        .with_annotations(ToolAnnotations::new().read_only(!write).open_world(false));
        routes.push(Route {
            tool,
            validator,
            command,
            write,
        });
        Ok(())
    };
    add(
        "cirnet_audit_read",
        "Read a page of redacted task audit events.",
        "audit-task",
        false,
        input(
            json!({"task_id":task_id,"limit":limit,"cursor":cursor}),
            &["task_id"],
        ),
    )?;
    add(
        "cirnet_diff_check",
        "Check the current scoped diff against an expected snapshot digest.",
        "diff-check",
        false,
        input(
            json!({"task_id":task_id,"expected_snapshot_sha256":digest}),
            &["task_id", "expected_snapshot_sha256"],
        ),
    )?;
    add(
        "cirnet_policy_lookup",
        "Read policy selection and provenance for a registered repository or task.",
        "authority-context",
        false,
        input(
            json!({"repo_id":label,"task_id":task_id,"asset_id":label,"kind":{
                "enum":["rule","policy","skill","profile","reference","hook","command"]
            }}),
            &["repo_id"],
        ),
    )?;
    add(
        "cirnet_repositories_discover",
        "Read a page of registered repository metadata and local checkout candidates.",
        "workspace-discover",
        false,
        input(
            json!({"namespace":{"const":"workspace"},"limit":limit,"cursor":cursor}),
            &[],
        ),
    )?;
    add(
        "cirnet_task_context",
        "Read the bounded context, exact task scopes and authorization assessment for the specified current revision.",
        "task-context",
        false,
        input(
            json!({"task_id":task_id,"expected_revision":number}),
            &["task_id", "expected_revision"],
        ),
    )?;
    add(
        "cirnet_task_status",
        "Read task state, revision and lease summary.",
        "coordination-status",
        false,
        input(json!({"task_id":task_id}), &["task_id"]),
    )?;
    if coordinated {
        add(
            "cirnet_task_claim",
            "Claim the startup-authorized task at its expected revision.",
            "coordination-claim",
            true,
            input(
                json!({"task_id":task_id,"expected_revision":number,"request_id":label}),
                &["task_id", "expected_revision", "request_id"],
            ),
        )?;
        let lease_properties = json!({"task_id":task_id,"generation":number,"request_id":label,"expected_revision":number});
        let lease_fields = ["task_id", "generation", "request_id"];
        add(
            "cirnet_task_renew",
            "Renew the current authorized lease generation.",
            "coordination-renew",
            true,
            input(lease_properties.clone(), &lease_fields),
        )?;
        add(
            "cirnet_task_release",
            "Release the current authorized lease generation.",
            "coordination-release",
            true,
            input(lease_properties, &lease_fields),
        )?;
        let text = |max| json!({"type":"string","minLength":1,"maxLength":max});
        let report = input(
            json!({
                "summary": text(2048),
                "changed_paths":{"type":"array","maxItems":128,"items":input(json!({"repo":label,"path":text(512)}), &["repo","path"])},
                "blockers":{"type":"array","maxItems":16,"items":text(512)},
                "artifacts":{"type":"array","maxItems":8,"uniqueItems":true,"items":digest},
                "commands":{"type":"array","maxItems":32,"items":input(json!({
                    "command":label,"exit_code":{"type":"integer","minimum":-2_147_483_648_i64,"maximum":2_147_483_647_i64},"result":text(128),"log_sha256":digest
                }), &["command"])}
            }),
            &["summary"],
        );
        add(
            "cirnet_task_report",
            "Submit a bounded report under the current authorized lease generation.",
            "coordination-report",
            true,
            input(
                json!({"task_id":task_id,"generation":number,"expected_revision":number,"request_id":label,"report":report}),
                &[
                    "task_id",
                    "generation",
                    "expected_revision",
                    "request_id",
                    "report",
                ],
            ),
        )?;
    }
    routes.sort_unstable_by(|left, right| left.tool.name.cmp(&right.tool.name));
    Ok(routes)
}

impl Route {
    pub(super) fn accepts(&self, arguments: &Value) -> bool {
        self.validator.is_valid(arguments)
            && arguments
                .get("task_id")
                .and_then(Value::as_str)
                .is_none_or(|id| crate::tasks::valid_id(id).is_ok())
            && ["expected_revision", "generation", "limit"]
                .iter()
                .all(|key| {
                    arguments
                        .get(key)
                        .is_none_or(|value| value.as_u64().is_some())
                })
    }

    pub(super) fn execute(
        &self,
        service: &GovernanceService,
        args: &JsonObject,
        capability: Option<&str>,
        cursor_key: &str,
    ) -> Result<Value, String> {
        let mut options = BTreeMap::new();
        for (argument, flag) in [
            ("task_id", "task"),
            ("repo_id", "repo"),
            ("request_id", "request-id"),
            ("generation", "generation"),
            ("report", "report"),
        ] {
            if let Some(value) = args.get(argument) {
                options.insert(flag.into(), scalar(value));
            }
        }
        if self.write {
            options.insert(
                "session".into(),
                capability
                    .ok_or("No authorized coordination session")?
                    .into(),
            );
            if let Some(revision) = args.get("expected_revision") {
                options.insert("revision".into(), scalar(revision));
            }
        }
        if self.command == "task-context" {
            let status = service.call("coordination-status", options.clone())?;
            if status["revision"] != args["expected_revision"] {
                return Err("Task revision changed. Reload task status.".into());
            }
        }
        let mut result = service.call(self.command, options)?;
        match self.command {
            "audit-task" => {
                result = result
                    .get_mut("record")
                    .ok_or("Audit record unavailable")?
                    .take();
                paginate(&mut result, &["events"], args, cursor_key, self.command)?;
            }
            "workspace-discover" => paginate(
                &mut result,
                &["repositories", "candidates"],
                args,
                cursor_key,
                self.command,
            )?,
            "authority-context" => filter_policy(&mut result, args)?,
            "task-context"
                if result["status"] == "PASS"
                    && result["spec_revision"] != args["expected_revision"] =>
            {
                return Err("Task revision changed. Reload task status.".into());
            }
            "diff-check"
                if !result["sha256"].is_null()
                    && result["sha256"] != args["expected_snapshot_sha256"] =>
            {
                return Err("Scoped diff snapshot changed. Reload the expected digest.".into());
            }
            _ => {}
        }
        if self.command == "task-context"
            && result["status"] == "PASS"
            && capability.is_some()
            && [
                "primary_repo",
                "descriptor_sha256",
                "task_scope",
                "authorization_summary",
                "execution_bindings",
                "acceptance",
                "allowed_command_ids",
            ]
            .iter()
            .any(|field| contains_capability(&result[*field].to_string(), capability))
        {
            return Err("Exact task context metadata contains sensitive text".into());
        }
        redact(&mut result, capability);
        Ok(result)
    }
}

fn scalar(value: &Value) -> String {
    value
        .as_str()
        .map_or_else(|| value.to_string(), str::to_owned)
}

fn filter_policy(result: &mut Value, args: &JsonObject) -> Result<(), String> {
    let mut found = false;
    for field in ["included", "deferred", "excluded"] {
        let Some(rows) = result[field].as_array_mut() else {
            continue;
        };
        rows.retain(|row| {
            args.get("asset_id").is_none_or(|id| row["id"] == *id)
                && args.get("kind").is_none_or(|kind| row["kind"] == *kind)
        });
        found |= !rows.is_empty();
    }
    if !found && args.contains_key("asset_id") {
        return Err("Asset was not found in the selected authority".into());
    }
    Ok(())
}

fn paginate(
    result: &mut Value,
    fields: &[&str],
    args: &JsonObject,
    key: &str,
    command: &str,
) -> Result<(), String> {
    let snapshot = docs::sha256(result.to_string().as_bytes());
    let limit = args
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(PAGE_SIZE) as usize;
    let cursor_for = |offset| {
        let scope = json!([
            key,
            command,
            snapshot,
            args.get("task_id"),
            "workspace",
            limit,
            offset
        ]);
        format!("{}.{offset}", docs::sha256(scope.to_string().as_bytes()))
    };
    let offset: usize = match args.get("cursor").and_then(Value::as_str) {
        None => 0,
        Some(cursor) => {
            let offset = cursor
                .rsplit_once('.')
                .and_then(|(_, offset)| offset.parse().ok())
                .ok_or("Invalid page cursor")?;
            if cursor_for(offset) != cursor {
                return Err("Page snapshot or scope changed. Restart pagination.".into());
            }
            offset
        }
    };
    let end = offset.saturating_add(limit);
    let mut start = 0usize;
    for field in fields {
        let rows = result[field]
            .as_array_mut()
            .ok_or("Page source unavailable")?;
        let local_start = offset.saturating_sub(start);
        let local_end = end.saturating_sub(start);
        start += rows.len();
        *rows = rows
            .drain(..)
            .skip(local_start)
            .take(local_end.saturating_sub(local_start))
            .collect();
    }
    let next = (end < start).then(|| cursor_for(end));
    result["next_cursor"] = json!(next);
    Ok(())
}

pub(super) fn redact(value: &mut Value, capability: Option<&str>) {
    match value {
        Value::String(text)
            if workspace::contains_known_secrets(text) || contains_capability(text, capability) =>
        {
            *text = "[redacted]".into()
        }
        Value::Array(items) => items.iter_mut().for_each(|value| redact(value, capability)),
        Value::Object(items) => {
            items.retain(|key, _| {
                !workspace::contains_known_secrets(key) && !contains_capability(key, capability)
            });
            items
                .values_mut()
                .for_each(|value| redact(value, capability));
        }
        _ => {}
    }
}

pub(super) fn contains_capability(text: &str, capability: Option<&str>) -> bool {
    capability
        .and_then(|token| token.split_once('.'))
        .is_some_and(|(_, secret)| text.contains(secret))
}
