use super::*;
use std::time::Duration;

fn options() -> Options {
    Options {
        dry_run: true,
        offline: false,
        locked: false,
        yes: false,
        timeout_ms: 1000,
    }
}

#[test]
fn fixture_process() {
    match std::env::var("CNR_TEST_PROCESS").as_deref() {
        Ok("noisy") => {
            println!("{}", "x".repeat(256 * 1024));
            eprintln!("stderr fixture");
            std::process::exit(17);
        }
        Ok("sleep") => std::thread::sleep(Duration::from_secs(20)),
        Ok("argv") => {
            println!(
                "{}",
                serde_json::to_string(&std::env::args().collect::<Vec<_>>()).unwrap()
            );
        }
        _ => {}
    }
}

fn fixture(mode: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args([
        "--exact",
        "execution::tests::fixture_process",
        "--nocapture",
    ]);
    command.env("CNR_TEST_PROCESS", mode);
    command
}

#[test]
fn native_run_bounds_output_and_preserves_child_failure_code() {
    let result = run_native(&mut fixture("noisy"), 5000).unwrap();
    assert_eq!(result.exit_code, Some(17));
    assert!(result.output_truncated);
    assert!(result.output.len() + result.error.len() <= 128 * 1024);
    assert!(!result.success());
}

#[test]
fn native_timeout_terminates_child_and_does_not_report_success() {
    let result = run_native(&mut fixture("sleep"), 100).unwrap();
    assert!(result.timed_out);
    assert!(!result.success());
    assert!(result.elapsed_ms < 5000);
}

#[test]
fn native_executable_preserves_unicode_path_and_literal_arguments() {
    let root = crate::test_support::temporary_directory("cnr-execution").unwrap();
    let binary = root.join(format!("đường dẫn có dấu{}", std::env::consts::EXE_SUFFIX));
    let source = std::env::current_exe().unwrap();
    std::fs::copy(source, &binary).unwrap();
    let mut command = Command::new(binary);
    command
        .args([
            "--exact",
            "execution::tests::fixture_process",
            "--nocapture",
            "--",
        ])
        .args(["with spaces", "tiếng Việt", "a&b%name%", r#"quotes"x\\"#])
        .env("CNR_TEST_PROCESS", "argv");
    let result = run_native(&mut command, 5000);
    std::fs::remove_dir_all(root).unwrap();
    let result = result.unwrap();
    assert!(result.success());
    let text = String::from_utf8(result.output).unwrap();
    let argv: Vec<String> = text
        .lines()
        .find_map(|line| serde_json::from_str(line).ok())
        .expect("fixture emitted argv");
    assert_eq!(
        &argv[argv.len() - 4..],
        &["with spaces", "tiếng Việt", "a&b%name%", r#"quotes"x\\"#]
    );
}

#[test]
fn exec_requires_explicit_approval_before_starting() {
    let root = std::env::current_dir().unwrap();
    let mut options = options();
    options.dry_run = false;
    let report = dispatch(
        &root,
        &["exec".into(), "--".into(), "missing".into()],
        &BTreeMap::new(),
        &options,
    )
    .unwrap();
    assert_eq!(report["status"], "BLOCKED");
    assert!(report["child_exit_code"].is_null());
}

#[test]
fn exec_dry_run_refuses_implicit_batch_shell() {
    let root = std::env::current_dir().unwrap();
    let report = dispatch(
        &root,
        &["exec".into(), "--".into(), "fixture.cmd".into()],
        &BTreeMap::new(),
        &options(),
    )
    .unwrap();
    assert_eq!(report["status"], "BLOCKED");
    assert!(report["reason"].as_str().unwrap().contains("batch"));
    assert!(report["child_exit_code"].is_null());
}

#[cfg(windows)]
#[test]
fn native_provider_arguments_use_ordinary_drive_paths() {
    let path = Path::new(r"\\?\C:\CNR fixture\Tiếng Việt");
    assert_eq!(
        resolve::provider_path(path),
        std::ffi::OsStr::new(r"C:\CNR fixture\Tiếng Việt")
    );
}
