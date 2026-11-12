use super::{RESULT_BYTES, tools::routes, unique_json};
use crate::{
    agent_config::mcp_config,
    execution::{Cancellation, process},
    service::GovernanceService,
};
use rmcp::model::{
    CallToolRequest, CallToolRequestParams, ClientCapabilities, ClientJsonRpcMessage,
    ClientRequest, DiscoverRequest, DiscoverRequestParams, GetMeta, Implementation,
    InitializeRequest, InitializeRequestParams, InitializedNotification, ListToolsRequest,
    PaginatedRequestParams, ProtocolVersion, ServerJsonRpcMessage, ServerResult,
};
use serde_json::{Value, json};
use std::{
    io::{self, BufRead, BufReader, Read, Write},
    path::Path,
    process::{Child, Command},
    sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender},
    thread,
    time::{Duration, Instant},
};

const PROBE_TIMEOUT: Duration = Duration::from_secs(30);
const POLL_INTERVAL: Duration = Duration::from_millis(25);
// The SDK includes the structured result and an escaped text representation.
const FRAME_LIMIT: u64 = (RESULT_BYTES * 3 + 4096) as u64;

#[derive(Debug)]
pub struct ProbeFailure {
    pub code: &'static str,
    pub message: &'static str,
}

impl ProbeFailure {
    const fn new(code: &'static str, message: &'static str) -> Self {
        Self { code, message }
    }
}

impl From<String> for ProbeFailure {
    fn from(_: String) -> Self {
        Self::new(
            "PROCESS_ERROR",
            "Could not start or clean up the owned MCP process.",
        )
    }
}

struct Pipe {
    input: SyncSender<ClientJsonRpcMessage>,
    frames: Receiver<Result<ServerJsonRpcMessage, ProbeFailure>>,
    deadline: Instant,
    next_id: u64,
    current: bool,
}

fn frame(reader: &mut impl BufRead) -> Result<Option<ServerJsonRpcMessage>, ProbeFailure> {
    let mut bytes = Vec::new();
    let length = reader
        .take(FRAME_LIMIT + 1)
        .read_until(b'\n', &mut bytes)
        .map_err(|_| ProbeFailure::new("PROTOCOL_ERROR", "Could not read MCP STDIO."))?;
    if length == 0 {
        return Ok(None);
    }
    if length as u64 > FRAME_LIMIT {
        return Err(ProbeFailure::new(
            "FRAME_TOO_LARGE",
            "MCP response exceeded its size limit.",
        ));
    }
    if bytes.last() != Some(&b'\n') || unique_json(&bytes).is_err() {
        return Err(ProbeFailure::new(
            "PROTOCOL_ERROR",
            "Malformed MCP response.",
        ));
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|_| ProbeFailure::new("PROTOCOL_ERROR", "Invalid MCP response envelope."))
}

impl Pipe {
    fn new(child: &mut Child, deadline: Instant, current: bool) -> Self {
        let mut input = child.stdin.take().expect("piped stdin");
        let mut stdout = BufReader::new(child.stdout.take().expect("piped stdout"));
        let mut stderr = child.stderr.take().expect("piped stderr");
        let (send_input, read_input) = mpsc::sync_channel::<ClientJsonRpcMessage>(2);
        let (send_frames, frames) = mpsc::sync_channel(2);
        thread::spawn(move || {
            for message in read_input {
                if serde_json::to_writer(&mut input, &message).is_err()
                    || input.write_all(b"\n").is_err()
                    || input.flush().is_err()
                {
                    break;
                }
            }
        });
        thread::spawn(move || {
            loop {
                match frame(&mut stdout) {
                    Ok(Some(response)) => {
                        if send_frames.send(Ok(response)).is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        let _ = send_frames.send(Err(error));
                        break;
                    }
                    _ => break,
                }
            }
        });
        thread::spawn(move || {
            // Keep stderr separate and discard it. It is not protocol evidence.
            let _ = io::copy(&mut stderr, &mut io::sink());
        });
        Self {
            input: send_input,
            frames,
            deadline,
            next_id: 0,
            current,
        }
    }

    fn send(&self, message: ClientJsonRpcMessage) -> Result<(), ProbeFailure> {
        self.input.try_send(message).map_err(|_| {
            ProbeFailure::new(
                "PROTOCOL_ERROR",
                "MCP STDIO could not accept a bounded request.",
            )
        })
    }

    fn request(
        &mut self,
        request: impl Into<ClientRequest>,
        is_cancelled: &impl Fn() -> bool,
    ) -> Result<ServerResult, ProbeFailure> {
        if is_cancelled() {
            return Err(ProbeFailure::new("CANCELLED", "MCP probe cancelled."));
        }
        self.next_id += 1;
        let id = rmcp::model::RequestId::Number(self.next_id as i64);
        let mut message = ClientJsonRpcMessage::request(request.into(), id.clone());
        if self.current
            && let ClientJsonRpcMessage::Request(request) = &mut message
        {
            let meta = request.request.get_meta_mut();
            meta.set_protocol_version(ProtocolVersion::V_2026_07_28);
            meta.set_client_capabilities(ClientCapabilities::default());
        }
        self.send(message)?;
        loop {
            if is_cancelled() {
                return Err(ProbeFailure::new("CANCELLED", "MCP probe cancelled."));
            }
            let remaining = self.deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(ProbeFailure::new("TIMEOUT", "MCP probe deadline expired."));
            }
            match self.frames.recv_timeout(remaining.min(POLL_INTERVAL)) {
                Ok(Ok(ServerJsonRpcMessage::Response(response))) if response.id == id => {
                    return Ok(response.result);
                }
                Ok(Err(error)) => return Err(error),
                Err(RecvTimeoutError::Timeout) => {}
                _ => {
                    return Err(ProbeFailure::new(
                        "PROTOCOL_ERROR",
                        "MCP server rejected the request or returned an unexpected frame.",
                    ));
                }
            }
        }
    }

    fn finish(self, is_cancelled: &impl Fn() -> bool) -> Result<(), ProbeFailure> {
        drop(self.input);
        loop {
            if is_cancelled() {
                return Err(ProbeFailure::new("CANCELLED", "MCP probe cancelled."));
            }
            let remaining = self.deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(ProbeFailure::new(
                    "TIMEOUT",
                    "MCP STDIO did not close before the deadline.",
                ));
            }
            match self.frames.recv_timeout(remaining.min(POLL_INTERVAL)) {
                Ok(Ok(_)) => {
                    return Err(ProbeFailure::new(
                        "PROTOCOL_ERROR",
                        "Unexpected MCP message after the final response.",
                    ));
                }
                Ok(Err(error)) => return Err(error),
                Err(RecvTimeoutError::Disconnected) => return Ok(()),
                Err(RecvTimeoutError::Timeout) => {}
            }
        }
    }
}

fn capabilities(
    server: &rmcp::model::ServerCapabilities,
    implementation: Option<Implementation>,
) -> bool {
    server.tools.is_some()
        && server.resources.is_none()
        && server.prompts.is_none()
        && implementation
            .is_some_and(|info| info.name == "cnr" && info.version == env!("CARGO_PKG_VERSION"))
}

fn handshake(pipe: &mut Pipe, is_cancelled: &impl Fn() -> bool) -> Result<(), ProbeFailure> {
    let verified = if pipe.current {
        match pipe.request(
            DiscoverRequest::new(DiscoverRequestParams::default()),
            is_cancelled,
        )? {
            ServerResult::DiscoverResult(result) => {
                result
                    .supported_versions
                    .contains(&ProtocolVersion::V_2026_07_28)
                    && result
                        .supported_versions
                        .contains(&ProtocolVersion::V_2025_11_25)
                    && capabilities(&result.capabilities, result.server_info())
            }
            _ => false,
        }
    } else {
        let client = Implementation::new("cnr-probe", env!("CARGO_PKG_VERSION"));
        let request = InitializeRequestParams::new(ClientCapabilities::default(), client)
            .with_protocol_version(ProtocolVersion::V_2025_11_25);
        let verified = match pipe.request(InitializeRequest::new(request), is_cancelled)? {
            ServerResult::InitializeResult(result) => {
                result.protocol_version == ProtocolVersion::V_2025_11_25
                    && capabilities(&result.capabilities, Some(result.server_info.clone()))
            }
            _ => false,
        };
        if verified {
            pipe.send(ClientJsonRpcMessage::notification(
                InitializedNotification::default().into(),
            ))?;
        }
        verified
    };
    if !verified {
        return Err(ProbeFailure::new(
            "PROTOCOL_ERROR",
            "Unexpected CNR protocol or capabilities.",
        ));
    }
    Ok(())
}

fn call(
    pipe: &mut Pipe,
    name: &'static str,
    args: Value,
    cancelled: &impl Fn() -> bool,
) -> Result<Value, ProbeFailure> {
    let arguments = args.as_object().expect("fixed probe arguments").clone();
    let request = CallToolRequest::new(CallToolRequestParams::new(name).with_arguments(arguments));
    let ServerResult::CallToolResult(result) = pipe.request(request, cancelled)? else {
        return Err(ProbeFailure::new(
            "PROTOCOL_ERROR",
            "MCP tool returned an incomplete response.",
        ));
    };
    let data = result.structured_content.ok_or(ProbeFailure::new(
        "PROTOCOL_ERROR",
        "MCP tool returned no structured content.",
    ))?;
    if data["status"] == "PASS" && result.is_error != Some(true) {
        let schema = routes(false)?
            .into_iter()
            .find(|route| route.tool.name == name)
            .and_then(|route| route.tool.output_schema)
            .ok_or(ProbeFailure::new(
                "TOOL_SCHEMA_MISMATCH",
                "Expected MCP output schema is missing.",
            ))?;
        let schema = Value::Object((*schema).clone());
        if jsonschema::is_valid(&schema, &data) {
            return Ok(data["data"].clone());
        }
        return Err(ProbeFailure::new(
            "TOOL_SCHEMA_MISMATCH",
            "MCP tool output failed schema validation.",
        ));
    }
    if matches!(data["status"].as_str(), Some("BLOCKED" | "NOT_RUN"))
        && result.is_error == Some(true)
    {
        return Err(ProbeFailure::new(
            "DOMAIN_BLOCKED",
            "A read-only governance tool could not complete.",
        ));
    }
    Err(ProbeFailure::new(
        "PROTOCOL_ERROR",
        "MCP tool result did not report consistent success.",
    ))
}

fn verify_session(
    pipe: &mut Pipe,
    repo: &str,
    task: Option<&str>,
    is_cancelled: &impl Fn() -> bool,
) -> Result<(), ProbeFailure> {
    handshake(pipe, is_cancelled)?;
    let listed = pipe.request(
        ListToolsRequest::with_param(PaginatedRequestParams::default()),
        is_cancelled,
    )?;
    let ServerResult::ListToolsResult(listed) = listed else {
        return Err(ProbeFailure::new(
            "TOOL_SCHEMA_MISMATCH",
            "MCP tools/list returned the wrong result type.",
        ));
    };
    let expected = routes(false)?;
    if listed.next_cursor.is_some()
        || listed.tools.len() != expected.len()
        || expected.iter().any(|route| {
            listed
                .tools
                .iter()
                .filter(|item| {
                    item.name == route.tool.name
                        && item.input_schema == route.tool.input_schema
                        && item.output_schema == route.tool.output_schema
                        && item.annotations == route.tool.annotations
                })
                .count()
                != 1
        })
    {
        return Err(ProbeFailure::new(
            "TOOL_SCHEMA_MISMATCH",
            "MCP tool list differs from the read-only contract.",
        ));
    }
    call(
        pipe,
        "cirnet_repositories_discover",
        json!({"limit":10}),
        is_cancelled,
    )?;
    let policy = call(
        pipe,
        "cirnet_policy_lookup",
        json!({"repo_id":repo}),
        is_cancelled,
    )?;
    if policy["repository"] != repo {
        return Err(ProbeFailure::new(
            "AUTHORITY_MISMATCH",
            "Policy response does not match the selected repository.",
        ));
    }
    if let Some(task) = task {
        let status = call(
            pipe,
            "cirnet_task_status",
            json!({"task_id":task}),
            is_cancelled,
        )?;
        let revision = status["revision"].as_u64().ok_or(ProbeFailure::new(
            "TOOL_SCHEMA_MISMATCH",
            "MCP task status did not return a revision.",
        ))?;
        call(
            pipe,
            "cirnet_task_context",
            json!({"task_id":task,"expected_revision":revision}),
            is_cancelled,
        )?;
    }
    Ok(())
}

/// Validate both supported MCP lifecycles over private pipes from this CNR binary.
pub fn probe(
    root: &Path,
    workspace: &Path,
    repo: Option<&str>,
    task: Option<&str>,
) -> Result<Value, ProbeFailure> {
    probe_with_cancel(root, workspace, repo, task, || false)
}

/// Observe caller cancellation while waiting on the private transport.
pub fn probe_with_cancel(
    root: &Path,
    workspace: &Path,
    repo: Option<&str>,
    task: Option<&str>,
    cancelled: impl Fn() -> bool,
) -> Result<Value, ProbeFailure> {
    let terminal_cancel = Cancellation::begin()?;
    let is_cancelled = || terminal_cancel.requested() || cancelled();
    let authority = GovernanceService::open(root, workspace).map_err(|_| {
        ProbeFailure::new(
            "AUTHORITY_MISMATCH",
            "MCP probe requires valid canonical governance and workspace roots.",
        )
    })?;
    let (binary, sha256) = mcp_config::current_executable().map_err(|_| {
        ProbeFailure::new(
            "EXECUTABLE_NOT_FOUND",
            "The running CNR executable is unavailable or unsafe.",
        )
    })?;
    let repo = repo.unwrap_or("cirnet-rules");
    if !repo
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
        || repo.is_empty()
        || repo.len() > 80
        || task.is_some_and(|task| crate::tasks::valid_id(task).is_err())
    {
        return Err(ProbeFailure::new(
            "INVALID_SCOPE",
            "Select a valid repository and task identifier.",
        ));
    }
    let deadline = Instant::now() + PROBE_TIMEOUT;
    // On Linux, the private process entry names the already-running image
    // even if the executable's directory entry is replaced before spawning.
    #[cfg(target_os = "linux")]
    let launch = Path::new("/proc/self/exe");
    #[cfg(not(target_os = "linux"))]
    let launch = binary.as_path();
    for current in [true, false] {
        if is_cancelled() {
            return Err(ProbeFailure::new("CANCELLED", "MCP probe cancelled."));
        }
        let mut command = Command::new(launch);
        command
            .current_dir(binary.parent().expect("validated executable directory"))
            .args(["mcp", "serve", "--governance-root"])
            .arg(root)
            .arg("--workspace")
            .arg(workspace)
            .args(["--access", "read-only"]);
        process::with_owned_stdio(&mut command, |child| {
            let mut pipe = Pipe::new(child, deadline, current);
            verify_session(&mut pipe, repo, task, &is_cancelled)?;
            pipe.finish(&is_cancelled)
        })?;
    }
    authority.ensure_current().map_err(|_| {
        ProbeFailure::new(
            "AUTHORITY_MISMATCH",
            "Governance changed during the probe. Run it again.",
        )
    })?;
    if !mcp_config::current_executable().is_ok_and(|actual| actual == (binary, sha256.clone())) {
        return Err(ProbeFailure::new(
            "CONFIG_DRIFT",
            "The running CNR executable changed during verification.",
        ));
    }
    Ok(json!({
        "status":"PASS",
        "mcp_protocol":"MCP_PROTOCOL_VERIFIED",
        "native_client":"NOT_RUN",
        "access":"read-only",
        "protocols":["2026-07-28","2025-11-25"],
        "tool_count":6,
        "binary_sha256":sha256
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_frames_reject_oversize_duplicate_and_unterminated_data() {
        for mut malformed in [
            &b"{\"jsonrpc\":\"2.0\",\"id\":1,\"id\":2,\"result\":{}}\n"[..],
            &b"{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}"[..],
            &b"CNR banner\n"[..],
        ] {
            assert_eq!(frame(&mut malformed).unwrap_err().code, "PROTOCOL_ERROR");
        }
        let oversize = vec![b'a'; FRAME_LIMIT as usize + 1];
        assert_eq!(
            frame(&mut oversize.as_slice()).unwrap_err().code,
            "FRAME_TOO_LARGE"
        );
    }
}
