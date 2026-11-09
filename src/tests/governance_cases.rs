use super::*;

fn method(name: &str) -> String {
    format!("governance.{name}")
}

fn create(runtime: &mut Runtime, key: &str, kind: &str, slug: &str) -> Value {
    runtime.call(
        &method("asset.create"),
        json!({
            "kind":kind,"slug":slug,"title":slug,"mutation_key":key
        }),
    )
}

fn save(
    runtime: &mut Runtime,
    asset: &Value,
    key: &str,
    en: &str,
    vi: &str,
    files: Value,
    dependencies: Value,
) -> Value {
    runtime.call(
        &method("draft.save"),
        json!({
            "id":asset["id"],"expected_revision":asset["revision"],"mutation_key":key,
            "en":en,"vi":vi,"format":"markdown","files":files,"dependencies":dependencies
        }),
    )
}

fn publish(runtime: &mut Runtime, asset: &Value, key: &str) -> Value {
    let reviewed = runtime.call(&method("change.review"), json!({
        "id":asset["id"],"expected_revision":asset["revision"],"mutation_key":format!("{key}-review")
    }));
    let params = json!({
        "id":asset["id"],"expected_revision":asset["revision"],
        "mutation_key":key,"review_token":reviewed["review_token"]
    });
    let result = runtime.call(&method("change.publish"), params.clone());
    assert_eq!(result, runtime.call(&method("change.publish"), params));
    result
}

#[test]
fn governance_draft_restart_publish_and_rollback_preserve_assets_and_history() {
    let fixture = Fixture::new();
    let mut runtime = Runtime::open(&fixture.0);
    let asset = create(&mut runtime, "create", "rule", "sample");
    let files = json!([{"path":"references/guide.md","content":"# Guide"}]);
    let draft = save(
        &mut runtime,
        &asset,
        "first",
        "# Scope",
        "# Phạm vi",
        files.clone(),
        json!([]),
    );
    assert_eq!(draft["active_revision"], Value::Null);
    runtime.close();

    let mut runtime = Runtime::open(&fixture.0);
    let recovered = runtime.call(&method("asset.get"), json!({"id":asset["id"]}));
    assert_eq!(recovered["document"]["files"], files);
    assert_eq!(recovered["revision"], draft["revision"]);
    assert_eq!(recovered["active_revision"], Value::Null);
    let active = publish(&mut runtime, &recovered, "first-publish");

    let stale = save(
        &mut runtime,
        &active,
        "stale",
        "# New scope",
        "# Phạm vi",
        json!([]),
        json!([]),
    );
    let validated = runtime.call(
        &method("revision.validate"),
        json!({
            "id":asset["id"],"expected_revision":stale["revision"]
        }),
    );
    assert_eq!(validated["valid"], false);
    assert_eq!(validated["translation"], "STALE");
    assert_eq!(stale["active_revision"], active["active_revision"]);

    let draft = save(
        &mut runtime,
        &stale,
        "updated",
        "# New scope",
        "# Phạm vi mới",
        json!([]),
        json!([]),
    );
    let updated = publish(&mut runtime, &draft, "second-publish");
    let rollback = runtime.call(&method("change.rollback"), json!({
        "id":asset["id"],"expected_revision":updated["revision"],"target_revision":active["active_revision"],
        "mutation_key":"rollback"
    }));
    assert_eq!(rollback["id"], asset["id"]);
    assert_eq!(rollback["document"]["files"], files);
    assert!(rollback["active_revision"].as_i64() > updated["active_revision"].as_i64());
    let history = runtime.call(
        &method("revision.history"),
        json!({"id":asset["id"],"limit":2}),
    );
    let old = runtime.call(
        &method("revision.get"),
        json!({"id":asset["id"],"revision":active["active_revision"]}),
    );
    assert_eq!(old["document"]["en"], "# Scope");
    let events = runtime.call(
        &method("change.history"),
        json!({"id":asset["id"],"limit":100}),
    );
    let events = events["items"].as_array().unwrap();
    assert!(
        events
            .iter()
            .any(|event| event["event"] == "REVIEWED_LOCAL_OPERATOR")
    );
    assert_eq!(history["items"][0]["event"], "ROLLBACK");
    assert_eq!(history["items"].as_array().unwrap().len(), 2);
    assert!(history["next_before"].is_number());
}

#[test]
fn governance_rejects_traversal_stale_writes_dependency_removal_and_unreviewed_publish() {
    let fixture = Fixture::new();
    let mut runtime = Runtime::open(&fixture.0);
    let parent = create(&mut runtime, "parent", "rule", "parent");
    let invalid = json!({
        "id":parent["id"],"expected_revision":parent["revision"],"mutation_key":"unsafe",
        "en":"# Parent","vi":"# Tài liệu cha","format":"markdown",
        "files":[{"path":"references/../.git/config","content":"ignored"}]
    });
    assert_eq!(
        runtime.request(2, &method("draft.save"), invalid)["error"]["code"],
        "INVALID_PATH"
    );
    let draft = save(
        &mut runtime,
        &parent,
        "parent-save",
        "# Parent",
        "# Tài liệu cha",
        json!([]),
        json!([]),
    );
    let stale = json!({
        "id":parent["id"],"expected_revision":parent["revision"],"mutation_key":"stale",
        "en":"# Wrong","vi":"# Sai","format":"markdown"
    });
    assert_eq!(
        runtime.request(2, &method("draft.save"), stale)["error"]["code"],
        "CONFLICT"
    );
    let unauthorized = json!({
        "id":parent["id"],"expected_revision":draft["revision"],"review_token":"unknown","mutation_key":"unreviewed"
    });
    assert_eq!(
        runtime.request(2, &method("change.publish"), unauthorized)["error"]["code"],
        "PERMISSION_DENIED"
    );
    let active = publish(&mut runtime, &draft, "parent-publish");
    let consumer = create(&mut runtime, "consumer", "rule", "consumer");
    let consumer = save(
        &mut runtime,
        &consumer,
        "consumer-save",
        "# Consumer",
        "# Tài liệu con",
        json!([]),
        json!([parent["id"]]),
    );
    let archive = runtime.request(
        2,
        &method("asset.archive"),
        json!({
            "id":parent["id"],"expected_revision":active["revision"],"mutation_key":"archive"
        }),
    );
    assert_eq!(archive["error"]["code"], "DEPENDENCY_IN_USE");
    assert_eq!(
        archive["error"]["details"]["used_by"][0]["id"],
        consumer["id"]
    );
}

#[test]
fn governance_validates_structured_content_and_blocks_sensitive_drafts_and_elevated_publish() {
    let fixture = Fixture::new();
    let mut runtime = Runtime::open(&fixture.0);
    let profile = create(&mut runtime, "profile", "profile", "profile");
    let save = |en: &str, key| {
        json!({
            "id":profile["id"],"expected_revision":profile["revision"],"mutation_key":key,
            "en":en,"vi":"# Hồ sơ thử nghiệm","format":"json"
        })
    };
    assert_eq!(
        runtime.request(
            2,
            &method("draft.save"),
            save(r#"{"api_key":"fixture-secret"}"#, "secret")
        )["error"]["code"],
        "SENSITIVE_INPUT"
    );
    let invalid = runtime.call(&method("draft.save"), save("{\n  broken json", "invalid"));
    let validated = runtime.call(
        &method("revision.validate"),
        json!({
            "id":profile["id"],"expected_revision":invalid["revision"]
        }),
    );
    assert_eq!(validated["diagnostics"][0]["code"], "INVALID_JSON");
    assert_eq!(validated["diagnostics"][0]["line"], 2);
    let elevated = runtime.call(&method("draft.save"), json!({
        "id":profile["id"],"expected_revision":invalid["revision"],"mutation_key":"hard-deny",
        "en":r#"{"allowed_actions":["hardware.control"]}"#,"vi":"# Hồ sơ thử nghiệm","format":"json"
    }));
    let validated = runtime.call(
        &method("revision.validate"),
        json!({
            "id":profile["id"],"expected_revision":elevated["revision"]
        }),
    );
    assert_eq!(validated["diagnostics"][0]["code"], "HARD_DENY");

    let policy = create(&mut runtime, "policy", "policy", "policy");
    let policy = runtime.call(&method("draft.save"), json!({
        "id":policy["id"],"expected_revision":policy["revision"],"mutation_key":"policy-draft",
        "en":r#"{"deny":["hardware.control"]}"#,"vi":"# Cấm điều khiển phần cứng","format":"json"
    }));
    assert_eq!(runtime.request(2, &method("change.review"), json!({
        "id":policy["id"],"expected_revision":policy["revision"],"mutation_key":"review-policy"
    }))["error"]["code"], "CAPABILITY_UNAVAILABLE");
}

#[test]
fn governance_session_pins_active_profiles_and_binding_assignments() {
    let fixture = Fixture::new();
    let repo = fixture.0.join("repo");
    fs::create_dir(&repo).unwrap();
    git(&repo, &["init"]);
    git(
        &repo,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--allow-empty",
            "-m",
            "Fixture",
        ],
    );
    let mut runtime = Runtime::open(&fixture.0.join("state"));
    let preview = runtime.call("repo.preview", json!({"path":repo}));
    let binding = runtime.call("repo.add", json!({"path":repo,"token":preview["token"]}));
    let rule = create(&mut runtime, "rule", "rule", "pinned-rule");
    let rule = save(
        &mut runtime,
        &rule,
        "rule-draft",
        "# Initial rule",
        "# Quy tắc ban đầu",
        json!([]),
        json!([]),
    );
    let rule = publish(&mut runtime, &rule, "publish-rule");
    let rule = runtime.call(
        &method("assignment.update"),
        json!({
            "id":rule["id"],"expected_revision":rule["revision"],"binding_id":binding["binding_id"],
            "enabled":true,"mutation_key":"assign-rule"
        }),
    );
    let profile = create(&mut runtime, "profile", "profile", "local-profile");
    let profile = runtime.call(&method("draft.save"), json!({
        "id":profile["id"],"expected_revision":profile["revision"],"mutation_key":"profile-draft",
        "en":r#"{"role":"review","use_remote":false,"budget":0}"#,"vi":"# Hồ sơ rà soát cục bộ","format":"json"
    }));
    let profile = publish(&mut runtime, &profile, "publish-profile");
    runtime.call(&method("profile.default"), json!({
        "id":profile["id"],"expected_revision":profile["revision"],"enabled":true,"mutation_key":"set-default"
    }));
    let conversation = runtime.call(
        "conversation.create",
        json!({
            "key":"chat-one","title":"First","bindings":[binding["binding_id"]]
        }),
    );
    let effective = runtime.call(
        &method("effective.preview"),
        json!({"binding_id":binding["binding_id"]}),
    );
    assert_eq!(effective["profile_id"], profile["id"]);
    assert_eq!(effective["included"], conversation["governance"]["assets"]);
    let assets = &conversation["governance"]["assets"];
    assert!(
        assets
            .as_array()
            .unwrap()
            .iter()
            .any(|asset| asset["id"] == rule["id"] && asset["revision"] == rule["active_revision"])
    );
    assert_eq!(conversation["governance"]["profile_id"], profile["id"]);
    assert_eq!(conversation["profile"], "handoff");

    let draft = save(
        &mut runtime,
        &rule,
        "rule-revised",
        "# New rule",
        "# Quy tắc mới",
        json!([]),
        json!([]),
    );
    let second = runtime.call(
        "conversation.create",
        json!({
            "key":"chat-two","title":"Second","bindings":[binding["binding_id"]]
        }),
    );
    assert_eq!(second["governance"], conversation["governance"]);
    let updated = publish(&mut runtime, &draft, "publish-updated-rule");
    let third = runtime.call(
        "conversation.create",
        json!({
            "key":"chat-three","title":"Third","bindings":[binding["binding_id"]]
        }),
    );
    assert!(
        third["governance"]["assets"]
            .as_array()
            .unwrap()
            .iter()
            .any(|asset| asset["id"] == rule["id"]
                && asset["revision"] == updated["active_revision"])
    );
    let reopened = runtime.call("conversation.get", json!({"id":conversation["id"]}));
    assert_eq!(
        reopened["conversation"]["governance"],
        conversation["governance"]
    );
}

#[test]
fn governance_export_uses_published_bytes_and_import_requires_exact_preview() {
    let fixture = Fixture::new();
    let mut runtime = Runtime::open(&fixture.0);
    let asset = create(&mut runtime, "create", "reference", "source");
    let draft = save(
        &mut runtime,
        &asset,
        "source-draft",
        "# Scope",
        "# Phạm vi",
        json!([
            {"path":"docs/a.md","content":"# Child"}
        ]),
        json!([]),
    );
    let active = publish(&mut runtime, &draft, "source-publish");
    let draft = save(
        &mut runtime,
        &active,
        "source-new",
        "# Later",
        "# Sau này",
        json!([]),
        json!([]),
    );
    let bundle = runtime.call(&method("asset.export"), json!({"id":asset["id"]}));
    assert_eq!(bundle["document"]["en"], "# Scope");
    assert_eq!(bundle["revision"], active["active_revision"]);
    assert_eq!(bundle["document"]["files"][0]["path"], "docs/a.md");

    let preview = runtime.call(&method("asset.import.preview"), json!({"bundle":bundle}));
    assert_eq!(preview["activates"], false);
    let mut corrupted = bundle.clone();
    corrupted["document"]["en"] = json!("# Corrupted");
    assert_eq!(
        runtime.request(
            2,
            &method("asset.import.preview"),
            json!({"bundle":corrupted})
        )["error"]["code"],
        "INTEGRITY"
    );
    let mut renamed = bundle.clone();
    renamed["title"] = json!("Different title");
    assert_eq!(runtime.request(2, &method("asset.import.apply"), json!({
        "bundle":renamed,"fingerprint":preview["fingerprint"],"slug":"import-copy","mutation_key":"tampered"
    }))["error"]["code"], "CONFLICT");
    let copy = runtime.call(&method("asset.import.apply"), json!({
        "bundle":bundle,"fingerprint":preview["fingerprint"],"slug":"import-copy","mutation_key":"new-copy"
    }));
    assert_ne!(copy["id"], asset["id"]);
    assert_eq!(copy["origin"], "local_import");
    assert_eq!(copy["source_revision"], bundle["revision"]);
    assert_eq!(copy["source_digest"], bundle["digest"]);
    assert_eq!(copy["active_revision"], Value::Null);
    assert_eq!(copy["document"], bundle["document"]);
    assert_ne!(copy["document"], draft["document"]);
}

#[test]
fn governance_attachment_aliases_and_executable_attachments_are_denied() {
    let fixture = Fixture::new();
    let mut runtime = Runtime::open(&fixture.0);
    let asset = create(&mut runtime, "skill", "skill", "fixture-skill");
    let request = |files| {
        json!({
            "id":asset["id"],"expected_revision":asset["revision"],"mutation_key":"draft",
            "en":"---\nname: fixture-skill\ndescription: Fixture skill\n---\n# How to review",
            "vi":"# Cách rà soát","format":"markdown","files":files
        })
    };
    for files in [
        json!([{"path":"docs/A.md","content":"a"},{"path":"docs/a.md","content":"b"}]),
        json!([{"path":"docs/CON.txt","content":"a"}]),
        json!([{"path":"docs/valid.md:zone","content":"a"}]),
        json!([{"path":"docs/valid.md.","content":"a"}]),
        json!([{"path":"//server/share/file.md","content":"a"}]),
    ] {
        assert_eq!(
            runtime.request(2, &method("draft.save"), request(files))["error"]["code"],
            "INVALID_PATH"
        );
    }
    let saved = runtime.call(
        &method("draft.save"),
        request(json!([{"path":"scripts/do.ps1","content":"exit 9"}])),
    );
    let checked = runtime.call(
        &method("revision.validate"),
        json!({
            "id":asset["id"],"expected_revision":saved["revision"]
        }),
    );
    assert_eq!(checked["valid"], false);
    assert_eq!(checked["diagnostics"][0]["code"], "ATTACHMENT_UNSUPPORTED");
    assert_eq!(runtime.request(2, &method("change.review"), json!({
        "id":asset["id"],"expected_revision":saved["revision"],"mutation_key":"unsafe-script"
    }))["error"]["code"], "VALIDATION_FAILED");
}

#[test]
fn governance_interrupted_sql_write_never_activates_partial_publication() {
    let fixture = Fixture::new();
    let mut runtime = Runtime::open(&fixture.0);
    let asset = create(&mut runtime, "create", "rule", "atomic");
    let draft = save(
        &mut runtime,
        &asset,
        "save",
        "# Atomic",
        "# Ghi nguyên tử",
        json!([{"path":"docs/child.md","content":"# Child"}]),
        json!([]),
    );
    let review = runtime.call(
        &method("change.review"),
        json!({
            "id":asset["id"],"expected_revision":draft["revision"],"mutation_key":"review"
        }),
    );
    runtime.close();

    // The fixture aborts after inserting the immutable revision but before moving
    // the active pointer. This exercises the real SQL transaction, not a test flag.
    let database = fixture.0.join("openrules.sqlite3");
    let sql = rusqlite::Connection::open(&database).unwrap();
    sql.execute_batch(
        "CREATE TRIGGER fixture_interrupt BEFORE UPDATE OF active_revision ON governance_assets
         BEGIN SELECT RAISE(ABORT,'injected failure'); END;",
    )
    .unwrap();
    drop(sql);

    let mut runtime = Runtime::open(&fixture.0);
    let apply = json!({"id":asset["id"],"expected_revision":draft["revision"],
                       "review_token":review["review_token"],"mutation_key":"apply"});
    assert_eq!(
        runtime.request(2, &method("change.publish"), apply.clone())["ok"],
        false
    );
    let recovered = runtime.call(&method("asset.get"), json!({"id":asset["id"]}));
    assert_eq!(recovered["active_revision"], Value::Null);
    assert_eq!(recovered["document"], draft["document"]);
    let history = runtime.call(&method("revision.history"), json!({"id":asset["id"]}));
    assert!(history["items"].as_array().unwrap().is_empty());
    runtime.close();

    let sql = rusqlite::Connection::open(database).unwrap();
    sql.execute_batch("DROP TRIGGER fixture_interrupt").unwrap();
    drop(sql);
    let mut runtime = Runtime::open(&fixture.0);
    let published = runtime.call(&method("change.publish"), apply);
    assert!(published["active_revision"].is_number());
    assert_eq!(published["document"], draft["document"]);
}

#[test]
fn governance_wire_schema_checks_mutation_identity_and_typed_input() {
    let schema: Value =
        serde_json::from_str(include_str!("../../en/schemas/dashboard-ipc.schema.json")).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let frame = |method, params| json!({"v":2,"id":"test","method":format!("openrules.governance.{method}"),"params":params});
    let good = json!({
        "id":"asset-0123456789abcdef0123456789abcdef","expected_revision":1,
        "mutation_key":"save-1","en":"# Input","vi":"# Dữ liệu","format":"markdown",
        "files":[{"path":"examples/guide.md","content":"Guide"}],"dependencies":[]
    });
    assert!(validator.is_valid(&frame("draft.save", good.clone())));
    let mut bad = good.clone();
    bad["mutation_key"] = json!("bad/key");
    assert!(!validator.is_valid(&frame("draft.save", bad)));
    let mut bad = good;
    bad["execute"] = json!(true);
    assert!(!validator.is_valid(&frame("draft.save", bad)));
    assert!(validator.is_valid(&frame(
        "effective.preview",
        json!({"mode":"WORK","budget_bytes":1024})
    )));
    assert!(!validator.is_valid(&frame("effective.preview", json!({"mode":"DEPLOY"}))));
}
