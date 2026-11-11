use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("cnr fixture Việt-{}-{now}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_cnr"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn json(output: &Output, exit: i32) -> Value {
    assert_eq!(
        output.status.code(),
        Some(exit),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn headless_menu_is_help_and_version_and_inventory_are_available_without_project() {
    let fixture = Fixture::new();
    let output = fixture.run(&["--no-banner"]);
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for name in ["search", "install", "venv", "workspace", "completion"] {
        assert!(help.contains(name), "{help}");
    }
    let version = json(&fixture.run(&["--json", "--version"]), 0);
    assert_eq!(version["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(
        json(&fixture.run(&["search", "rust", "--json", "--offline"]), 0)["status"],
        "PASS"
    );
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 0);
}

#[test]
fn init_and_dry_run_have_distinct_write_effects() {
    let fixture = Fixture::new();
    assert_eq!(
        json(&fixture.run(&["init", "--dry-run", "--json"]), 0)["status"],
        "PASS"
    );
    assert!(!fixture.0.join("cnr.toml").exists());
    assert!(!fixture.0.join(".cnr").exists());
    json(&fixture.run(&["init", "--json"]), 0);
    let before = fs::read(fixture.0.join("cnr.toml")).unwrap();
    json(&fixture.run(&["init", "--json"]), 0);
    assert_eq!(fs::read(fixture.0.join("cnr.toml")).unwrap(), before);
    assert_ne!(
        json(
            &fixture.run(&[
                "install",
                "rust@1.99.1",
                "--scope",
                "system",
                "--yes",
                "--json"
            ]),
            2
        )["status"],
        "PASS"
    );
    assert_eq!(fs::read_dir(fixture.0.join(".cnr")).unwrap().count(), 1);
}

#[test]
fn project_lock_detects_native_pin_drift_without_touching_dependency_locks() {
    let fixture = Fixture::new();
    fs::write(
        fixture.0.join("rust-toolchain.toml"),
        "[toolchain]\nchannel = \"1.99.1\"\n",
    )
    .unwrap();
    json(&fixture.run(&["init", "--json"]), 0);
    let report = json(&fixture.run(&["env", "lock", "--json"]), 0);
    assert_eq!(report["versions"]["rust"], "1.99.1");
    json(&fixture.run(&["env", "plan", "--locked", "--json"]), 0);
    let pinned = fs::read(fixture.0.join("cnr.lock")).unwrap();
    fs::write(
        fixture.0.join("rust-toolchain.toml"),
        "[toolchain]\nchannel = \"1.99.2\"\n",
    )
    .unwrap();
    let stale = json(&fixture.run(&["env", "plan", "--locked", "--json"]), 2);
    assert_eq!(stale["status"], "BLOCKED");
    assert_eq!(fs::read(fixture.0.join("cnr.lock")).unwrap(), pinned);
    assert!(!fixture.0.join("Cargo.lock").exists());
}

#[test]
fn exec_keeps_child_output_separate_and_denies_implicit_shell_execution() {
    let fixture = Fixture::new();
    let path = fixture
        .0
        .join(format!("CNR công cụ{}", std::env::consts::EXE_SUFFIX));
    fs::copy(env!("CARGO_BIN_EXE_cnr"), &path).unwrap();
    let executable = path.to_str().unwrap();
    let blocked = json(
        &fixture.run(&["exec", "--json", "--", executable, "--version"]),
        2,
    );
    assert_eq!(blocked["child_exit_code"], Value::Null);
    let allowed = fixture.run(&["exec", "--yes", "--json", "--", executable, "--version"]);
    assert_eq!(json(&allowed, 0)["child_exit_code"], 0);
    assert!(String::from_utf8_lossy(&allowed.stderr).contains("cnr "));
    #[cfg(windows)]
    {
        fs::write(fixture.0.join("unsafe.cmd"), "@exit /b 42\r\n").unwrap();
        let batch = fixture.0.join("unsafe.cmd");
        let blocked = json(
            &fixture.run(&["exec", "--yes", "--json", "--", batch.to_str().unwrap()]),
            2,
        );
        assert_eq!(blocked["child_exit_code"], Value::Null);
        assert!(blocked["reason"].as_str().unwrap().contains("batch"));
    }
}

#[test]
fn governance_bridge_retains_the_maintenance_envelope() {
    let fixture = Fixture::new();
    let root = env!("CARGO_MANIFEST_DIR");
    let bridge = fixture.run(&[
        "governance",
        "check-translations",
        "--json",
        "--",
        "--governance",
        root,
    ]);
    let old = Command::new(env!("CARGO_BIN_EXE_cirnet-governance"))
        .current_dir(&fixture.0)
        .args(["check-translations", "--governance", root])
        .output()
        .unwrap();
    assert_eq!(json(&bridge, 0), json(&old, 0));
    let workspace = fixture.0.to_str().unwrap();
    let direct = fixture.run(&[
        "--project",
        workspace,
        "governance",
        "validate",
        "--governance-root",
        root,
        "--json",
    ]);
    let old = Command::new(env!("CARGO_BIN_EXE_cirnet-governance"))
        .current_dir(workspace)
        .args(["validate", "--governance", root, "--workspace", workspace])
        .output()
        .unwrap();
    assert_eq!(json(&direct, 0), json(&old, 0));
}

#[test]
fn governance_native_flags_reject_unknown_and_duplicate_options() {
    let fixture = Fixture::new();
    let root = env!("CARGO_MANIFEST_DIR");
    let inspection = fixture.run(&["governance", "inspect", "--governance-root", root, "--json"]);
    assert!(inspection.stderr.is_empty());
    assert_eq!(json(&inspection, 0)["status"], "PASS");
    let agents = fixture.run(&["agent", "list", "--governance-root", root, "--json"]);
    assert_eq!(json(&agents, 0)["status"], "PASS");
    let missing = fixture.0.join("missing-legacy.sqlite3");
    let migration = fixture.run(&[
        "governance",
        "migrate",
        "inspect",
        "--source",
        missing.to_str().unwrap(),
        "--governance-root",
        root,
        "--json",
    ]);
    assert_eq!(json(&migration, 2)["result"]["status"], "NOT_FOUND");

    for extra in [
        vec!["--unknown"],
        vec!["--governance-root", root],
        vec!["--", "--governance", root],
        vec!["--yes"],
    ] {
        let mut args = vec!["governance", "inspect", "--governance-root", root, "--json"];
        args.extend(extra);
        assert_eq!(json(&fixture.run(&args), 1)["status"], "FAIL");
    }
}
