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
    let root = fixture("openrules-identity");
    let locations = Locations::new(root.path().join("OPENRULES"), true);
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
    let root = fixture("openrules-identity-resume");
    let locations = Locations::new(root.path().to_path_buf(), false);
    legacy(&locations.legacy);
    locations.migrate().unwrap();
    fs::remove_file(locations.data_dir.join(COMPLETE)).unwrap();
    assert_eq!(locations.preview().unwrap()["status"], "RESUME");
    assert!(locations.open().is_err());
    locations.migrate().unwrap();
    assert_eq!(locations.preview().unwrap()["status"], "READY");

    let root = fixture("openrules-identity-conflict");
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
fn legacy_writer_blocks_migration_and_partial_stage_retries_safely() {
    let root = fixture("openrules-identity-in-use");
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
    locations
        .write_marker(STARTED, b"migration-prepared\n")
        .unwrap();
    fs::write(root.path().join(STAGED), b"partial copy").unwrap();
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
