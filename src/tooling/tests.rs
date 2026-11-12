use super::*;
use std::fs;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = env::temp_dir().join(format!(
            "cnr-provider-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }

    fn run(&self, args: &[&str], options: Options) -> Result<Value, String> {
        let args: Vec<_> = args.iter().map(|word| (*word).to_owned()).collect();
        dispatch(Some(&self.0), &args, &options)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn dry_run_never_creates_provider_state_and_versions_are_exact() {
    let fixture = Fixture::new();
    let options = || Options {
        dry_run: true,
        ..Options::default()
    };
    let plan = fixture.run(&["install", "rust@1.99.1"], options()).unwrap();
    assert_eq!(plan["status"], "PASS");
    assert_eq!(plan["plan"]["provider"], "rustup");
    assert_eq!(plan["plan"]["version"], "1.99.1");
    assert_eq!(plan["plan"]["scope"], "project");
    for target in [
        "rust@stable",
        "node@../../anything",
        "node@v01.2.3",
        "uv@-1",
    ] {
        assert!(
            fixture.run(&["install", target], options()).is_err(),
            "{target}"
        );
    }
    assert!(!fixture.0.join(".cnr").exists());
}

#[test]
fn update_without_targets_is_read_only_even_when_confirmed() {
    let fixture = Fixture::new();
    let result = fixture
        .run(
            &["update"],
            Options {
                yes: true,
                ..Options::default()
            },
        )
        .unwrap();
    assert_eq!(result["state"], "PLANNED");
    assert!(!fixture.0.join(".cnr").exists());
}

#[test]
fn offline_install_and_user_scope_do_not_create_directories() {
    let fixture = Fixture::new();
    let offline = Options {
        offline: true,
        yes: true,
        ..Options::default()
    };
    let result = fixture
        .run(&["install", "python@3.12.10"], offline)
        .unwrap();
    assert_eq!(result["status"], "BLOCKED");
    let user = Options {
        scope: "user".into(),
        yes: true,
        ..Options::default()
    };
    let result = fixture.run(&["install", "python@3.12.10"], user).unwrap();
    assert_eq!(result["status"], "BLOCKED");
    assert!(!fixture.0.join(".cnr").exists());
}

#[test]
fn unmanaged_directories_are_never_removed() {
    let fixture = Fixture::new();
    let target = fixture.0.join(".cnr/tools/python/3.12.10");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("sentinel"), b"unowned").unwrap();
    let options = Options {
        yes: true,
        ..Options::default()
    };
    assert!(fixture.run(&["remove", "python@3.12.10"], options).is_err());
    assert_eq!(fs::read(target.join("sentinel")).unwrap(), b"unowned");
}

#[test]
fn owned_interrupted_install_can_be_cleaned_explicitly() {
    let fixture = Fixture::new();
    let target = store::reserve(&fixture.0, &store::Identity::tool("python", "3.12.10")).unwrap();
    fs::write(target.join("partial"), b"interrupted").unwrap();
    let lock = store::exclusive(&fixture.0).unwrap();
    assert!(lock_use(&fixture.0).is_err());
    drop(lock);
    let job = lock_use(&fixture.0).unwrap();
    let options = || Options {
        yes: true,
        ..Options::default()
    };
    assert!(
        fixture
            .run(&["remove", "python@3.12.10"], options())
            .is_err()
    );
    drop(job);
    let result = fixture
        .run(&["remove", "python@3.12.10"], options())
        .unwrap();
    assert_eq!(result["status"], "PASS");
    assert!(!target.exists());
    let audit = fixture.run(&["audit", "list"], Options::default()).unwrap();
    assert_eq!(audit["events"][0]["action"], "remove");
}

#[test]
fn catalog_works_without_a_project_and_rejects_wrong_type() {
    let words = |args: &[&str]| {
        args.iter()
            .map(|word| (*word).to_owned())
            .collect::<Vec<_>>()
    };
    let result = dispatch(None, &words(&["search", "qt"]), &Options::default()).unwrap();
    assert_eq!(result["tools"][0]["tool"], "qt");
    let options = Options {
        kind: Some("runtime".into()),
        ..Options::default()
    };
    assert!(dispatch(None, &words(&["info", "qt"]), &options).is_err());
}

#[cfg(any(windows, target_os = "linux", target_os = "macos"))]
#[test]
fn completion_inventory_reads_pins_and_ready_receipts() {
    let fixture = Fixture::new();
    fs::write(
        fixture.0.join("cnr.toml"),
        "schema = 1\nprofile = \"default\"\n[tools]\nrust = \"1.99.1\"\n",
    )
    .unwrap();
    let installed = store::Identity::tool("rust", "1.98.0");
    let owned = store::reserve(&fixture.0, &installed).unwrap();
    let executable = owned.join("rustc-fixture");
    fs::write(&executable, b"receipt fixture, never executed").unwrap();
    store::publish(&fixture.0, &installed, &executable).unwrap();
    store::reserve(&fixture.0, &store::Identity::tool("rust", "1.97.0")).unwrap();
    fs::create_dir(owned.parent().unwrap().join("1.100.0")).unwrap();

    let catalog = completion_catalog(Some(&fixture.0));
    let rust = catalog.iter().find(|tool| tool.id == "rust").unwrap();
    assert_eq!(
        rust.versions
            .iter()
            .map(|version| (version.version.as_str(), version.pinned, version.installed))
            .collect::<Vec<_>>(),
        [("1.99.1", true, false), ("1.98.0", false, true)]
    );
}

#[test]
fn project_and_venv_references_keep_an_owned_python() {
    let fixture = Fixture::new();
    let target = store::reserve(&fixture.0, &store::Identity::tool("python", "3.12.10")).unwrap();
    let version_file = fixture.0.join(".python-version");
    fs::write(&version_file, b"3.12\n").unwrap();
    let options = || Options {
        yes: true,
        ..Options::default()
    };
    let result = fixture
        .run(&["remove", "python@3.12.10"], options())
        .unwrap();
    assert_eq!(result["status"], "BLOCKED");
    fs::remove_file(version_file).unwrap();
    store::reserve_with_python(
        &fixture.0,
        &store::Identity::venv("test"),
        Some("3.12.10"),
        None,
    )
    .unwrap();
    let result = fixture
        .run(&["remove", "python@3.12.10"], options())
        .unwrap();
    assert_eq!(result["status"], "BLOCKED");
    assert!(target.is_dir());
}

#[test]
#[ignore = "requires externally installed uv and Python"]
fn uv_creates_runs_and_removes_a_real_fixture_venv() {
    let fixture = Fixture::new();
    let options = || Options {
        yes: true,
        ..Options::default()
    };
    let created = fixture
        .run(&["venv", "create", "smoke"], options())
        .unwrap();
    assert_eq!(created["status"], "PASS", "{created}");
    let run = fixture
        .run(
            &[
                "venv",
                "run",
                "smoke",
                "--",
                "-c",
                "import sys; assert sys.prefix != sys.base_prefix",
            ],
            options(),
        )
        .unwrap();
    assert_eq!(run["status"], "PASS", "{run}");
    assert_eq!(run["child_exit_code"], 0);
    let removed = fixture
        .run(&["venv", "remove", "smoke"], options())
        .unwrap();
    assert_eq!(removed["state"], "REMOVED");
    assert!(!fixture.0.join(".cnr/venvs/smoke").exists());
}
