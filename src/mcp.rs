mod probe;
mod tools;
mod transport;

pub use probe::{ProbeFailure, probe, probe_with_cancel};
pub(crate) use transport::unique_json;

use crate::{coordination, service::GovernanceService};
use rmcp::{
    ErrorData, RoleServer, ServerHandler, ServiceExt,
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, Implementation, ListToolsResult,
        PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerConfig,
    },
    service::{RequestContext, ServerInitializeError},
};
use serde_json::{Value, json};
use std::{borrow::Cow, path::Path, sync::Arc, time::Duration};
use tokio::sync::Semaphore;
use tools::{Route, contains_capability, redact, routes};
use transport::BoundedStdio;

const WORKERS: usize = 4;
const RESULT_BYTES: usize = 192 * 1024;
const HANDSHAKE_DEADLINE: Duration = Duration::from_secs(30);
const WORK_DEADLINE: Duration = Duration::from_secs(12);
const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);

struct Server {
    service: Arc<GovernanceService>,
    capability: Option<Arc<str>>,
    cursor_key: Arc<str>,
    routes: Arc<[Route]>,
    workers: Arc<Semaphore>,
}

/// Serve one local STDIO client with roots and access fixed at startup.
pub fn serve(
    root: &Path,
    workspace: &Path,
    access: &str,
    capability: Option<&Path>,
) -> Result<(), String> {
    let service = GovernanceService::open(root, workspace)?;
    let capability = match (access, capability) {
        ("read-only", None) => None,
        ("read-only", Some(_)) => return Err("A session file requires coordinated access".into()),
        ("coordinated", None) => {
            return Err("Coordinated access requires a prepared local session file".into());
        }
        ("coordinated", Some(path)) => Some(open_capability(root, workspace, path)?.into()),
        _ => return Err("MCP access must be read-only or coordinated".into()),
    };
    let mut key = [0u8; 32];
    getrandom::fill(&mut key).map_err(|_| "Cannot create local MCP cursor scope")?;
    let server = Server {
        service: Arc::new(service),
        cursor_key: crate::docs::sha256(&key).into(),
        routes: routes(capability.is_some())?.into(),
        capability,
        workers: Arc::new(Semaphore::new(WORKERS)),
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        // Windows STDIO uses Tokio's blocking pool. Keep slots for both pipes.
        .max_blocking_threads(WORKERS + 2)
        .build()
        .map_err(|_| "Cannot start local MCP runtime")?;
    let result = runtime.block_on(async {
        let connection =
            tokio::time::timeout(HANDSHAKE_DEADLINE, server.serve(BoundedStdio::new()))
                .await
                .map_err(|_| "MCP handshake timed out")?;
        match connection {
            Ok(connection) => connection
                .waiting()
                .await
                .map(|_| ())
                .map_err(|_| "MCP service failed".into()),
            Err(ServerInitializeError::ConnectionClosed(_)) => Ok(()),
            Err(_) => Err("MCP handshake failed".into()),
        }
    });
    runtime.shutdown_timeout(SHUTDOWN_GRACE);
    result
}

fn open_capability(root: &Path, workspace: &Path, file: &Path) -> Result<String, String> {
    if !file.is_absolute() {
        return Err("MCP session file must be an absolute path".into());
    }
    let canonical = root
        .canonicalize()
        .map_err(|_| "Governance checkout is unavailable")?;
    let relative = file
        .strip_prefix(root)
        .or_else(|_| file.strip_prefix(&canonical))
        .map_err(|_| "MCP session file must be inside the governance checkout")?;
    let relative = relative
        .to_str()
        .ok_or("MCP session file must use a valid Unicode path")?;
    let token = coordination::session_token(&canonical, &relative.replace('\\', "/"))?;
    coordination::check_session(&canonical, Some(workspace), &token)?;
    Ok(token)
}

impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("cnr", env!("CARGO_PKG_VERSION")))
            .with_protocol_version(ProtocolVersion::V_2026_07_28)
    }

    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        const VERSIONS: &[ProtocolVersion] =
            &[ProtocolVersion::V_2025_11_25, ProtocolVersion::V_2026_07_28];
        Cow::Borrowed(VERSIONS)
    }

    async fn list_tools(
        &self,
        params: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        if params.is_some_and(|params| params.cursor.is_some()) {
            return Err(ErrorData::invalid_params("Tool list has one page", None));
        }
        Ok(ListToolsResult::with_all_items(
            self.routes.iter().map(|route| route.tool.clone()).collect(),
        ))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let index = self
            .routes
            .iter()
            .position(|route| route.tool.name == request.name)
            .ok_or_else(|| {
                ErrorData::invalid_params("Tool is unavailable in this profile", None)
            })?;
        let args = Value::Object(request.arguments.unwrap_or_default());
        if request.input_responses.is_some()
            || request.request_state.is_some()
            || !self.routes[index].accepts(&args)
        {
            return Err(ErrorData::invalid_params(
                "Arguments do not match the tool input schema",
                None,
            ));
        }
        if contains_capability(&args.to_string(), self.capability.as_deref()) {
            return Err(ErrorData::invalid_params(
                "Credential material is not a tool argument",
                None,
            ));
        }
        let Ok(permit) = self.workers.clone().try_acquire_owned() else {
            return Ok(
                blocked("MCP workers are busy. Retry after the active requests finish.").into(),
            );
        };
        let routes = self.routes.clone();
        let service = self.service.clone();
        let capability = self.capability.clone();
        let cursor_key = self.cursor_key.clone();
        let cancellation = context.ct.clone();
        let worker = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            if cancellation.is_cancelled() {
                return blocked("MCP request was cancelled before dispatch");
            }
            let args = args.as_object().expect("validated arguments object");
            let result = routes[index].execute(&service, args, capability.as_deref(), &cursor_key);
            match result {
                Ok(data) => domain_result(data),
                Err(reason) => {
                    let mut reason = Value::String(reason);
                    redact(&mut reason, capability.as_deref());
                    blocked(reason.as_str().unwrap_or("Governance operation blocked"))
                }
            }
        });
        let response = tokio::select! {
            biased;
            () = context.ct.cancelled() => blocked("MCP request cancelled"),
            outcome = tokio::time::timeout(WORK_DEADLINE, worker) => match outcome {
                Ok(Ok(response)) => response,
                Ok(Err(_)) => blocked("MCP worker failed"),
                Err(_) => blocked("MCP request deadline exceeded. Check task status before retrying."),
            }
        };
        Ok(response.into())
    }
}

fn domain_result(data: Value) -> CallToolResult {
    let Some(status @ ("PASS" | "FAIL" | "BLOCKED" | "NOT_RUN")) = data["status"].as_str() else {
        return blocked("Governance service returned an invalid result");
    };
    let is_error = status != "PASS";
    let value = json!({"status": status, "data": data});
    if serde_json::to_vec(&value).map_or(true, |bytes| bytes.len() > RESULT_BYTES) {
        return blocked(
            "Result exceeds the MCP byte budget. Narrow the task scope or request a smaller page.",
        );
    }
    if is_error {
        CallToolResult::structured_error(value)
    } else {
        CallToolResult::structured(value)
    }
}

fn blocked(reason: &str) -> CallToolResult {
    let reason = if reason.len() <= 2048 {
        reason
    } else {
        "Governance operation blocked"
    };
    CallToolResult::structured_error(json!({"status":"BLOCKED","reason":reason}))
}
