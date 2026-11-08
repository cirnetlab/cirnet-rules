use super::*;
use std::sync::{Arc, Barrier};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "cgacp-task-test-{}-{}",
            std::process::id(),
            NEXT_STAGE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let root = root.canonicalize().unwrap();
        fs::write(root.join("governance.yaml"), "test policy\n").unwrap();
        Self(root)
    }

    fn descriptor(&self, repo: &str, id: &str) -> Value {
        let mut record = json!({
            "id":id, "request":"Update toolbar spacing", "objective":"Fit narrow viewports",
            "non_goals":["Change API"], "primary_repo":repo, "hints":[],
            "read_scope":{}, "write_scope":{}, "verification_scope":{}, "documentation_scope":{},
            "risk":"low", "role":"implementer", "mode":"WORK", "owner":"worker",
            "authorization":{"basis":"owner request","owner":"authority","actions":["read"]},
            "acceptance":["Toolbar remains accessible"], "commands":[], "docs_targets":[],
            "docs_not_required":"Visual-only adjustment with no docs impact"
        });
        for (key, folder) in [("spec", "specs"), ("plan", "plans")] {
            let path = format!("en/{folder}/{repo}/{id}/{key}.md");
            let file = self.0.join(&path);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, format!("# {key}\n\nReviewed fixture input.\n")).unwrap();
            record[key] = json!(path);
        }
        record
    }

    fn create(&self, descriptor: Value) -> Value {
        create(&self.0, descriptor, &registry()).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // Every fixture root was created with create_dir above and uniquely owned.
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn registry() -> Value {
    json!({"repositories":{"ui":{},"api":{}}, "control_plane":{"commands":{}}})
}

#[test]
fn descriptor_rejects_traversal_and_noncentral_plans() {
    let f = Fixture::new();
    let mut descriptor = f.descriptor("ui", "safe");
    descriptor["write_scope"]["ui"] = json!(["../sensitive.txt"]);
    assert!(create(&f.0, descriptor, &registry()).is_err());
    let mut descriptor = f.descriptor("ui", "safe");
    descriptor["plan"] = json!("en/plans/ui/other/plan.md");
    assert!(create(&f.0, descriptor, &registry()).is_err());
    assert!(parse_json(r#"{"key":1,"key":2}"#).is_err());
    assert!(relative("docs/CON.md").is_err());
    assert!(relative("docs/a:b.md").is_err());
}

#[test]
fn lifecycle_prevents_generic_acceptance_and_stale_revision() {
    let f = Fixture::new();
    f.create(f.descriptor("ui", "toolbar"));
    let mut options = BTreeMap::from([
        ("task".to_owned(), "toolbar".to_owned()),
        ("revision".to_owned(), "1".to_owned()),
        ("actor".to_owned(), "worker".to_owned()),
        ("to".to_owned(), "READY".to_owned()),
    ]);
    assert!(run("task-transition", &f.0, &options, &registry()).is_err());
    options.insert("actor".into(), "authority".into());
    let ready = run("task-transition", &f.0, &options, &registry()).unwrap();
    assert_eq!(ready["task"]["state"], "READY");
    options.insert("revision".into(), "2".into());
    options.insert("to".into(), "ACCEPTED".into());
    assert!(run("task-transition", &f.0, &options, &registry()).is_err());
    assert!(update(&f.0, "toolbar", 1, |_| Ok(())).is_err());
    assert_eq!(load(&f.0, "toolbar").unwrap()["revision"], 2);
}

#[test]
fn readiness_matches_command_mode_to_its_distinct_action() {
    let f = Fixture::new();
    let mut descriptor = f.descriptor("ui", "verify");
    descriptor["commands"] = json!(["read-check"]);
    descriptor["authorization"]["actions"] = json!(["read", "verify"]);
    let mut registry = registry();
    registry["control_plane"]["commands"]["read-check"] = json!({"mode":"VERIFY"});
    let record = create(&f.0, descriptor, &registry).unwrap();
    assert!(ready(&f.0, &record, &registry).is_ok());
    registry["control_plane"]["commands"]["read-check"]["mode"] = json!("WORK");
    assert!(ready(&f.0, &record, &registry).unwrap_err().contains("run"));
}

#[test]
fn compare_and_swap_allows_only_one_writer_and_releases_the_lock() {
    let f = Fixture::new();
    f.create(f.descriptor("ui", "toolbar"));
    let barrier = Arc::new(Barrier::new(2));
    let attempts = std::thread::scope(|scope| {
        let handles = (0..2)
            .map(|_| {
                let barrier = barrier.clone();
                let root = &f.0;
                scope.spawn(move || {
                    barrier.wait();
                    update(root, "toolbar", 1, |task| {
                        task["checkpoint"] = json!({"next_action":"continue"});
                        Ok(())
                    })
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(attempts.iter().filter(|r| r.is_ok()).count(), 1);
    assert!(update(&f.0, "toolbar", 2, |_| Err("fixture abort".into())).is_err());
    assert_eq!(load(&f.0, "toolbar").unwrap()["revision"], 2);
    assert!(update(&f.0, "toolbar", 2, |_| Ok(())).is_ok());
}

#[test]
fn completed_task_lock_is_released_when_a_duplicate_handle_remains_open() {
    let f = Fixture::new();
    let held = lock(&f.0, "toolbar").unwrap();
    let duplicate = held.0.try_clone().unwrap();
    assert!(lock(&f.0, "toolbar").is_err());

    drop(held);
    assert!(lock(&f.0, "toolbar").is_ok());
    drop(duplicate);
}

#[test]
fn changing_plan_makes_existing_grant_stale() {
    let f = Fixture::new();
    let descriptor = f.descriptor("ui", "toolbar");
    let path = f.0.join(descriptor["plan"].as_str().unwrap());
    f.create(descriptor);
    fs::write(path, "# Modified plan\n").unwrap();
    assert!(load(&f.0, "toolbar").is_ok());
    assert!(load_current(&f.0, "toolbar").unwrap_err().contains("stale"));
    assert!(update(&f.0, "toolbar", 1, |_| Ok(())).is_err());
    let options = BTreeMap::from([
        ("task".to_owned(), "toolbar".to_owned()),
        ("revision".to_owned(), "1".to_owned()),
        ("actor".to_owned(), "authority".to_owned()),
        ("to".to_owned(), "CANCELLED".to_owned()),
        ("reason".to_owned(), "Plan changed".to_owned()),
    ]);
    let cancelled = run("task-transition", &f.0, &options, &registry()).unwrap();
    assert_eq!(cancelled["task"]["state"], "CANCELLED");
}

#[test]
fn authority_reapproval_requires_explicit_revision_and_resets_evidence() {
    let f = Fixture::new();
    let descriptor = f.descriptor("ui", "toolbar");
    f.create(descriptor.clone());
    let record = update(&f.0, "toolbar", 1, |record| {
        record["evidence"] = json!([{"result":"PASS"}]);
        Ok(())
    })
    .unwrap();
    assert_eq!(record["revision"], 2);
    fs::write(f.0.join("governance.yaml"), "revised policy\n").unwrap();
    fs::write(
        f.0.join("proposal.json"),
        serde_json::to_vec(&descriptor).unwrap(),
    )
    .unwrap();
    let mut options = BTreeMap::from([
        ("task".to_owned(), "toolbar".to_owned()),
        ("revision".to_owned(), "2".to_owned()),
        ("actor".to_owned(), "worker".to_owned()),
        ("descriptor".to_owned(), "proposal.json".to_owned()),
    ]);
    assert!(run("task-revise", &f.0, &options, &registry()).is_err());
    options.insert("actor".into(), "authority".into());
    let revised = run("task-revise", &f.0, &options, &registry()).unwrap();
    assert_eq!(revised["task"]["revision"], 3);
    assert_eq!(revised["task"]["approval_epoch"], 2);
    assert_eq!(revised["task"]["evidence"], json!([]));
    assert_eq!(
        revised["task"]["history"][0]["evidence"],
        json!([{"result":"PASS"}])
    );
    assert!(load_current(&f.0, "toolbar").is_ok());
}

#[test]
fn overlapping_claims_are_refused_and_expired_permissions_fail_closed() {
    let f = Fixture::new();
    let mut first = f.descriptor("ui", "toolbar");
    first["write_scope"]["ui"] = json!(["src/toolbar.ts"]);
    first["authorization"]["actions"] = json!(["read", "write"]);
    f.create(first);
    update(&f.0, "toolbar", 1, |record| {
        record["state"] = json!("IN_PROGRESS");
        Ok(())
    })
    .unwrap();
    let mut overlapping_task = f.descriptor("ui", "theme");
    overlapping_task["write_scope"]["ui"] = json!(["src/toolbar.ts"]);
    overlapping_task["authorization"]["actions"] = json!(["read", "write"]);
    let candidate = f.create(overlapping_task);
    assert!(
        overlapping(&f.0, &candidate)
            .unwrap_err()
            .contains("overlaps")
    );
    let mut expired = f.descriptor("ui", "expired");
    expired["authorization"]["expires_at"] = json!(1);
    let expired = f.create(expired);
    assert!(authorize(&expired, "read").unwrap_err().contains("expired"));
    assert!(load_current(&f.0, "expired").is_err());
}

#[test]
fn untrusted_descriptor_text_never_grants_or_stores_credentials() {
    let f = Fixture::new();
    let mut descriptor = f.descriptor("cirnet-rules", "policy");
    descriptor["write_scope"]["cirnet-rules"] = json!(["governance.yaml"]);
    descriptor["authorization"]["actions"] = json!(["write"]);
    assert!(
        create(&f.0, descriptor, &registry())
            .unwrap_err()
            .contains("authority")
    );
    let mut descriptor = f.descriptor("ui", "credential");
    descriptor["request"] = json!(format!("copy token ghp_{}", "A".repeat(20)));
    assert!(
        create(&f.0, descriptor, &registry())
            .unwrap_err()
            .contains("credential")
    );
    assert!(load(&f.0, "credential").is_err());
}

#[test]
fn initiative_cannot_close_while_a_required_child_is_pending() {
    let f = Fixture::new();
    let mut parent = f.descriptor("_workspace", "initiative");
    parent["required_children"] = json!(["toolbar"]);
    let parent = f.create(parent);
    let mut child = f.descriptor("ui", "toolbar");
    child["parent"] = json!("initiative");
    f.create(child);
    assert!(ensure_children_closed(&f.0, &parent).is_err());
    update(&f.0, "toolbar", 1, |task| {
        task["state"] = json!("CLOSED");
        Ok(())
    })
    .unwrap();
    assert!(ensure_children_closed(&f.0, &parent).is_ok());
}
