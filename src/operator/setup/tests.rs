use super::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const REPO: &str = "cirnet-interface";
const TASK: &str = "setup-fixture";
const SPEC: &str = "en/specs/cirnet-interface/setup-fixture/spec.md";
const PLAN: &str = "en/plans/cirnet-interface/setup-fixture/plan.md";

struct Fixture {
    workspace: PathBuf,
    root: PathBuf,
    input: SetupInput,
}

impl Fixture {
    fn new() -> Self {
        let workspace = crate::test_support::temporary_directory("cnr-setup-proposal").unwrap();
        let root = workspace.join("cirnet-rules");
        let product = workspace.join(REPO);
        let target = workspace.join("interface-setup");
        let fixture = Self {
            input: SetupInput {
                scope: Scope {
                    governance_root: Some(root.clone()),
                    workspace: Some(workspace.clone()),
                    target: Some(target.clone()),
                    repo: Some(REPO.into()),
                    binding: Some("fixture".into()),
                    agent: Some("codex".into()),
                    bundle: Some(workspace.join("bundle")),
                    ..Default::default()
                },
                actor: Some("fixture-owner".into()),
                proposal: Some(TASK.into()),
                spec: Some(SPEC.into()),
                plan: Some(PLAN.into()),
                ..Default::default()
            },
            workspace,
            root,
        };
        let source = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut registry: Value =
            serde_json::from_str(include_str!("../../../governance.yaml")).unwrap();
        for path in [
            ".gitignore",
            "VERSION",
            "AGENTS.md",
            "en/schemas/governance.schema.json",
            "en/schemas/cirnetrules-policy.schema.json",
        ]
        .into_iter()
        .chain(
            registry["assets"]["entries"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|asset| asset["status"] == "ACTIVE")
                .map(|asset| asset["path"].as_str().unwrap()),
        ) {
            let destination = fixture.root.join(path);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::copy(source.join(path), destination).unwrap();
        }
        let repo = registry["repositories"][REPO].as_object_mut().unwrap();
        for key in ["snapshot", "plan", "spec", "remote"] {
            repo.remove(key);
        }
        fs::write(
            fixture.root.join("governance.yaml"),
            serde_json::to_vec_pretty(&registry).unwrap(),
        )
        .unwrap();
        for path in [SPEC, PLAN] {
            let path = fixture.root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "# Review generated setup files\n").unwrap();
        }
        fs::create_dir(&product).unwrap();
        fs::write(product.join("README.md"), "# Interface fixture\n").unwrap();
        for (path, origin) in [
            (
                &fixture.root,
                "https://github.com/cirnetlab/cirnet-rules.git",
            ),
            (
                &product,
                "https://github.com/cirnetlab/cirnet-interface.git",
            ),
        ] {
            git(path, &["init", "--quiet"]);
            git(path, &["remote", "add", "origin", origin]);
            git(path, &["add", "--", "."]);
            git(path, &["commit", "--quiet", "-m", "Initialize fixture"]);
        }
        // Keep the bundle's source_dirty observation stable while testing
        // independent changes to the proposal's spec and plan digests.
        fs::write(fixture.root.join("fixture-note.txt"), "Fixture note\n").unwrap();
        let context = Context::open(&fixture.input.scope).unwrap();
        let mut args = context.arguments().unwrap();
        args.extend([
            ("repo".into(), REPO.into()),
            ("path".into(), REPO.into()),
            ("actor".into(), "fixture-owner".into()),
        ]);
        workspace::run("workspace-bind", &context.root, &args, &context.registry).unwrap();
        git(
            &product,
            &["worktree", "add", "--detach", "../interface-setup", "HEAD"],
        );
        args.insert("path".into(), "interface-setup".into());
        args.insert("binding".into(), "fixture".into());
        workspace::run(
            "workspace-binding-add",
            &context.root,
            &args,
            &context.registry,
        )
        .unwrap();
        let export = preview_setup(SetupAction::ExportInstructions, &fixture.input).unwrap();
        assert_eq!(perform_setup(export, false).unwrap()["state"], "EXPORTED");
        fixture
    }

    fn proposal(&self) -> SetupPreview {
        preview_setup(SetupAction::ProposeTask, &self.input)
            .unwrap_or_else(|error| panic!("production setup proposal preview failed: {error}"))
    }

    fn assert_no_task(&self) {
        assert!(
            !self
                .root
                .join(".cirnetrules/tasks/setup-fixture.json")
                .exists()
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if self.workspace.canonicalize().ok().as_ref() == Some(&self.workspace) {
            let _ = fs::remove_dir_all(&self.workspace);
        }
    }
}

fn git(path: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(path)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args([
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "-c",
            "core.hooksPath=.git/disabled-hooks",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn proposal_previews_without_writes_and_creates_only_a_proposed_task() {
    let fixture = Fixture::new();
    let preview = fixture.proposal();
    assert_eq!(preview.report["state"], "DRY_RUN");
    let descriptor = &preview.report["descriptor"];
    assert_eq!(descriptor["authorization"]["actions"], json!([]));
    assert_eq!(descriptor["docs_targets"], json!([]));
    assert!(descriptor["docs_not_required"].is_string());
    assert_eq!(descriptor["bindings"][REPO], "fixture");
    let scope = descriptor["write_scope"][REPO].as_array().unwrap();
    assert!(scope.contains(&json!("AGENTS.md")));
    assert!(scope.contains(&json!(".codex/config.toml")));
    fixture.assert_no_task();

    let duplicate = fixture.proposal();
    let applied = perform_setup(preview, false).unwrap();
    assert_eq!(applied["state"], "PROPOSED");
    assert_eq!(applied["revision"], 1);
    let saved = tasks::load_current(&fixture.root, TASK).unwrap();
    assert_eq!(saved["state"], "PROPOSED");
    assert_eq!(saved["descriptor"]["authorization"]["actions"], json!([]));
    assert!(tasks::authorize(&saved, "write").is_err());
    assert!(
        !fixture
            .input
            .scope
            .target
            .as_ref()
            .unwrap()
            .join("AGENTS.md")
            .exists()
    );
    assert!(
        perform_setup(duplicate, false)
            .unwrap_err()
            .contains("already in use")
    );
}

#[test]
fn proposal_rejects_changes_to_reviewed_inputs_and_legacy_store() {
    let fixture = Fixture::new();
    for (path, expected) in [
        (fixture.root.join(SPEC), "Task proposal changed"),
        (fixture.root.join(PLAN), "Task proposal changed"),
        (
            fixture
                .input
                .scope
                .bundle
                .as_ref()
                .unwrap()
                .join("AGENTS.md"),
            "bundle",
        ),
    ] {
        let preview = fixture.proposal();
        let before = fs::read(&path).unwrap();
        let after = [before.as_slice(), b"\n# Changed after review\n"].concat();
        fs::write(&path, after).unwrap();
        let error = perform_setup(preview, false).unwrap_err();
        assert!(
            error
                .to_ascii_lowercase()
                .contains(&expected.to_ascii_lowercase()),
            "{error}"
        );
        fixture.assert_no_task();
        fs::write(path, before).unwrap();
    }
    let preview = fixture.proposal();
    fs::create_dir(fixture.root.join(concat!(".open", "rules"))).unwrap();
    assert!(
        perform_setup(preview, false)
            .unwrap_err()
            .contains("current task store")
    );
    fixture.assert_no_task();
}

#[test]
fn proposal_validator_requires_exactly_one_documentation_disposition() {
    let fixture = Fixture::new();
    let context = Context::open(&fixture.input.scope).unwrap();
    let preview = fixture.proposal();
    let mut descriptor = preview.report["descriptor"].clone();
    descriptor
        .as_object_mut()
        .unwrap()
        .remove("docs_not_required");
    assert!(
        tasks::preview_proposal(&context.root, &descriptor, &context.registry)
            .unwrap_err()
            .contains("Specify documentation targets or a reason")
    );

    let path = "docs/en/setup.md";
    descriptor["documentation_scope"] = json!({(REPO): [path]});
    descriptor["docs_targets"] = json!([{"repo": REPO, "path": path, "locale": "en"}]);
    assert!(tasks::preview_proposal(&context.root, &descriptor, &context.registry).is_ok());
    descriptor["docs_not_required"] = json!("A conflicting disposition");
    assert!(
        tasks::preview_proposal(&context.root, &descriptor, &context.registry)
            .unwrap_err()
            .contains("Specify documentation targets or a reason")
    );
    fixture.assert_no_task();
}
