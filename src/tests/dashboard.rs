use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

struct Runtime {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
    request: u64,
}
impl Runtime {
    fn open(data: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_cgacp-runtime"))
            .arg("--stdio")
            .arg("--data-dir")
            .arg(data)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        Self {
            input: child.stdin.take(),
            output: BufReader::new(child.stdout.take().unwrap()),
            child,
            request: 0,
        }
    }
    fn request(&mut self, version: u8, method: &str, params: Value) -> Value {
        self.request += 1;
        let id = self.request.to_string();
        let input = self.input.as_mut().unwrap();
        serde_json::to_writer(
            &mut *input,
            &json!({"v":version,"id":id,"method":method,"params":params}),
        )
        .unwrap();
        input.write_all(b"\n").unwrap();
        input.flush().unwrap();
        let mut response = String::new();
        assert_ne!(
            self.output.read_line(&mut response).unwrap(),
            0,
            "Runtime closed during {method}"
        );
        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["id"], id);
        response
    }
    fn call(&mut self, method: &str, params: Value) -> Value {
        let response = self.request(1, method, params);
        assert_eq!(response["ok"], true, "{method}: {response}");
        response["result"].clone()
    }
    fn close(&mut self) {
        self.input.take();
        assert!(self.child.wait().unwrap().success());
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.input.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("dashboard-fixture-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn git(path: &Path, args: &[&str]) {
    let result = Command::new("git")
        .args(args)
        .current_dir(path)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn closing_request_pipe_drains_accepted_replies() {
    let fixture = Fixture::new();
    let data = fixture.0.join("eof");
    let mut initializer = Runtime::open(&data);
    initializer.call("storage.verify", json!({}));
    initializer.close();
    {
        let db = rusqlite::Connection::open(data.join("cgacp.sqlite3")).unwrap();
        db.execute("INSERT INTO conversations VALUES(1,'fixture','digest','EOF fixture','handoff','ACTIVE','DASHBOARD',NULL,1,1,1)", []).unwrap();
        for id in 1..=4 {
            db.execute(
                "INSERT INTO messages VALUES(?1,1,?2,'digest',?1,'USER',?3,1)",
                rusqlite::params![id, format!("message-{id}"), "x".repeat(16_000)],
            )
            .unwrap();
            db.execute("INSERT INTO task_proposals VALUES(?1,1,?1,'WAITING_APPROVAL','WORK','SCOPED','{}',1)",[id]).unwrap();
        }
    }
    let mut child = Command::new(env!("CARGO_BIN_EXE_cgacp-runtime"))
        .arg("--stdio")
        .arg("--data-dir")
        .arg(data)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut stdin = child.stdin.take().unwrap();
        writeln!(stdin, "{}", json!({"v":1,"id":"1","method":"runtime.info"})).unwrap();
        for id in 2..=8 {
            writeln!(
                stdin,
                "{}",
                json!({"v":1,"id":id.to_string(),"method":"conversation.get","params":{"id":"1"}})
            )
            .unwrap();
        }
    }
    std::thread::sleep(Duration::from_millis(200));
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let ids: Vec<_> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| {
            let reply: Value = serde_json::from_str(line).unwrap();
            assert_eq!(reply["ok"], true);
            reply["id"].as_str().unwrap().to_owned()
        })
        .collect();
    assert_eq!(ids, ["1", "2", "3", "4", "5", "6", "7", "8"]);
}

#[test]
fn private_runtime_persists_repo_conversation_and_one_shot_then_rejects_stale_binding() {
    let fixture = Fixture::new();
    let data = fixture.0.join("data");
    let repo = fixture.0.join("Repository thử mới");
    let file = repo.join("src/sample.txt");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, "original\n").unwrap();
    git(&repo, &["init", "--quiet"]);
    git(&repo, &["add", "--", "src"]);
    git(
        &repo,
        &[
            "-c",
            "user.name=CGACP Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "--quiet",
            "-m",
            "Fixture",
        ],
    );

    let start = Instant::now();
    let mut client = Runtime::open(&data);
    let info = client.call("runtime.info", json!({}));
    assert_eq!(info["product_generation"], "CGACP v1");
    assert_eq!(info["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(
        info["release_name"],
        format!("cirnet-rules-{}", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(
        client.request(2, "runtime.info", json!({}))["error"]["code"],
        "PROTOCOL_VERSION"
    );
    let preview = client.call("repo.preview", json!({"path":file}));
    assert_eq!(preview["file_hint"], "src/sample.txt");
    assert!(
        client.call("repo.list", json!({}))["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let binding = client.call(
        "repo.add",
        json!({"path":file,"token":preview["token"],"display_name":"Local fixture"}),
    );
    let id = binding["binding_id"].clone();
    assert_eq!(binding["permissions_granted"], false);
    let folder = client.call("repo.preview", json!({"path":repo}));
    let existing = client.call("repo.add", json!({"path":repo,"token":folder["token"]}));
    assert_eq!(existing["binding_id"], id);

    let conversation = client.call(
        "conversation.create",
        json!({"key":"conversation-1","title":"Local review","bindings":[id]}),
    );
    let conversation_id = conversation["id"].clone();
    let message = json!({"id":conversation_id,"key":"message-1","content":"Inspect one selected file","read_scope":{id.as_str().unwrap():["src/sample.txt"]}});
    let first = client.call("conversation.send", message.clone());
    let duplicate = client.call("conversation.send", message);
    assert_eq!(first["message"]["id"], duplicate["message"]["id"]);
    assert_eq!(first["task"]["state"], "WAITING_APPROVAL");
    assert_eq!(first["task"]["route"]["release_checked"], false);
    let rejected = client.request(
        1,
        "conversation.send",
        json!({"id":conversation_id,"key":"escape",
        "content":"Try disallowed path","read_scope":{id.as_str().unwrap():["../other.txt"]}}),
    );
    assert_eq!(rejected["ok"], false);

    let definition = client.call(
        "automation.create",
        json!({"key":"definition-1","name":"Fixture review",
        "binding_id":id,"prompt":"Inspect fixture","trigger":{"kind":"MANUAL"},"enabled":false,
        "read_paths":["src/sample.txt"]}),
    );
    let def_id = definition["id"].clone();
    let request = json!({"id":def_id,"key":"occurrence-1","confirm_inactive":true});
    let run = client.call("automation.run", request.clone());
    assert_eq!(run["id"], client.call("automation.run", request)["id"]);
    let mut status = Value::Null;
    for _ in 0..35 {
        let history = client.call("automation.runs", json!({"id":def_id}));
        status = history["items"][0]["status"].clone();
        if status == "WAITING_APPROVAL" {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(status, "WAITING_APPROVAL");
    assert_eq!(
        client.call("automation.list", json!({}))["items"][0]["enabled"],
        false
    );
    assert_eq!(fs::read_to_string(&file).unwrap(), "original\n");
    client.close();

    let relocated = fixture.0.join("Moved repository");
    fs::rename(&repo, &relocated).unwrap();
    let mut client = Runtime::open(&data);
    assert_eq!(
        client.call("conversation.get", json!({"id":conversation_id}))["messages"][0]["id"],
        first["message"]["id"]
    );
    let old = client.call("repo.health", json!({"binding_id":id}));
    assert_eq!(old["status"], "MISSING");
    let preview = client.call(
        "repo.preview",
        json!({"path":relocated.join("src/sample.txt")}),
    );
    let rebound = client.call(
        "repo.relink",
        json!({"binding_id":id,"revision":binding["revision"],
        "path":relocated.join("src/sample.txt"),"token":preview["token"]}),
    );
    assert_eq!(rebound["binding_id"], id);
    assert_eq!(
        rebound["revision"].as_i64(),
        Some(binding["revision"].as_i64().unwrap() + 1)
    );
    assert_eq!(
        client.request(
            1,
            "automation.run",
            json!({"id":def_id,"key":"stale","confirm_inactive":true})
        )["ok"],
        false
    );
    let snapshot = client.call("storage.backup", json!({}));
    let restored = fixture.0.join("restore");
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new().mode(0o700).create(&restored).unwrap();
    }
    #[cfg(not(unix))]
    fs::create_dir(&restored).unwrap();
    fs::copy(
        snapshot["path"].as_str().unwrap(),
        restored.join("cgacp.sqlite3"),
    )
    .unwrap();
    client.close();
    let mut copy = Runtime::open(&restored);
    assert_eq!(copy.call("storage.verify", json!({}))["status"], "PASS");
    assert_eq!(
        copy.call("repo.list", json!({}))["items"][0]["binding_id"],
        id
    );
    copy.close();
    println!("Runtime fixture elapsed_ms={}", start.elapsed().as_millis());
}
