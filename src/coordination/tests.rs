use super::*;
use serde_json::json;
use std::{
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = crate::test_support::temporary_directory("cnr-coordination").unwrap();
        fs::write(root.join("governance.yaml"), "test: governance\n").unwrap();
        fs::write(
            root.join(".gitignore"),
            ".cirnetrules/\nconsumer-checkout/\n",
        )
        .unwrap();
        fs::write(root.join("first.txt"), "first before\n").unwrap();
        fs::write(root.join("second.txt"), "second before\n").unwrap();
        for id in ["first", "second"] {
            for (dir, kind) in [("plans", "plan"), ("specs", "spec")] {
                let path = root.join(format!("en/{dir}/cirnet-rules/{id}/{kind}.md"));
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(path, "# Fixture\n").unwrap();
            }
            fs::write(
                root.join(format!("{id}.json")),
                serde_json::to_vec(&descriptor(id)).unwrap(),
            )
            .unwrap();
        }
        Self::git(&root, &["init", "--quiet"]);
        Self::git(&root, &["add", "--", "."]);
        Self::git(
            &root,
            &[
                "-c",
                "user.name=Coordination fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "--quiet",
                "-m",
                "seed",
            ],
        );
        Self(root)
    }

    fn git(root: &Path, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn ready(&self, id: &str) {
        tasks::run(
            "task-create",
            &self.0,
            &options([("descriptor", &format!("{id}.json"))]),
            &registry(),
        )
        .unwrap();
        tasks::run(
            "task-transition",
            &self.0,
            &options([
                ("task", id),
                ("revision", "1"),
                ("actor", "operator"),
                ("to", "READY"),
            ]),
            &registry(),
        )
        .unwrap();
    }

    fn session(&self, id: &str, revision: &str) -> Value {
        self.run(
            "session-prepare",
            options([("task", id), ("revision", revision), ("actor", "operator")]),
        )
        .unwrap()
    }

    fn run(&self, action: &str, args: BTreeMap<String, String>) -> Result<Value, String> {
        run(&self.0, Some(&self.0), &registry(), action, &args)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn descriptor(id: &str) -> Value {
    json!({
        "id":id,"request":"Fixture write","objective":"Observe coordinated source",
        "non_goals":["External writes"],"primary_repo":"cirnet-rules","hints":[],
        "read_scope":{}, "write_scope":{"cirnet-rules":[format!("{id}.txt")]},
        "verification_scope":{}, "documentation_scope":{},
        "risk":"low","role":"implementer","mode":"WORK","owner":"worker",
        "authorization":{"basis":"fixture","owner":"operator","actions":["read","write"]},
        "acceptance":["Fixture bytes sealed"],"commands":[],"docs_targets":[],
        "docs_not_required":"Fixture","spec":format!("en/specs/cirnet-rules/{id}/spec.md"),
        "plan":format!("en/plans/cirnet-rules/{id}/plan.md"),
    })
}

fn registry() -> Value {
    json!({"repositories":{"cirnet-rules":{}},"control_plane":{"commands":{}}})
}

fn options<const N: usize>(items: [(&str, &str); N]) -> BTreeMap<String, String> {
    items
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

fn file(session: &Value) -> &str {
    session["session_file"].as_str().unwrap()
}

#[test]
fn child_process_claim() {
    let Some(root) = std::env::var_os("CNR_FIXTURE_CLAIM_ROOT").map(PathBuf::from) else {
        return;
    };
    let session = std::env::var("CNR_FIXTURE_CLAIM_FILE").unwrap();
    let request = std::env::var("CNR_FIXTURE_CLAIM_REQUEST").unwrap();
    std::io::stdin().read_exact(&mut [0_u8]).unwrap();
    let result = run(
        &root,
        Some(&root),
        &registry(),
        "claim",
        &options([
            ("task", "first"),
            ("revision", "4"),
            ("session-file", &session),
            ("request-id", &request),
        ]),
    );
    if result.as_ref().is_ok_and(|value| value["status"] == "PASS") {
        println!("CNR_CLAIM_WON");
    } else {
        println!("CNR_CLAIM_LOST");
    }
}

#[test]
fn concurrent_processes_commit_only_one_claim_event() {
    let fixture = Fixture::new();
    fixture.ready("first");
    let one = fixture.session("first", "2");
    let two = fixture.session("first", "3");
    let test = std::env::current_exe().unwrap();
    let mut children =
        [("race-one", file(&one)), ("race-two", file(&two))].map(|(request, session)| {
            Command::new(&test)
                .args([
                    "--exact",
                    "coordination::tests::child_process_claim",
                    "--nocapture",
                ])
                .env("CNR_FIXTURE_CLAIM_ROOT", &fixture.0)
                .env("CNR_FIXTURE_CLAIM_FILE", session)
                .env("CNR_FIXTURE_CLAIM_REQUEST", request)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
        });
    for child in &mut children {
        child.stdin.take().unwrap().write_all(&[1]).unwrap();
    }
    let outputs = children.map(|child| child.wait_with_output().unwrap());
    assert!(outputs.iter().all(|output| output.status.success()));
    let won =
        |index: usize| String::from_utf8_lossy(&outputs[index].stdout).contains("CNR_CLAIM_WON");
    assert_ne!(won(0), won(1), "{outputs:?}");
    let state = tasks::load(&fixture.0, "first").unwrap();
    let events = audit_events(&state).unwrap();
    assert_eq!(state["revision"], 5);
    assert_eq!(
        events
            .iter()
            .filter(|event| event["operation"] == "claim")
            .count(),
        1
    );
}

#[test]
fn committed_claim_receipt_replays_without_an_extra_event() {
    let fixture = Fixture::new();
    fixture.ready("first");
    let session = fixture.session("first", "2");
    let mut args = options([
        ("task", "first"),
        ("revision", "3"),
        ("request-id", "retry"),
        ("session-file", file(&session)),
    ]);
    let claimed = fixture.run("claim", args.clone()).unwrap();
    args.insert("ttl".into(), "900".into());
    assert_eq!(fixture.run("claim", args.clone()).unwrap(), claimed);
    args.insert("ttl".into(), "899".into());
    assert!(
        fixture
            .run("claim", args)
            .unwrap_err()
            .contains("Request ID conflict")
    );
    assert_eq!(tasks::load(&fixture.0, "first").unwrap()["revision"], 4);
}

#[test]
fn owner_generation_is_fenced_and_compatibility_writer_conflicts_on_same_worktree() {
    let fixture = Fixture::new();
    fixture.ready("first");
    fixture.ready("second");
    let one = fixture.session("first", "2");
    let two = fixture.session("first", "3");
    fixture
        .run(
            "claim",
            options([
                ("task", "first"),
                ("revision", "4"),
                ("request-id", "claim"),
                ("session-file", file(&one)),
            ]),
        )
        .unwrap();
    let renew = |session, generation| {
        fixture.run(
            "renew",
            options([
                ("task", "first"),
                ("generation", generation),
                ("request-id", "renew"),
                ("session-file", session),
            ]),
        )
    };
    assert!(renew(file(&two), "1").unwrap_err().contains("owner"));
    assert!(renew(file(&one), "2").unwrap_err().contains("generation"));
    assert_eq!(renew(file(&one), "1").unwrap()["lease"]["status"], "ACTIVE");
    let result = tasks::run(
        "task-transition",
        &fixture.0,
        &options([
            ("task", "second"),
            ("revision", "2"),
            ("actor", "worker"),
            ("to", "IN_PROGRESS"),
        ]),
        &registry(),
    )
    .unwrap_err();
    assert!(result.contains("Physical worktree is reserved"), "{result}");
}

#[test]
fn report_is_an_assertion_and_post_report_bytes_are_rejected() {
    let fixture = Fixture::new();
    fixture.ready("first");
    let session = fixture.session("first", "2");
    fixture
        .run(
            "claim",
            options([
                ("task", "first"),
                ("revision", "3"),
                ("request-id", "claim"),
                ("session-file", file(&session)),
            ]),
        )
        .unwrap();
    fs::write(fixture.0.join("first.txt"), "first edit\n").unwrap();
    let log = b"fixture log bytes\n";
    let log_digest = crate::docs::sha256(log);
    let artifacts = fixture.0.join(".cirnetrules/artifacts");
    fs::create_dir(&artifacts).unwrap();
    let log_path = artifacts.join(&log_digest);
    fs::write(&log_path, log).unwrap();
    let report_input = json!({
        "summary":"Fixture claims all checks passed",
        "changed_paths":[{"repo":"cirnet-rules","path":"first.txt"}],
        "artifacts":[log_digest],
    })
    .to_string();
    let report = fixture
        .run(
            "report",
            options([
                ("task", "first"),
                ("revision", "4"),
                ("generation", "1"),
                ("request-id", "report"),
                ("session-file", file(&session)),
                ("report", &report_input),
            ]),
        )
        .unwrap();
    assert_eq!(report["state"], "IMPLEMENTED");
    assert_eq!(report["evidence_kind"], "AGENT_ASSERTION");
    assert_eq!(report["lease"]["status"], "ACTIVE");
    assert_eq!(
        fixture
            .run(
                "release",
                options([
                    ("task", "first"),
                    ("generation", "1"),
                    ("request-id", "release"),
                    ("session-file", file(&session)),
                ])
            )
            .unwrap()["lease"]["status"],
        "SEALED"
    );
    let task = tasks::load(&fixture.0, "first").unwrap();
    assert_eq!(task["evidence"], json!([]));
    let snapshot =
        |task| runner::snapshot(&fixture.0, task, &registry(), Some(&fixture.0)).unwrap();
    check_evidence(
        &fixture.0,
        Some(&fixture.0),
        &registry(),
        &task,
        &snapshot(&task),
    )
    .unwrap();
    fs::write(
        fixture.0.join("first.txt"),
        "same status, different bytes\n",
    )
    .unwrap();
    assert!(
        check_evidence(
            &fixture.0,
            Some(&fixture.0),
            &registry(),
            &task,
            &snapshot(&task)
        )
        .is_err()
    );
    let audit = crate::audit::task(&fixture.0, "first").unwrap().to_string();
    assert!(!audit.contains("Fixture claims all checks passed"));
    assert!(!audit.contains(&session_token(&fixture.0, file(&session)).unwrap()));
}

#[test]
fn dirty_expired_worktree_stays_reserved_for_operator_recovery() {
    let fixture = Fixture::new();
    fixture.ready("first");
    fixture.ready("second");
    let session = fixture.session("first", "2");
    fixture
        .run(
            "claim",
            options([
                ("task", "first"),
                ("revision", "3"),
                ("request-id", "claim"),
                ("session-file", file(&session)),
            ]),
        )
        .unwrap();
    let mut task = tasks::load(&fixture.0, "first").unwrap();
    let mut state = coordinate(&task).unwrap();
    state.lease.as_mut().unwrap().expires_at = clock().unwrap().unix_seconds.saturating_sub(1);
    put(&mut task, state).unwrap();
    tasks::persist(&fixture.0, "first", &task).unwrap();
    fs::write(fixture.0.join("first.txt"), "first edit\n").unwrap();
    assert_eq!(
        fixture.run("status", options([("task", "first")])).unwrap()["lease"]["status"],
        "RECOVERY_REQUIRED"
    );
    let renewal = fixture
        .run(
            "renew",
            options([
                ("task", "first"),
                ("generation", "1"),
                ("request-id", "expire"),
                ("session-file", file(&session)),
            ]),
        )
        .unwrap();
    assert_eq!(renewal["status"], "BLOCKED");
    assert_eq!(renewal["state"], "BLOCKED");
    let recovery = fixture.run(
        "recover",
        options([
            ("task", "first"),
            ("revision", "5"),
            ("generation", "1"),
            ("request-id", "recovery"),
            ("actor", "operator"),
            ("stopped", "yes"),
        ]),
    );
    assert!(recovery.unwrap_err().contains("unowned change"));
    assert!(
        tasks::run(
            "task-transition",
            &fixture.0,
            &options([
                ("task", "second"),
                ("revision", "2"),
                ("actor", "worker"),
                ("to", "IN_PROGRESS"),
            ]),
            &registry()
        )
        .unwrap_err()
        .contains("Physical worktree is reserved")
    );
}

#[test]
fn prior_record_migration_requires_review_and_fresh_grants() {
    let fixture = Fixture::new();
    fixture.ready("first");
    let mut legacy = tasks::load(&fixture.0, "first").unwrap();
    for field in ["task_schema", "store_identity", "coordination"] {
        legacy.as_object_mut().unwrap().remove(field);
    }
    tasks::persist(&fixture.0, "first", &legacy).unwrap();
    let mut args = options([
        ("task", "first"),
        ("revision", "2"),
        ("actor", "operator"),
        ("confirm", "preview"),
    ]);
    let preview = fixture.run("migrate", args.clone()).unwrap();
    assert_eq!(preview["target_grants"], json!([]));
    assert_eq!(tasks::load(&fixture.0, "first").unwrap(), legacy);
    args.insert(
        "confirm".into(),
        preview["review_sha256"].as_str().unwrap().into(),
    );
    let result = fixture.run("migrate", args).unwrap();
    let state = tasks::load(&fixture.0, "first").unwrap();
    assert_eq!(state["state"], "PROPOSED");
    assert_eq!(state["descriptor"]["authorization"]["actions"], json!([]));
    assert_eq!(state["store_identity"]["writer_version"], 2);
    assert_eq!(state["approval_epoch"], 2);
    let archive = fixture.0.join(result["archive"].as_str().unwrap());
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(archive).unwrap()).unwrap(),
        legacy
    );
}

#[test]
fn accepted_output_and_consumer_bytes_are_pinned_across_repositories() {
    let fixture = Fixture::new();
    let product = fixture.0.join("consumer-checkout");
    fs::create_dir(&product).unwrap();
    fs::write(product.join("artifact.txt"), "accepted bytes\n").unwrap();
    Fixture::git(&product, &["init", "--quiet"]);
    Fixture::git(&product, &["add", "--", "artifact.txt"]);
    Fixture::git(
        &product,
        &[
            "-c",
            "user.name=Coordination fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--quiet",
            "-m",
            "seed",
        ],
    );
    let mut policy = registry();
    policy["repositories"]["product"] = json!({});
    policy["control_plane"]["commands"]["check-produced"] = json!({
        "mode":"VERIFY","program":"check-utf8","repository":"cirnet-rules",
        "args":[],"paths":["first.txt"],"timeout_ms":5000,"max_output_bytes":2048,
    });
    let work = fixture.0.to_str().unwrap();
    workspace::run(
        "workspace-bind",
        &fixture.0,
        &options([
            ("workspace", work),
            ("repo", "product"),
            ("path", "consumer-checkout"),
            ("actor", "operator"),
        ]),
        &policy,
    )
    .unwrap();
    let mut producer = descriptor("first");
    producer["authorization"]["actions"] = json!(["read", "write", "verify", "accept"]);
    producer["verification_scope"]["cirnet-rules"] = json!(["first.txt"]);
    producer["commands"] = json!(["check-produced"]);
    let mut consumer = descriptor("second");
    consumer["depends_on"] = json!(["first"]);
    consumer["read_scope"]["product"] = json!(["artifact.txt"]);
    consumer["dependency_inputs"] = json!([{
        "task_id":"first","producer_repo":"cirnet-rules","producer_path":"first.txt",
        "consumer_repo":"product","consumer_path":"artifact.txt",
    }]);
    for (name, descriptor) in [("first.json", producer), ("second.json", consumer)] {
        fs::write(
            fixture.0.join(name),
            serde_json::to_vec(&descriptor).unwrap(),
        )
        .unwrap();
    }
    let create = |name| {
        tasks::run(
            "task-create",
            &fixture.0,
            &options([("descriptor", name)]),
            &policy,
        )
        .unwrap()
    };
    let transition = |task, revision, actor, to| {
        tasks::run(
            "task-transition",
            &fixture.0,
            &options([
                ("task", task),
                ("revision", revision),
                ("actor", actor),
                ("to", to),
                ("workspace", work),
            ]),
            &policy,
        )
    };
    create("first.json");
    transition("first", "1", "operator", "READY").unwrap();
    transition("first", "2", "worker", "IN_PROGRESS").unwrap();
    fs::write(fixture.0.join("first.txt"), "accepted bytes\n").unwrap();
    transition("first", "3", "worker", "IMPLEMENTED").unwrap();
    for (action, revision) in [
        ("task-verify", "4"),
        ("task-accept", "5"),
        ("task-close", "6"),
    ] {
        let result = crate::lifecycle::run(
            action,
            &fixture.0,
            &options([
                ("task", "first"),
                ("actor", "operator"),
                ("revision", revision),
                ("workspace", work),
            ]),
            &policy,
        )
        .unwrap();
        assert_eq!(result["status"], "PASS", "{result}");
    }
    create("second.json");
    let ready = transition("second", "1", "operator", "READY").unwrap();
    let dependency = &ready["task"]["coordination"]["dependencies"][0];
    assert_eq!(
        dependency["outputs"][0]["sha256"],
        dependency["selected_inputs"][0]["sha256"]
    );
    assert_eq!(dependency["selected_inputs"][0]["repository"], "product");

    fs::write(product.join("artifact.txt"), "swapped bytes\n").unwrap();
    assert!(
        transition("second", "2", "worker", "IN_PROGRESS")
            .unwrap_err()
            .contains("selected input bytes")
    );
    fs::write(product.join("artifact.txt"), "accepted bytes\n").unwrap();
    fs::write(fixture.0.join("first.txt"), "stale producer bytes\n").unwrap();
    assert!(
        transition("second", "2", "worker", "IN_PROGRESS")
            .unwrap_err()
            .contains("accepted source has changed")
    );
}

#[test]
fn child_process_revise_graph() {
    let Some(root) = std::env::var_os("CNR_FIXTURE_REVISE_ROOT").map(PathBuf::from) else {
        return;
    };
    let id = std::env::var("CNR_FIXTURE_REVISE_ID").unwrap();
    std::io::stdin().read_exact(&mut [0_u8]).unwrap();
    if revise_graph(&root, &id).is_ok() {
        println!("CNR_REVISE_WON");
    } else {
        println!("CNR_REVISE_LOST");
    }
}

fn revise_graph(root: &Path, id: &str) -> Result<Value, String> {
    tasks::run(
        "task-revise",
        root,
        &options([
            ("task", id),
            ("revision", "2"),
            ("actor", "operator"),
            ("descriptor", &format!("{id}.json")),
        ]),
        &registry(),
    )
}

#[test]
fn opposite_dependency_revisions_cannot_commit_a_cycle() {
    let fixture = Fixture::new();
    fixture.ready("first");
    fixture.ready("second");
    for (id, dependency) in [("first", "second"), ("second", "first")] {
        let mut proposed = descriptor(id);
        proposed["depends_on"] = json!([dependency]);
        fs::write(
            fixture.0.join(format!("{id}.json")),
            serde_json::to_vec(&proposed).unwrap(),
        )
        .unwrap();
    }
    let test = std::env::current_exe().unwrap();
    let ids = ["first", "second"];
    let mut children = ids.map(|id| {
        Command::new(&test)
            .args([
                "--exact",
                "coordination::tests::child_process_revise_graph",
                "--nocapture",
            ])
            .env("CNR_FIXTURE_REVISE_ROOT", &fixture.0)
            .env("CNR_FIXTURE_REVISE_ID", id)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
    });
    for child in &mut children {
        child.stdin.take().unwrap().write_all(&[1]).unwrap();
    }
    let outputs = children.map(|child| child.wait_with_output().unwrap());
    assert!(outputs.iter().all(|output| output.status.success()));
    let won =
        |index: usize| String::from_utf8_lossy(&outputs[index].stdout).contains("CNR_REVISE_WON");
    assert_ne!(won(0), won(1), "{outputs:?}");
    let loser = if won(0) { ids[1] } else { ids[0] };
    assert!(
        revise_graph(&fixture.0, loser)
            .unwrap_err()
            .contains("form a cycle")
    );
}
