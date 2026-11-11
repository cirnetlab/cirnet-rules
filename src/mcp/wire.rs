use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Write};
use std::{
    fs,
    path::{Path, PathBuf},
};
#[path = "../test_support.rs"]
mod test_support;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const DEADLINE: Duration = Duration::from_secs(15);

struct Wire {
    process: Child,
    input: Option<ChildStdin>,
    frames: Receiver<String>,
}

impl Wire {
    fn spawn(access: &str) -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        Self::with_roots(&root, root.parent().unwrap(), access, None)
    }

    fn with_roots(root: &Path, work: &Path, access: &str, session: Option<&Path>) -> Self {
        let mut process = Self::spawn_process(root, work, access, session);
        let (sender, frames) = mpsc::channel();
        let stdout = process.stdout.take().unwrap();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                if sender.send(line.unwrap()).is_err() {
                    break;
                }
            }
        });
        Self {
            input: process.stdin.take(),
            process,
            frames,
        }
    }

    fn spawn_process(root: &Path, work: &Path, access: &str, session: Option<&Path>) -> Child {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cnr"));
        command
            .current_dir(work)
            .args(["mcp", "serve", "--governance-root"])
            .arg(root)
            .arg("--workspace")
            .arg(work)
            .args(["--access", access])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        if let Some(session) = session {
            command.arg("--session-file").arg(session);
        }
        command.spawn().unwrap()
    }

    fn send_raw(&mut self, frame: &str) {
        let input = self.input.as_mut().unwrap();
        writeln!(input, "{frame}").expect("write one MCP frame");
        input.flush().unwrap();
    }

    fn send(&mut self, frame: Value) {
        self.send_raw(&frame.to_string());
    }

    fn receive(&self) -> Value {
        let frame = self.frames.recv_timeout(DEADLINE).expect("MCP response");
        let value: Value = serde_json::from_str(&frame).expect("stdout holds JSON-RPC only");
        assert_eq!(value["jsonrpc"], "2.0", "{value}");
        value
    }

    fn request(&mut self, id: u64, method: &str, params: Value) -> Value {
        self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}));
        let response = self.receive();
        assert_eq!(response["id"], id, "{response}");
        response
    }

    fn initialize(&mut self) {
        let response = self.request(
            1,
            "initialize",
            json!({
                "protocolVersion":"2025-11-25",
                "capabilities":{},
                "clientInfo":{"name":"untrusted-administrator","version":"1"}
            }),
        );
        assert_eq!(
            response["result"]["protocolVersion"], "2025-11-25",
            "{response}"
        );
        assert!(response["result"]["capabilities"]["tools"].is_object());
        assert!(response["result"]["capabilities"]["resources"].is_null());
        self.send(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    }

    fn close_input(&mut self) -> std::process::ExitStatus {
        self.input.take();
        self.wait_exit()
    }

    fn wait_exit(&mut self) -> std::process::ExitStatus {
        let start = Instant::now();
        loop {
            if let Some(exit) = self.process.try_wait().unwrap() {
                return exit;
            }
            assert!(
                start.elapsed() < DEADLINE,
                "MCP process must stop before deadline"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for Wire {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

#[test]
fn legacy_stdio_discovery_is_read_only_and_structured() {
    let mut server = Wire::spawn("read-only");
    server.initialize();
    let tools = server.request(2, "tools/list", json!({}));
    let tools = tools["result"]["tools"].as_array().expect("tools listed");
    let names: Vec<_> = tools
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "cirnet_audit_read",
            "cirnet_diff_check",
            "cirnet_policy_lookup",
            "cirnet_repositories_discover",
            "cirnet_task_context",
            "cirnet_task_status"
        ]
    );
    assert!(tools.iter().all(|tool| {
        tool["inputSchema"]["additionalProperties"] == false
            && tool["outputSchema"]["type"] == "object"
            && tool["annotations"]["readOnlyHint"] == true
    }));
    let result = server.request(
        3,
        "tools/call",
        json!({"name":"cirnet_repositories_discover"}),
    );
    assert_eq!(
        result["result"]["structuredContent"]["status"], "PASS",
        "{result}"
    );
    let denied = server.request(
        4,
        "tools/call",
        json!({"name":"cirnet_task_claim","arguments":{
            "task_id":"EXAMPLE", "expected_revision":1, "request_id":"attempt", "actor":"admin"
        }}),
    );
    assert_eq!(denied["error"]["code"], -32602, "{denied}");
    let denied = server.request(
        5,
        "tools/call",
        json!({"name":"cirnet_repositories_discover","arguments":{
            "workspace":"some-other-workspace"
        }}),
    );
    assert_eq!(denied["error"]["code"], -32602, "{denied}");
    assert!(server.close_input().success());
}

#[test]
fn current_stdio_discovery_uses_per_request_metadata() {
    let mut server = Wire::spawn("read-only");
    let meta = json!({
        "io.modelcontextprotocol/protocolVersion":"2026-07-28",
        "io.modelcontextprotocol/clientCapabilities":{}
    });
    let discovery = server.request(1, "server/discover", json!({"_meta":meta}));
    assert_eq!(
        discovery["result"]["capabilities"]["tools"],
        json!({}),
        "{discovery}"
    );
    assert!(
        discovery["result"].to_string().contains("2026-07-28"),
        "{discovery}"
    );
    assert!(
        discovery["result"].to_string().contains("2025-11-25"),
        "{discovery}"
    );
    let tools = server.request(2, "tools/list", json!({"_meta":meta}));
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 6);
    let invalid = server.request(3, "tools/list", json!({}));
    assert!(invalid["error"]["code"].is_number(), "{invalid}");
    let blocked = server.request(
        4,
        "tools/call",
        json!({"_meta":meta,
            "name":"cirnet_task_status","arguments":{"task_id":"TASK_DOES_NOT_EXIST_WIRE_FIXTURE"}
        }),
    );
    assert_eq!(
        blocked["result"]["structuredContent"]["status"], "BLOCKED",
        "{blocked}"
    );
    assert_eq!(blocked["result"]["isError"], true, "{blocked}");
    assert!(server.close_input().success());
}

#[test]
fn duplicate_arguments_and_overlong_frames_are_bounded() {
    let mut server = Wire::spawn("read-only");
    server.initialize();
    server.send_raw(r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"cirnet_repositories_discover","arguments":{"limit":1,"limit":2}}}"#);
    let duplicate = server.receive();
    assert_eq!(duplicate["error"]["code"], -32600, "{duplicate}");
    let invalid = server.request(
        3,
        "tools/call",
        json!({"name":"cirnet_repositories_discover","arguments":{"limit":41}}),
    );
    assert_eq!(invalid["error"]["code"], -32602, "{invalid}");
    assert!(server.request(4, "tools/list", json!({}))["result"]["tools"].is_array());
    // A single byte beyond the frame cap is enough to stop reading the peer.
    let _ = writeln!(
        server.input.as_mut().unwrap(),
        "{}",
        "x".repeat(64 * 1024 + 1)
    );
    assert!(server.wait_exit().success());
}

#[test]
fn coordinated_profile_requires_operator_prepared_session() {
    let mut server = Wire::spawn("coordinated");
    let exit = server.close_input();
    assert!(!exit.success());
    assert!(
        server.frames.recv_timeout(DEADLINE).is_err(),
        "startup error leaked to stdout"
    );
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        Self(test_support::temporary_directory("cnr MCP Việt").unwrap())
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // The path was allocated exclusively for this fixture.
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn pagination_rejects_changed_snapshot_filter_and_process() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let work = Scratch::new();
    let add_candidate = |name: &str| fs::create_dir_all(work.0.join(name).join(".git")).unwrap();
    add_candidate("one");
    add_candidate("two");
    let mut first = Wire::with_roots(&root, &work.0, "read-only", None);
    let mut second = Wire::with_roots(&root, &work.0, "read-only", None);
    first.initialize();
    second.initialize();
    let discover = |server: &mut Wire, id, arguments| {
        server.request(
            id,
            "tools/call",
            json!({
                "name":"cirnet_repositories_discover","arguments":arguments
            }),
        )["result"]["structuredContent"]
            .clone()
    };
    let page = discover(&mut first, 2, json!({"limit":1}));
    assert_eq!(page["status"], "PASS", "{page}");
    let cursor = page["data"]["next_cursor"].as_str().expect("next page");
    let next = discover(&mut first, 3, json!({"cursor":cursor,"limit":1}));
    assert_eq!(next["status"], "PASS", "{next}");
    let limit = discover(&mut first, 4, json!({"cursor":cursor,"limit":2}));
    assert_eq!(limit["status"], "BLOCKED", "{limit}");
    let other = discover(&mut second, 2, json!({"cursor":cursor,"limit":1}));
    assert_eq!(other["status"], "BLOCKED", "{other}");
    add_candidate("three");
    let changed = discover(&mut first, 5, json!({"cursor":cursor,"limit":1}));
    assert_eq!(changed["status"], "BLOCKED", "{changed}");
    assert!(first.close_input().success());
    assert!(second.close_input().success());
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(root)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn commit_fixture(root: &Path) {
    git(
        root,
        &[
            "-c",
            "user.name=fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "-c",
            "core.hooksPath=.git/disabled-hooks",
            "commit",
            "--quiet",
            "-m",
            "Initialize fixture",
        ],
    );
}
fn operator(root: &Path, work: &Path, args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_cnr"))
        .current_dir(work)
        .env("CIRNETRULES_GOVERNANCE", root)
        .args(args)
        .arg("--json")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{args:?}: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    value.get("result").unwrap_or(&value).clone()
}

#[test]
fn coordinated_wire_and_cli_share_lease_revision_report_and_audit() {
    let work = Scratch::new();
    let root = work.0.join("cirnet-rules");
    let product = work.0.join("Interface fixture Việt");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut registry: Value =
        serde_json::from_slice(&fs::read(source.join("governance.yaml")).unwrap()).unwrap();
    for path in [
        ".gitignore",
        "VERSION",
        "AGENTS.md",
        "en/schemas/governance.schema.json",
        "en/schemas/cirnetrules-policy.schema.json",
    ]
    .into_iter()
    .chain(
        registry["assets"]["entries"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|asset| asset["status"] == "ACTIVE")
            .map(|asset| asset["path"].as_str().unwrap()),
    ) {
        let destination = root.join(path);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(source.join(path), destination).unwrap();
    }
    let repo = registry["repositories"]["cirnet-interface"]
        .as_object_mut()
        .unwrap();
    for key in ["snapshot", "plan", "spec", "remote"] {
        repo.remove(key);
    }
    fs::write(
        root.join("governance.yaml"),
        serde_json::to_vec_pretty(&registry).unwrap(),
    )
    .unwrap();
    git(&root, &["init", "--quiet"]);
    git(
        &root,
        &[
            "remote",
            "add",
            "origin",
            "https://github.com/cirnetlab/cirnet-rules.git",
        ],
    );
    git(&root, &["add", "--", "."]);
    commit_fixture(&root);
    let target = "src/components/Toolbar.vue";
    fs::create_dir_all(product.join(target).parent().unwrap()).unwrap();
    fs::write(product.join(target), "<button>Fixture</button>\n").unwrap();
    git(&product, &["init", "--quiet"]);
    git(
        &product,
        &[
            "remote",
            "add",
            "origin",
            "https://github.com/cirnetlab/cirnet-interface.git",
        ],
    );
    git(&product, &["add", "--", target]);
    commit_fixture(&product);
    operator(
        &root,
        &work.0,
        &[
            "workspace",
            "bind",
            "--repo",
            "cirnet-interface",
            "--path",
            "Interface fixture Việt",
            "--actor",
            "maintainer",
        ],
    );
    let spec = "en/specs/cirnet-interface/mcp-fixture/spec.md";
    let plan = "en/plans/cirnet-interface/mcp-fixture/plan.md";
    for path in [spec, plan] {
        let file = root.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, "# Test shared MCP coordination\n").unwrap();
    }
    git(&root, &["add", "--", spec, plan]);
    commit_fixture(&root);
    let descriptor = root.join("mcp-task.json");
    fs::write(&descriptor, serde_json::to_vec(&json!({
        "id":"mcp-fixture","request":"Exercise MCP","objective":"Observe shared lease state","non_goals":["No release"],
        "primary_repo":"cirnet-interface","hints":[target],
        "read_scope":{"cirnet-interface":[target]},"write_scope":{"cirnet-interface":[target]},
        "verification_scope":{"cirnet-interface":[target]},"documentation_scope":{},"risk":"low","role":"implementer","mode":"WORK","owner":"builder",
        "authorization":{"owner":"maintainer","basis":"Test local coordination","actions":["read","write","verify"]},
        "spec":spec,"plan":plan,"acceptance":["Observe one shared lease"],"commands":["interface-diff"],
        "docs_targets":[],"docs_not_required":"Fixture has no documentation changes"
    })).unwrap()).unwrap();
    operator(
        &root,
        &work.0,
        &["task", "create", "--descriptor", "mcp-task.json"],
    );
    fs::remove_file(&descriptor).unwrap();
    operator(
        &root,
        &work.0,
        &[
            "task",
            "transition",
            "--task",
            "mcp-fixture",
            "--revision",
            "1",
            "--actor",
            "maintainer",
            "--to",
            "READY",
        ],
    );
    let session = operator(
        &root,
        &work.0,
        &[
            "task",
            "session-prepare",
            "--task",
            "mcp-fixture",
            "--revision",
            "2",
            "--actor",
            "maintainer",
            "--request-id",
            "prepare-fixture",
        ],
    );
    let file = root.join(
        session["session_file"]
            .as_str()
            .expect("prepared credential file"),
    );
    let token = fs::read_to_string(&file).unwrap();
    let mut writer = Wire::with_roots(&root, &work.0, "coordinated", Some(&file));
    let mut reader = Wire::with_roots(&root, &work.0, "read-only", None);
    writer.initialize();
    reader.initialize();
    let tools = writer.request(2, "tools/list", json!({}));
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 10);
    assert!(!tools.to_string().contains(token.trim()));
    let call = |server: &mut Wire, id, name, arguments| {
        server.request(id, "tools/call", json!({"name":name,"arguments":arguments}))["result"]["structuredContent"].clone()
    };
    let claim = call(
        &mut writer,
        3,
        "cirnet_task_claim",
        json!({
            "task_id":"mcp-fixture","expected_revision":session["revision"],"request_id":"claim-fixture"
        }),
    );
    assert_eq!(claim["status"], "PASS", "{claim}");
    let lease = &claim["data"]["lease"];
    let local = operator(&root, &work.0, &["task", "status", "--task", "mcp-fixture"]);
    assert_eq!(local["revision"], claim["data"]["revision"]);
    assert_eq!(local["lease"]["generation"], lease["generation"]);
    let remote = call(
        &mut reader,
        2,
        "cirnet_task_status",
        json!({"task_id":"mcp-fixture"}),
    );
    assert_eq!(remote["data"]["revision"], local["revision"], "{remote}");
    let policy = call(
        &mut reader,
        3,
        "cirnet_policy_lookup",
        json!({"repo_id":"cirnet-interface","asset_id":"SEC-01","kind":"rule"}),
    );
    assert_eq!(policy["status"], "PASS", "{policy}");
    assert_eq!(policy["data"]["included"][0]["id"], "SEC-01", "{policy}");
    assert_eq!(
        policy["data"]["included"][0]["enforcement_class"], "BLOCK",
        "{policy}"
    );
    let stale = call(
        &mut reader,
        4,
        "cirnet_task_context",
        json!({"task_id":"mcp-fixture","expected_revision":session["revision"]}),
    );
    assert_eq!(stale["status"], "BLOCKED", "{stale}");
    let context = call(
        &mut reader,
        5,
        "cirnet_task_context",
        json!({"task_id":"mcp-fixture","expected_revision":local["revision"]}),
    );
    assert_eq!(context["status"], "PASS", "{context}");
    assert_eq!(context["data"]["cached"], false, "{context}");
    assert!(!root.join(".cirnetrules/cache").exists());
    let snapshot = operator(
        &root,
        &work.0,
        &["task", "diff-check", "--task", "mcp-fixture"],
    );
    let diff = call(
        &mut reader,
        6,
        "cirnet_diff_check",
        json!({"task_id":"mcp-fixture","expected_snapshot_sha256":snapshot["sha256"]}),
    );
    assert_eq!(diff["status"], "PASS", "{diff}");
    let renewed = call(
        &mut writer,
        4,
        "cirnet_task_renew",
        json!({
            "task_id":"mcp-fixture","generation":lease["generation"],"request_id":"renew-fixture"
        }),
    );
    assert_eq!(renewed["status"], "PASS", "{renewed}");
    assert_eq!(renewed["data"]["lease"]["generation"], lease["generation"]);
    let artifact_body = b"Fixture artifact body remains private";
    let artifact_digest = format!("{:x}", Sha256::digest(artifact_body));
    let artifact_dir = root.join(".cirnetrules/artifacts");
    fs::create_dir_all(&artifact_dir).unwrap();
    fs::write(artifact_dir.join(&artifact_digest), artifact_body).unwrap();
    fs::write(product.join(target), "<button>Reviewed fixture</button>\n").unwrap();
    let report = call(
        &mut writer,
        5,
        "cirnet_task_report",
        json!({
            "task_id":"mcp-fixture","generation":lease["generation"],"expected_revision":renewed["data"]["revision"],
            "request_id":"report-fixture","report":{"summary":"Fixture report text remains private","artifacts":[artifact_digest]}
        }),
    );
    assert_eq!(report["status"], "PASS", "{report}");
    let released = call(
        &mut writer,
        6,
        "cirnet_task_release",
        json!({
            "task_id":"mcp-fixture","generation":lease["generation"],"request_id":"release-fixture"
        }),
    );
    assert_eq!(released["status"], "PASS", "{released}");
    let audit = call(
        &mut reader,
        7,
        "cirnet_audit_read",
        json!({"task_id":"mcp-fixture"}),
    );
    assert_eq!(audit["status"], "PASS", "{audit}");
    let events = audit["data"]["events"].as_array().unwrap();
    for operation in ["claim", "renew", "report", "release"] {
        assert!(
            events.iter().any(|event| event["operation"] == operation),
            "{audit}"
        );
    }
    assert!(
        !audit
            .to_string()
            .contains("Fixture report text remains private")
    );
    assert!(
        !audit
            .to_string()
            .contains("Fixture artifact body remains private")
    );
    assert!(!audit.to_string().contains(token.trim()));
    assert!(reader.close_input().success());
    assert!(writer.close_input().success());

    let bundle = work.0.join("reviewed-candidate.json");
    let evidence = operator(
        &root,
        &work.0,
        &[
            "task",
            "evidence",
            "export",
            "--task",
            "mcp-fixture",
            "--output",
            bundle.to_str().unwrap(),
        ],
    );
    let inspected = operator(
        &root,
        &work.0,
        &[
            "task",
            "evidence",
            "inspect",
            "--bundle",
            bundle.to_str().unwrap(),
        ],
    );
    assert_eq!(inspected["trust"], "UNTRUSTED");
    assert_eq!(evidence["bundle_sha256"], inspected["bundle_sha256"]);
    let authority = Command::new("git")
        .current_dir(&root)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(authority.status.success());
    let authority = String::from_utf8(authority.stdout).unwrap();
    git(&product, &["add", "--", target]);
    commit_fixture(&product);
    let gated = operator(
        &root,
        &work.0,
        &[
            "ci",
            "verify",
            "--governance-root",
            root.to_str().unwrap(),
            "--authority-commit",
            authority.trim(),
            "--expected-digest",
            evidence["bundle_sha256"].as_str().unwrap(),
            "--bundle",
            bundle.to_str().unwrap(),
            "--target",
            product.to_str().unwrap(),
        ],
    );
    assert_eq!(gated["status"], "PASS");
    assert_eq!(gated["checks"][0]["id"], "interface-diff");
}

#[test]
fn cancellation_does_not_stop_an_independent_request() {
    let mut server = Wire::spawn("read-only");
    server.initialize();
    let cancelled = json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{
        "name":"cirnet_task_status","arguments":{"task_id":"TASK_CANCELLED_WIRE_FIXTURE"}
    }});
    let cancel =
        json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":2}});
    let independent = json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{
        "name":"cirnet_repositories_discover"
    }});
    server.send_raw(&format!("{cancelled}\n{cancel}\n{independent}"));
    let response = server.receive();
    // An already completed response can win the race against cancellation.
    let independent = if response["id"] == 2 {
        server.receive()
    } else {
        response
    };
    assert_eq!(independent["id"], 3, "{independent}");
    assert_eq!(
        independent["result"]["structuredContent"]["status"], "PASS",
        "{independent}"
    );
    assert!(server.close_input().success());
}

#[test]
fn unread_stdout_exits_before_the_handshake_deadline() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut process = Wire::spawn_process(&root, root.parent().unwrap(), "read-only", None);
    let mut input = process.stdin.take().unwrap();
    let mut wire = Wire {
        process,
        input: None,
        frames: mpsc::channel().1,
    };
    // Hold the stdout pipe unread and keep stdin open after the final request.
    let (stop, hold) = mpsc::channel();
    let sender = std::thread::spawn(move || {
        let meta = json!({
            "io.modelcontextprotocol/protocolVersion":"2026-07-28",
            "io.modelcontextprotocol/clientCapabilities":{}
        });
        for id in 0..512 {
            let request =
                json!({"jsonrpc":"2.0","id":id,"method":"server/discover","params":{"_meta":meta}});
            if writeln!(input, "{request}").is_err() {
                break;
            }
        }
        let _ = hold.recv_timeout(DEADLINE);
    });
    let exit = wire.wait_exit();
    let _ = stop.send(());
    sender.join().unwrap();
    assert!(!exit.success());
}
