use super::*;
use crate::authority::{AssetKind, Resolved, Source};

fn source(id: &str, kind: AssetKind, path: &str, body: &str) -> Source {
    Source {
        id: id.into(),
        kind,
        path: path.into(),
        revision: 1,
        dependencies: Vec::new(),
        sha256: docs::sha256(body.as_bytes()),
        trust: "governance_instruction",
        bytes: body.as_bytes().to_vec(),
    }
}

fn selection() -> Resolved {
    let mut skill = source(
        "SKILL-test",
        AssetKind::Skill,
        "en/skills/test/SKILL.md",
        "<!-- doc-id: SKILL-test -->\n---\nname: test\ndescription: Fixture workflow\n---\n\nRead the [rule](../../rules/test.md).\n",
    );
    skill.dependencies.push("RULE-test".into());
    Resolved {
        repository: "example".into(),
        task: None,
        task_provenance: None,
        profile: None,
        digest: "a".repeat(64),
        commit: "b".repeat(40),
        dirty: true,
        binding: None,
        included: vec![
            source(
                "RULE-test",
                AssetKind::Rule,
                "en/rules/test.md",
                "# Fixture rule\n",
            ),
            skill,
            source(
                "SKILL-executable",
                AssetKind::Skill,
                "en/skills/executable/SKILL.md",
                "---\nname: executable\ndescription: Fixture\nallowed-tools: Bash\n---\n\n!`git status`\n",
            ),
        ],
        manual: vec![source(
            "HOOK-test",
            AssetKind::Hook,
            "en/hooks/test.md",
            "# Hook procedure\n",
        )],
        deferred: Vec::new(),
        excluded: Vec::new(),
    }
}

#[test]
fn separate_and_combined_bundles_share_native_paths_and_preserve_inert_references() {
    let expected = [
        ("codex", "AGENTS.md", ".agents/skills/test/SKILL.md"),
        ("claude", "CLAUDE.md", ".claude/skills/test/SKILL.md"),
        ("antigravity", "AGENTS.md", ".agents/skills/test/SKILL.md"),
    ];
    for (agent, entry, skill) in expected {
        let agents = targets(agent).unwrap();
        let bundle = render::bundle(&selection(), &agents, None).unwrap();
        assert_eq!(bundle.files[entry].owners, vec![agent.to_owned()]);
        let instructions = std::str::from_utf8(&bundle.files[entry].bytes).unwrap();
        assert!(instructions.contains(
            "\n\nEnglish policy and task authority come from the cirnet-rules Git repository.\n\
             Use `cnr task context` with the approved task ID to load its exact scope.\n\
             Check exact file grants before editing and record verification with `cnr task verify`.\n\n"
        ));
        let rendered = std::str::from_utf8(&bundle.files[skill].bytes).unwrap();
        assert!(rendered.starts_with("---\nname: test\ndescription: \"Fixture workflow\"\n---\n"));
        assert!(rendered.contains("[rule](../../../docs/cirnetrules/en/rules/test.md)"));
        assert_eq!(
            bundle.manifest.capabilities[agent]["native_discovery"],
            "NOT_RUN"
        );
        assert!(
            bundle
                .files
                .contains_key("docs/cirnetrules/en/hooks/test.md")
        );
        assert!(
            !bundle
                .files
                .contains_key(&skill.replace("/test/", "/executable/"))
        );
    }
    let agents = targets("antigravity,claude,codex").unwrap();
    let one = render::bundle(&selection(), &agents, None).unwrap();
    let two = render::bundle(
        &selection(),
        &targets("codex,claude,antigravity").unwrap(),
        None,
    )
    .unwrap();
    assert_eq!(one.manifest_bytes().unwrap(), two.manifest_bytes().unwrap());
    assert_eq!(
        one.files["AGENTS.md"].owners,
        vec!["antigravity".to_owned(), "codex".to_owned()]
    );
    assert_eq!(one.files["CLAUDE.md"].bytes, b"@AGENTS.md\n");
    assert_eq!(one.files.len(), 8);
}

#[test]
fn unsafe_paths_and_case_collisions_fail_closed() {
    for path in [
        "../AGENTS.md",
        "/AGENTS.md",
        "C:/AGENTS.md",
        "//host/share",
        "docs/CON.md",
        "docs/LPT1.md",
    ] {
        assert!(paths::validate(path).is_err(), "{path}");
    }
    assert!(paths::validate_names(["docs/Some/a.md", "docs/some/b.md"]).is_err());
    assert!(paths::validate_names(["docs/Some", "docs/Some/a.md"]).is_err());
}

#[test]
fn preview_never_adopts_user_files_and_atomic_install_is_digest_guarded() {
    let root = crate::test_support::temporary_directory("cirnet-agent-config").unwrap();
    let target = root.canonicalize().unwrap();
    {
        let bundle = render::bundle(&selection(), &targets("codex").unwrap(), None).unwrap();
        let entry = "AGENTS.md";
        let first = paths::observe(&target, entry).unwrap();
        assert!(first.sha256.is_none());
        fs::write(target.join(entry), b"user-owned").unwrap();
        let different = paths::observe(&target, entry).unwrap();
        assert_ne!(first, different);
        let bytes = &bundle.files[entry].bytes;
        assert!(paths::replace(&target, entry, &first, Some(bytes)).is_err());
        assert_eq!(fs::read(target.join(entry)).unwrap(), b"user-owned");
        let interrupted =
            paths::replace_with_intent(&target, entry, &different, Some(bytes), |intended| {
                assert_eq!(intended.sha256, Some(docs::sha256(bytes)));
                assert_eq!(fs::read(target.join(entry)).unwrap(), b"user-owned");
                Err("fixture journal unavailable".into())
            });
        assert!(interrupted.is_err());
        assert_eq!(fs::read_dir(&target).unwrap().count(), 1);
        assert_eq!(paths::observe(&target, entry).unwrap(), different);
        paths::replace(&target, entry, &different, Some(bytes)).unwrap();
        let installed = paths::observe(&target, entry).unwrap();
        assert_eq!(installed.sha256, Some(docs::sha256(bytes)));
        paths::replace(&target, entry, &installed, None).unwrap();
        assert!(!target.join(entry).exists());
    }
    assert_eq!(target, root.canonicalize().unwrap());
    assert!(
        target
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("cirnet-agent-config")
    );
    fs::remove_dir_all(target).unwrap();
}

struct GitFixture {
    root: std::path::PathBuf,
    governance: std::path::PathBuf,
    product: std::path::PathBuf,
    registry: Value,
}

impl GitFixture {
    fn new() -> Self {
        let root = crate::test_support::temporary_directory("cirnet-agent-config-git").unwrap();
        let governance = root.join("governance");
        let product = root.join("product");
        fs::create_dir(&governance).unwrap();
        fs::create_dir(&product).unwrap();
        let registry = json!({
            "repositories":{"cirnet-rules":{"classification":"supporting"},"product":{"classification":"active","ownership":"Fixture ownership."}},
            "control_plane":{"entrypoint":"en/AGENTS.md","skills":{"WORK":"en/skills/work/SKILL.md"}},
            "context":{"tasks":{"implementation":{"profile":"en/profiles/developer.md","skill":"en/skills/work/SKILL.md"}}},
            "assets":{"schema":1,"entries":[
                {"id":"AGENT-ENTRY","kind":"policy","path":"en/AGENTS.md","revision":1,"status":"ACTIVE","depends_on":["RULE-fixture"]},
                {"id":"RULE-fixture","kind":"rule","path":"en/rules/fixture.md","revision":1,"status":"ACTIVE","depends_on":[]},
                {"id":"WORK","kind":"skill","path":"en/skills/work/SKILL.md","revision":1,"status":"ACTIVE","depends_on":["RULE-fixture"]},
                {"id":"ROLE-developer","kind":"profile","path":"en/profiles/developer.md","revision":1,"status":"ACTIVE","depends_on":[]}
            ]}
        });
        for (path, content) in [
            (".gitignore", "/.cirnetrules/\n"),
            ("en/AGENTS.md", "# Fixture authority\n"),
            ("en/rules/fixture.md", "# Fixture rule\n"),
            (
                "en/skills/work/SKILL.md",
                "---\nname: work\ndescription: Fixture skill\n---\n\nUse the [fixture rule](../../rules/fixture.md).\n",
            ),
            ("en/profiles/developer.md", "# Fixture developer\n"),
            (
                "en/specs/product/config/spec.md",
                "# Fixture specification\n",
            ),
            ("en/plans/product/config/plan.md", "# Fixture plan\n"),
        ] {
            let file = governance.join(path);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, content).unwrap();
        }
        fs::write(
            governance.join("governance.yaml"),
            serde_json::to_vec(&registry).unwrap(),
        )
        .unwrap();
        fs::write(product.join("README.md"), b"# Product fixture\n").unwrap();
        for checkout in [&governance, &product] {
            git(checkout, &["init", "-q"]);
            git(checkout, &["add", "--", "."]);
            git(checkout, &["commit", "-qm", "fixture"]);
        }
        crate::workspace::run(
            "workspace-bind",
            &governance,
            &options(&[
                ("repo", "product"),
                ("path", "product"),
                ("actor", "operator"),
            ]),
            &registry,
        )
        .unwrap();
        Self {
            root,
            governance,
            product,
            registry,
        }
    }

    fn bundle(&self) -> Bundle {
        generate(
            &self.governance,
            None,
            &self.registry,
            "product",
            None,
            None,
            &targets("codex,claude,antigravity").unwrap(),
        )
        .unwrap()
    }

    fn grant(&self, paths: &[&str]) {
        let descriptor = json!({
            "id":"config","request":"Install project instructions","objective":"Verify the reviewed exporter",
            "non_goals":["Global configuration"],"primary_repo":"product","hints":[],
            "read_scope":{"product":paths},"write_scope":{"product":paths},
            "verification_scope":{},"documentation_scope":{},"risk":"normal","role":"implementer",
            "mode":"WORK","owner":"worker","authorization":{"basis":"Fixture request","owner":"operator","actions":["read","write"]},
            "acceptance":["Check fixture configuration"],"commands":[],"docs_targets":[],
            "docs_not_required":"Isolated configuration fixture","spec":"en/specs/product/config/spec.md","plan":"en/plans/product/config/plan.md"
        });
        let proposal = self.governance.join("proposal.json");
        fs::write(&proposal, serde_json::to_vec(&descriptor).unwrap()).unwrap();
        crate::tasks::run(
            "task-create",
            &self.governance,
            &options(&[("descriptor", "proposal.json")]),
            &self.registry,
        )
        .unwrap();
        fs::remove_file(proposal).unwrap();
        for (revision, to, actor) in [("1", "READY", "operator"), ("2", "IN_PROGRESS", "worker")] {
            crate::tasks::run(
                "task-transition",
                &self.governance,
                &options(&[
                    ("task", "config"),
                    ("revision", revision),
                    ("to", to),
                    ("actor", actor),
                ]),
                &self.registry,
            )
            .unwrap();
        }
    }
}

impl Drop for GitFixture {
    fn drop(&mut self) {
        if self.root.canonicalize().ok().as_ref() == Some(&self.root)
            && self
                .root
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("cirnet-agent-config-git-")
        {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn options(args: &[(&str, &str)]) -> BTreeMap<String, String> {
    args.iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn git(path: &Path, args: &[&str]) {
    let outcome = std::process::Command::new("git")
        .current_dir(path)
        .args([
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        outcome.status.success(),
        "{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
}

#[test]
fn real_fixture_grant_owns_only_installed_paths_and_can_rollback() {
    let fixture = GitFixture::new();
    let bundle = fixture.bundle();
    fixture.grant(&bundle.files.keys().map(String::as_str).collect::<Vec<_>>());
    let dir = fixture.root.join("bundle");
    assert_eq!(export(&dir, &bundle).unwrap()["state"], "EXPORTED");
    assert_eq!(export(&dir, &bundle).unwrap()["state"], "UNCHANGED");
    let args = options(&[
        ("bundle", dir.to_str().unwrap()),
        ("target", fixture.product.to_str().unwrap()),
        ("dry-run", "true"),
    ]);
    let sync = |args: &BTreeMap<String, String>| {
        sync::sync(&fixture.governance, None, &fixture.registry, args)
    };
    let dry = sync(&args).unwrap();
    assert_eq!(dry["state"], "DRY_RUN");
    assert!(!fixture.product.join("AGENTS.md").exists());
    assert!(
        !fixture
            .governance
            .join(".cirnetrules/agent-config")
            .exists()
    );
    fs::write(fixture.product.join("AGENTS.md"), b"User instructions\n").unwrap();
    let conflict = sync(&args).unwrap();
    assert_eq!(conflict["state"], "CONFLICT");
    assert!(
        conflict["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["action"] == "CONFLICT" && row["patch"]["replacement"].is_string())
    );
    let mut apply = args.clone();
    apply.remove("dry-run");
    apply.extend(options(&[
        ("yes", "true"),
        ("expect", dry["plan_sha256"].as_str().unwrap()),
        ("task", "config"),
        ("actor", "worker"),
    ]));
    assert!(sync(&apply).unwrap_err().contains("preview changed"));
    assert_eq!(
        fs::read(fixture.product.join("AGENTS.md")).unwrap(),
        b"User instructions\n"
    );
    fs::remove_file(fixture.product.join("AGENTS.md")).unwrap();
    apply.insert("actor".into(), "impostor".into());
    assert!(sync(&apply).unwrap_err().contains("declared owner"));
    assert!(!fixture.product.join("AGENTS.md").exists());
    apply.insert("actor".into(), "worker".into());
    let legacy = fixture.governance.join(".openrules");
    fs::create_dir(&legacy).unwrap();
    assert!(sync(&apply).unwrap_err().contains("reviewed recovery"));
    assert!(!fixture.product.join("AGENTS.md").exists());
    fs::remove_dir(legacy).unwrap();
    let applied = sync(&apply).unwrap();
    assert_eq!(applied["state"], "APPLIED", "{applied}");
    assert_eq!(sync(&apply).unwrap()["state"], "UNCHANGED");
    let receipt = applied["receipt"].as_str().unwrap();
    assert_eq!(
        audit(&fixture.governance, receipt).unwrap()["result"],
        "APPLIED"
    );
    let unchanged = sync(&args).unwrap();
    assert_eq!(unchanged["state"], "UNCHANGED");
    apply.insert(
        "expect".into(),
        unchanged["plan_sha256"].as_str().unwrap().into(),
    );
    assert_eq!(sync(&apply).unwrap()["state"], "UNCHANGED");
    let rollback = |args: &BTreeMap<String, String>| {
        sync::rollback(&fixture.governance, None, &fixture.registry, args)
    };
    let mut reverse = options(&[
        ("target", fixture.product.to_str().unwrap()),
        ("receipt", receipt),
        ("dry-run", "true"),
    ]);
    let native = fixture.product.join("CLAUDE.md");
    let expected = fs::read(&native).unwrap();
    fs::write(&native, b"Local edit\n").unwrap();
    assert_eq!(rollback(&reverse).unwrap()["state"], "CONFLICT");
    fs::write(&native, &expected).unwrap();
    let reverse_dry = rollback(&reverse).unwrap();
    assert_eq!(reverse_dry["state"], "DRY_RUN", "{reverse_dry}");
    reverse.remove("dry-run");
    reverse.extend(options(&[
        ("yes", "true"),
        ("expect", reverse_dry["plan_sha256"].as_str().unwrap()),
        ("task", "config"),
        ("actor", "worker"),
    ]));
    assert_eq!(rollback(&reverse).unwrap()["state"], "ROLLED_BACK");
    assert!(!native.exists());
    assert!(fixture.product.join("README.md").exists());
    assert_eq!(
        audit(&fixture.governance, receipt).unwrap()["result"],
        "ROLLED_BACK"
    );
    let source = fixture.governance.join("en/rules/fixture.md");
    fs::write(&source, b"# Changed rule\n").unwrap();
    assert!(
        checked(&fixture.governance, None, &fixture.registry, &dir)
            .err()
            .unwrap()
            .contains("authority")
    );
    fs::write(&source, b"# Fixture rule\n").unwrap();
    fs::write(dir.join("AGENTS.md"), b"Tampered payload\n").unwrap();
    assert!(checked(&fixture.governance, None, &fixture.registry, &dir).is_err());
}

#[test]
fn task_bound_bundle_requires_current_plan_spec_and_approval() {
    let fixture = GitFixture::new();
    let plan = fixture.governance.join("en/plans/product/config/plan.md");
    let spec = fixture.governance.join("en/specs/product/config/spec.md");
    fs::write(&plan, b"# Initial approved plan\n").unwrap();
    fixture.grant(&["AGENTS.md"]);
    let agents = targets("codex").unwrap();
    let generate = || {
        generate(
            &fixture.governance,
            None,
            &fixture.registry,
            "product",
            Some("config"),
            None,
            &agents,
        )
        .unwrap()
    };
    let first = generate();
    let dir = fixture.root.join("task-bundle");
    export(&dir, &first).unwrap();
    let check = || checked(&fixture.governance, None, &fixture.registry, &dir);
    assert!(check().is_ok());

    let task = |command: &str, args: &[(&str, &str)]| {
        let mut args = options(args);
        args.insert("task".into(), "config".into());
        crate::tasks::run(command, &fixture.governance, &args, &fixture.registry).unwrap()
    };
    fs::write(
        fixture
            .governance
            .join(".cirnetrules/fixture-checkpoint.json"),
        br#"{"next_action":"Review the configuration"}"#,
    )
    .unwrap();
    task(
        "task-checkpoint",
        &[
            ("revision", "3"),
            ("actor", "worker"),
            ("checkpoint", ".cirnetrules/fixture-checkpoint.json"),
        ],
    );
    assert!(
        check().is_ok(),
        "Checkpoint updates preserve task provenance"
    );

    let new_plan = b"# Revised approved plan\n";
    let new_spec = b"# Revised approved specification\n";
    fs::write(&plan, new_plan).unwrap();
    fs::write(&spec, new_spec).unwrap();
    let blocked = task(
        "task-transition",
        &[
            ("revision", "4"),
            ("actor", "worker"),
            ("to", "BLOCKED"),
            ("reason", "Plan and spec revised"),
        ],
    );
    fs::write(
        fixture
            .governance
            .join(".cirnetrules/fixture-revision.json"),
        serde_json::to_vec(&blocked["task"]["descriptor"]).unwrap(),
    )
    .unwrap();
    task(
        "task-revise",
        &[
            ("revision", "5"),
            ("actor", "operator"),
            ("descriptor", ".cirnetrules/fixture-revision.json"),
        ],
    );
    task(
        "task-transition",
        &[("revision", "6"), ("actor", "operator"), ("to", "READY")],
    );

    let fresh = generate();
    assert_eq!(first.manifest.source_commit, fresh.manifest.source_commit);
    assert!(first.manifest.source_dirty && fresh.manifest.source_dirty);
    assert_eq!(
        first.manifest.authority_sha256,
        fresh.manifest.authority_sha256
    );
    let provenance = fresh.manifest.task_provenance.as_ref().unwrap();
    assert_eq!(provenance.spec_sha256, docs::sha256(new_spec));
    assert_eq!(provenance.plan_sha256, docs::sha256(new_plan));
    assert_eq!(provenance.approval_epoch, 2);
    assert!(check().err().unwrap().contains("Export a fresh bundle"));
}

#[cfg(unix)]
#[test]
fn symlinked_output_ancestry_is_rejected() {
    use std::os::unix::fs::symlink;
    let fixture = GitFixture::new();
    symlink(&fixture.governance, fixture.product.join(".agents")).unwrap();
    assert!(paths::observe(&fixture.product, ".agents/skills/work/SKILL.md").is_err());
    assert!(!fixture.governance.join("skills").exists());
}
