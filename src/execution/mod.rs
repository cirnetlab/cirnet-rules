mod cancel;
mod plan;
pub(crate) mod process;
mod resolve;

#[cfg(test)]
mod tests;

use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

type Selection = BTreeMap<String, String>;
const OUTPUT_LIMIT: usize = 128 * 1024;
const MAX_TIMEOUT_MS: u64 = 1_800_000;

pub struct Options {
    pub dry_run: bool,
    pub offline: bool,
    pub locked: bool,
    pub yes: bool,
    pub timeout_ms: u64,
}

pub struct NativeResult {
    pub output: Vec<u8>,
    pub error: Vec<u8>,
    pub output_truncated: bool,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub cancelled: bool,
    pub elapsed_ms: u64,
}

impl NativeResult {
    pub fn success(&self) -> bool {
        self.exit_code == Some(0) && !self.timed_out && !self.cancelled
    }
}

impl From<process::ResultRecord> for NativeResult {
    fn from(record: process::ResultRecord) -> Self {
        Self {
            exit_code: record.exit.code(),
            elapsed_ms: record.elapsed_ms,
            timed_out: record.timed_out,
            cancelled: record.cancelled,
            output_truncated: record.output.truncated || record.error.truncated,
            output: record.output.prefix,
            error: record.error.prefix,
        }
    }
}

pub fn run_native(command: &mut Command, timeout_ms: u64) -> Result<NativeResult, String> {
    if !(1..=MAX_TIMEOUT_MS).contains(&timeout_ms) {
        return Err("Timeout must be between 1 and 1800000 milliseconds".into());
    }
    if !Path::new(command.get_program()).is_absolute() {
        return Err("Native process requires an absolute executable".into());
    }
    resolve::executable(Path::new(command.get_program()))?;
    let cancellation = cancel::Cancellation::begin()?;
    process::run_process_with(
        command,
        Duration::from_millis(timeout_ms),
        OUTPUT_LIMIT,
        || cancellation.requested(),
    )
    .map(NativeResult::from)
}

/// Capture a bounded local metadata command with an owned process tree.
pub fn run_quiet_native(
    command: &mut Command,
    timeout: Duration,
    is_cancelled: impl Fn() -> bool,
) -> Result<NativeResult, String> {
    if timeout.is_zero() || timeout.as_millis() > u128::from(MAX_TIMEOUT_MS) {
        return Err("Metadata command timeout is outside the supported range".into());
    }
    let path = Path::new(command.get_program());
    if !path.is_absolute() {
        return Err("Native process requires an absolute executable".into());
    }
    resolve::executable(path)?;
    process::run_process_captured(command, timeout, OUTPUT_LIMIT, true, is_cancelled)
        .map(NativeResult::from)
}

pub fn dispatch(
    root: &Path,
    words: &[String],
    selected: &Selection,
    options: &Options,
) -> Result<Value, String> {
    if !(1..=MAX_TIMEOUT_MS).contains(&options.timeout_ms) {
        return Err("Timeout must be between 1 and 1800000 milliseconds".into());
    }
    if !options.dry_run && !options.yes {
        return Ok(blocked(
            "Review the dry run, then explicitly authorize local project execution with --yes",
        ));
    }
    let root = root
        .canonicalize()
        .map_err(|_| "Cannot resolve project directory")?;
    let _lease = match crate::tooling::lock_use(&root) {
        Ok(lease) => lease,
        Err(reason) => return Ok(blocked(&reason)),
    };
    let plan = match plan::create(&root, words, selected, options) {
        Ok(plan) => plan,
        Err(reason) => return Ok(blocked(&reason)),
    };
    if options.dry_run {
        return Ok(json!({
            "status":"PASS", "dry_run":true, "plan":plan.preview(&root, options),
            "child_exit_code":null
        }));
    }
    let start = Instant::now();
    let mut report = Value::Null;
    let mut output_truncated = false;
    for (index, step) in plan.steps.iter().enumerate() {
        let elapsed = start.elapsed().as_millis() as u64;
        if elapsed >= options.timeout_ms {
            return Ok(blocked(
                "The execution deadline elapsed before the next native step",
            ));
        }
        let paths: Vec<_> = step.path_tools.iter().map(|path| path.as_path()).collect();
        let mut command = resolve::command(&step.program, &root, &paths, &root);
        command
            .args(&step.args)
            .envs(step.env.iter().map(|(key, value)| (*key, value)));
        let result = match run_native(&mut command, options.timeout_ms - elapsed) {
            Ok(result) => result,
            Err(reason) => return Ok(blocked(&reason)),
        };
        write_output(&result.output);
        write_output(&result.error);
        output_truncated |= result.output_truncated;
        let status = if result.cancelled {
            "CANCELLED"
        } else if result.timed_out {
            "BLOCKED"
        } else if result.success() {
            "PASS"
        } else {
            "FAIL"
        };
        report = json!({
            "status": status,
            "engine": plan.engine,
            "step": index + 1,
            "child_exit_code": result.exit_code.filter(|_| !result.cancelled && !result.timed_out),
            "elapsed_ms": start.elapsed().as_millis() as u64,
            "timed_out": result.timed_out,
            "cancelled": result.cancelled,
            "output_truncated": output_truncated,
        });
        if status != "PASS" {
            return Ok(report);
        }
    }
    Ok(report)
}

pub(crate) fn write_output(bytes: &[u8]) {
    use std::io::Write;
    let safe: String = String::from_utf8_lossy(bytes)
        .chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
        .collect();
    let _ = std::io::stderr().lock().write_all(safe.as_bytes());
}

fn blocked(reason: &str) -> Value {
    json!({"status":"BLOCKED","reason":reason,"child_exit_code":null})
}
