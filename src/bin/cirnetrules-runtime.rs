//! Private local operator transport. All database access stays in the runtime.
#[path = "../application_identity.rs"]
mod application_identity;
#[path = "../dashboard/mod.rs"]
mod dashboard;
#[path = "runtime/dispatcher.rs"]
mod dispatcher;
#[path = "../dashboard/operations.rs"]
mod operations;
#[cfg(test)]
#[path = "../test_support.rs"]
mod test_support;

use application_identity::{
    API_NAMESPACE, DISPLAY_NAME, IPC_PROTOCOL as PROTOCOL, PRODUCT_ID, PRODUCT_SYMBOL, VERSION,
    release_name,
};
use dashboard::{
    automations, catalog, integrations, preferences, sessions,
    storage::{self, AppError, Database},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    borrow::Cow,
    env, fmt,
    io::{self, BufRead, Read, Write},
    path::PathBuf,
    process::ExitCode,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const MAX_FRAME: usize = 256 * 1024;
const TICK: Duration = Duration::from_secs(1);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    v: u8,
    id: String,
    method: String,
    #[serde(default = "empty_object")]
    params: Value,
}

fn empty_object() -> Value {
    json!({})
}

fn info() -> Value {
    json!({
        "product_generation":PRODUCT_SYMBOL, "product_id":PRODUCT_ID, "display_name":DISPLAY_NAME, "version":VERSION,
        "release_name":release_name(), "protocol":PROTOCOL,
        "operator_channel":"private_inherited_stdio", "release_actions":"RELEASE_BLOCKED",
        "native_agent":{"codex":"HANDOFF_ONLY","claude":"HANDOFF_ONLY","antigravity":"HANDOFF_ONLY"},
        "execution_enforcement":{"READ_ONLY_BUILTIN":"AVAILABLE","TRUSTED_LOCAL_PROCESS":"LIMITED",
            "ISOLATED_EXECUTION":"BLOCKED","authenticated_independent_review":"BLOCKED"},
        "scheduler":"runtime while running; manual --scheduler supports a separate foreground process",
        "storage_jobs":operations::limits(),
        "read_cancellation":true, "conversation_event_watermark":true
    })
}

fn dispatch(db: &mut Database, method: &str, params: Value) -> Result<Value, AppError> {
    match method.split_once('.').map(|(kind, _)| kind) {
        Some("repo") => catalog::dispatch(db, method, params),
        Some("storage") => storage::dispatch(db, method, params),
        Some("preferences") => preferences::dispatch(db, method, params),
        Some("legacy") => dashboard::legacy::dispatch(db, method, params),
        Some("conversation" | "events" | "message") => sessions::dispatch(db, method, params),
        Some("automation") => automations::dispatch(db, method, params),
        Some("governance") => dashboard::governance::dispatch(db, method, params),
        Some("integration") => integrations::dispatch(method, params),
        _ => Err(AppError::new(
            "METHOD_NOT_FOUND",
            "Unknown local runtime operation",
        )),
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}

#[derive(Clone, Default)]
pub(crate) struct Diagnostics(Option<mpsc::SyncSender<String>>);

impl Diagnostics {
    fn start() -> Self {
        let (sender, receiver) = mpsc::sync_channel::<String>(64);
        // The inherited stderr pipe may never drain. Only this thread can
        // wait on that pipe, and stdio shutdown does not join it.
        let worker = thread::Builder::new()
            .name("cirnetrules-stderr".into())
            .spawn(move || {
                for line in receiver {
                    if writeln!(io::stderr().lock(), "{line}").is_err() {
                        break;
                    }
                }
            });
        Self(worker.ok().map(|_| sender))
    }

    fn line(&self, message: fmt::Arguments<'_>) {
        if let Some(sender) = &self.0 {
            let mut line = message.to_string();
            line.truncate(line.floor_char_boundary(512));
            let _ = sender.try_send(line);
        }
    }
}

struct Trace {
    remaining: usize,
    diagnostics: Diagnostics,
}

impl Trace {
    fn new(diagnostics: Diagnostics) -> Self {
        let enabled =
            env::var_os(application_identity::TRACE_IPC_ENV).is_some_and(|value| value == "1");
        Self {
            remaining: if enabled { 2048 } else { 0 },
            diagnostics,
        }
    }

    fn request(&mut self, id: &str, timing: dispatcher::Timing, egress: Duration, size: usize) {
        if self.remaining > 0 {
            self.remaining -= 1;
            // The Dashboard generates UUIDs. Pseudonymize arbitrary caller IDs
            // while retaining a stable identifier for local correlation.
            let id = if id.len() == 36
                && id.bytes().enumerate().all(|(index, byte)| {
                    if matches!(index, 8 | 13 | 18 | 23) {
                        byte == b'-'
                    } else {
                        byte.is_ascii_hexdigit()
                    }
                }) {
                Cow::Borrowed(id)
            } else {
                Cow::Owned(format!("{:x}", Sha256::digest(id.as_bytes()))[..16].to_owned())
            };
            self.diagnostics.line(format_args!(
                "cirnetrules-ipc id={id} method={} queued_us={} dispatch_us={} storage_pending={} serialize_us={} egress_us={} response_bytes={size}",
                timing.method, timing.queued.as_micros(), timing.execution.as_micros(), timing.storage_pending,
                timing.serialization.as_micros(), egress.as_micros(),
            ));
        }
    }
}

fn frame(reader: &mut impl BufRead) -> io::Result<Option<Vec<u8>>> {
    let mut frame = Vec::new();
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return if frame.is_empty() {
                Ok(None)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Incomplete frame",
                ))
            };
        }
        let newline = available.iter().position(|&byte| byte == b'\n');
        let count = newline.unwrap_or(available.len());
        if frame.len() + count > MAX_FRAME {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Frame exceeds limit",
            ));
        }
        frame.extend_from_slice(&available[..count]);
        reader.consume(count + usize::from(newline.is_some()));
        if newline.is_some() {
            return Ok(Some(frame));
        }
    }
}

struct LocalScheduler {
    stop: mpsc::Sender<()>,
    worker: Option<thread::JoinHandle<()>>,
    diagnostics: Diagnostics,
}

impl LocalScheduler {
    fn start(mut db: Database, diagnostics: Diagnostics) -> io::Result<Self> {
        let (stop, stopped) = mpsc::channel();
        let worker_diagnostics = diagnostics.clone();
        let worker = thread::Builder::new()
            .name("cirnetrules-scheduler".into())
            .spawn(move || {
                loop {
                    tick(&mut db, Some(&worker_diagnostics));
                    if stopped.recv_timeout(TICK) != Err(mpsc::RecvTimeoutError::Timeout) {
                        break;
                    }
                }
            })?;
        Ok(Self {
            stop,
            worker: Some(worker),
            diagnostics,
        })
    }
}

impl Drop for LocalScheduler {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(worker) = self.worker.take() {
            let until = Instant::now() + Duration::from_secs(1);
            while !worker.is_finished() && Instant::now() < until {
                thread::sleep(Duration::from_millis(10));
            }
            if worker.is_finished() {
                let _ = worker.join();
            } else {
                self.diagnostics.line(format_args!(
                    "Scheduler shutdown did not finish before process exit"
                ));
            }
        }
    }
}

fn tick(db: &mut Database, diagnostics: Option<&Diagnostics>) {
    if let Err(error) = automations::tick(db) {
        if let Some(diagnostics) = diagnostics {
            diagnostics.line(format_args!("Scheduler tick deferred: {}", error.code));
        } else {
            eprintln!("Scheduler tick deferred: {}", error.code);
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let mut args = env::args_os().skip(1);
    let mode = args
        .next()
        .ok_or("Expected --stdio, --scheduler, or --version")?;
    if mode == "--version" && args.len() == 0 {
        println!(
            "{}",
            serde_json::to_string(&info()).map_err(|error| error.to_string())?
        );
        return Ok(ExitCode::SUCCESS);
    }
    let (path, use_default) = match args.next() {
        Some(flag) if flag == "--data-dir" => {
            let path = PathBuf::from(args.next().ok_or("--data-dir requires a path")?);
            if !path.is_absolute() || args.len() != 0 {
                return Err("Expected one absolute --data-dir path".into());
            }
            (path, false)
        }
        None => (
            storage::default_data_dir().map_err(|error| error.to_string())?,
            true,
        ),
        _ => return Err("Unknown runtime argument".into()),
    };
    let locations = storage::identity::Locations::new(path, use_default);
    let identity = match mode.to_str() {
        Some("--identity-preview") => Some(locations.preview()),
        Some("--identity-migrate") => {
            use std::sync::{
                Arc,
                atomic::{AtomicBool, Ordering},
            };
            let cancelled = Arc::new(AtomicBool::new(false));
            let watcher = Arc::clone(&cancelled);
            thread::spawn(move || {
                let mut line = String::new();
                if io::stdin().lock().take(16).read_line(&mut line).is_ok()
                    && line.trim() == "cancel"
                {
                    watcher.store(true, Ordering::Relaxed);
                }
            });
            Some(locations.migrate_checked(&cancelled, |phase| eprintln!("identity:{phase}")))
        }
        Some("--identity-use-current") => Some(locations.use_current()),
        Some("--stdio" | "--scheduler") => None,
        _ => return Err("Expected --stdio or --scheduler".into()),
    };
    if let Some(result) = identity {
        println!("{}", {
            let mut result =
                result.unwrap_or_else(|error| json!({"status":"ERROR","error":error.wire()}));
            result["product_id"] = json!(PRODUCT_ID);
            result["protocol"] = json!(PROTOCOL);
            result
        });
        return Ok(ExitCode::SUCCESS);
    }
    let db = locations.open().map_err(|error| error.to_string())?;
    if mode == "--stdio" {
        return Ok(dispatcher::stdio(db));
    }
    let mut db = db;
    loop {
        tick(&mut db, None);
        thread::sleep(TICK);
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn bounded_frames_handle_partial_reads_and_multiple_requests() {
        let mut reader = BufReader::with_capacity(2, Cursor::new(b"abc\nxyz\n"));
        assert_eq!(
            frame(&mut reader).unwrap().as_deref(),
            Some(b"abc".as_slice())
        );
        assert_eq!(
            frame(&mut reader).unwrap().as_deref(),
            Some(b"xyz".as_slice())
        );
        assert_eq!(frame(&mut reader).unwrap(), None);
        let mut partial = Cursor::new(b"{");
        assert_eq!(
            frame(&mut partial).unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
        let mut large = Cursor::new(vec![b'x'; MAX_FRAME + 1]);
        assert_eq!(
            frame(&mut large).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
}
