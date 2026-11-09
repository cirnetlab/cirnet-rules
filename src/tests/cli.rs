use serde_json::Value;
mod openrules;
#[path = "../test_support.rs"]
mod test_support;
use std::{path::Path, process::Command};

fn run(arguments: &[&str]) -> (i32, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_cirnet-governance"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(arguments)
        .output()
        .expect("start governance CLI");
    let result = serde_json::from_slice(&output.stdout).expect("JSON CLI result");
    (output.status.code().expect("exit code"), result)
}

#[test]
fn repository_validation_is_nonempty() {
    let (exit, result) = run(&["validate"]);
    assert_eq!(exit, 0, "{result}");
    assert_eq!(result["status"], "PASS");
    assert!(result["result"]["paired_documents"].as_u64().unwrap() >= 30);
}

#[test]
fn missing_product_inputs_block_release() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("en")
        .join("tests");
    let (exit, result) = run(&["check-release", "--workspace", workspace.to_str().unwrap()]);
    assert_eq!(exit, 2, "{result}");
    assert_eq!(result["status"], "BLOCKED");
}

#[test]
fn unrelated_repository_never_gets_agent_context() {
    let (exit, result) = run(&["context", "--workspace", "..", "--repo", "unrelated"]);
    assert_eq!(exit, 1, "{result}");
    assert_eq!(
        result["result"]["reason"],
        "Repository outside CIRNET scope"
    );
    assert!(result["result"].get("manifest").is_none());
}
