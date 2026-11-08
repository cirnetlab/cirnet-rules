use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::{self, Read},
    path::Path,
    process::{Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

pub(super) struct Output {
    pub prefix: Vec<u8>,
    pub truncated: bool,
}

fn capture(mut reader: impl Read, limit: usize) -> io::Result<Output> {
    let mut prefix = Vec::new();
    let mut truncated = false;
    let mut buffer = [0; 4096];
    loop {
        let bytes = reader.read(&mut buffer)?;
        if bytes == 0 {
            break;
        }
        let retained = bytes.min(limit - prefix.len());
        prefix.extend_from_slice(&buffer[..retained]);
        truncated |= retained != bytes;
    }
    Ok(Output { prefix, truncated })
}

pub(super) struct ResultRecord {
    pub exit: ExitStatus,
    pub output: Output,
    error: Output,
    elapsed_ms: u64,
    pub timed_out: bool,
    pub cancelled: bool,
}

pub(super) fn run_process(
    command: &mut Command,
    timeout: Duration,
    limit: usize,
    cancel: Option<&Path>,
) -> Result<ResultRecord, String> {
    let start = Instant::now();
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| "Unable to start the approved executable")?;
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let output = thread::spawn(move || capture(stdout, limit));
    let error = thread::spawn(move || capture(stderr, limit));
    let mut timed_out = false;
    let mut cancelled = false;
    let exit = loop {
        match child.try_wait() {
            Ok(Some(exit)) => break exit,
            Ok(None) => {}
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("Unable to observe the owned process".into());
            }
        }
        cancelled = cancel.is_some_and(Path::is_file);
        timed_out = !cancelled && start.elapsed() >= timeout;
        if cancelled || timed_out {
            let _ = child.kill();
            break child
                .wait()
                .map_err(|_| "Unable to stop the owned process")?;
        }
        thread::sleep(Duration::from_millis(10));
    };
    let output = output
        .join()
        .map_err(|_| "Unable to collect approved command stdout")?
        .map_err(|_| "Unable to read approved command stdout")?;
    let mut error = error
        .join()
        .map_err(|_| "Unable to collect approved command stderr")?
        .map_err(|_| "Unable to read approved command stderr")?;
    let remaining = limit - output.prefix.len();
    if error.prefix.len() > remaining {
        error.prefix.truncate(remaining);
        error.truncated = true;
    }
    Ok(ResultRecord {
        exit,
        output,
        error,
        elapsed_ms: start.elapsed().as_millis().min(u64::MAX as u128) as u64,
        timed_out,
        cancelled,
    })
}

pub(super) fn process_receipt(result: ResultRecord) -> Value {
    let mut digest = Sha256::new();
    digest.update((result.output.prefix.len() as u64).to_le_bytes());
    digest.update(&result.output.prefix);
    digest.update((result.error.prefix.len() as u64).to_le_bytes());
    digest.update(&result.error.prefix);
    let truncated = result.output.truncated || result.error.truncated;
    let status = if result.timed_out || result.cancelled || truncated {
        "BLOCKED"
    } else if result.exit.success() {
        "PASS"
    } else {
        "FAIL"
    };
    json!({
        "status": status,
        "exit_code": result.exit.code(),
        "elapsed_ms": result.elapsed_ms,
        "output_sha256": format!("{:x}", digest.finalize()),
        "output_bytes": result.output.prefix.len() + result.error.prefix.len(),
        "output_truncated": truncated,
        "output_retained": false,
        "timed_out": result.timed_out,
        "cancelled": result.cancelled
    })
}
