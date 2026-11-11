use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

struct Fixture {
    workspace: PathBuf,
    governance: PathBuf,
    product: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let workspace = crate::test_support::temporary_directory("cirnetrules-e2e").unwrap();
        let governance = workspace.join("cirnet-rules");
        let product = workspace.join("Interface feature worktree");
        fs::create_dir_all(&governance).unwrap();
        fs::create_dir_all(&product).unwrap();
        let source = Path::new(env!("CARGO_MANIFEST_DIR"));
        for path in [
            "VERSION",
            "governance.yaml",
            "en/schemas/governance.schema.json",
        ] {
            Self::put(&governance, path, &fs::read(source.join(path)).unwrap());
        }
        let mut registry: Value =
            serde_json::from_slice(&fs::read(governance.join("governance.yaml")).unwrap()).unwrap();
        for asset in registry["assets"]["entries"].as_array().unwrap() {
            if asset["status"] == "ACTIVE" {
                let path = asset["path"].as_str().unwrap();
                Self::put(&governance, path, &fs::read(source.join(path)).unwrap());
            }
        }
        let product_registry = registry["repositories"]["cirnet-interface"]
            .as_object_mut()
            .unwrap();
        for key in ["snapshot", "plan", "spec", "remote"] {
            product_registry.remove(key);
        }
        registry["repositories"]["cirnet-docs"] = json!({
            "profile":"documentation","ownership":"fixture manuals",
            "classification":"supporting","depends_on":[]
        });
        Self::put(
            &governance,
            "governance.yaml",
            &serde_json::to_vec_pretty(&registry).unwrap(),
        );
        Self::git(&governance, &["init", "--quiet"]);
        Self::git(
            &governance,
            &["add", "--", "VERSION", "governance.yaml", "en"],
        );
        Self::git(
            &governance,
            &[
                "-c",
                "user.name=CIRNETRULES fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgSign=false",
                "commit",
                "--quiet",
                "-m",
                "Seed governance fixture",
            ],
        );
        Self::put(
            &product,
            "src/components/Toolbar.vue",
            b"<button>Before</button>\n",
        );
        Self::put(
            &product,
            "tests/toolbar.test.ts",
            b"// Fixture assertion input\n",
        );
        Self::put(
            &product,
            "docs/en/workbench.md",
            b"# Workbench\n\nBefore.\n",
        );
        Self::put(
            &product,
            "docs/vi/workbench.md",
            "# Không gian\n\nTrước.\n".as_bytes(),
        );
        Self::git(&product, &["init", "--quiet"]);
        Self::git(&product, &["add", "--", "."]);
        Self::git(
            &product,
            &[
                "-c",
                "user.name=CIRNETRULES fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgSign=false",
                "commit",
                "--quiet",
                "-m",
                "Seed fixture",
            ],
        );
        Self {
            workspace,
            governance,
            product,
        }
    }
    fn put(root: &Path, path: &str, bytes: &[u8]) {
        let dest = root.join(path);
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::write(dest, bytes).unwrap();
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
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fn cli(&self, args: &[&str], code: i32) -> Value {
        let output = Command::new(env!("CARGO_BIN_EXE_cirnet-governance"))
            .current_dir(&self.product)
            .env_remove("CIRNETRULES_GOVERNANCE")
            .args(args)
            .output()
            .unwrap();
        let body: Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stderr)));
        assert_eq!(output.status.code(), Some(code), "{args:?}: {body}");
        body["result"].clone()
    }
    fn json(&self, path: &str, value: &Value) {
        Self::put(
            &self.governance,
            path,
            &serde_json::to_vec_pretty(value).unwrap(),
        );
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.workspace);
    }
}

fn sha(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

#[test]
fn legacy_maintenance_state_does_not_get_shadowed_by_a_new_store() {
    let fixture = Fixture::new();
    let original = b"{\"legacy\":true}\n";
    Fixture::put(&fixture.governance, ".cgacp/tasks/legacy.json", original);
    let blocked = fixture.cli(&["task-show", "--task", "legacy"], 2);
    assert!(
        blocked["reason"]
            .as_str()
            .unwrap()
            .contains("reviewed migration")
    );
    assert!(!fixture.governance.join(".cirnetrules").exists());
    assert_eq!(
        fs::read(fixture.governance.join(".cgacp/tasks/legacy.json")).unwrap(),
        original,
    );
}

#[test]
fn reviewed_legacy_task_recovery_preserves_evidence_and_requires_a_new_grant() {
    let f = Fixture::new();
    let mut proposal = json!({
        "id":"recovered","request":"Resume toolbar work","objective":"Review the toolbar label",
        "non_goals":["No release"],"primary_repo":"cirnet-interface",
        "hints":["src/components/Toolbar.vue"],
        "read_scope":{"cirnet-interface":["src/components/Toolbar.vue"]},
        "write_scope":{},"verification_scope":{},"documentation_scope":{},
        "risk":"low","role":"developer","mode":"WORK","owner":"builder",
        "authorization":{"owner":"maintainer","basis":"New operator review","actions":[]},
        "spec":"en/specs/cirnet-interface/recovered/spec.md",
        "plan":"en/plans/cirnet-interface/recovered/plan.md",
        "acceptance":["Review current toolbar"],"commands":[],
        "docs_targets":[],"docs_not_required":"Read-only recovery"
    });
    let plan = proposal["plan"].as_str().unwrap().to_owned();
    Fixture::put(
        &f.governance,
        proposal["spec"].as_str().unwrap(),
        b"# Spec\n",
    );
    Fixture::put(&f.governance, &plan, b"# Plan\n");
    f.json("descriptor.json", &proposal);
    let old_path = ".openrules/tasks/recovered.json";
    let mut old = json!({
        "descriptor":proposal,"state":"IN_PROGRESS","revision":37,"approval_epoch":9,
        "checkpoint":{"summary":"Previous checkpoint"},"evidence":[{"sha256":"preserve this"}],
        "review":{"actor":"old-reviewer"},"acceptance":{"actor":"old-owner"}
    });
    old["descriptor"]["authorization"]["actions"] = json!(["read", "write", "release"]);
    f.json(old_path, &old);
    let original = fs::read(f.governance.join(old_path)).unwrap();
    let lease_path = ".openrules/runs/recovered/active.json";
    let lease =
        b"{\"actor\":\"builder\",\"expires_at_ms\":4102444800000,\"token\":\"fixture-lease\"}\n";
    Fixture::put(&f.governance, lease_path, lease);

    let recover = |review: &str, code| {
        f.cli(
            &[
                "task-create",
                "--descriptor",
                "descriptor.json",
                "--legacy-review",
                review,
            ],
            code,
        )
    };
    f.cli(&["task-create", "--descriptor", "descriptor.json"], 2);
    let preview = recover("preview", 0);
    let digest = preview["review_sha256"].as_str().unwrap();
    assert_eq!(preview["source"]["sha256"], sha(&original));
    assert_eq!(preview["source"]["revision"], 37);
    assert_eq!(preview["source"]["state"], "IN_PROGRESS");
    assert_eq!(preview["proposal"]["descriptor"], proposal);
    assert!(!f.governance.join(".cirnetrules").exists());
    proposal["depends_on"] = json!(["missing-task"]);
    f.json("descriptor.json", &proposal);
    recover("preview", 1);
    proposal.as_object_mut().unwrap().remove("depends_on");
    f.json("descriptor.json", &proposal);

    Fixture::put(&f.governance, &plan, b"# Changed plan\n");
    recover(digest, 1);
    Fixture::put(&f.governance, &plan, b"# Plan\n");
    let mut changed = original.clone();
    changed.push(b'\n');
    Fixture::put(&f.governance, old_path, &changed);
    recover(digest, 1);
    Fixture::put(&f.governance, old_path, &original);
    let ambiguous = ".cgacp/tasks/recovered.json";
    Fixture::put(&f.governance, ambiguous, &original);
    recover(digest, 1);
    fs::remove_file(f.governance.join(ambiguous)).unwrap();
    let active = ".cirnetrules/tasks/recovered.json";
    let unrelated = json!({"descriptor":{"id":"recovered"},"state":"PROPOSED","revision":1});
    f.json(active, &unrelated);
    recover(digest, 1);
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(f.governance.join(active)).unwrap()).unwrap(),
        unrelated
    );
    fs::remove_file(f.governance.join(active)).unwrap();

    let restored = recover(digest, 0)["task"].clone();
    assert_eq!(recover(digest, 0)["task"], restored);
    assert_eq!(
        f.cli(&["task-show", "--task", "recovered"], 0)["task"],
        restored
    );
    assert_eq!(restored["recovery"]["review_sha256"], digest);
    assert_eq!(restored["recovery"]["source"], preview["source"]);
    assert_eq!(restored["state"], "PROPOSED");
    assert_eq!(restored["checkpoint"], Value::Null);
    assert_eq!(restored["baseline"], Value::Null);
    assert_eq!(restored["evidence"], json!([]));
    assert_eq!(restored["review"], Value::Null);
    assert_eq!(restored["acceptance"], Value::Null);
    assert!(!f.governance.join(".cirnetrules/runs").exists());
    f.cli(&["route", "--task", "recovered"], 2);
    f.cli(
        &[
            "task-transition",
            "--task",
            "recovered",
            "--revision",
            "1",
            "--actor",
            "maintainer",
            "--to",
            "READY",
        ],
        1,
    );

    f.cli(&["workspace-discover"], 0);
    f.cli(
        &[
            "workspace-bind",
            "--repo",
            "cirnet-interface",
            "--path",
            "Interface feature worktree",
            "--actor",
            "maintainer",
        ],
        0,
    );
    proposal["authorization"]["actions"] = json!(["read"]);
    f.json("descriptor.json", &proposal);
    recover("preview", 1);
    let revised = f.cli(
        &[
            "task-revise",
            "--task",
            "recovered",
            "--revision",
            "1",
            "--actor",
            "maintainer",
            "--descriptor",
            "descriptor.json",
        ],
        0,
    );
    assert_eq!(revised["task"]["recovery"], restored["recovery"]);
    f.cli(
        &[
            "task-transition",
            "--task",
            "recovered",
            "--revision",
            "2",
            "--actor",
            "maintainer",
            "--to",
            "READY",
        ],
        0,
    );
    let context = f.cli(
        &[
            "task-context",
            "--task",
            "recovered",
            "--session",
            "recovery",
        ],
        0,
    );
    assert!(context["metrics"]["emitted_bytes"].as_u64().unwrap() > 0);
    assert_eq!(
        f.cli(
            &[
                "task-context",
                "--task",
                "recovered",
                "--session",
                "recovery"
            ],
            0
        )["metrics"]["emitted_bytes"],
        0
    );
    assert_eq!(fs::read(f.governance.join(old_path)).unwrap(), original);
    assert_eq!(fs::read(f.governance.join(lease_path)).unwrap(), lease);
}

#[test]
fn a_ui_task_routes_verifies_and_resumes_two_docs_destinations() {
    let f = Fixture::new();
    let discovered = f.cli(&["workspace-discover"], 0);
    assert!(
        !discovered.to_string().contains("github.com"),
        "{discovered}"
    );
    f.cli(
        &[
            "workspace-bind",
            "--repo",
            "cirnet-interface",
            "--path",
            "Interface feature worktree",
            "--actor",
            "maintainer",
        ],
        0,
    );
    Fixture::put(
        &f.governance,
        "en/specs/cirnet-interface/ui-docs/spec.md",
        b"# Confirm fixture toolbar label\n",
    );
    Fixture::put(
        &f.governance,
        "en/plans/cirnet-interface/ui-docs/plan.md",
        b"# Edit label and verify tests\n",
    );
    f.json("descriptor.json",&json!({
        "id":"ui-docs","request":"Adjust toolbar label","objective":"Render the requested label","non_goals":["No release"],
        "primary_repo":"cirnet-interface","hints":["src/components/Toolbar.vue"],
        "read_scope":{"cirnet-interface":["src/components/Toolbar.vue","tests/toolbar.test.ts"]},
        "write_scope":{"cirnet-interface":["src/components/Toolbar.vue"]},
        "verification_scope":{"cirnet-interface":["src/components/Toolbar.vue","tests/toolbar.test.ts"]},
        "documentation_scope":{"cirnet-interface":["docs/en/workbench.md"],"cirnet-docs":["docs/vi/workbench.md"]},
        "risk":"low","role":"developer","mode":"WORK","owner":"builder",
        "authorization":{"owner":"maintainer","basis":"fixture-authorized scoped work","actions":["read","write","verify","review","accept","docs"]},
        "spec":"en/specs/cirnet-interface/ui-docs/spec.md","plan":"en/plans/cirnet-interface/ui-docs/plan.md",
        "acceptance":["Diff has valid whitespace and expected files"],"commands":["interface-diff"],
        "docs_targets":[{"repo":"cirnet-interface","path":"docs/en/workbench.md","locale":"en"},
                        {"repo":"cirnet-docs","path":"docs/vi/workbench.md","locale":"vi"}]
    }));
    f.cli(&["task-create", "--descriptor", "descriptor.json"], 0);
    let routed = f.cli(&["route", "--task", "ui-docs"], 0);
    assert_eq!(routed["release_checked"], false);
    assert_eq!(routed["selected"].as_object().unwrap().len(), 1);
    let first = f.cli(&["task-context", "--task", "ui-docs", "--session", "ui"], 0);
    let second = f.cli(&["task-context", "--task", "ui-docs", "--session", "ui"], 0);
    assert_eq!(first["metrics"]["repos_accessed"], 1);
    assert_eq!(second["metrics"]["emitted_bytes"], 0);
    assert!(first["metrics"]["emitted_bytes"].as_u64().unwrap() > 0);
    let session = f.workspace.join("Agent session");
    fs::create_dir(&session).unwrap();
    for agent in ["codex", "claude", "antigravity"] {
        let handoff = f.cli(
            &[
                "task-bootstrap",
                "--task",
                "ui-docs",
                "--agent",
                agent,
                "--session",
                session.to_str().unwrap(),
                "--dry-run",
            ],
            0,
        );
        assert_eq!(handoff["bundlePreviewed"], true);
        assert_eq!(handoff["nativeDiscovery"], "NOT_RUN");
        assert_eq!(handoff["manifest"]["taskPacket"]["mode"], "WORK");
    }
    assert_eq!(fs::read_dir(&session).unwrap().count(), 0);
    f.cli(
        &[
            "release-check",
            "--task",
            "ui-docs",
            "--actor",
            "maintainer",
        ],
        2,
    );
    f.cli(
        &[
            "task-transition",
            "--task",
            "ui-docs",
            "--revision",
            "1",
            "--actor",
            "maintainer",
            "--to",
            "READY",
        ],
        0,
    );
    f.cli(
        &[
            "task-transition",
            "--task",
            "ui-docs",
            "--revision",
            "2",
            "--actor",
            "builder",
            "--to",
            "IN_PROGRESS",
        ],
        0,
    );
    Fixture::put(
        &f.product,
        "src/components/Toolbar.vue",
        b"<button>After</button>\n",
    );
    Fixture::put(
        &f.product,
        "src/auth/session.ts",
        b"// boundary scope creep\n",
    );
    let guard = f.cli(&["diff-check", "--task", "ui-docs"], 2);
    assert!(guard.to_string().contains("src/auth/session.ts"));
    fs::remove_file(f.product.join("src/auth/session.ts")).unwrap();
    f.cli(
        &[
            "task-transition",
            "--task",
            "ui-docs",
            "--revision",
            "3",
            "--actor",
            "builder",
            "--to",
            "IMPLEMENTED",
        ],
        0,
    );
    let verified = f.cli(
        &[
            "task-verify",
            "--task",
            "ui-docs",
            "--revision",
            "4",
            "--actor",
            "maintainer",
        ],
        0,
    );
    assert_eq!(
        verified["evidence"]["verification_kind"],
        "tool_verification"
    );
    let review = f.cli(&["review-context", "--task", "ui-docs"], 0);
    assert!(
        review["review"]["diff"]
            .to_string()
            .contains("+<button>After</button>")
    );
    assert!(
        review["sources"]
            .to_string()
            .contains("tests/toolbar.test.ts")
    );
    Fixture::put(
        &f.product,
        "tests/toolbar.test.ts",
        b"// Post-verification change\n",
    );
    f.cli(
        &[
            "task-accept",
            "--task",
            "ui-docs",
            "--revision",
            "5",
            "--actor",
            "maintainer",
        ],
        2,
    );
    Fixture::put(
        &f.product,
        "tests/toolbar.test.ts",
        b"// Fixture assertion input\n",
    );
    f.cli(
        &[
            "task-accept",
            "--task",
            "ui-docs",
            "--revision",
            "5",
            "--actor",
            "builder",
        ],
        1,
    );
    let manuals = f.workspace.join("Manuals worktree");
    let replacement = f.workspace.join("Replacement worktree");
    for worktree in [&manuals, &replacement] {
        Fixture::put(
            worktree,
            "docs/vi/workbench.md",
            "# Không gian\n\nTrước.\n".as_bytes(),
        );
        Fixture::git(worktree, &["init", "--quiet"]);
        Fixture::git(worktree, &["add", "--", "docs"]);
        Fixture::git(
            worktree,
            &[
                "-c",
                "user.name=CIRNETRULES fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgSign=false",
                "commit",
                "--quiet",
                "-m",
                "Seed manual",
            ],
        );
    }
    let docs_binding = f.cli(
        &[
            "workspace-bind",
            "--repo",
            "cirnet-docs",
            "--path",
            "Manuals worktree",
            "--actor",
            "maintainer",
        ],
        0,
    );
    f.cli(
        &[
            "task-accept",
            "--task",
            "ui-docs",
            "--revision",
            "5",
            "--actor",
            "maintainer",
        ],
        0,
    );
    let en = f.product.join("docs/en/workbench.md");
    let vi = manuals.join("docs/vi/workbench.md");
    let proposal = |path: &str, locale: &str, base: &[u8], content: &str| {
        json!({"repo":if locale=="vi" {"cirnet-docs"} else {"cirnet-interface"},
            "path":path,"locale":locale,"base_sha256":sha(base),"content":content})
    };
    let english = proposal(
        "docs/en/workbench.md",
        "en",
        &fs::read(&en).unwrap(),
        "# Workbench\n\nAfter.\n",
    );
    let vietnamese = proposal(
        "docs/vi/workbench.md",
        "vi",
        &fs::read(&vi).unwrap(),
        "# Không gian\n\nSau.\n",
    );
    f.json("docs.json", &json!({"targets":[english,vietnamese]}));
    f.cli(
        &[
            "docs-preview",
            "--task",
            "ui-docs",
            "--proposal",
            "docs.json",
        ],
        0,
    );
    assert!(fs::read_to_string(&en).unwrap().contains("Before."));
    let other = f.cli(
        &[
            "workspace-bind",
            "--repo",
            "cirnet-docs",
            "--path",
            "Replacement worktree",
            "--actor",
            "maintainer",
            "--replace",
            docs_binding["binding"].as_str().unwrap(),
        ],
        0,
    );
    let blocked = f.cli(
        &[
            "docs-preview",
            "--task",
            "ui-docs",
            "--proposal",
            "docs.json",
        ],
        2,
    );
    assert!(blocked.to_string().contains("outside the accepted binding"));
    f.cli(
        &[
            "workspace-bind",
            "--repo",
            "cirnet-docs",
            "--path",
            "Manuals worktree",
            "--actor",
            "maintainer",
            "--replace",
            other["binding"].as_str().unwrap(),
        ],
        0,
    );
    Fixture::put(
        &manuals,
        "docs/vi/workbench.md",
        "# Không gian\n\nMột thay đổi khác.\n".as_bytes(),
    );
    f.cli(
        &[
            "docs-apply",
            "--task",
            "ui-docs",
            "--proposal",
            "docs.json",
            "--revision",
            "6",
            "--actor",
            "maintainer",
        ],
        1,
    );
    let partial = f.cli(
        &[
            "docs-apply",
            "--task",
            "ui-docs",
            "--proposal",
            "docs.json",
            "--revision",
            "6",
            "--actor",
            "builder",
        ],
        2,
    );
    assert_eq!(partial["revision"], 7);
    assert!(fs::read_to_string(&en).unwrap().contains("After."));
    assert!(fs::read_to_string(&vi).unwrap().contains("thay đổi khác"));
    let vietnamese = proposal(
        "docs/vi/workbench.md",
        "vi",
        &fs::read(&vi).unwrap(),
        "# Không gian\n\nSau.\n",
    );
    f.json("docs.json", &json!({"targets":[english,vietnamese]}));
    let synced = f.cli(
        &[
            "docs-apply",
            "--task",
            "ui-docs",
            "--proposal",
            "docs.json",
            "--revision",
            "7",
            "--actor",
            "builder",
        ],
        0,
    );
    assert_eq!(synced["state"], "DOCS_SYNCED");
    assert_eq!(synced["targets"][0]["result"]["unchanged"], true);
    f.cli(&["docs-check", "--task", "ui-docs"], 0);
    let closed = f.cli(
        &[
            "task-close",
            "--task",
            "ui-docs",
            "--revision",
            "8",
            "--actor",
            "maintainer",
        ],
        0,
    );
    assert_eq!(closed["state"], "CLOSED");
    assert!(!f.product.join(".cirnetrules").exists());
    assert!(!f.product.join("en/plans").exists());
    println!(
        "CIRNETRULES UI cold {} warm {}",
        first["metrics"], second["metrics"]
    );
}
