use super::*;
use std::process::Command;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        Self(crate::test_support::temporary_directory("cgacp-catalog").unwrap())
    }

    fn git(path: &Path, args: &[&str]) {
        let executable = git_executable(path).unwrap();
        let result = Command::new(executable)
            .current_dir(path)
            .args(args)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }

    fn checkout(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(&path).unwrap();
        Self::git(&path, &["init", "-q"]);
        Self::git(
            &path,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "--allow-empty",
                "-qm",
                "Fixture",
            ],
        );
        path
    }

    fn add(db: &mut Database, path: &Path) -> Value {
        let preview = dispatch(db, "repo.preview", json!({"path":path})).unwrap();
        assert_eq!(preview["status"], "VERIFIED");
        dispatch(
            db,
            "repo.add",
            json!({"path":path,"token":preview["token"]}),
        )
        .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Ok(base) = std::env::temp_dir().canonicalize()
            && self.0.parent() == Some(base.as_path())
            && self
                .0
                .file_name()
                .and_then(|x| x.to_str())
                .is_some_and(|x| x.starts_with("cgacp-catalog-"))
        {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

#[test]
fn file_folder_reopen_and_duplicate_preserve_binding() {
    let fixture = Fixture::new();
    let checkout = fixture.checkout("Mạch thử nghiệm");
    let sub = checkout.join("source files");
    fs::create_dir(&sub).unwrap();
    let file = sub.join("tín hiệu.txt");
    fs::write(&file, "test").unwrap();
    let data = fixture.0.join("app data");
    let mut db = Database::open(&data).unwrap();
    assert_eq!(
        dispatch(&mut db, "repo.preview", json!({"path":&data}))
            .unwrap_err()
            .code,
        "NOT_A_REPOSITORY"
    );
    let preview = dispatch(&mut db, "repo.preview", json!({"path":file})).unwrap();
    assert_eq!(preview["file_hint"], "source files/tín hiệu.txt");
    assert_eq!(
        dispatch(&mut db, "repo.list", json!({})).unwrap()["items"],
        json!([])
    );
    let saved = dispatch(
        &mut db,
        "repo.add",
        json!({"path":file,"token":preview["token"],"display_name":"Mạch"}),
    )
    .unwrap();
    assert_eq!(saved["display_name"], "Mạch");
    assert_eq!(
        Fixture::add(&mut db, &sub)["binding_id"],
        saved["binding_id"]
    );
    drop(db);
    let mut db = Database::open(&data).unwrap();
    let listed = dispatch(&mut db, "repo.list", json!({})).unwrap();
    assert_eq!(listed["items"][0]["repo_id"], saved["repo_id"]);
    let id = saved["binding_id"].as_str().unwrap();
    let selected = dispatch(&mut db, "repo.select", json!({"binding_id":id})).unwrap();
    assert!(selected["selected_at"].is_i64());
    assert_eq!(selected["permissions_granted"], false);
}

#[test]
fn linked_worktree_and_equal_basenames_stay_distinct() {
    let fixture = Fixture::new();
    let first = fixture.checkout("A/same");
    let other = fixture.checkout("B/same");
    let linked = fixture.0.join("alternate workspace");
    Fixture::git(
        &first,
        &[
            "worktree",
            "add",
            "--detach",
            "../../alternate workspace",
            "HEAD",
        ],
    );
    let mut db = Database::open(&fixture.0.join("db")).unwrap();
    let normal = Fixture::add(&mut db, &first);
    let twin = Fixture::add(&mut db, &other);
    let second = Fixture::add(&mut db, &linked);
    assert_ne!(normal["repo_id"], twin["repo_id"]);
    assert_eq!(normal["repo_id"], second["repo_id"]);
    assert_ne!(normal["binding_id"], second["binding_id"]);
    assert_eq!(second["linked_worktree"], true);
}

#[test]
fn concurrent_add_and_online_backup_preserve_a_single_registration() {
    let fixture = Fixture::new();
    let root = fixture.checkout("concurrent");
    let data = fixture.0.join("db");
    let mut first = Database::open(&data).unwrap();
    let mut second = Database::open(&data).unwrap();
    let both = std::sync::Barrier::new(2);
    let (one, two) = std::thread::scope(|scope| {
        let left = scope.spawn(|| {
            both.wait();
            Fixture::add(&mut first, &root)
        });
        let right = scope.spawn(|| {
            both.wait();
            Fixture::add(&mut second, &root)
        });
        (left.join().unwrap(), right.join().unwrap())
    });
    assert_eq!(one["binding_id"], two["binding_id"]);
    assert_eq!(
        storage::dispatch(&mut first, "storage.verify", json!({})).unwrap()["repositories"],
        1
    );
    let snapshot = storage::dispatch(&mut first, "storage.backup", json!({})).unwrap();
    let saved = storage::dispatch(
        &mut first,
        "storage.verify",
        json!({"backup":snapshot["backup"]}),
    )
    .unwrap();
    assert_eq!(saved["bindings"], 1);

    let restoration = fixture.0.join("restore");
    drop(Database::open(&restoration).unwrap());
    fs::copy(
        snapshot["path"].as_str().unwrap(),
        restoration.join("cgacp.sqlite3"),
    )
    .unwrap();
    let mut restored = Database::open(&restoration).unwrap();
    let list = dispatch(&mut restored, "repo.list", json!({})).unwrap();
    assert_eq!(list["items"][0]["binding_id"], one["binding_id"]);
}

#[test]
fn preview_cannot_authorize_replacement_or_be_reused_in_another_runtime() {
    let fixture = Fixture::new();
    let root = fixture.checkout("expected");
    let alternate = fixture.checkout("alternate");
    let data = fixture.0.join("db");
    let mut db = Database::open(&data).unwrap();
    let preview = dispatch(&mut db, "repo.preview", json!({"path":root})).unwrap();
    assert_eq!(
        dispatch(
            &mut db,
            "repo.add",
            json!({"path":alternate,"token":preview["token"]})
        )
        .unwrap_err()
        .code,
        "STALE_PREVIEW"
    );
    drop(db);
    let mut db = Database::open(&data).unwrap();
    assert_eq!(
        dispatch(
            &mut db,
            "repo.add",
            json!({"path":root,"token":preview["token"]})
        )
        .unwrap_err()
        .code,
        "STALE_PREVIEW"
    );
    assert_eq!(
        dispatch(&mut db, "repo.list", json!({})).unwrap()["items"],
        json!([])
    );
}

#[test]
fn sqlite_busy_and_future_schema_fail_without_mutating_existing_state() {
    let fixture = Fixture::new();
    let data = fixture.0.join("db");
    let mut first = Database::open(&data).unwrap();
    let second = Database::open(&data).unwrap();
    second
        .conn
        .busy_timeout(std::time::Duration::from_millis(10))
        .unwrap();
    let tx = first
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    tx.execute(
        "UPDATE catalog_settings SET value='uncommitted' WHERE key='machine_id'",
        [],
    )
    .unwrap();
    let error = second
        .conn
        .execute(
            "UPDATE catalog_settings SET value='bad' WHERE key='machine_id'",
            [],
        )
        .unwrap_err();
    assert_eq!(AppError::from(error).code, "STORAGE_BUSY");
    tx.rollback().unwrap();
    let persisted: String = second
        .conn
        .query_row(
            "SELECT value FROM catalog_settings WHERE key='machine_id'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(persisted, first.machine_id);
    second
        .conn
        .pragma_update(None, "user_version", 9001)
        .unwrap();
    drop(second);
    drop(first);
    assert_eq!(
        Database::open(&data).err().unwrap().code,
        "MIGRATION_REQUIRED"
    );
    let original = Connection::open(data.join("cgacp.sqlite3")).unwrap();
    assert_eq!(
        original
            .pragma_query_value::<i64, _>(None, "user_version", |row| row.get(0))
            .unwrap(),
        9001
    );
}

#[test]
fn legacy_archive_preserves_revision_quarantines_lease_and_links_only_reviewed_workspace() {
    use crate::dashboard::legacy;

    let fixture = Fixture::new();
    let checkout = fixture.checkout("reviewed checkout");
    let source = fixture.0.join(".cgacp");
    let task = source.join("tasks/test.json");
    let lease = source.join("runs/test/active.json");
    fs::create_dir_all(task.parent().unwrap()).unwrap();
    fs::create_dir_all(lease.parent().unwrap()).unwrap();
    let task_record = json!({
        "descriptor":{"id":"test","authorization":{"actor":"historical-label"}},
        "revision":8,"state":"IN_PROGRESS","checkpoint":{"test":"recorded"},
        "evidence":[{"digest":"evidence fixture"}]
    });
    fs::write(&task, task_record.to_string()).unwrap();
    fs::write(
        &lease,
        json!({"token":"123-456","actor":"historical-label","expires_at_ms":50}).to_string(),
    )
    .unwrap();
    let namespace = digest(fixture.0.to_str().unwrap().as_bytes());
    let relative = format!("bindings/{namespace}/former-name.json");
    let old = source.join(&relative);
    fs::create_dir_all(old.parent().unwrap()).unwrap();
    let mut db = Database::open(&fixture.0.join("db")).unwrap();
    let registered = Fixture::add(&mut db, &checkout);
    fs::write(
        &old,
        json!({
            "schema":1,"relative":"reviewed checkout","anchor":registered["commit"],
            "identity":"0".repeat(64),"actor":"historical-label"
        })
        .to_string(),
    )
    .unwrap();
    let dry = legacy::dispatch(
        &mut db,
        "legacy.preview",
        json!({"source":source,"workspace":fixture.0}),
    )
    .unwrap();
    assert_eq!(dry["record_count"], 3);
    let request = json!({
        "source":source,"workspace":fixture.0,"manifest_sha256":dry["manifest_sha256"],"writers_stopped":true
    });
    let archived = legacy::dispatch(&mut db, "legacy.apply", request.clone()).unwrap();
    assert_eq!(archived["status"], "ARCHIVED");
    assert_eq!(archived["active_storage_cutover"], false);
    let again = legacy::dispatch(&mut db, "legacy.apply", request.clone()).unwrap();
    assert_eq!(again["status"], "UNCHANGED");
    let saved = legacy::dispatch(
        &mut db,
        "legacy.inspect",
        json!({"source_id":archived["source_id"],"path":"tasks/test.json"}),
    )
    .unwrap();
    assert_eq!(saved["revision"], 8);
    assert_eq!(saved["body"], task_record);
    assert_eq!(saved["authenticated_approval"], false);
    let lease = legacy::dispatch(
        &mut db,
        "legacy.inspect",
        json!({"source_id":archived["source_id"],"path":"runs/test/active.json"}),
    )
    .unwrap();
    assert_eq!(lease["state"], "RECOVERY_REQUIRED");
    assert!(lease["body"]["token"].is_null());
    let linked = legacy::dispatch(
        &mut db,
        "legacy.link",
        json!({
            "source_id":archived["source_id"],"path":relative,"binding_id":registered["binding_id"],
            "binding_revision":registered["revision"]
        }),
    )
    .unwrap();
    assert_eq!(linked["repo_id"], registered["repo_id"]);
    assert_eq!(linked["permissions_granted"], false);
    fs::write(
        &task,
        json!({"descriptor":{"id":"test"},"revision":9,"state":"CLOSED"}).to_string(),
    )
    .unwrap();
    assert_eq!(
        legacy::dispatch(&mut db, "legacy.apply", request)
            .unwrap_err()
            .code,
        "STALE_PREVIEW"
    );
    assert!(source.join("tasks/test.json").exists());
}

#[test]
fn replacement_is_detected_and_relink_and_archive_revoke_old_binding() {
    let fixture = Fixture::new();
    let root = fixture.checkout("existing checkout");
    let mut db = Database::open(&fixture.0.join("db")).unwrap();
    let saved = Fixture::add(&mut db, &root);
    let id = saved["binding_id"].as_str().unwrap();
    let relocated = fixture.0.join("new path");
    fs::rename(&root, &relocated).unwrap();
    assert_eq!(
        dispatch(&mut db, "repo.health", json!({"binding_id":id})).unwrap()["status"],
        "MISSING"
    );
    let wrong = fixture.checkout("existing checkout");
    assert_eq!(
        dispatch(&mut db, "repo.health", json!({"binding_id":id})).unwrap()["status"],
        "REBIND_REQUIRED"
    );
    let bad = dispatch(&mut db, "repo.preview", json!({"path":wrong})).unwrap();
    assert_eq!(
        dispatch(
            &mut db,
            "repo.relink",
            json!({"binding_id":id,"revision":1,"path":wrong,"token":bad["token"]})
        )
        .unwrap_err()
        .code,
        "REBIND_REQUIRED"
    );
    let preview = dispatch(&mut db, "repo.preview", json!({"path":relocated})).unwrap();
    let fixed = dispatch(
        &mut db,
        "repo.relink",
        json!({"binding_id":id,"revision":1,"path":relocated,"token":preview["token"]}),
    )
    .unwrap();
    assert_eq!(fixed["repo_id"], saved["repo_id"]);
    assert_eq!(fixed["revision"], 2);
    assert_eq!(
        resolve_binding(&db, id, Some(1)).unwrap_err().code,
        "CONFLICT"
    );
    let renamed = dispatch(
        &mut db,
        "repo.rename",
        json!({"repo_id":saved["repo_id"],"revision":1,"display_name":"Renamed"}),
    )
    .unwrap();
    assert_eq!(renamed["revision"], 2);
    dispatch(
        &mut db,
        "repo.archive",
        json!({"repo_id":saved["repo_id"],"revision":2}),
    )
    .unwrap();
    assert_eq!(
        resolve_binding(&db, id, Some(2)).unwrap_err().code,
        "ARCHIVED"
    );
    assert!(relocated.join(".git").is_dir());
    assert_eq!(
        dispatch(&mut db, "repo.list", json!({"include_archived":true})).unwrap()["items"][0]["status"],
        "ARCHIVED"
    );
}
