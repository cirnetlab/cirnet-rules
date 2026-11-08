use super::*;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha2::Sha512;
use std::fs;

const TRAIN: &str = "0.2.4";
const FAMILY: &str = "0.2";

#[test]
fn exact_training_tuple_and_compatibility_allowlist_are_independent() {
    let mut manifest = json!({
        "releaseTrain": "0.2.4",
        "family": "0.2",
        "repositories": {"cirnet-remote": {"version": "0.2.4"}},
        "contracts": {"http": {"revision": "0.2.0"}},
        "rollback": {"automaticDowngrade": false}
    });
    let reviewed = selection_digest(&manifest).unwrap();
    let matrix = json!({
        "schemaVersion": 1,
        "releaseTrain": "0.2.4",
        "family": "0.2",
        "allowedSelections": [reviewed]
    });
    assert!(check_matrix(&matrix, &manifest, TRAIN, FAMILY).is_ok());
    manifest["contracts"]["http"]["revision"] = json!("0.2.1");
    assert!(check_matrix(&matrix, &manifest, TRAIN, FAMILY).is_err());
    manifest["releaseTrain"] = json!("0.2.5");
    assert!(check_matrix(&matrix, &manifest, TRAIN, FAMILY).is_err());
    manifest["releaseTrain"] = json!("0.2.4");
    manifest["family"] = json!(0.2);
    assert!(check_matrix(&matrix, &manifest, TRAIN, FAMILY).is_err());
}

#[test]
fn future_train_and_family_are_configurable_but_must_be_approved_exactly() {
    let registry = json!({"baseline":{"release_train":"0.3.7","family":"0.3"}});
    let (train, family) = baseline(&registry).unwrap();
    let mut manifest = json!({"releaseTrain":train,"family":family,"repositories":{},"contracts":{},"rollback":{}});
    let selection = selection_digest(&manifest).unwrap();
    let matrix = json!({"schemaVersion":1,"releaseTrain":train,"family":family,"allowedSelections":[selection]});
    assert!(check_matrix(&matrix, &manifest, train, family).is_ok());
    assert!(check_matrix(&matrix, &manifest, TRAIN, FAMILY).is_err());
    manifest["contracts"] = json!({"http":{"revision":"0.3.7"}});
    assert!(check_matrix(&matrix, &manifest, train, family).is_err());
    let mut grants = json!({});
    for name in REPOS {
        grants[name] = json!(inventory::release_inputs(name));
    }
    assert!(release_scope_matches(&grants));
    grants[REMOTE] = json!(["README.md"]);
    assert!(!release_scope_matches(&grants));
}

#[test]
fn scope_keeps_python_tests_and_historical_fixtures_distinct() {
    for path in [
        "tests/probe.py",
        "native/benchmarks/perf.py",
        "docs/history/run.py",
        "conftest.py",
    ] {
        assert!(!unreviewed_python(path), "{path}");
    }
    for path in [
        "services/api/app.py",
        "tools/bootstrap.py",
        "scripts/generate_sdk.py",
    ] {
        assert!(unreviewed_python(path), "{path}");
    }
}

#[test]
fn metadata_rejects_an_unavailable_or_unmeasured_capability_claim() {
    let clean = json!({"simulation": {"implemented": true, "validated": true, "available": true}});
    assert!(check_capabilities(&clean).is_ok());
    assert!(
        check_capabilities(&json!({
            "simulation": {"implemented": true, "validated": false, "available": true}
        }))
        .is_err()
    );
    assert!(
        check_capabilities(&json!({
            "physical": {"implemented": true, "validated": true, "available": true}
        }))
        .is_err()
    );
}

#[test]
fn filenames_cannot_select_paths_or_reserved_windows_devices() {
    for value in [
        "../../sample.zip",
        "other\\sample.zip",
        "C:sample",
        "NUL.zip",
        ".git",
        "abc.",
    ] {
        assert!(!portable_filename(value), "{value}");
    }
    assert!(portable_filename("cirnet-remote-0.2.4-windows-x64.zip"));
}

#[test]
fn gate_evidence_has_exact_source_governance_selection_and_type() {
    let commit = "a".repeat(40);
    let governance = "b".repeat(40);
    let selection = "c".repeat(64);
    let mut evidence = json!({
        "schemaVersion": 1, "gate": "clean_windows", "repository": "cirnet-remote",
        "sourceCommit": commit, "governanceCommit": governance,
        "selectionSha256": selection, "command": ["cirnet-verify", "--isolated"],
        "toolVersions": {"Windows": "11 24H2"}, "platform": "windows-x64-msvc",
        "recordedAt": "2026-11-08T00:00:00Z", "exitCode": 0, "result": "PASS",
        "evidenceType": "clean_install", "resultFile": "clean-windows.json",
        "resultSha256": "d".repeat(64), "limitations": "No physical HIL"
    });
    let gate = Gate {
        id: "clean_windows",
        repo: "cirnet-remote",
        kind: "clean_install",
    };
    assert!(check_evidence(&evidence, &gate, &commit, &governance, &selection).is_ok());
    evidence["governanceCommit"] = json!("0".repeat(40));
    assert!(check_evidence(&evidence, &gate, &commit, &governance, &selection).is_err());
    evidence["governanceCommit"] = json!(governance);
    evidence["evidenceType"] = json!("simulation");
    assert!(check_evidence(&evidence, &gate, &commit, &governance, &selection).is_err());
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let parent = std::env::temp_dir().canonicalize().unwrap();
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = parent.join(format!(
            "cirnet-governance-test-{}-{id}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if self.0.canonicalize().ok().as_ref() == Some(&self.0)
            && self.0.parent() == std::env::temp_dir().canonicalize().ok().as_deref()
        {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

fn write(root: &Path, file: &str, bytes: impl AsRef<[u8]>) {
    let path = root.join(file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn write_json(root: &Path, file: &str, value: &Value) -> String {
    let bytes = serde_json::to_vec(value).unwrap();
    let sha = digest(&bytes);
    write(root, file, bytes);
    sha
}

fn fixture_git(repo: &Path, args: &[&str]) {
    let mut command = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key
            .to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("GIT_")
        {
            command.env_remove(key);
        }
    }
    let output = command
        .current_dir(repo)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "../fixture-git-config")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "fixture git failed: {:?} {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn fixture_commit(repo: &Path, subject: &str) {
    fixture_git(
        repo,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-qm",
            subject,
        ],
    );
}

#[test]
fn exact_local_release_requires_review_and_activation_and_rejects_repackaging() {
    let scratch = Scratch::new();
    let root = &scratch.0;
    write(root, "fixture-git-config", "");
    let governance = "1".repeat(40);
    let sdk_bytes = b"fixture SDK archive";
    let sdk_sha = digest(sdk_bytes);
    let sdk_sri = format!("sha512-{}", STANDARD.encode(Sha512::digest(sdk_bytes)));
    let sdk_filename = "cirnet-contracts-0.2.4-sdk.tgz";
    let sdk_path = format!("file:.artifacts/{sdk_filename}");
    let mut entries = Map::new();
    let mut registration = Map::new();
    let mut payloads = Vec::new();
    let mut contracts_commit = String::new();
    for name in REPOS {
        let repo = root.join(name);
        fs::create_dir(&repo).unwrap();
        fixture_git(&repo, &["init", "-q", "--template="]);
        fixture_git(
            &repo,
            &[
                "remote",
                "add",
                "origin",
                &format!("https://github.com/cirnetlab/{name}.git"),
            ],
        );
        let lockfile = match name {
            "cirnet-fpga" => {
                write(&repo, "VERSION", "0.2.4\n");
                write(
                    &repo,
                    "integration/toolchain-lock.json",
                    "{\"tag\":\"fixture\"}",
                );
                "integration/toolchain-lock.json"
            }
            "cirnet-simulator" => {
                write(
                    &repo,
                    "CMakeLists.txt",
                    "project(cirnet_simulator VERSION 0.2.4 LANGUAGES CXX)\n",
                );
                "CMakeLists.txt"
            }
            "cirnet-interface" => {
                write_json(
                    &repo,
                    "package.json",
                    &json!({"version": TRAIN, "dependencies": {"@cirnet/contracts": sdk_path}}),
                );
                write_json(
                    &repo,
                    "package-lock.json",
                    &json!({"version": TRAIN, "packages": {
                        "": {"version": TRAIN, "dependencies": {"@cirnet/contracts": sdk_path}},
                        "node_modules/@cirnet/contracts": {
                            "version": TRAIN, "resolved": sdk_path, "integrity": sdk_sri
                        }
                    }}),
                );
                write_json(
                    &repo,
                    "integration/contracts-artifacts.json",
                    &json!({"contractsVersion": TRAIN, "npm": {
                        "version": TRAIN, "filename": sdk_filename, "sha256": sdk_sha
                    }}),
                );
                "package-lock.json"
            }
            _ => {
                let crate_name = format!("{name}-native");
                let mut manifest =
                    format!("[package]\nname = \"{crate_name}\"\nversion = \"{TRAIN}\"\n");
                let mut lock =
                    format!("[[package]]\nname = \"{crate_name}\"\nversion = \"{TRAIN}\"\n");
                if name != "cirnet-contracts" {
                    for section in ["dependencies", "build-dependencies"] {
                        manifest += &format!(
                            "[{section}]\ncirnet-contracts-native = {{ git = \"https://github.com/cirnetlab/cirnet-contracts.git\", rev = \"{contracts_commit}\", version = \"={TRAIN}\" }}\n"
                        );
                    }
                    lock += &format!(
                        "[[package]]\nname = \"cirnet-contracts-native\"\nversion = \"{TRAIN}\"\nsource = \"git+https://github.com/cirnetlab/cirnet-contracts.git?rev={contracts_commit}#{contracts_commit}\"\n"
                    );
                }
                write(&repo, "Cargo.toml", manifest);
                write(&repo, "Cargo.lock", lock);
                "Cargo.lock"
            }
        };
        fixture_git(&repo, &["add", "-f", "--", "."]);
        fixture_commit(&repo, "Synthetic release source");
        let commit = git_text(&repo, &["rev-parse", "HEAD"]).unwrap();
        if name == "cirnet-contracts" {
            contracts_commit.clone_from(&commit);
        }
        let filename = format!("{name}-{TRAIN}-fixture.zip");
        let bytes = format!("synthetic input for {name}").into_bytes();
        let target = match name {
            "cirnet-fpga" => "portable-source",
            "cirnet-interface" => "web-static",
            _ => "x86_64-pc-windows-msvc",
        };
        let mut artifacts = vec![json!({
            "filename": filename, "kind": expected_kind(name), "sha256": digest(&bytes), "size": bytes.len()
        })];
        payloads.push((format!("artifacts/{name}/{filename}"), bytes));
        if name == "cirnet-contracts" {
            artifacts.push(json!({"filename": sdk_filename, "kind": "npm-sdk", "sha256": sdk_sha, "size": sdk_bytes.len()}));
            payloads.push((
                format!("artifacts/{name}/{sdk_filename}"),
                sdk_bytes.to_vec(),
            ));
        }
        entries.insert(name.to_owned(), json!({
            "version": TRAIN, "tag": format!("v{TRAIN}"), "commit": commit,
            "target": target, "toolchain": {"fixture": "1.0"}, "lockSha256": digest(&fs::read(repo.join(lockfile)).unwrap()),
            "capabilities": {"simulation": {"implemented": true, "validated": true, "available": true}},
            "artifacts": artifacts
        }));
        registration.insert(
            name.to_owned(),
            json!({"remote": format!("https://github.com/cirnetlab/{name}.git")}),
        );
    }
    let mut domains = Map::new();
    for domain in DOMAINS {
        let owner = match domain {
            "http" | "realtime" | "data_storage" | "release_manifest" => REMOTE,
            "simulator_ipc" => "cirnet-simulator",
            _ => "cirnet-contracts",
        };
        let bytes = format!("synthetic contract {domain}").into_bytes();
        domains.insert(domain.to_owned(), json!({"family": FAMILY, "revision": "0.2.0", "filename": "contract.json",
            "sha256": digest(&bytes), "repository": owner, "commit": entries[owner]["commit"], "encoding": "fixture"}));
        payloads.push((format!("contracts/{domain}/contract.json"), bytes));
    }
    let mut manifest = json!({
        "schemaVersion": 2, "releaseTrain": TRAIN, "family": FAMILY, "distribution": "candidate",
        "repositories": entries, "contracts": domains, "evidence": {},
        "rollback": {"automaticStartReplay": false, "matchingDatabaseBackupRequired": true,
            "matchingCaptureBackupRequired": true, "preserveQuarantine": true, "automaticDowngrade": false}
    });
    let selection = selection_digest(&manifest).unwrap();
    let remote = root.join(REMOTE);
    let matrix_sha = write_json(
        &remote,
        MATRIX_PATH,
        &json!({
            "schemaVersion": 1, "releaseTrain": TRAIN, "family": FAMILY, "allowedSelections": [selection]
        }),
    );
    manifest["compatibilityMatrixSha256"] = json!(matrix_sha);
    let evidence_root = root.join(format!("cirnet-release-evidence/{selection}"));
    for (path, bytes) in payloads {
        write(&evidence_root, &path, bytes);
    }
    let mut gates = Map::new();
    for gate in &GATES {
        let filename = format!("{}.json", gate.id);
        let result = format!("fixture result for {}", gate.id);
        write(
            &evidence_root,
            &format!("results/{filename}"),
            result.as_bytes(),
        );
        let evidence_sha = write_json(
            &evidence_root,
            &format!("gates/{filename}"),
            &json!({
                "schemaVersion": 1, "gate": gate.id, "repository": gate.repo,
                "sourceCommit": manifest["repositories"][gate.repo]["commit"], "governanceCommit": governance,
                "selectionSha256": selection, "command": ["synthetic-test"], "toolVersions": {"fixture": "1.0"},
                "platform": "fixture", "recordedAt": "2026-11-08T00:00:00Z", "exitCode": 0, "result": "PASS",
                "evidenceType": gate.kind, "resultFile": filename, "resultSha256": digest(result.as_bytes()),
                "limitations": "synthetic fixture only"
            }),
        );
        manifest["evidence"][gate.id] = json!({"filename": filename, "sha256": evidence_sha});
        gates.insert(
            gate.id.to_owned(),
            json!({"repository": gate.repo, "evidence_type": gate.kind}),
        );
    }
    write_json(&remote, RELEASE_PATH, &manifest);
    let mut registry = json!({
        "baseline": {"release_train": TRAIN, "family": FAMILY, "activation": "NOT_ACTIVATED"},
        "authority": {"release_set": {"repository": REMOTE, "path": RELEASE_PATH}}, "repositories": registration,
        "release_policy": {"approved_matrix_sha256": matrix_sha, "required_domains": DOMAINS, "required_gates": gates},
        "_governance_commit": governance, "_governance_clean": true, "_governance_origin_verified": true
    });
    let unapproved = check_release(root, &registry).unwrap();
    assert_eq!(unapproved["status"], "BLOCKED", "{unapproved}");
    assert_eq!(unapproved["integrity_validation"], "PASS", "{unapproved}");
    assert!(unapproved["candidate_validation"].is_null(), "{unapproved}");
    registry["release_policy"]["approved_evidence"] = json!({
        "governance_commit": governance, "index_sha256": canonical_digest(&manifest["evidence"]).unwrap()
    });
    registry["_approved_evidence_verified"] = json!(true);
    registry["_governance_commit"] = json!("2".repeat(40));
    let accepted = check_release(root, &registry).unwrap();
    assert_eq!(accepted["status"], "BLOCKED", "{accepted}");
    assert_eq!(accepted["candidate_validation"], "PASS", "{accepted}");
    let gate_path = "gates/clean_windows.json";
    let result_path = "results/clean_windows.json";
    let original_gate = fs::read(evidence_root.join(gate_path)).unwrap();
    let original_result = fs::read(evidence_root.join(result_path)).unwrap();
    let mut forged_gate = read_json(&original_gate, gate_path).unwrap();
    let forged_result = b"replacement result with self-reported PASS";
    write(&evidence_root, result_path, forged_result);
    forged_gate["resultSha256"] = json!(digest(forged_result));
    let mut forged_manifest = manifest.clone();
    forged_manifest["evidence"]["clean_windows"]["sha256"] =
        json!(write_json(&evidence_root, gate_path, &forged_gate));
    write_json(&remote, RELEASE_PATH, &forged_manifest);
    let rejected = check_release(root, &registry).unwrap();
    assert_eq!(rejected["status"], "FAIL", "{rejected}");
    assert!(
        rejected["reason"]
            .as_str()
            .unwrap()
            .contains("reviewed index digest")
    );
    write(&evidence_root, gate_path, original_gate);
    write(&evidence_root, result_path, original_result);

    for name in REPOS {
        fixture_git(
            &root.join(name),
            &[
                "update-ref",
                &format!("refs/tags/v{TRAIN}"),
                manifest["repositories"][name]["commit"].as_str().unwrap(),
            ],
        );
    }
    let mut published = manifest.clone();
    published["distribution"] = json!("published");
    let published_sha = write_json(&remote, RELEASE_PATH, &published);
    fixture_git(&remote, &["add", "-f", "--", RELEASE_PATH, MATRIX_PATH]);
    fixture_commit(&remote, "Synthetic release owner publication");
    assert_ne!(
        git_text(&remote, &["rev-parse", "HEAD"]).unwrap(),
        manifest["repositories"][REMOTE]["commit"]
    );
    let mut activated = registry.clone();
    activated["baseline"]["activation"] = json!("ACTIVATED");
    let unpublished_approval = check_release(root, &activated).unwrap();
    assert_eq!(
        unpublished_approval["status"], "BLOCKED",
        "{unpublished_approval}"
    );
    assert!(
        unpublished_approval["reason"]
            .as_str()
            .unwrap()
            .contains("reviewed publication manifest")
    );
    activated["release_policy"]["approved_publication_sha256"] = json!(published_sha);
    let accepted = check_release(root, &activated).unwrap();
    assert_eq!(accepted["status"], "PASS", "{accepted}");
    assert_eq!(accepted["activation"], "ACTIVATED", "{accepted}");

    write(
        &evidence_root,
        "artifacts/cirnet-remote/cirnet-remote-0.2.4-fixture.zip",
        b"tampered",
    );
    let rejected = check_release(root, &activated).unwrap();
    assert_eq!(rejected["status"], "FAIL", "{rejected}");
    assert!(rejected["reason"].as_str().unwrap().contains("wrong size"));
    fixture_git(
        &remote,
        &["config", "--local", "filter.fixture.clean", "exit 87"],
    );
    fixture_git(
        &remote,
        &["config", "--local", "filter.fixture.required", "true"],
    );
    write(&remote, ".gitattributes", "Cargo.toml filter=fixture\n");
    write(&remote, "Cargo.toml", "synthetic dirty worktree\n");
    let dirty = git_status(&remote).unwrap();
    assert!(String::from_utf8_lossy(&dirty).contains(" M Cargo.toml"));
}

#[test]
fn publication_requires_reviewed_raw_manifest_bytes() {
    let manifest = b"{\"distribution\":\"published\"}";
    let mut registry = json!({"release_policy": {"approved_publication_sha256": null}});
    assert!(matches!(
        check_publication(&registry, manifest),
        Err(GateError::Blocked(_))
    ));
    registry["release_policy"]["approved_publication_sha256"] = json!(digest(manifest));
    assert!(check_publication(&registry, manifest).is_ok());
    assert!(matches!(
        check_publication(&registry, b"{ \"distribution\": \"published\" }"),
        Err(GateError::Failed(_))
    ));
}
