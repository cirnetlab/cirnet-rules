use super::*;

struct Fixture(PathBuf);
impl Fixture {
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn fixture(name: &str) -> Fixture {
    Fixture(crate::test_support::temporary_directory(name).unwrap())
}

fn legacy(path: &Path) {
    let mut db = Connection::open(path).unwrap();
    db.execute_batch(
        "CREATE TABLE schema_migrations (
             revision INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE,
             checksum TEXT NOT NULL, applied_at INTEGER NOT NULL) STRICT;",
    )
    .unwrap();
    let tx = db.transaction().unwrap();
    for (i, (name, sql)) in super::super::MIGRATIONS[..6].iter().enumerate() {
        tx.execute_batch(sql).unwrap();
        tx.execute(
            "INSERT INTO schema_migrations VALUES (?1, ?2, ?3, 19)",
            rusqlite::params![i + 1, name, format!("{:x}", Sha256::digest(sql.as_bytes()))],
        )
        .unwrap();
    }
    tx.pragma_update(None, "user_version", 6).unwrap();
    tx.execute_batch(
        "INSERT INTO repositories VALUES ('repo-original','test','Original',1,0,4,5);
         INSERT INTO bindings (id,repo_id,machine_id,selected_path,root,git_dir,common_dir,identity,anchor,linked_worktree,status,verified_at)
           SELECT 'binding-original','repo-original',value,'local','root','git','common','identity','anchor',0,'ACTIVE',19
           FROM catalog_settings WHERE key='machine_id';
         INSERT INTO conversations VALUES (41,'create-identity','digest','Legacy chat','handoff','ARCHIVED','DASHBOARD',NULL,2,4,5);
         INSERT INTO conversation_bindings VALUES (41,'binding-original',1,1);
         INSERT INTO messages VALUES (17,41,'message-identity','digest',1,'USER','Nội dung CGACP',6);
         INSERT INTO task_proposals VALUES (6,41,17,'WAITING_APPROVAL','WORK','SCOPED','{}',6);
         INSERT INTO automation_definitions
           (id,create_key,create_digest,name,binding_id,binding_revision,profile,prompt,paths_json,mode,risk,revision,enabled,trigger_json,created_at,updated_at)
           VALUES (9,'automation-key','digest','Old','binding-original',1,'handoff','Prompt','[]','WORK','SCOPED',1,0,'{}',4,5);
         INSERT INTO automation_runs
           (id,automation_id,definition_revision,binding_id,binding_revision,kind,request_key,status,snapshot_json,conversation_id,task_id,created_at,updated_at)
           VALUES (8,9,1,'binding-original',1,'MANUAL','run-key','WAITING_APPROVAL','{}',41,6,4,5);
         INSERT INTO session_events VALUES (18,41,6,8,1,'run-key','fixture','{}',7);
         INSERT INTO desktop_preferences VALUES (1,7,'{}');",
    ).unwrap();
    tx.commit().unwrap();
    db.pragma_update(None, "journal_mode", "WAL").unwrap();
}

#[test]
fn cutover_retains_revisions_ids_preferences_and_idempotency_keys() {
    let root = fixture("cirnetrules-identity");
    let locations = Locations::new(root.path().join("CIRNETRULES"), true);
    ensure_directory(locations.legacy.parent().unwrap()).unwrap();
    legacy(&locations.legacy);
    assert_eq!(
        locations.preview().unwrap()["status"],
        "MIGRATION_AVAILABLE"
    );
    assert!(locations.open().is_err());
    assert!(!locations.active.exists());
    locations.migrate().unwrap();
    let db = locations.open().unwrap();
    assert_eq!(
        db.conn.query_row("SELECT content FROM messages WHERE id=17 AND request_key='message-identity' AND created_at=6", [], |row| row.get::<_, String>(0)).unwrap(),
        "Nội dung CGACP"
    );
    assert_eq!(db.conn.query_row("SELECT count(*) FROM automation_runs r JOIN task_proposals t ON r.task_id=t.id JOIN session_events e ON e.run_id=r.id JOIN bindings b ON b.id=r.binding_id WHERE r.request_key='run-key' AND b.repo_id='repo-original'", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
    assert_eq!(
        db.conn
            .query_row("SELECT revision FROM desktop_preferences", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        7
    );
    assert_eq!(
        db.conn
            .query_row(
                "SELECT applied_at FROM schema_migrations WHERE revision=6",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        19
    );
    assert!(locations.legacy.is_file());
    assert_eq!(locations.migrate().unwrap()["status"], "READY");
}

#[test]
fn resumes_after_activation_and_refuses_two_independent_databases() {
    let root = fixture("cirnetrules-identity-resume");
    let locations = Locations::new(root.path().to_path_buf(), false);
    legacy(&locations.legacy);
    locations.migrate().unwrap();
    fs::remove_file(locations.journal.join(COMPLETE)).unwrap();
    assert_eq!(locations.preview().unwrap()["status"], "RESUME");
    assert!(locations.open().is_err());
    locations.migrate().unwrap();
    assert_eq!(locations.preview().unwrap()["status"], "READY");

    let root = fixture("cirnetrules-identity-conflict");
    let locations = Locations::new(root.path().to_path_buf(), false);
    drop(locations.open().unwrap());
    legacy(&locations.legacy);
    assert_eq!(locations.preview().unwrap()["status"], "CONFLICT");
    assert!(locations.open().is_err());
    assert_eq!(locations.migrate().unwrap_err().code, "IDENTITY_CONFLICT");
    locations.use_current().unwrap();
    let db = locations.open().unwrap();
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM conversations", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert!(locations.legacy.is_file());
}

#[test]
fn completed_receipt_without_separate_backup_digest_remains_readable() {
    let root = fixture("identity-old-receipt");
    let locations = Locations::new(root.path().to_path_buf(), false);
    legacy(&locations.previous);
    let source = lock_source(&locations.previous).unwrap();
    let backup = Database::backup_from(&source, root.path()).unwrap();
    drop(source);
    let backup_name = backup["backup"].as_str().unwrap();
    let backup_path = root.path().join("backups").join(backup_name);
    fs::copy(&backup_path, &locations.active).unwrap();
    ensure_directory(&locations.journal).unwrap();
    let intent = locations.intent(Generation::Previous);
    locations.write_json(STARTED, &intent).unwrap();
    locations
        .write_json(
            RECEIPT,
            &json!({
                "intent": intent,
                "snapshot_sha256": file_digest(&backup_path).unwrap(),
                "source_schema": 6,
                "backup": backup_name
            }),
        )
        .unwrap();
    locations.complete(Generation::Previous).unwrap();
    let before = fs::read(locations.journal.join(RECEIPT)).unwrap();
    let db = locations.open().unwrap();
    verify(&db.conn).unwrap();
    assert_eq!(
        ensure_supported(&db.conn).unwrap(),
        super::super::MIGRATIONS.len() as i64
    );
    assert_eq!(fs::read(locations.journal.join(RECEIPT)).unwrap(), before);
}

#[test]
fn legacy_writer_blocks_migration_and_partial_stage_retries_safely() {
    let root = fixture("cirnetrules-identity-in-use");
    let locations = Locations::new(root.path().to_path_buf(), false);
    legacy(&locations.legacy);
    let legacy = Connection::open(&locations.legacy).unwrap();
    legacy
        .execute_batch("BEGIN IMMEDIATE; UPDATE messages SET content='Uncommitted';")
        .unwrap();
    assert_eq!(locations.migrate().unwrap_err().code, "STORAGE_BUSY");
    assert!(!locations.active.exists());
    legacy.execute_batch("ROLLBACK;").unwrap();
    drop(legacy);
    ensure_directory(&locations.journal).unwrap();
    locations
        .write_json(STARTED, &locations.intent(Generation::Legacy))
        .unwrap();
    fs::write(locations.journal.join(STAGED), b"partial copy").unwrap();
    locations.migrate().unwrap();
    let db = locations.open().unwrap();
    assert_eq!(
        db.conn
            .query_row("SELECT content FROM messages WHERE id=17", [], |row| row
                .get::<_, String>(
                0
            ))
            .unwrap(),
        "Nội dung CGACP"
    );
}

#[test]
fn discovers_previous_product_database_before_opening_a_new_store() {
    let root = fixture("identity-predecessor");
    let path = root.path().join(PREVIOUS_FILE);
    legacy(&path);
    let locations = Locations::new(root.path().to_path_buf(), false);
    let preview = locations.preview().unwrap();
    assert_eq!(preview["status"], "MIGRATION_AVAILABLE");
    assert_eq!(preview["source"], json!(path));
    assert!(locations.open().is_err());
    assert!(!root.path().join("cirnetrules.sqlite3").exists());
}

#[test]
fn previous_product_with_wal_preserves_governance_and_unmodified_payloads() {
    let root = fixture("identity-predecessor-wal");
    let data = root
        .path()
        .join("Kho dữ liệu thử nghiệm")
        .join("long path".repeat(12))
        .join("thư mục ứng dụng".repeat(8));
    ensure_directory(&data).unwrap();
    let locations = Locations::new(data.clone(), false);
    legacy(&locations.previous);
    let mut conn = Connection::open(&locations.previous).unwrap();
    super::super::migrate(&mut conn, Some((&data, &locations.previous))).unwrap();
    conn.set_db_config(
        rusqlite::config::DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE,
        true,
    )
    .unwrap();
    conn.execute_batch(
        "UPDATE messages SET content='User wrote OPENRULES and openrules' WHERE id=17;
         UPDATE session_events SET payload_json='{\"source\":\"OPENRULES\"}' WHERE sequence=18;
         INSERT INTO governance_assets
          (id,kind,slug,title,origin,source_name,revision,draft_json,created_at,updated_at)
          VALUES ('asset-old','rule','old','Title','import','OPENRULES.md',8,'{\"en\":\"OPENRULES\"}',4,7);
         INSERT INTO governance_revisions VALUES
           ('asset-old',7,'evidence-digest','{\"en\":\"OPENRULES\",\"vi\":\"Tài liệu\",\"files\":[{\"path\":\"notes.md\",\"content\":\"OPENRULES\"}]}',
            'Title','old',9,'PUBLISHED',NULL,6);
         UPDATE governance_assets SET active_revision=7 WHERE id='asset-old';
         INSERT INTO governance_assignments VALUES ('binding-original','asset-old',9);
         INSERT INTO governance_reviews VALUES ('asset-old',8,'review-digest','original-token',9999999999);
         INSERT INTO governance_events VALUES (11,'asset-old',7,'PUBLISHED',7);
         INSERT INTO governance_mutations VALUES ('mutation-key','original-fingerprint','asset-old',8,7);
         INSERT INTO governance_session_snapshots VALUES (41,'{\"origin\":\"OPENRULES\"}');"
    ).unwrap();
    drop(conn);
    let read = |connection: &Connection| {
        let sql = "SELECT json_array(m.id,m.content,m.request_key,m.created_at,e.payload_json,
             a.id,a.source_name,a.revision,a.draft_json,v.digest,v.document_json,
             x.binding_id,b.root,b.identity,u.key,u.fingerprint,r.token,r.expires_at,p.manifest_json)
            FROM messages m JOIN session_events e ON e.sequence=18
            JOIN governance_assets a ON a.id='asset-old'
            JOIN governance_revisions v ON v.asset_id=a.id AND v.revision=a.active_revision
            JOIN governance_assignments x ON x.asset_id=a.id
            JOIN bindings b ON b.id=x.binding_id
            JOIN governance_mutations u ON u.asset_id=a.id
            JOIN governance_reviews r ON r.asset_id=a.id
            JOIN governance_session_snapshots p ON p.conversation_id=m.conversation_id
            WHERE m.id=17";
        connection
            .query_row(sql, [], |row| row.get::<_, String>(0))
            .unwrap()
    };
    let preview = locations.preview().unwrap();
    assert_eq!(preview["source_product"], PREVIOUS_PRODUCT);
    assert!(preview["candidates"][0]["wal_bytes"].as_u64().unwrap() > 0);
    let source =
        Connection::open_with_flags(&locations.previous, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let expected = read(&source);
    drop(source);
    locations.migrate().unwrap();
    let db = locations.open().unwrap();
    assert_eq!(read(&db.conn), expected);
    assert_eq!(verify(&db.conn).unwrap()["status"], "PASS");
    verify_history(&db.conn).unwrap();
    assert_eq!(locations.migrate().unwrap()["status"], "READY");
}

#[test]
fn completed_previous_receipt_selects_latest_predecessor_without_rewriting_history() {
    let root = fixture("identity-lineage");
    let locations = Locations::new(root.path().to_path_buf(), false);
    legacy(&locations.legacy);
    let conn = Connection::open(&locations.legacy).unwrap();
    let old = Database::backup_from(&conn, root.path()).unwrap();
    let backup = PathBuf::from(old["path"].as_str().unwrap());
    fs::copy(&backup, &locations.previous).unwrap();
    drop(conn);
    let previous = Connection::open(&locations.previous).unwrap();
    previous
        .execute(
            "UPDATE messages SET content='Latest OPENRULES data' WHERE id=17",
            [],
        )
        .unwrap();
    drop(previous);
    assert_eq!(locations.preview().unwrap()["status"], "CONFLICT");
    assert_eq!(
        locations.use_current().unwrap_err().code,
        "IDENTITY_CONFLICT"
    );
    let receipt = serde_json::to_vec(&json!({"source":locations.legacy,
        "digest":file_digest(&backup).unwrap(),"backup":old["backup"]}))
    .unwrap();
    let path = root.path().join("identity-verified.json");
    fs::write(&path, &receipt).unwrap();
    fs::write(
        root.path().join("identity-activated"),
        b"migration-activated\n",
    )
    .unwrap();
    assert_eq!(
        locations.preview().unwrap()["source_product"],
        PREVIOUS_PRODUCT
    );
    let ancestor = Connection::open(&locations.legacy).unwrap();
    ancestor
        .execute("UPDATE messages SET created_at=99 WHERE id=17", [])
        .unwrap();
    drop(ancestor);
    assert_eq!(locations.migrate().unwrap_err().code, "IDENTITY_CONFLICT");
    assert!(!locations.active.exists());
    let ancestor = Connection::open(&locations.legacy).unwrap();
    ancestor
        .execute("UPDATE messages SET created_at=6 WHERE id=17", [])
        .unwrap();
    drop(ancestor);
    locations.migrate().unwrap();
    assert_eq!(fs::read(path).unwrap(), receipt);
    let db = locations.open().unwrap();
    assert_eq!(
        db.conn
            .query_row("SELECT content FROM messages WHERE id=17", [], |row| row
                .get::<_, String>(
                0
            ))
            .unwrap(),
        "Latest OPENRULES data"
    );
}

#[test]
fn cancellation_and_publish_failure_resume_without_a_blank_store() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let root = fixture("identity-checkpoints");
    let locations = Locations::new(root.path().to_path_buf(), false);
    legacy(&locations.previous);
    let cancellation = AtomicBool::new(false);
    assert_eq!(
        locations
            .migrate_checked(&cancellation, |phase| {
                if phase == "VERIFY" {
                    cancellation.store(true, Ordering::Relaxed);
                }
            })
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    assert_eq!(locations.preview().unwrap()["status"], "RESUME");
    assert!(!locations.active.exists());
    fs::remove_file(locations.journal.join(STAGED)).unwrap();
    fs::create_dir(locations.journal.join(STAGED)).unwrap();
    assert!(locations.migrate().is_err());
    assert!(!locations.active.exists());
    fs::remove_dir(locations.journal.join(STAGED)).unwrap();
    locations.migrate().unwrap();
    assert_eq!(locations.preview().unwrap()["status"], "READY");
    fs::remove_file(&locations.active).unwrap();
    assert_eq!(locations.preview().unwrap_err().code, "IDENTITY_INTEGRITY");
    assert!(locations.open().is_err());
    assert!(!locations.active.exists());
}

#[test]
fn sql_upgrade_failure_leaves_source_and_verified_backup_for_retry() {
    let root = fixture("identity-sql-failure");
    let locations = Locations::new(root.path().to_path_buf(), false);
    legacy(&locations.previous);
    let source = Connection::open(&locations.previous).unwrap();
    source
        .execute_batch("CREATE TABLE governance_assets (conflicting INTEGER);")
        .unwrap();
    drop(source);

    let failure = locations.migrate().unwrap_err();
    assert_eq!(failure.code, "STORAGE_UNAVAILABLE");
    assert!(failure.message.contains("upgrade the staged database"));
    assert_eq!(locations.preview().unwrap()["status"], "RESUME");
    assert!(!locations.active.exists());
    assert!(!locations.journal.join(RECEIPT).exists());
    let backup = fs::read_dir(root.path().join("backups"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let backup_hash = file_digest(&backup).unwrap();
    let source = Connection::open(&locations.previous).unwrap();
    assert_eq!(ensure_supported(&source).unwrap(), 6);
    verify(&source).unwrap();
    source
        .execute_batch("DROP TABLE governance_assets;")
        .unwrap();
    drop(source);

    assert_eq!(locations.migrate().unwrap()["status"], "READY");
    let activated =
        Connection::open_with_flags(&locations.active, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    assert_eq!(
        ensure_supported(&activated).unwrap(),
        super::super::MIGRATIONS.len() as i64
    );
    verify(&activated).unwrap();
    assert_eq!(file_digest(&backup).unwrap(), backup_hash);
}

#[cfg(windows)]
#[test]
fn windows_acl_denial_retains_backup_and_resumes_same_journal() {
    use std::{
        process::Command,
        sync::atomic::{AtomicBool, Ordering},
    };

    struct DenyCreate<'a> {
        path: &'a Path,
        trustee: String,
    }
    impl DenyCreate<'_> {
        fn edit(&self, operation: &str, trustee: &str) -> io::Result<std::process::Output> {
            Command::new("icacls.exe")
                .arg(self.path)
                .args([operation, trustee])
                .output()
        }
    }
    impl Drop for DenyCreate<'_> {
        fn drop(&mut self) {
            let _ = self.edit("/remove:d", &self.trustee);
        }
    }

    let root = fixture("identity-acl");
    let locations = Locations::new(root.path().to_path_buf(), false);
    legacy(&locations.previous);
    let cancellation = AtomicBool::new(false);
    assert_eq!(
        locations
            .migrate_checked(&cancellation, |phase| {
                if phase == "VERIFY" {
                    cancellation.store(true, Ordering::Relaxed);
                }
            })
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    let dir = root.path().join("backups");
    let backup = fs::read_dir(&dir).unwrap().next().unwrap().unwrap().path();
    let backup_hash = file_digest(&backup).unwrap();
    let started = fs::read(locations.journal.join(STARTED)).unwrap();

    let whoami = Command::new("whoami.exe")
        .args(["/user", "/fo", "csv", "/nh"])
        .output()
        .unwrap();
    assert!(whoami.status.success());
    let output = String::from_utf8_lossy(&whoami.stdout);
    let sid = output.trim().rsplit(',').next().unwrap().trim_matches('"');
    assert!(sid.starts_with("S-1-") && sid[4..].bytes().all(|b| b.is_ascii_digit() || b == b'-'));
    let acl = DenyCreate {
        path: &dir,
        trustee: format!("*{sid}"),
    };
    let denial = acl.edit("/deny", &format!("{}:(WD)", acl.trustee)).unwrap();
    assert!(
        denial.status.success(),
        "Could not deny fixture directory writes: {denial:?}"
    );
    let denied = fs::write(dir.join("denial-probe"), b"test").unwrap_err();
    assert_eq!(denied.raw_os_error(), Some(5));

    let failure = locations.migrate().unwrap_err();
    assert_eq!(failure.code, "PERMISSION_DENIED");
    assert!(failure.message.contains("backup"));
    assert_eq!(locations.preview().unwrap()["status"], "RESUME");
    assert!(locations.open().is_err());
    assert!(!locations.active.exists());
    assert_eq!(fs::read(locations.journal.join(STARTED)).unwrap(), started);
    assert_eq!(file_digest(&backup).unwrap(), backup_hash);
    let source = Connection::open(&locations.previous).unwrap();
    assert_eq!(ensure_supported(&source).unwrap(), 6);
    verify(&source).unwrap();
    drop(source);

    drop(acl);
    fs::write(dir.join("recovery-probe"), b"test").unwrap();
    assert_eq!(locations.migrate().unwrap()["status"], "READY");
    let db = locations.open().unwrap();
    assert_eq!(
        db.conn
            .query_row("SELECT count(*) FROM messages WHERE id=17", [], |row| row
                .get::<_, i64>(
                0
            ))
            .unwrap(),
        1
    );
    assert_eq!(file_digest(&backup).unwrap(), backup_hash);
    assert_eq!(fs::read(locations.journal.join(STARTED)).unwrap(), started);
    assert_eq!(locations.migrate().unwrap()["status"], "READY");
}
