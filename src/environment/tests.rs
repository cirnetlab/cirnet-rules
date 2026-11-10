use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("cnr-é môi trường-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).unwrap();
        Self(root)
    }

    fn put(&self, path: &str, contents: &str) {
        fs::write(self.0.join(path), contents).unwrap();
    }

    fn run(&self, words: &[&str], locked: bool, dry_run: bool) -> Result<Value, String> {
        let project = Project::open(&self.0).unwrap();
        let words: Vec<_> = words.iter().map(|word| (*word).to_owned()).collect();
        dispatch(&project, &words, locked, dry_run)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn lock_preserves_native_pins_and_blocks_manifest_drift() {
    let f = Fixture::new();
    f.put("rust-toolchain.toml", "[toolchain]\nchannel = '1.99.1'\n");
    f.put("Cargo.toml", "[package]\nrust-version = '1.90'\n");
    f.put(
        "package.json",
        r#"{"engines":{"node":"^20.19.0 || >=22.12.0"}}"#,
    );
    f.run(&["init"], false, true).unwrap();
    assert!(!f.0.join("cnr.toml").exists());
    f.run(&["init"], false, false).unwrap();
    assert!(f.run(&["env", "lock"], false, false).is_err());
    f.run(&["pin", "node@22.14.0"], false, false).unwrap();
    f.run(&["env", "lock"], false, false).unwrap();
    let p = Project::open(&f.0).unwrap();
    assert_eq!(p.selected_locked().unwrap()["rust"], "1.99.1");
    assert_eq!(p.selected_locked().unwrap()["node"], "22.14.0");
    f.put("Cargo.toml", "[package]\nrust-version = '1.91'\n");
    assert!(p.selected_locked().is_err());
    assert!(p.references("rust", "1.99.1").unwrap());
}

#[test]
fn static_cmake_minimum_is_checked_and_locked_without_configuring_the_project() {
    let f = Fixture::new();
    f.put(
        "CMakeLists.txt",
        "cmake_minimum_required(\n  VERSION 3.24...3.30 # policy ceiling\n)\n",
    );
    let plan = f.run(&["env", "plan"], false, false).unwrap();
    assert_eq!(plan["unresolved"], json!(["cmake"]));
    assert!(f.run(&["pin", "cmake@3.23.9"], false, true).is_err());
    f.run(&["pin", "cmake@3.31.2"], false, false).unwrap();
    f.run(&["env", "lock"], false, false).unwrap();
    assert_eq!(
        Project::open(&f.0).unwrap().selected_locked().unwrap()["cmake"],
        "3.31.2"
    );
    assert!(!f.0.join("CMakeCache.txt").exists());
}

#[test]
fn conflicting_native_pins_and_unsupported_ranges_do_not_mutate_config() {
    let f = Fixture::new();
    f.run(&["init"], false, false).unwrap();
    let original = fs::read(f.0.join("cnr.toml")).unwrap();
    f.put(".python-version", "3.12.2\n");
    f.put(
        "pyproject.toml",
        "[project]\nrequires-python = '>=3.13, <4'\n",
    );
    assert!(f.run(&["pin", "python@3.12.2"], false, false).is_err());
    assert_eq!(fs::read(f.0.join("cnr.toml")).unwrap(), original);
    f.put(".python-version", "3.13.1\n");
    f.put(".tool-versions", "python 3.13.2\n");
    assert!(f.run(&["env", "plan"], false, false).is_err());
}

#[test]
fn profiles_export_and_import_keep_inactive_references() {
    let f = Fixture::new();
    f.run(&["init"], false, false).unwrap();
    f.run(&["pin", "node@22.14.0"], false, false).unwrap();
    f.run(
        &["config", "set", "profiles.ci.node", "20.19.0"],
        false,
        false,
    )
    .unwrap();
    f.run(&["config", "set", "profile", "ci"], false, false)
        .unwrap();
    let p = Project::open(&f.0).unwrap();
    assert_eq!(p.selected().unwrap()["node"], "20.19.0");
    assert!(p.references("node", "22.14.0").unwrap());
    assert!(!p.references("node", "23.1.0").unwrap());
    let exported = f.run(&["env", "export"], false, false).unwrap();
    let other = Fixture::new();
    other.put("transfer.json", &serde_json::to_string(&exported).unwrap());
    other
        .run(&["env", "import", "transfer.json"], false, false)
        .unwrap();
    let p = Project::open(&other.0).unwrap();
    assert_eq!(p.selected().unwrap()["node"], "20.19.0");
    assert!(p.references("node", "22.14.0").unwrap());
}

#[test]
fn removing_a_selection_keeps_the_old_lock_authoritative_until_refreshed() {
    let f = Fixture::new();
    f.run(&["pin", "node@22.14.0"], false, false).unwrap();
    f.run(&["env", "lock"], false, false).unwrap();
    f.run(&["config", "unset", "tools.node"], false, false)
        .unwrap();
    let project = Project::open(&f.0).unwrap();
    assert!(!project.selected().unwrap().contains_key("node"));
    assert!(project.selected_locked().is_err());
    assert!(project.references("node", "22.14.0").unwrap());
    f.run(&["env", "lock"], false, false).unwrap();
    assert!(!project.references("node", "22.14.0").unwrap());
}

#[test]
fn malformed_import_busy_lock_and_redirection_preserve_configuration() {
    let f = Fixture::new();
    f.run(&["init"], false, false).unwrap();
    let original = fs::read(f.0.join("cnr.toml")).unwrap();
    f.put(
        "transfer.json",
        r#"{"schema":1,"profile":"default","tools":{"node":"../../bin"}}"#,
    );
    assert!(
        f.run(&["env", "import", "transfer.json"], false, false)
            .is_err()
    );
    let p = Project::open(&f.0).unwrap();
    let lock = p.lock().unwrap();
    assert!(f.run(&["use", "node@22.14.0"], false, false).is_err());
    drop(lock);
    #[cfg(unix)]
    {
        let other = Fixture::new();
        other.put("configuration", "unchanged");
        std::os::unix::fs::symlink(other.0.join("configuration"), f.0.join("cnr.lock")).unwrap();
        assert!(f.run(&["env", "lock"], false, false).is_err());
        assert_eq!(
            fs::read_to_string(other.0.join("configuration")).unwrap(),
            "unchanged"
        );
    }
    assert_eq!(fs::read(f.0.join("cnr.toml")).unwrap(), original);
}
