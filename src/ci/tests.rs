use super::*;

fn git(path: &Path, args: &[&str]) -> String {
    let result = std::process::Command::new("git")
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
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).unwrap().trim().into()
}

fn init(path: &Path, repo: &str) {
    fs::create_dir(path).unwrap();
    git(path, &["init", "--quiet"]);
    git(
        path,
        &[
            "remote",
            "add",
            "origin",
            &format!("https://github.com/cirnetlab/{repo}.git"),
        ],
    );
}

fn commit(path: &Path, message: &str) -> String {
    git(path, &["add", "--all"]);
    git(path, &["commit", "--quiet", "-m", message]);
    git(path, &["rev-parse", "HEAD"])
}

#[test]
fn portable_gate_uses_pinned_authority_and_observes_real_consumer_artifact() {
    let base = crate::test_support::temporary_directory("cirnet-portable-ci").unwrap();
    let authority = base.join("authority");
    let candidate = base.join("candidate");
    init(&authority, "cirnet-rules");
    init(&candidate, "cirnet-interface");
    let producer = base.join("producer");
    init(&producer, "cirnet-contracts");
    let source_path = "en/rule.md";
    fs::create_dir(authority.join("en")).unwrap();
    fs::write(authority.join(source_path), b"# Trusted rule\n").unwrap();
    fs::write(producer.join("artifact.json"), br#"{"generated":1}"#).unwrap();
    commit(&producer, "Produce generated contract");

    let path = "generated-contract.json";
    let registry = json!({
        "repositories":{"cirnet-interface":{}},
        "control_plane":{"commands":{"check-contract":{
            "repository":"cirnet-interface", "mode":"VERIFY","program":"check-json","args":[],"paths":[path]
        }}}
    });
    fs::write(
        authority.join("governance.yaml"),
        serde_json::to_vec(&registry).unwrap(),
    )
    .unwrap();
    let authority_commit = commit(&authority, "Trusted authority");
    fs::write(candidate.join(path), b"{}\n").unwrap();
    let base_commit = commit(&candidate, "Consumer base");
    let artifact = fs::read(producer.join("artifact.json")).unwrap();
    fs::write(candidate.join(path), &artifact).unwrap();
    let commit_id = commit(&candidate, "Consume generated artifact");
    let authority = authority.canonicalize().unwrap();
    let candidate = candidate.canonicalize().unwrap();
    let sha256 = docs::sha256(&artifact);
    let mut bundle = Bundle {
        schema: 1,
        task: "consumer".into(),
        repository: "cirnet-interface".into(),
        base_commit,
        authority_commit: authority_commit.clone(),
        registry_sha256: registry_digest(&authority).unwrap(),
        authority_sha256: "a".repeat(64),
        authority_sources: BTreeMap::from([(
            source_path.into(),
            docs::sha256(b"# Trusted rule\n"),
        )]),
        descriptor_sha256: "b".repeat(64),
        approval_epoch: 1,
        submission_sha256: "c".repeat(64),
        source_sha256: "d".repeat(64),
        write_scope: BTreeSet::from([path.into()]),
        verification_scope: BTreeSet::from([path.into()]),
        changed: BTreeSet::from([path.into()]),
        files: BTreeMap::from([(
            path.into(),
            FileEvidence {
                sha256: Some(sha256.clone()),
                executable: false,
            },
        )]),
        commands: BTreeSet::from(["check-contract".into()]),
        budget: BTreeMap::new(),
        dependencies: vec![Dependency {
            task_id: "producer".into(),
            revision: 4,
            approval_epoch: 2,
            source_sha256: "e".repeat(64),
            evidence_sha256: "f".repeat(64),
            outputs: vec![Artifact {
                repository: "cirnet-contracts".into(),
                path: "artifact.json".into(),
                sha256: Some(sha256.clone()),
            }],
            selected_inputs: vec![Artifact {
                repository: "cirnet-interface".into(),
                path: path.into(),
                sha256: Some(sha256.clone()),
            }],
        }],
    };
    validate(&bundle).unwrap();
    let digest = |bundle: &Bundle| docs::sha256(&serde_json::to_vec(bundle).unwrap());
    let verify = |bundle: &Bundle, expected: &str| {
        gate(
            &authority,
            &registry,
            bundle,
            &digest(bundle),
            expected,
            &authority_commit,
            &candidate,
        )
    };
    let verified = verify(&bundle, &digest(&bundle)).unwrap();
    assert_eq!(verified["status"], "PASS");
    assert_eq!(verified["candidate_commit"], commit_id);
    assert_eq!(verified["observed_inputs"][path], sha256);
    assert_eq!(
        verify(&bundle, &"0".repeat(64)).unwrap()["status"],
        "BLOCKED"
    );

    bundle.dependencies[0].selected_inputs[0].sha256 = Some("0".repeat(64));
    assert_eq!(
        verify(&bundle, &digest(&bundle)).unwrap()["status"],
        "BLOCKED"
    );
    bundle.dependencies[0].selected_inputs[0].sha256 = Some(sha256);
    fs::write(authority.join("governance.yaml"), b"{}").unwrap();
    assert_eq!(
        verify(&bundle, &digest(&bundle)).unwrap()["status"],
        "BLOCKED"
    );
    fs::write(
        authority.join("governance.yaml"),
        serde_json::to_vec(&registry).unwrap(),
    )
    .unwrap();
    fs::write(candidate.join("unauthorized.txt"), b"changed\n").unwrap();
    commit(&candidate, "Out of scope");
    assert!(
        verify(&bundle, &digest(&bundle))
            .unwrap_err()
            .contains("changed paths")
    );
    fs::remove_dir_all(base).unwrap();
}
