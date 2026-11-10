use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::{self, Read},
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};

mod owned;
const CLEANUP_LIMIT: Duration = Duration::from_secs(2);

pub(crate) struct Output {
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

pub(crate) struct ResultRecord {
    pub exit: ExitStatus,
    pub output: Output,
    pub error: Output,
    pub elapsed_ms: u64,
    pub timed_out: bool,
    pub cancelled: bool,
    drained: bool,
}

fn wait_bounded(child: &mut Child) -> Result<ExitStatus, String> {
    let deadline = Instant::now() + CLEANUP_LIMIT;
    loop {
        if let Some(exit) = child
            .try_wait()
            .map_err(|_| "Unable to observe the owned process")?
        {
            return Ok(exit);
        }
        if Instant::now() >= deadline {
            return Err(
                "The owned process has not confirmed termination within the cleanup deadline"
                    .into(),
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn start_capture(reader: impl Read + Send + 'static, limit: usize) -> Receiver<io::Result<Output>> {
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let _ = sender.send(capture(reader, limit));
    });
    receiver
}

fn drain(
    receiver: Receiver<io::Result<Output>>,
    deadline: Instant,
) -> Result<(Output, bool), String> {
    match receiver.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
        Ok(Ok(output)) => Ok((output, true)),
        Err(RecvTimeoutError::Timeout) => {
            // Never wait forever for a pipe still held by a process outside
            // the owned tree. Treat absent output as incomplete evidence.
            Ok((
                Output {
                    prefix: Vec::new(),
                    truncated: true,
                },
                false,
            ))
        }
        _ => Err("Unable to collect the approved process output".into()),
    }
}

pub(crate) fn run_process(
    command: &mut Command,
    timeout: Duration,
    limit: usize,
    cancel: Option<&Path>,
) -> Result<ResultRecord, String> {
    run_process_with(command, timeout, limit, || {
        cancel.is_some_and(Path::is_file)
    })
}

pub(super) fn run_process_with(
    command: &mut Command,
    timeout: Duration,
    limit: usize,
    is_cancelled: impl Fn() -> bool,
) -> Result<ResultRecord, String> {
    let start = Instant::now();
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let (mut child, tree) = owned::spawn(command)?;
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let output = start_capture(stdout, limit);
    let error = start_capture(stderr, limit);
    let mut timed_out = false;
    let mut cancelled = false;
    let exit = loop {
        if let Some(exit) = tree.poll(&mut child)? {
            break exit;
        }
        cancelled = is_cancelled();
        timed_out = !cancelled && start.elapsed() >= timeout;
        if cancelled || timed_out {
            tree.stop()?;
            break wait_bounded(&mut child)?;
        }
        thread::sleep(Duration::from_millis(10));
    };
    let deadline = Instant::now() + CLEANUP_LIMIT;
    let (mut output, output_drained) = drain(output, deadline)?;
    let (mut error, error_drained) = drain(error, deadline)?;
    let drained = output_drained && error_drained;
    output.truncated |= !drained;
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
        drained,
    })
}

pub(crate) fn process_receipt(result: ResultRecord) -> Value {
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
        "output_drained": result.drained,
        "process_ownership": owned::OWNERSHIP,
        "timed_out": result.timed_out,
        "cancelled": result.cancelled
    })
}
