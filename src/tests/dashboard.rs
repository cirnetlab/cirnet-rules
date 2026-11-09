#[path = "../test_support.rs"]
mod test_support;

use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::{Duration, Instant},
};

struct Runtime {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
    request: u64,
}
impl Runtime {
    fn open(data: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_openrules-runtime"))
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
        let wire_method = if version == 1 {
            method.to_owned()
        } else {
            format!("openrules.{method}")
        };
        serde_json::to_writer(
            &mut *input,
            &json!({"v":version,"id":id,"method":wire_method,"params":params}),
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
        let response = self.request(2, method, params);
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
        Self(test_support::temporary_directory("dashboard-fixture").unwrap())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn integration_inventory_and_import_preview_cross_private_ipc() {
    let fixture = Fixture::new();
    let mut runtime = Runtime::open(&fixture.0.join("integration"));
    let adapters = runtime.call("integration.list", json!({}));
    let rows = adapters["adapters"].as_array().unwrap();
    let opencode = rows.iter().find(|row| row["id"] == "opencode").unwrap();
    assert_eq!(opencode["version"], "2.2.5");
    assert_eq!(opencode["capabilities"]["probe"], "AVAILABLE");
    assert_eq!(opencode["capabilities"]["create_session"], "BLOCKED");
    let result = runtime.call(
        "integration.import_preview",
        json!({"format":"opencode-v1-agent","source":r#"{"disable":false,"permission":"allow"}"#}),
    );
    assert_eq!(result["metadata"], json!({"disabled":false}));
    assert!(
        result["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| { entry["field"] == "permission" && entry["status"] == "REJECTED" })
    );
    runtime.close();
}

#[test]
fn slow_integration_probe_keeps_private_dispatch_responsive_and_bounded() {
    use std::{io::Read, net::TcpListener, sync::mpsc, thread};

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    let (accepted, await_accept) = mpsc::sync_channel(1);
    let (release, await_release) = mpsc::sync_channel(1);
    let server = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error)
                    if error.kind() == std::io::ErrorKind::WouldBlock
                        && Instant::now() < deadline =>
                {
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => {
                    panic!("loopback integration fixture could not accept the probe: {error}")
                }
            }
        };
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut request = Vec::new();
        let mut byte = [0; 1];
        while !request.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
            assert!(request.len() < 4096);
        }
        assert!(request.starts_with(b"GET /server_info HTTP/1.0\r\n"));
        accepted.send(()).unwrap();
        await_release.recv_timeout(Duration::from_secs(2)).unwrap();
        let body = r#"{"version":"1.60.1"}"#;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
    });

    let fixture = Fixture::new();
    let mut runtime = Runtime::open(&fixture.0.join("slow-integration"));
    let params = json!({"adapter":"openhands","port":port});
    let input = runtime.input.as_mut().unwrap();
    writeln!(
        input,
        "{}",
        json!({"v":2,"id":"slow","method":"openrules.integration.probe","params":params})
    )
    .unwrap();
    input.flush().unwrap();
    await_accept.recv_timeout(Duration::from_secs(2)).unwrap();
    let started = Instant::now();
    assert_eq!(runtime.call("runtime.info", json!({}))["protocol"], 2);
    assert_eq!(
        runtime.request(2, "integration.probe", params)["error"]["code"],
        "PROBE_BUSY"
    );
    eprintln!(
        "Concurrent runtime.info and rejected probe IPC took {} us",
        started.elapsed().as_micros()
    );
    release.send(()).unwrap();

    let mut line = String::new();
    assert_ne!(runtime.output.read_line(&mut line).unwrap(), 0);
    let response: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(response["id"], "slow");
    assert_eq!(response["result"]["contract"], "MATCHED");
    assert_eq!(response["result"]["native_execution"], "BLOCKED");
    server.join().unwrap();
    runtime.close();
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

fn finish_storage_job(runtime: &mut Runtime, start: Value) -> Value {
    let id = start["operation_id"].as_str().unwrap();
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        let view = runtime.call("operation.get", json!({"operation_id":id}));
        match view["state"].as_str() {
            Some("SUCCEEDED") => return view["result"].clone(),
            Some("QUEUED" | "RUNNING") if Instant::now() < until => {
                std::thread::sleep(Duration::from_millis(10));
            }
            _ => panic!("Storage job did not succeed: {view}"),
        }
    }
}

#[test]
fn chat_storage_counts_only_local_message_bytes_by_conversation_state() {
    let fixture = Fixture::new();
    let data = fixture.0.join("chat-storage");
    let mut runtime = Runtime::open(&data);
    runtime.call("runtime.info", json!({}));
    let measure = |runtime: &mut Runtime| {
        let started = runtime.call("storage.chats", json!({}));
        assert_eq!(started["kind"], "CHAT_USAGE");
        finish_storage_job(runtime, started)
    };
    let empty = measure(&mut runtime);
    assert_eq!(
        empty["active"],
        json!({"conversations":0,"message_bytes":0})
    );
    assert_eq!(empty["archived"], empty["active"]);
    let active = "Tiếng Việt \0 🧪";
    let archived = "Đã lưu";
    {
        let db = rusqlite::Connection::open(data.join("openrules.sqlite3")).unwrap();
        db.busy_timeout(Duration::from_secs(1)).unwrap();
        db.execute_batch(
            "INSERT INTO conversations
               (id,create_key,create_digest,title,profile,state,source,revision,created_at,updated_at)
             VALUES (1,'active','1','Active','handoff','ACTIVE','DASHBOARD',1,1,1),
                    (2,'archived','2','Archived','handoff','ARCHIVED','AUTOMATION',1,1,1),
                    (3,'empty','3','Empty archived','handoff','ARCHIVED','DASHBOARD',1,1,1);
             CREATE TABLE fixture_unrelated(payload BLOB NOT NULL);
             INSERT INTO fixture_unrelated VALUES(zeroblob(1048576));",
        )
        .unwrap();
        for (id, message) in [(1, active), (2, archived)] {
            db.execute(
                "INSERT INTO messages
                   (id,conversation_id,request_key,request_digest,ordinal,role,content,created_at)
                 VALUES (?1,?1,'message','digest',1,'USER',?2,1)",
                rusqlite::params![id, message],
            )
            .unwrap();
        }
    }
    let usage = measure(&mut runtime);
    assert_eq!(
        usage["active"],
        json!({"conversations":1,"message_bytes":active.len()})
    );
    assert_eq!(
        usage["archived"],
        json!({"conversations":2,"message_bytes":archived.len()})
    );
    assert_eq!(usage.as_object().unwrap().len(), 3);
    assert!(usage["measured_at"].is_u64());
    assert_eq!(
        runtime.request(2, "storage.chats", json!({"root":"."}))["error"]["code"],
        "INVALID_INPUT"
    );
    runtime.close();
}

#[test]
fn storage_jobs_measure_verify_and_compact_an_isolated_store() {
    let fixture = Fixture::new();
    let data = fixture.0.join("storage-jobs");
    let mut runtime = Runtime::open(&data);
    assert_eq!(
        runtime.call("runtime.info", json!({}))["storage_jobs"]["workers"],
        1
    );
    {
        let db = rusqlite::Connection::open(data.join("openrules.sqlite3")).unwrap();
        db.busy_timeout(Duration::from_secs(1)).unwrap();
        db.execute_batch(
            "CREATE TABLE fixture_padding(payload BLOB NOT NULL);
             INSERT INTO fixture_padding VALUES(zeroblob(2097152));
             DELETE FROM fixture_padding;",
        )
        .unwrap();
    }

    let backup_job = runtime.call("storage.backup.start", json!({}));
    assert_eq!(backup_job["kind"], "BACKUP");
    let backup = finish_storage_job(&mut runtime, backup_job);
    let name = backup["backup"].as_str().unwrap();
    let backup_dir = data.join("backups");
    let backup_size = fs::metadata(backup_dir.join(name)).unwrap().len();
    if let Err(error) = fs::hard_link(
        backup_dir.join(name),
        backup_dir.join("snapshot-ffffffffffffffffffffffffffffffff.sqlite3"),
    ) {
        assert!(matches!(
            error.kind(),
            std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::Unsupported
        ));
        eprintln!("Hard-link alias unavailable in this fixture environment: {error}");
    }
    let verify = runtime.call("storage.verify.start", json!({"backup":name}));
    assert_eq!(finish_storage_job(&mut runtime, verify)["status"], "PASS");

    let scan_job = runtime.call("storage.scan", json!({}));
    let scan = finish_storage_job(&mut runtime, scan_job);
    let physical = &scan["physical"];
    assert_eq!(scan["partial"], false, "{scan}");
    assert_eq!(physical["backup_bytes"], backup_size);
    assert_eq!(scan["backup_count"], 1);
    let sum: u64 = physical
        .as_object()
        .unwrap()
        .iter()
        .filter(|(key, _)| key.as_str() != "total_bytes")
        .map(|(_, bytes)| bytes.as_u64().unwrap())
        .sum();
    assert_eq!(physical["total_bytes"], sum);
    let page_count = scan["logical"]["page_count"].as_u64().unwrap();
    let free_pages = scan["logical"]["free_pages"].as_u64().unwrap();
    assert!(free_pages > 0);

    let stale = runtime.call(
        "storage.compact.start",
        json!({
            "expected_page_count":page_count + 1, "expected_free_pages":free_pages
        }),
    );
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        let view = runtime.call(
            "operation.get",
            json!({"operation_id":stale["operation_id"]}),
        );
        if view["state"] == "FAILED" {
            assert_eq!(view["error"]["code"], "CONFLICT");
            break;
        }
        assert!(Instant::now() < until, "{view}");
        std::thread::sleep(Duration::from_millis(10));
    }

    let compact = runtime.call(
        "storage.compact.start",
        json!({
            "expected_page_count":page_count, "expected_free_pages":free_pages
        }),
    );
    let receipt = finish_storage_job(&mut runtime, compact);
    assert_eq!(receipt["before_page_count"], page_count);
    assert!(receipt["after_page_count"].as_u64().unwrap() < page_count);
    assert_eq!(receipt["after_free_pages"], 0);
    assert_eq!(runtime.call("storage.verify", json!({}))["status"], "PASS");
    runtime.close();
}

#[test]
fn reduced_motion_defaults_for_existing_preferences_and_persists() {
    let fixture = Fixture::new();
    let data = fixture.0.join("motion");
    let mut initializer = Runtime::open(&data);
    assert_eq!(
        initializer.call("storage.verify", json!({}))["schema_revision"],
        7
    );
    initializer.close();
    {
        let database = rusqlite::Connection::open(data.join("openrules.sqlite3")).unwrap();
        let previous = json!({
            "theme":"dark","density":"comfortable","text_scale":100,
            "sidebar_width":260,"sidebar_collapsed":false,"notification_level":"all"
        });
        database
            .execute(
                "INSERT INTO desktop_preferences (singleton, revision, settings) VALUES (1, 1, ?1)",
                [previous.to_string()],
            )
            .unwrap();
    }

    let mut client = Runtime::open(&data);
    let initial = client.call("preferences.get", json!({}));
    assert_eq!(initial["revision"], 1);
    assert_eq!(initial["settings"]["reduced_motion"], false);
    let mut settings = initial["settings"].clone();
    settings["reduced_motion"] = json!(true);
    let updated = client.call(
        "preferences.set",
        json!({"expected_revision":1,"settings":settings}),
    );
    assert_eq!(updated, json!({"revision":2,"settings":settings}));
    settings["reduced_motion"] = json!("false");
    assert_eq!(
        client.request(
            2,
            "preferences.set",
            json!({"expected_revision":2,"settings":settings}),
        )["error"]["code"],
        "INVALID_INPUT"
    );
    client.close();

    let mut reopened = Runtime::open(&data);
    assert_eq!(reopened.call("preferences.get", json!({})), updated);
    assert_eq!(
        reopened.call("storage.verify", json!({}))["schema_revision"],
        7
    );
    reopened.close();
}

#[test]
fn preferences_upgrade_backs_up_and_preserves_operator_data() {
    use sha2::{Digest, Sha256};

    let fixture = Fixture::new();
    let data = fixture.0.join("previous-schema");
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new().mode(0o700).create(&data).unwrap();
    }
    #[cfg(not(unix))]
    fs::create_dir(&data).unwrap();
    let database = data.join("openrules.sqlite3");
    {
        let conn = rusqlite::Connection::open(&database).unwrap();
        conn.execute_batch(
            "PRAGMA foreign_keys=ON;
            CREATE TABLE schema_migrations (
                revision INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE,
                checksum TEXT NOT NULL, applied_at INTEGER NOT NULL
            ) STRICT;",
        )
        .unwrap();
        let migrations = [
            (
                "001_catalog.sql",
                include_str!("../../migrations/sqlite/001_catalog.sql"),
            ),
            (
                "002_sessions.sql",
                include_str!("../../migrations/sqlite/002_sessions.sql"),
            ),
            (
                "003_legacy.sql",
                include_str!("../../migrations/sqlite/003_legacy.sql"),
            ),
        ];
        for (index, (name, sql)) in migrations.into_iter().enumerate() {
            conn.execute_batch(sql).unwrap();
            conn.execute(
                "INSERT INTO schema_migrations VALUES (?1, ?2, ?3, 1)",
                rusqlite::params![
                    (index + 1) as i64,
                    name,
                    format!("{:x}", Sha256::digest(sql))
                ],
            )
            .unwrap();
        }
        conn.execute_batch(
            "PRAGMA user_version=3;
             INSERT INTO repositories VALUES ('repo-fixture','common','Fixture repo',1,0,1,1);
             INSERT INTO bindings VALUES (
                'binding-fixture','repo-fixture','machine','/fixture','/fixture','/fixture/.git',
                '/fixture/.git','identity','anchor','main',NULL,0,'ACTIVE',1,1,NULL
             );
             INSERT INTO conversations VALUES (2,'create','digest','Existing chat','handoff',
                'ACTIVE','DASHBOARD',NULL,1,1,1);
             INSERT INTO conversation_bindings VALUES (2,'binding-fixture',1,1);
             INSERT INTO messages VALUES (3,2,'message','digest',1,'USER','Existing prompt',1);
             INSERT INTO task_proposals VALUES (4,2,3,'WAITING_APPROVAL','WORK','SCOPED','{}',1);
             INSERT INTO automation_definitions VALUES (5,'automation','digest','Existing automation',
                'binding-fixture',1,'handoff','Existing workflow','[]','scoped-code-review',1,
                'WORK','SCOPED',1,0,'{\"kind\":\"MANUAL\"}',NULL,1,1);
             INSERT INTO automation_runs VALUES (6,5,1,'binding-fixture',1,'MANUAL','run',NULL,
                'WAITING_APPROVAL','{}',NULL,NULL,0,2,4,NULL,1,1);",
        )
        .unwrap();
    }

    let blocked_backup = data.join("backups");
    fs::write(&blocked_backup, "not a backup directory").unwrap();
    let refused = Command::new(env!("CARGO_BIN_EXE_openrules-runtime"))
        .arg("--stdio")
        .arg("--data-dir")
        .arg(&data)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("STORAGE_UNAVAILABLE"));
    {
        let conn = rusqlite::Connection::open(&database).unwrap();
        let revision: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(revision, 3);
    }
    fs::remove_file(&blocked_backup).unwrap();

    let mut client = Runtime::open(&data);
    let initial = client.call("preferences.get", json!({}));
    assert_eq!(initial["revision"], 0);
    assert_eq!(initial["settings"]["theme"], "dark");
    assert_eq!(
        client.call("storage.verify", json!({}))["schema_revision"],
        7
    );
    let mut backups = fs::read_dir(data.join("backups")).unwrap();
    let previous = backups
        .next()
        .unwrap()
        .unwrap()
        .file_name()
        .into_string()
        .unwrap();
    assert!(backups.next().is_none());
    assert_eq!(
        client.call("storage.verify", json!({"backup":previous}))["schema_revision"],
        3
    );

    let mut settings = initial["settings"].clone();
    settings["theme"] = json!("light");
    settings["repository_layout"] = json!({
        "widths":[160,380,136,100,112],"order":[1,0,2,3,4],"sort_column":1,
        "sort_descending":true,"show_archived":true,"status":"archived",
        "selected_id":"binding-1234567890abcdef1234567890abcdef","scroll":42,"horizontal_scroll":100
    });
    let saved = client.call(
        "preferences.set",
        json!({"expected_revision":0,"settings":settings}),
    );
    assert_eq!(saved, json!({"revision":1,"settings":settings}));
    let mut invalid = settings.clone();
    invalid["repository_layout"]["path"] = json!("C:/private");
    assert_eq!(
        client.request(
            2,
            "preferences.set",
            json!({"expected_revision":1,"settings":invalid}),
        )["error"]["code"],
        "INVALID_INPUT"
    );
    assert_eq!(
        client.request(
            2,
            "preferences.set",
            json!({"expected_revision":0,"settings":settings}),
        )["error"]["code"],
        "CONFLICT"
    );
    settings["sidebar_collapsed"] = json!(true);
    let updated = client.call(
        "preferences.set",
        json!({"expected_revision":1,"settings":settings}),
    );
    assert_eq!(updated, json!({"revision":2,"settings":settings}));
    client.close();
    let mut reopened = Runtime::open(&data);
    assert_eq!(reopened.call("preferences.get", json!({})), updated);
    reopened.close();

    let records = "SELECT bindings.id, conversations.id, messages.content, task_proposals.id,
                          automation_runs.id, automation_definitions.template_id
                   FROM automation_runs
                   JOIN automation_definitions ON automation_definitions.id=automation_runs.automation_id
                   JOIN bindings ON bindings.id=automation_runs.binding_id
                   JOIN task_proposals ON task_proposals.id=automation_runs.task_id
                   JOIN messages ON messages.id=task_proposals.message_id
                   JOIN conversations ON conversations.id=messages.conversation_id";
    for path in [database, data.join("backups").join(previous)] {
        let conn = rusqlite::Connection::open(path).unwrap();
        let values: (String, i64, String, i64, i64, String) = conn
            .query_row(records, [], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })
            .unwrap();
        assert_eq!(
            values,
            (
                "binding-fixture".into(),
                2,
                "Existing prompt".into(),
                4,
                6,
                "scoped-code-review".into()
            )
        );
    }
}

#[test]
fn closing_request_pipe_drains_accepted_replies() {
    let fixture = Fixture::new();
    let data = fixture.0.join("eof");
    let mut initializer = Runtime::open(&data);
    initializer.call("storage.verify", json!({}));
    initializer.close();
    {
        let db = rusqlite::Connection::open(data.join("openrules.sqlite3")).unwrap();
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
    let mut child = Command::new(env!("CARGO_BIN_EXE_openrules-runtime"))
        .arg("--stdio")
        .arg("--data-dir")
        .arg(data)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut stdin = child.stdin.take().unwrap();
        writeln!(
            stdin,
            "{}",
            json!({"v":2,"id":"1","method":"openrules.runtime.info"})
        )
        .unwrap();
        for id in 2..=8 {
            writeln!(
                stdin,
                "{}",
                json!({"v":2,"id":id.to_string(),"method":"openrules.conversation.get","params":{"id":"1"}})
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
fn conversation_lifecycle_survives_restart_and_fences_stale_requests() {
    let fixture = Fixture::new();
    let data = fixture.0.join("lifecycle");
    let repo = fixture.0.join("lifecycle-repo");
    fs::create_dir(&repo).unwrap();
    git(&repo, &["init", "--quiet"]);
    git(
        &repo,
        &[
            "-c",
            "user.name=OPENRULES Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "--allow-empty",
            "--quiet",
            "-m",
            "Fixture",
        ],
    );
    let mut client = Runtime::open(&data);
    let checkout = client.call("repo.preview", json!({"path":repo}));
    let binding =
        client.call("repo.add", json!({"path":repo,"token":checkout["token"]}))["binding_id"]
            .clone();
    let create = json!({"key":"lifecycle-chat","title":"Lifecycle fixture","bindings":[binding]});
    let chat = client.call("conversation.create", create.clone());
    let cid = &chat["id"];
    let other = client.call(
        "conversation.create",
        json!({"key":"other-chat","title":"Other fixture","bindings":[binding]}),
    );
    let first = client.call("conversation.send",
        json!({"id":cid,"revision":chat["revision"],"key":"turn-first","content":"First retained payload"}));
    let first_id = &first["message"]["id"];
    let active = client.call(
        "message.delete.preview",
        json!({"id":cid,"message_id":first_id}),
    );
    let active_delete = json!({"id":cid,"message_id":first_id,"revision":active["revision"],
        "token":active["token"],"key":"delete-first"});
    let denied = client.request(2, "message.delete", active_delete.clone());
    assert_eq!(denied["error"]["code"], "ACTIVE_DEPENDENCY");
    assert_eq!(
        denied["error"]["details"]["blockers"][0]["id"],
        first["task"]["id"]
    );
    assert_eq!(
        client.request(
            2,
            "message.delete.preview",
            json!({"id":other["id"],"message_id":first_id})
        )["error"]["code"],
        "NOT_FOUND"
    );
    assert_eq!(
        client.request(
            2,
            "conversation.send",
            json!({
                "id":cid,"revision":chat["revision"],"key":"stale-send","content":"Stale request"
            })
        )["error"]["code"],
        "CONFLICT"
    );

    let send_second = json!({"id":cid,"revision":first["conversation_revision"],
        "key":"turn-second","content":"Second payload to remove"});
    let second = client.call("conversation.send", send_second.clone());
    let second_id = &second["message"]["id"];
    let forward = client.call("conversation.get", json!({"id":cid,"after":"1"}));
    assert_eq!(forward["messages"].as_array().unwrap().len(), 1);
    assert_eq!(forward["messages"][0]["id"], *second_id);
    assert_eq!(forward["next_after"], "2");
    assert_eq!(forward["has_more"], false);

    for task in [&first["task"]["id"], &second["task"]["id"]] {
        client.call("conversation.cancel", json!({"id":cid,"task_id":task}));
    }
    assert_eq!(
        client.request(2, "message.delete", active_delete)["error"]["code"],
        "STALE_PREVIEW"
    );
    let scope = client.call(
        "message.delete.preview",
        json!({"id":cid,"message_id":second_id}),
    );
    let delete_second = json!({"id":cid,"message_id":second_id,"revision":scope["revision"],
        "token":scope["token"],"key":"delete-second"});
    let receipt = client.call("message.delete", delete_second.clone());
    assert_eq!(receipt["status"], "DELETED");
    assert_eq!(
        client.call("message.delete", delete_second.clone()),
        receipt
    );
    assert_eq!(
        client.request(2, "conversation.send", send_second)["error"]["code"],
        "DELETED"
    );
    let remaining = client.call("conversation.get", json!({"id":cid}));
    assert_eq!(remaining["messages"].as_array().unwrap().len(), 1);
    assert_eq!(remaining["messages"][0]["id"], *first_id);
    let replacement = client.call(
        "conversation.send",
        json!({"id":cid,"key":"turn-third","content":"Third payload"}),
    );
    assert_eq!(replacement["message"]["ordinal"], 3);
    assert!(
        replacement["message"]["id"]
            .as_str()
            .unwrap()
            .parse::<i64>()
            .unwrap()
            > second_id.as_str().unwrap().parse::<i64>().unwrap()
    );
    client.call(
        "conversation.cancel",
        json!({"id":cid,"task_id":replacement["task"]["id"]}),
    );
    let current = client.call("conversation.get", json!({"id":cid}));
    let archived = client.call(
        "conversation.archive",
        json!({"id":cid,"revision":current["conversation"]["revision"]}),
    );
    assert_eq!(archived["state"], "ARCHIVED");
    assert_eq!(
        client.call("conversation.list", json!({"query":"Lifecycle"}))["items"],
        json!([])
    );
    client.close();

    let mut client = Runtime::open(&data);
    assert_eq!(client.call("message.delete", delete_second), receipt);
    let archived_page = client.call(
        "conversation.list",
        json!({"state":"ARCHIVED","query":"Lifecycle","binding_id":binding,"limit":1}),
    );
    assert_eq!(archived_page["total"], 1);
    assert_eq!(archived_page["items"][0]["id"], *cid);
    assert_eq!(archived_page["next_before"], Value::Null);
    let restored = client.call(
        "conversation.unarchive",
        json!({"id":cid,"revision":archived["revision"]}),
    );
    assert_eq!(restored["state"], "ACTIVE");
    assert_eq!(restored["id"], *cid);
    let archived = client.call(
        "conversation.archive",
        json!({"id":cid,"revision":restored["revision"]}),
    );
    let scope = client.call("conversation.delete.preview", json!({"id":cid}));
    assert_eq!(scope["message_count"], 2);
    assert_eq!(scope["revision"], archived["revision"]);
    let delete_chat =
        json!({"id":cid,"revision":scope["revision"],"token":scope["token"],"key":"delete-chat"});
    let receipt = client.call("conversation.delete", delete_chat.clone());
    assert_eq!(receipt["status"], "DELETED");
    client.close();

    let mut client = Runtime::open(&data);
    assert_eq!(client.call("conversation.delete", delete_chat), receipt);
    assert_eq!(
        client.request(2, "conversation.get", json!({"id":cid}))["error"]["code"],
        "NOT_FOUND"
    );
    assert_eq!(
        client.request(2, "conversation.create", create)["error"]["code"],
        "DELETED"
    );
    assert_eq!(
        client.call("conversation.list", json!({"state":"ARCHIVED"}))["total"],
        0
    );
    assert_eq!(
        client.request(
            2,
            "conversation.send",
            json!({"id":cid,"key":"late","content":"Late writer"})
        )["error"]["code"],
        "NOT_FOUND"
    );
    assert_eq!(
        client.call("conversation.get", json!({"id":other["id"]}))["conversation"]["id"],
        other["id"]
    );
    let next = client.call(
        "conversation.create",
        json!({"key":"new-chat","title":"New fixture","bindings":[binding]}),
    );
    assert!(
        next["id"].as_str().unwrap().parse::<i64>().unwrap()
            > other["id"].as_str().unwrap().parse::<i64>().unwrap()
    );
    client.close();

    let conn = rusqlite::Connection::open(data.join("openrules.sqlite3")).unwrap();
    let remaining: i64 = conn
        .query_row(
            "SELECT count(*) FROM messages WHERE conversation_id=?1",
            [cid.as_str().unwrap()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(remaining, 0);
    let checks: i64 = conn
        .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(checks, 0);
}

#[test]
fn conversation_deletion_retires_only_its_originating_automation() {
    let fixture = Fixture::new();
    let data = fixture.0.join("data");
    let repo = fixture.0.join("repo");
    fs::create_dir(&repo).unwrap();
    fs::write(repo.join("sample.txt"), "fixture\n").unwrap();
    git(&repo, &["init", "--quiet"]);
    git(&repo, &["add", "--", "sample.txt"]);
    git(
        &repo,
        &[
            "-c",
            "user.name=Fixture",
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
    let mut client = Runtime::open(&data);
    let repo_preview = client.call("repo.preview", json!({"path":repo}));
    let binding = client.call(
        "repo.add",
        json!({"path":repo,"token":repo_preview["token"]}),
    );
    let binding_id = &binding["binding_id"];
    let define = |key: &str| {
        json!({
            "key":key,"name":"Same display name","binding_id":binding_id,"prompt":"Review the fixture",
            "trigger":{"kind":"MANUAL"},"enabled":false,"read_paths":["sample.txt"]
        })
    };
    let owned = client.call("automation.create", define("linked"));
    let unrelated = client.call("automation.create", define("unrelated"));
    let owned_id = &owned["id"];
    let unrelated_id = &unrelated["id"];
    let start_run = |client: &mut Runtime, key: &str| {
        let run = client.call(
            "automation.run",
            json!({"id":owned_id,"key":key,"confirm_inactive":true}),
        );
        let start = Instant::now();
        loop {
            let history = client.call("automation.runs", json!({"id":owned_id}));
            let row = history["items"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["id"] == run["id"])
                .unwrap();
            if row["status"] == "WAITING_APPROVAL" {
                return row.clone();
            }
            assert!(start.elapsed() < Duration::from_secs(5), "{row}");
            std::thread::sleep(Duration::from_millis(20));
        }
    };
    let first = start_run(&mut client, "first");
    let chat = &first["conversation_id"];
    client.call("automation.cancel", json!({"run_id":first["id"]}));
    let chat_revision =
        client.call("conversation.get", json!({"id":chat}))["conversation"]["revision"].clone();
    let archived = client.call(
        "conversation.archive",
        json!({"id":chat,"revision":chat_revision}),
    );
    assert_eq!(archived["state"], "ARCHIVED");
    assert!(
        client.call("automation.list", json!({}))["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["id"] == *owned_id)
    );
    client.call(
        "conversation.unarchive",
        json!({"id":chat,"revision":archived["revision"]}),
    );
    let message = client.call("conversation.get", json!({"id":chat}))["messages"][0]["id"].clone();
    let message_preview = client.call(
        "message.delete.preview",
        json!({"id":chat,"message_id":message}),
    );
    client.call(
        "message.delete",
        json!({"id":chat,"message_id":message,"revision":message_preview["revision"],
        "token":message_preview["token"],"key":"message-only"}),
    );
    assert_eq!(
        client.call("automation.list", json!({}))["items"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let sibling = start_run(&mut client, "sibling");
    let blocked = client.call("conversation.delete.preview", json!({"id":chat}));
    assert_eq!(blocked["automations"][0]["id"], *owned_id);
    assert!(
        blocked["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["id"] == sibling["id"] && row["kind"] == "run")
    );
    assert_eq!(
        client.request(
            2,
            "conversation.delete",
            json!({"id":chat,"revision":blocked["revision"],
        "token":blocked["token"],"key":"while-active"})
        )["error"]["code"],
        "ACTIVE_DEPENDENCY"
    );
    client.call("automation.cancel", json!({"run_id":sibling["id"]}));

    let stale = client.call("conversation.delete.preview", json!({"id":chat}));
    client.call(
        "automation.pause",
        json!({"id":owned_id,"revision":owned["revision"],"enabled":true}),
    );
    assert_eq!(
        client.request(
            2,
            "conversation.delete",
            json!({"id":chat,"revision":stale["revision"],
        "token":stale["token"],"key":"stale-preview"})
        )["error"]["code"],
        "STALE_PREVIEW"
    );
    let ready = client.call("conversation.delete.preview", json!({"id":chat}));
    let remove = json!({"id":chat,"revision":ready["revision"],"token":ready["token"],"key":"linked-delete"});
    let receipt = client.call("conversation.delete", remove.clone());
    assert_eq!(receipt["status"], "DELETED");
    assert_eq!(
        client.call("automation.list", json!({}))["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        client.call("automation.list", json!({}))["items"][0]["id"],
        *unrelated_id
    );
    client.close();

    let mut reopened = Runtime::open(&data);
    assert_eq!(reopened.call("conversation.delete", remove), receipt);
    assert_eq!(
        reopened.request(2, "conversation.get", json!({"id":chat}))["error"]["code"],
        "NOT_FOUND"
    );
    assert_eq!(
        reopened.call("conversation.get", json!({"id":sibling["conversation_id"]}))["conversation"]
            ["id"],
        sibling["conversation_id"]
    );
    let history = reopened.call("automation.runs", json!({"id":owned_id}));
    assert_eq!(history["items"].as_array().unwrap().len(), 2);
    assert!(
        history["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["status"] == "CANCELLED")
    );
    assert_eq!(
        reopened.request(
            2,
            "automation.run",
            json!({"id":owned_id,"key":"after-delete"})
        )["ok"],
        false
    );
    assert_eq!(
        reopened.request(2, "automation.create", define("linked"))["ok"],
        false
    );
    let remove_detached = json!({"id":unrelated_id,"revision":unrelated["revision"]});
    let detached_receipt = reopened.call("automation.delete", remove_detached.clone());
    assert_eq!(detached_receipt["status"], "DELETED");
    assert_eq!(
        reopened.call("automation.delete", remove_detached),
        detached_receipt
    );
    assert!(
        reopened.call("automation.list", json!({}))["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    reopened.close();
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
            "user.name=OPENRULES Fixture",
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
    assert_eq!(info["product_generation"], "OPENRULES");
    assert_eq!(info["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(
        info["release_name"],
        format!("cirnet-rules-{}", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(
        client.request(1, "repo.add", json!({"path":file,"token":"unreviewed"}))["error"]["code"],
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
        2,
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
            2,
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
        restored.join("openrules.sqlite3"),
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

#[path = "governance_cases.rs"]
mod governance_cases;
