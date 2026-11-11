use super::*;
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

const SCHEMA: &[(&str, &str)] = &[
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
    (
        "004_desktop_preferences.sql",
        include_str!("../../migrations/sqlite/004_desktop_preferences.sql"),
    ),
    (
        "005_conversation_lifecycle.sql",
        include_str!("../../migrations/sqlite/005_conversation_lifecycle.sql"),
    ),
    (
        "006_automation_retirement.sql",
        include_str!("../../migrations/sqlite/006_automation_retirement.sql"),
    ),
    (
        "007_governance.sql",
        include_str!("../../migrations/sqlite/007_governance.sql"),
    ),
];

const RULE: &str = "asset-11111111111111111111111111111111";
const DRAFT: &str = "asset-22222222222222222222222222222222";
const ARCHIVED: &str = "asset-33333333333333333333333333333333";
const DELETED: &str = "asset-44444444444444444444444444444444";

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        Self(crate::test_support::temporary_directory("cirnetrules-migration").unwrap())
    }

    fn db(&self, name: &str) -> Connection {
        self.versioned_db(name, 7)
    }

    fn versioned_db(&self, name: &str, version: usize) -> Connection {
        let db = Connection::open(self.0.join(name)).unwrap();
        db.execute_batch("CREATE TABLE schema_migrations (revision INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE, checksum TEXT NOT NULL, applied_at INTEGER NOT NULL) STRICT;").unwrap();
        for (index, (name, sql)) in SCHEMA.iter().take(version).enumerate() {
            db.execute_batch(sql).unwrap();
            db.execute(
                "INSERT INTO schema_migrations VALUES(?1, ?2, ?3, 1)",
                params![index + 1, name, hash(sql.as_bytes())],
            )
            .unwrap();
        }
        db.pragma_update(None, "user_version", version).unwrap();
        db.pragma_update(None, "foreign_keys", true).unwrap();
        db
    }

    fn command(&self, action: &str, options: &[(&str, &str)]) -> Result<Value, String> {
        run(
            &self.0,
            action,
            &options
                .iter()
                .map(|&(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        )
    }

    fn path(&self, name: &str) -> String {
        self.0.join(name).to_str().unwrap().to_owned()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn document(en: &str, vi: &str, dependency: Option<&str>) -> String {
    // The old serializer stores fields in declaration order. The migration
    // must verify the digest against these stored bytes.
    format!(
        r#"{{"en":{},"vi":{},"format":"markdown","description":"fixture","files":[{{"path":"details/example.md","content":"attachment bytes"}}],"dependencies":{}}}"#,
        serde_json::json!(en),
        serde_json::json!(vi),
        serde_json::json!(dependency.into_iter().collect::<Vec<_>>())
    )
}

fn add_asset(db: &Connection, id: &str, slug: &str, draft: Option<String>) {
    db.execute("INSERT INTO governance_assets(id,kind,slug,title,revision,draft_json,created_at,updated_at)
        VALUES(?1,'rule',?2,'Fixture',5,?3,1,1)", params![id, slug, draft]).unwrap();
}

fn add_revision(db: &Connection, id: &str, number: i64, en: &str, prior: Option<i64>) {
    let body = document(en, "Bản dịch", None);
    db.execute(
        "INSERT INTO governance_revisions VALUES(?1,?2,?3,?4,'Fixture','example',?5,?6,?7,1)",
        params![
            id,
            number,
            hash(body.as_bytes()),
            body,
            en.len(),
            if number == 5 { "ROLLBACK" } else { "PUBLISHED" },
            prior
        ],
    )
    .unwrap();
    db.execute(
        "UPDATE governance_assets SET active_revision=?1 WHERE id=?2",
        params![number, id],
    )
    .unwrap();
}

#[test]
fn migration_backup_reads_committed_wal_and_preserves_every_governance_layer() {
    let fixture = Fixture::new();
    let db = fixture.db("source.sqlite");
    db.pragma_update(None, "journal_mode", "WAL").unwrap();
    db.pragma_update(None, "wal_autocheckpoint", 0).unwrap();
    let source = fixture.path("source.sqlite");
    let destination = fixture.path("archive");
    let before = fs::read(&source).unwrap();
    add_asset(&db, RULE, "example", None);
    add_asset(
        &db,
        DRAFT,
        "unpublished",
        Some(document("Draft", "", Some(RULE))),
    );
    add_asset(&db, ARCHIVED, "historical", None);
    add_asset(&db, DELETED, "deleted", None);
    add_revision(&db, RULE, 2, "First", None);
    add_revision(&db, RULE, 3, "Second", Some(2));
    add_revision(&db, RULE, 5, "First", Some(2));
    db.execute_batch(&format!("
        UPDATE governance_assets SET archived=1 WHERE id='{ARCHIVED}';
        UPDATE governance_assets SET deleted_at=5 WHERE id='{DELETED}';
        INSERT INTO governance_dependencies VALUES('{DRAFT}','{RULE}','draft');
        INSERT INTO repositories VALUES('repo-legacy','repo-key','Display Name',1,0,1,1);
        INSERT INTO bindings VALUES('binding-legacy','repo-legacy','machine','checkout','checkout','git','common','identity','anchor',NULL,NULL,0,'ACTIVE',1,1,NULL);
        INSERT INTO governance_assignments VALUES('binding-legacy','{RULE}',1);
        UPDATE governance_assets SET kind='profile' WHERE id='{ARCHIVED}';
        INSERT INTO governance_defaults VALUES(1,'{ARCHIVED}');
        INSERT INTO governance_events VALUES(1,'{RULE}',5,'ROLLBACK',1);
        INSERT INTO governance_reviews VALUES('{DRAFT}',5,'review-digest','private-review-token',9999999999);
        INSERT INTO governance_mutations VALUES('old-key','fingerprint','{RULE}',5,1);
        INSERT INTO conversations VALUES(1,'conversation-key','digest','fixture','handoff','ACTIVE','DASHBOARD',NULL,1,1,1);
        INSERT INTO messages VALUES(1,1,'message-key','digest',1,'USER','private transcript marker',1);
        INSERT INTO automation_definitions VALUES(1,'automation-key','digest','fixture','binding-legacy',1,'handoff','private schedule prompt','[]',NULL,NULL,'WORK','SCOPED',1,0,'{{}}',NULL,1,1,NULL);
        INSERT INTO automation_runs VALUES(1,1,1,'binding-legacy',1,'MANUAL','run-key',NULL,'CANCELLED','{{}}',NULL,NULL,0,1,NULL,NULL,1,1);
        INSERT INTO governance_session_snapshots VALUES(1,'{{\"pinned\":true}}');
        INSERT INTO governance_run_snapshots VALUES(1,'{{\"pinned\":true}}');
    ")).unwrap();
    let wal = fs::read(format!("{source}-wal")).unwrap();
    let shm = fs::read(format!("{source}-shm")).unwrap();
    assert_eq!(fs::read(&source).unwrap(), before);

    let inspect = fixture.command("inspect", &[("source", &source)]).unwrap();
    assert_eq!(inspect["counts"]["governance_assets"], 4);
    assert_eq!(inspect["counts"]["governance_revisions"], 3);
    let exported = fixture
        .command(
            "export",
            &[("source", &source), ("destination", &destination)],
        )
        .unwrap();
    assert_eq!(exported["status"], "PASS");
    assert_eq!(exported["counts"]["governance_session_snapshots"], 1);
    assert_eq!(exported["counts"]["governance_run_snapshots"], 1);
    assert_eq!(exported["counts"]["governance_assignments"], 1);
    assert_eq!(exported["counts"]["governance_defaults"], 1);
    assert!(!exported.to_string().contains("private"));
    assert_eq!(fs::read(&source).unwrap(), before);
    assert_eq!(fs::read(format!("{source}-wal")).unwrap(), wal);
    assert_eq!(fs::read(format!("{source}-shm")).unwrap(), shm);
    let validated = fixture
        .command("validate", &[("archive", &destination)])
        .unwrap();
    assert_eq!(validated["snapshot_sha256"], exported["snapshot_sha256"]);
    assert_eq!(validated["export_sha256"], exported["export_sha256"]);
    let archived = Connection::open(fixture.0.join("archive/snapshot.sqlite")).unwrap();
    let transcript: String = archived
        .query_row("SELECT content FROM messages", [], |r| r.get(0))
        .unwrap();
    assert_eq!(transcript, "private transcript marker");
    let rows = fs::read_to_string(fixture.0.join("archive/governance.jsonl")).unwrap();
    assert!(!rows.contains("private transcript marker"));
    assert!(!rows.contains("private schedule prompt"));
    assert!(rows.contains("attachment bytes"));
    assert!(rows.contains("private-review-token"));
    assert!(rows.contains("old-key"));
    assert!(rows.contains("ROLLBACK"));
}

#[test]
fn migration_rejects_unknown_schema_bad_history_and_foreign_keys() {
    let fixture = Fixture::new();
    let db = fixture.db("source.sqlite");
    let source = fixture.path("source.sqlite");
    db.pragma_update(None, "user_version", 8).unwrap();
    assert!(fixture.command("inspect", &[("source", &source)]).is_err());
    db.pragma_update(None, "user_version", 7).unwrap();
    db.execute(
        "UPDATE schema_migrations SET checksum='bad' WHERE revision=7",
        [],
    )
    .unwrap();
    assert!(fixture.command("inspect", &[("source", &source)]).is_err());
    db.execute(
        "UPDATE schema_migrations SET checksum=?1 WHERE revision=7",
        [hash(SCHEMA[6].1.as_bytes())],
    )
    .unwrap();
    db.pragma_update(None, "foreign_keys", false).unwrap();
    db.execute(
        "INSERT INTO governance_events VALUES(1,'missing',1,'PUBLISHED',1)",
        [],
    )
    .unwrap();
    assert!(fixture.command("inspect", &[("source", &source)]).is_err());
}

#[test]
fn migration_preserves_broken_revision_history_and_flags_it_for_review() {
    let fixture = Fixture::new();
    let db = fixture.db("source.sqlite");
    add_asset(&db, RULE, "example", None);
    add_revision(&db, RULE, 5, "Missing prior", Some(2));
    let source = fixture.path("source.sqlite");
    let destination = fixture.path("archive");
    let receipt = fixture
        .command(
            "export",
            &[("source", &source), ("destination", &destination)],
        )
        .unwrap();
    assert_eq!(receipt["counts"]["governance_revisions"], 1);
    assert_eq!(receipt["issues"]["MISSING_PRIOR_REVISION"], 1);
    assert_eq!(
        fixture
            .command("validate", &[("archive", &destination)])
            .unwrap()["issues"]["MISSING_PRIOR_REVISION"],
        1
    );
    fs::write(fixture.0.join("archive/governance.jsonl"), "tampered").unwrap();
    assert!(
        fixture
            .command("validate", &[("archive", &destination)])
            .is_err()
    );
}

#[test]
fn migration_preserves_known_pre_governance_schema_without_upgrading_it() {
    let fixture = Fixture::new();
    let db = fixture.versioned_db("old.sqlite", 6);
    db.execute(
        "INSERT INTO repositories VALUES('repo','common','Historical',1,0,1,1)",
        [],
    )
    .unwrap();
    let source = fixture.path("old.sqlite");
    let archive = fixture.path("archive");
    let receipt = fixture
        .command("export", &[("source", &source), ("destination", &archive)])
        .unwrap();
    assert_eq!(receipt["schema"], 6);
    assert_eq!(receipt["counts"]["repositories"], 1);
    assert!(receipt["counts"].get("governance_assets").is_none());
    assert_eq!(
        fixture
            .command("validate", &[("archive", &archive)])
            .unwrap()["schema"],
        6
    );
    let snapshot = Connection::open(fixture.0.join("archive/snapshot.sqlite")).unwrap();
    assert_eq!(
        snapshot
            .pragma_query_value::<i64, _>(None, "user_version", |row| row.get(0))
            .unwrap(),
        6
    );
}

#[test]
fn migration_copies_checkpointed_wal_without_creating_original_sidecars() {
    let fixture = Fixture::new();
    let db = fixture.db("source.sqlite");
    db.pragma_update(None, "journal_mode", "WAL").unwrap();
    add_asset(&db, RULE, "checkpointed", None);
    drop(db);
    let source = fixture.path("source.sqlite");
    let before = fs::read(&source).unwrap();
    assert_eq!(&before[18..20], &[2, 2]);
    assert!(!Path::new(&format!("{source}-wal")).exists());
    assert!(!Path::new(&format!("{source}-shm")).exists());
    assert_eq!(
        fixture.command("inspect", &[("source", &source)]).unwrap()["status"],
        "BLOCKED"
    );
    let archive = fixture.path("archive");
    assert_eq!(
        fixture
            .command("export", &[("source", &source), ("destination", &archive)])
            .unwrap()["counts"]["governance_assets"],
        1
    );
    assert_eq!(
        fixture
            .command("validate", &[("archive", &archive)])
            .unwrap()["status"],
        "PASS"
    );
    assert_eq!(fs::read(&source).unwrap(), before);
    assert!(!Path::new(&format!("{source}-wal")).exists());
    assert!(!Path::new(&format!("{source}-shm")).exists());
}

#[test]
fn migration_fails_closed_on_integrity_errors_and_aliases() {
    let fixture = Fixture::new();
    let source = fixture.path("source.sqlite");
    assert_eq!(
        fixture.command("inspect", &[("source", &source)]).unwrap()["status"],
        "NOT_FOUND"
    );
    assert!(
        fixture
            .command("inspect", &[("source", &fixture.path("."))])
            .is_err()
    );
    let corrupt_path = fixture.path("corrupt.sqlite");
    let db = fixture.db("corrupt.sqlite");
    add_asset(&db, RULE, "example", None);
    let page_size: usize = db
        .pragma_query_value(None, "page_size", |row| row.get(0))
        .unwrap();
    let page_count: u32 = db
        .pragma_query_value(None, "page_count", |row| row.get(0))
        .unwrap();
    drop(db);
    // SQLite loads CHECK expressions only for a writable connection.
    // An orphaned B-tree page triggers integrity checking in read-only mode.
    let mut bytes = fs::read(&corrupt_path).unwrap();
    bytes[28..32].copy_from_slice(&(page_count + 1).to_be_bytes());
    bytes.resize(bytes.len() + page_size, 0);
    fs::write(&corrupt_path, bytes).unwrap();
    assert!(
        fixture
            .command("inspect", &[("source", &corrupt_path)])
            .unwrap_err()
            .contains("integrity")
    );
    let _db = fixture.db("source.sqlite");
    fs::create_dir(fixture.0.join("Archive")).unwrap();
    let destination = fixture.path("archive");
    assert!(
        fixture
            .command(
                "export",
                &[("source", &source), ("destination", &destination)]
            )
            .is_err()
    );
    // Preserve the spelling sent to the parser. PathBuf::join normalizes
    // parent components when appending to a Windows verbatim path.
    let separator = std::path::MAIN_SEPARATOR;
    let traversing = format!(
        "{}{separator}Archive{separator}..{separator}snapshot",
        fixture.0.display()
    );
    assert!(
        fixture
            .command(
                "export",
                &[("source", &source), ("destination", &traversing)]
            )
            .is_err()
    );
}

fn fixture_git(root: &Path, args: &[&str]) {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "fixture Git command failed: {:?}",
        args
    );
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir(target).unwrap();
    for child in fs::read_dir(source).unwrap() {
        let child = child.unwrap();
        let destination = target.join(child.file_name());
        if child.file_type().unwrap().is_dir() {
            copy_tree(&child.path(), &destination);
        } else {
            fs::copy(child.path(), destination).unwrap();
        }
    }
}

#[test]
fn migration_proposal_resumes_and_rollback_preserves_other_files() {
    let fixture = Fixture::new();
    let db = fixture.db("source.sqlite");
    add_asset(&db, RULE, "example", None);
    add_revision(&db, RULE, 2, "First rule", None);
    add_asset(&db, DRAFT, "unpublished", Some(document("Draft", "", None)));
    let source = fixture.path("source.sqlite");
    let archive = fixture.path("archive");
    fixture
        .command("export", &[("source", &source), ("destination", &archive)])
        .unwrap();

    let root = fixture.0.join("governance");
    fs::create_dir(&root).unwrap();
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"));
    for directory in ["en", "vi"] {
        copy_tree(&checkout.join(directory), &root.join(directory));
    }
    for path in [".gitignore", "VERSION", "governance.yaml"] {
        fs::copy(checkout.join(path), root.join(path)).unwrap();
    }
    fixture_git(&root, &["init", "-q"]);
    fixture_git(&root, &["add", "."]);
    fixture_git(
        &root,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-qm",
            "fixture",
        ],
    );
    let root = root.canonicalize().unwrap();
    let mut options = BTreeMap::from([("archive".to_owned(), archive)]);
    let mut duplicate = options.clone();
    duplicate.insert("additional-archive".into(), fixture.path("archive"));
    assert!(run(&root, "preview", &duplicate).is_err());
    let second = fixture.db("second.sqlite");
    add_asset(&second, ARCHIVED, "example", None);
    add_revision(&second, ARCHIVED, 2, "Conflicting rule", None);
    let secondary_archive = fixture.path("second-archive");
    fixture
        .command(
            "export",
            &[
                ("source", &fixture.path("second.sqlite")),
                ("destination", &secondary_archive),
            ],
        )
        .unwrap();
    duplicate.insert("additional-archive".into(), secondary_archive);
    let collision = run(&root, "preview", &duplicate).unwrap();
    assert_eq!(collision["summary"]["issues"]["MULTI_SOURCE_COLLISION"], 2);
    let preview = run(&root, "preview", &options).unwrap();
    assert_eq!(preview["summary"]["review_candidates"], 2);
    assert_eq!(preview["summary"]["issues"]["MISSING_VI"], 1);
    assert_eq!(preview["file_changes"][0]["action"], "ADD");
    assert!(!preview.to_string().contains("First rule"));
    let expected = preview["preview_sha256"].as_str().unwrap().to_owned();
    let mut review = options.clone();
    review.insert("preview-out".into(), fixture.path("review"));
    let local_preview = run(&root, "preview", &review).unwrap();
    assert_eq!(local_preview["preview_sha256"], expected);
    assert_eq!(local_preview["local_proposal_written"], true);
    assert_eq!(
        hash(&fs::read(fixture.0.join("review/proposal.json")).unwrap()),
        expected
    );
    options.insert("expected-digest".into(), expected.clone());
    let dest = root.join("en/rules/example.md");
    fs::write(&dest, "concurrent edit").unwrap();
    assert!(run(&root, "apply", &options).is_err());
    fs::remove_file(&dest).unwrap();

    assert_eq!(run(&root, "apply", &options).unwrap()["state"], "STAGED");
    let state = root.join(".cirnetrules/migration").join(&expected);
    let proposal = fs::read(state.join("proposal.json")).unwrap();
    let proposal_text = std::str::from_utf8(&proposal).unwrap();
    assert!(proposal_text.contains("First rule"));
    assert!(proposal_text.contains("en/rules/example.md"));
    assert!(proposal_text.contains("attachment bytes"));
    assert!(proposal_text.contains("\"status\":\"DRAFT\""));
    assert!(!dest.exists());
    fs::remove_file(state.join("receipt.json")).unwrap();
    assert_eq!(run(&root, "apply", &options).unwrap()["state"], "RESUMED");
    assert_eq!(run(&root, "apply", &options).unwrap()["state"], "UNCHANGED");
    assert_eq!(fs::read(state.join("proposal.json")).unwrap(), proposal);
    let rollback = BTreeMap::from([("expected-digest".to_owned(), expected)]);
    fs::write(state.join("proposal.json"), "operator change").unwrap();
    assert!(run(&root, "rollback", &rollback).is_err());
    assert_eq!(
        fs::read(state.join("proposal.json")).unwrap(),
        b"operator change"
    );
    fs::write(state.join("proposal.json"), proposal).unwrap();
    assert_eq!(
        run(&root, "rollback", &rollback).unwrap()["state"],
        "ROLLED_BACK"
    );
    assert_eq!(
        run(&root, "rollback", &rollback).unwrap()["state"],
        "UNCHANGED"
    );
    let audit = audit(&root, rollback["expected-digest"].as_str()).unwrap();
    assert_eq!(audit["state"], "ROLLED_BACK");
    assert_eq!(audit["sources"], preview["sources"]);
    assert!(!state.join("proposal.json").exists());
    assert!(state.join("receipt.json").is_file());
}

#[test]
fn migration_007_accepts_only_observed_checksums_and_verifies_table_schema() {
    const LF: &str = "43481e81aadc4791aba0e34e9ac18afd5f3ec85425f87a36093b5e79da726f36";
    const WINDOWS: &str = "3b0f5974923ef09b5f05ae70035ce1d08c05d1bbe220c893563ae85280ca7b6b";
    let fixture = Fixture::new();
    let db = fixture.db("variants.sqlite");
    let source = fixture.path("variants.sqlite");
    for checksum in [LF, WINDOWS] {
        db.execute(
            "UPDATE schema_migrations SET checksum=?1 WHERE revision=7",
            [checksum],
        )
        .unwrap();
        assert_eq!(
            fixture.command("inspect", &[("source", &source)]).unwrap()["schema"],
            7
        );
    }
    db.execute(
        "UPDATE schema_migrations SET checksum=?1 WHERE revision=7",
        ["a".repeat(64)],
    )
    .unwrap();
    assert!(
        fixture
            .command("inspect", &[("source", &source)])
            .unwrap_err()
            .contains("checksum")
    );
    db.execute(
        "UPDATE schema_migrations SET checksum=?1 WHERE revision=7",
        [LF],
    )
    .unwrap();
    db.execute_batch("ALTER TABLE governance_assets ADD COLUMN unexpected TEXT")
        .unwrap();
    assert!(fixture.command("inspect", &[("source", &source)]).is_err());
}
