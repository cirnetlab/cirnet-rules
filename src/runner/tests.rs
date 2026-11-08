use super::process::{process_receipt, run_process};
use super::snapshot::{boundary_signals, git_state, parse_dirty};
use super::*;
use std::time::Duration;

#[test]
fn dirty_paths_and_boundaries_are_explicit() {
    let dirty =
        parse_dirty(b" M src/ui/View.vue\0?? src/services/auth/session.rs\0 D src/old.rs\0")
            .unwrap();
    assert_eq!(dirty["src/ui/View.vue"], " M");
    assert!(is_boundary("src/services/auth/session.rs"));
    assert!(is_boundary("Cargo.lock"));
    assert!(!is_boundary("src/ui/View.vue"));
    assert!(parse_dirty(b"?? ../outside\0").is_err());
    assert_eq!(boundary_signals(b"<button>Run</button>"), None);
    assert_ne!(
        boundary_signals(b"<button @click=\"startRun()\">Run</button>"),
        boundary_signals(b"<button @click=\"stopRun()\">Run</button>")
    );
}

#[test]
fn runner_rejects_unapproved_programs_paths_and_expired_grants() {
    let mut task = json!({
        "state":"IMPLEMENTED", "descriptor":{
            "id":"fixture", "owner":"author", "mode":"WORK", "commands":["fixture"],
            "verification_scope":{"ui":["src/ui.vue"]}, "authorization":{"actions":["verify"],"owner":"reviewer"}
        }
    });
    let mut registry = json!({
        "control_plane":{"commands":{"fixture":{
            "program":"git","args":["diff","--check"],"repository":"ui","paths":["src/ui.vue"],
            "mode":"VERIFY","timeout_ms":1500,"max_output_bytes":256
        }}}
    });
    assert!(step(&task, &registry, "fixture", "reviewer", false).is_ok());
    assert!(
        step(&task, &registry, "fixture", "unrelated", false)
            .unwrap_err()
            .contains("authorized verification actor")
    );
    registry["control_plane"]["commands"]["fixture"]["program"] = json!("cmd.exe");
    assert!(
        step(&task, &registry, "fixture", "reviewer", false)
            .unwrap_err()
            .contains("reviewed runner profile")
    );
    registry["control_plane"]["commands"]["fixture"]["program"] = json!("git");
    registry["control_plane"]["commands"]["fixture"]["paths"] = json!(["../outside"]);
    assert!(
        step(&task, &registry, "fixture", "reviewer", false)
            .unwrap_err()
            .contains("explicit task scope")
    );
    task["descriptor"]["authorization"]["expires_at"] = json!(1);
    assert!(
        step(&task, &registry, "fixture", "reviewer", false)
            .unwrap_err()
            .contains("expired")
    );
}

#[test]
fn read_only_git_guard_suppresses_checkout_callbacks() {
    use std::{
        env, fs,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };
    let tmp = env::temp_dir().canonicalize().unwrap();
    let fixture = tmp.join(format!(
        "cgacp runner fixture {}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let root = fixture.join("governance");
    let checkout = fixture.join("fixture product");
    fs::create_dir(&fixture).unwrap();
    fs::create_dir(&root).unwrap();
    fs::create_dir(&checkout).unwrap();
    let setup = |args: &[&str]| {
        let output = Command::new("git")
            .current_dir(&checkout)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "Git fixture setup failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    setup(&["init", "--quiet"]);
    fs::write(checkout.join(".gitattributes"), "*.txt filter=fixture\n").unwrap();
    fs::write(checkout.join("view.txt"), "first\n").unwrap();
    setup(&["add", "--", ".gitattributes", "view.txt"]);
    setup(&[
        "config",
        "filter.fixture.clean",
        "echo triggered > filter-called; cat",
    ]);
    setup(&[
        "config",
        "core.fsmonitor",
        "echo triggered > monitor-called",
    ]);
    fs::write(checkout.join("view.txt"), "updated\n").unwrap();
    let status = git_state(
        &root.canonicalize().unwrap(),
        &checkout.canonicalize().unwrap(),
    )
    .unwrap();
    assert!(status.dirty.contains_key("view.txt"));
    assert!(!checkout.join("filter-called").exists());
    assert!(!checkout.join("monitor-called").exists());
    assert!(fixture.canonicalize().unwrap().starts_with(&tmp));
    // Sandboxed Windows can retain Git's hidden metadata even after Git exits.
    let _ = fs::remove_dir_all(fixture);
}

#[test]
fn runner_captures_bounded_output_without_exposing_secret_bytes() {
    let mut command = fixture_command("emit");
    let result = run_process(&mut command, Duration::from_secs(10), 64, None).unwrap();
    assert!(result.output.truncated);
    assert_eq!(result.output.prefix.len(), 64);
    let public = process_receipt(result);
    assert!(public["output_truncated"].as_bool().unwrap());
    assert!(!public.to_string().contains("SECRET_FIXTURE"));
}

#[test]
fn runner_times_out_and_honors_a_cancel_request() {
    let mut command = fixture_command("wait");
    let result = run_process(&mut command, Duration::from_millis(40), 128, None).unwrap();
    assert!(result.timed_out);
    assert!(!result.cancelled);

    let mut command = fixture_command("wait");
    let cancel = std::env::temp_dir().join(format!("cgacp-cancel-fixture-{}", std::process::id()));
    std::fs::write(&cancel, b"cancel").unwrap();
    let result = run_process(&mut command, Duration::from_secs(10), 128, Some(&cancel)).unwrap();
    std::fs::remove_file(cancel).unwrap();
    assert!(result.cancelled);
    assert!(!result.timed_out);
}

#[test]
fn runner_finishes_when_an_owned_descendant_inherits_the_pipes() {
    let fixture = std::env::temp_dir().join(format!(
        "cgacp-runner-tree-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&fixture).unwrap();
    let mut command = fixture_command("parent");
    command.env("CGACP_TREE_FIXTURE", &fixture);
    let result = run_process(&mut command, Duration::from_secs(5), 2048, None).unwrap();
    assert!(result.exit.success());
    assert!(
        fixture.join("ready").is_file(),
        "Fixture descendant never started"
    );
    assert!(
        !fixture.join("late").exists(),
        "Owned descendant outlived the leader"
    );
    std::fs::remove_file(fixture.join("ready")).unwrap();
    std::fs::remove_dir(fixture).unwrap();
}

#[test]
fn timeout_and_cancel_stop_owned_descendants_before_they_write() {
    for cancelled in [false, true] {
        let fixture = std::env::temp_dir().join(format!(
            "cgacp-runner-stop-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&fixture).unwrap();
        let ready = fixture.join("ready");
        let marker = fixture.join("cancel");
        let signal = cancelled.then(|| {
            let ready = ready.clone();
            let marker = marker.clone();
            std::thread::spawn(move || {
                let until = std::time::Instant::now() + Duration::from_secs(4);
                while !ready.is_file() && std::time::Instant::now() < until {
                    std::thread::sleep(Duration::from_millis(10));
                }
                std::fs::write(marker, b"cancel").unwrap();
            })
        });
        let mut command = fixture_command("parent-wait");
        command.env("CGACP_TREE_FIXTURE", &fixture);
        let timeout = Duration::from_millis(if cancelled { 5000 } else { 1100 });
        let receipt = process_receipt(
            run_process(
                &mut command,
                timeout,
                2048,
                cancelled.then_some(marker.as_path()),
            )
            .unwrap(),
        );
        if let Some(signal) = signal {
            signal.join().unwrap();
            std::fs::remove_file(marker).unwrap();
        }
        assert!(ready.is_file(), "Fixture descendant never started");
        assert_eq!(receipt["cancelled"], cancelled);
        assert_eq!(receipt["timed_out"], !cancelled);
        assert_eq!(receipt["output_drained"], true);
        assert!(
            !fixture.join("late").exists(),
            "Descendant outlived tree cancellation"
        );
        std::fs::remove_file(ready).unwrap();
        std::fs::remove_dir(fixture).unwrap();
    }
}

fn fixture_command(mode: &str) -> std::process::Command {
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "runner::tests::subprocess_fixture",
            "--nocapture",
        ])
        .env("CGACP_RUNNER_FIXTURE", mode);
    command
}

#[test]
fn subprocess_fixture() {
    match std::env::var("CGACP_RUNNER_FIXTURE").as_deref() {
        Ok("emit") => println!("SECRET_FIXTURE{}", "x".repeat(5000)),
        Ok("wait") => std::thread::sleep(std::time::Duration::from_secs(30)),
        Ok(mode @ ("parent" | "parent-wait")) => {
            let dir = std::path::PathBuf::from(std::env::var_os("CGACP_TREE_FIXTURE").unwrap());
            let mut descendant = fixture_command("descendant").spawn().unwrap();
            let until = std::time::Instant::now() + Duration::from_secs(3);
            while !dir.join("ready").is_file() {
                assert!(std::time::Instant::now() < until);
                std::thread::sleep(Duration::from_millis(10));
            }
            assert!(descendant.try_wait().unwrap().is_none());
            if mode == "parent-wait" {
                std::thread::sleep(Duration::from_secs(30));
            }
        }
        Ok("descendant") => {
            let dir = std::path::PathBuf::from(std::env::var_os("CGACP_TREE_FIXTURE").unwrap());
            std::fs::write(dir.join("ready"), b"1").unwrap();
            std::thread::sleep(Duration::from_secs(3));
            std::fs::write(dir.join("late"), b"1").unwrap();
        }
        _ => {}
    }
}
