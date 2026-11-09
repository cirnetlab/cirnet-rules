//! Private local operator transport. All database access stays in the runtime.
#[path = "../application_identity.rs"]
mod application_identity;
#[path = "../dashboard/mod.rs"]
mod dashboard;
#[path = "../dashboard/operations.rs"]
mod operations;
#[cfg(test)]
#[path = "../test_support.rs"]
mod test_support;

use application_identity::{GENERATION, VERSION, release_name};
use dashboard::{
    automations, catalog, preferences, sessions,
    storage::{self, AppError, Database},
};
use operations::Operations;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    env,
    io::{self, BufRead, Write},
    path::PathBuf,
    process::ExitCode,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const PROTOCOL: u8 = 1;
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
        "product_generation":GENERATION, "version":VERSION,
        "release_name":release_name(), "protocol":PROTOCOL,
        "operator_channel":"private_inherited_stdio", "release_actions":"RELEASE_BLOCKED",
        "native_agent":{"codex":"HANDOFF_ONLY","claude":"HANDOFF_ONLY","antigravity":"HANDOFF_ONLY"},
        "execution_enforcement":{"READ_ONLY_BUILTIN":"AVAILABLE","TRUSTED_LOCAL_PROCESS":"LIMITED",
            "ISOLATED_EXECUTION":"BLOCKED","authenticated_independent_review":"BLOCKED"},
        "scheduler":"runtime while running; manual --scheduler supports a separate foreground process",
        "storage_jobs":operations::limits()
    })
}

fn dispatch(db: &mut Database, method: &str, params: Value) -> Result<Value, AppError> {
    match method.split_once('.').map(|(kind, _)| kind) {
        Some("runtime") if method == "runtime.info" => Ok(info()),
        Some("repo") => catalog::dispatch(db, method, params),
        Some("storage") => storage::dispatch(db, method, params),
        Some("preferences") => preferences::dispatch(db, method, params),
        Some("legacy") => dashboard::legacy::dispatch(db, method, params),
        Some("conversation" | "events" | "message") => sessions::dispatch(db, method, params),
        Some("automation") => automations::dispatch(db, method, params),
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

fn envelope(id: &str, outcome: Result<Value, AppError>) -> Value {
    match outcome {
        Ok(result) => json!({"v":PROTOCOL,"id":id,"ok":true,"result":result}),
        Err(issue) => json!({"v":PROTOCOL,"id":id,"ok":false,"error":issue.wire()}),
    }
}

struct Trace {
    remaining: usize,
    lines: Option<mpsc::SyncSender<String>>,
}

impl Trace {
    fn new() -> Self {
        let enabled = env::var_os("CGACP_TRACE_IPC").is_some_and(|value| value == "1");
        let lines = if enabled {
            let (sender, receiver) = mpsc::sync_channel::<String>(64);
            thread::spawn(move || {
                for line in receiver {
                    if writeln!(io::stderr().lock(), "{line}").is_err() {
                        break;
                    }
                }
            });
            Some(sender)
        } else {
            None
        };
        Self {
            remaining: 2048,
            lines,
        }
    }

    fn request(
        &mut self,
        id: &str,
        method: &str,
        queued: Duration,
        execution: Duration,
        pending: usize,
    ) {
        if let Some(lines) = &self.lines
            && self.remaining > 0
        {
            self.remaining -= 1;
            let _ = lines.try_send(format!(
                "cgacp-ipc id={id} method={method} queued_us={} dispatch_us={} storage_pending={pending}",
                queued.as_micros(), execution.as_micros(),
            ));
        }
    }
}

fn reply(
    db: &mut Database,
    operations: &mut Operations,
    trace: &mut Trace,
    bytes: &[u8],
    enqueued: Instant,
) -> Option<Value> {
    let started = Instant::now();
    let error =
        |id: &str, code, message: &str| Some(envelope(id, Err(AppError::new(code, message))));
    let Ok(request) = serde_json::from_slice::<Request>(bytes) else {
        return error(
            "",
            "INVALID_REQUEST",
            "Expected one valid JSON request frame",
        );
    };
    if !valid_id(&request.id) {
        return error("", "INVALID_REQUEST", "Request id is invalid");
    }
    if request.v != PROTOCOL {
        return error(
            &request.id,
            "PROTOCOL_VERSION",
            "Unsupported local protocol version",
        );
    }
    if !valid_id(&request.method) || !request.params.is_object() {
        return error(
            &request.id,
            "INVALID_REQUEST",
            "Expected a method and object parameters",
        );
    }
    let legacy = matches!(request.method.as_str(), "storage.backup" | "storage.verify");
    let Request {
        id, method, params, ..
    } = request;
    let outcome = match method.as_str() {
        "storage.chats"
        | "storage.scan"
        | "storage.backup"
        | "storage.verify"
        | "storage.backup.start"
        | "storage.verify.start"
        | "storage.compact.start" => {
            operations.start(&method, params, legacy.then_some(id.as_str()))
        }
        "operation.get" | "operation.cancel" => {
            operations.inspect(params, method == "operation.cancel")
        }
        _ => dispatch(db, &method, params),
    };
    trace.request(
        &id,
        &method,
        started.duration_since(enqueued),
        started.elapsed(),
        operations.pending(),
    );
    if legacy && outcome.is_ok() {
        None
    } else {
        Some(envelope(&id, outcome))
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

fn stdio(db: &mut Database) -> io::Result<()> {
    let nonce = db.operation_nonce().map_err(io::Error::other)?;
    let mut operations = Operations::new(db.data_dir().to_owned(), &nonce)?;
    let scheduler = LocalScheduler::start(db.data_dir().to_owned())?;
    let mut trace = Trace::new();
    let (sender, requests) = mpsc::sync_channel(16);
    thread::spawn(move || {
        let stdin = io::stdin();
        let mut input = stdin.lock();
        loop {
            match frame(&mut input) {
                Ok(Some(bytes)) => {
                    if sender.send(Ok((Instant::now(), bytes))).is_err() {
                        break;
                    }
                }
                Ok(None) => break,
                Err(issue) => {
                    let _ = sender.send(Err(issue));
                    break;
                }
            }
        }
    });
    let (responses, outgoing) = mpsc::sync_channel::<Vec<u8>>(8);
    let (report, delivery) = mpsc::channel();
    thread::spawn(move || {
        let stdout = io::stdout();
        let mut output = stdout.lock();
        let result = (|| -> io::Result<()> {
            for bytes in outgoing {
                output.write_all(&bytes)?;
                output.flush()?;
            }
            Ok(())
        })();
        let _ = report.send(result);
    });
    loop {
        if let Ok(result) = delivery.try_recv() {
            return result;
        }
        for (id, outcome) in operations.reap() {
            send(&responses, envelope(&id, outcome))?;
        }
        let interval = if operations.busy() {
            Duration::from_millis(5)
        } else {
            TICK
        };
        match requests.recv_timeout(interval) {
            Ok(Ok((enqueued, bytes))) => {
                if let Some(response) = reply(db, &mut operations, &mut trace, &bytes, enqueued) {
                    send(&responses, response)?;
                }
            }
            Ok(Err(issue)) => return Err(issue),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                // A one-shot stdio caller may close stdin before reading its legacy response.
                let until = Instant::now() + Duration::from_secs(2);
                while operations.has_deferred() && Instant::now() < until {
                    for (id, outcome) in operations.reap() {
                        send(&responses, envelope(&id, outcome))?;
                    }
                    if operations.has_deferred() {
                        thread::sleep(Duration::from_millis(5));
                    }
                }
                for id in operations.take_deferred() {
                    send(
                        &responses,
                        envelope(
                            &id,
                            Err(AppError::new(
                                "OPERATION_OUTCOME_UNKNOWN",
                                "The operator channel closed before the storage outcome was confirmed.",
                            )),
                        ),
                    )?;
                }
                drop(operations);
                drop(scheduler);
                drop(responses);
                return delivery.recv_timeout(Duration::from_secs(2)).map_err(
                    |error| match error {
                        mpsc::RecvTimeoutError::Timeout => io::Error::new(
                            io::ErrorKind::TimedOut,
                            "Operator response drain exceeded its deadline",
                        ),
                        mpsc::RecvTimeoutError::Disconnected => io::Error::new(
                            io::ErrorKind::BrokenPipe,
                            "Operator response writer stopped",
                        ),
                    },
                )?;
            }
        }
    }
}

fn send(responses: &mpsc::SyncSender<Vec<u8>>, response: Value) -> io::Result<()> {
    let mut serialized = serde_json::to_vec(&response)?;
    if serialized.len() > MAX_FRAME {
        serialized = serde_json::to_vec(&json!({"v":PROTOCOL,"id":response["id"],"ok":false,
            "error":{"code":"RESPONSE_LIMIT","message":"Use pagination or a narrower request"}}))?;
    }
    serialized.push(b'\n');
    // A slow reader applies backpressure through both bounded queues.
    responses
        .send(serialized)
        .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Operator channel closed"))
}

struct LocalScheduler {
    stop: mpsc::Sender<()>,
    worker: Option<thread::JoinHandle<()>>,
}

impl LocalScheduler {
    fn start(data_dir: PathBuf) -> io::Result<Self> {
        let (stop, stopped) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("cgacp-scheduler".into())
            .spawn(move || {
                let mut db = match Database::open(&data_dir) {
                    Ok(db) => db,
                    Err(error) => {
                        eprintln!("Scheduler initialization failed: {}", error.code);
                        return;
                    }
                };
                loop {
                    tick(&mut db);
                    if stopped.recv_timeout(TICK) != Err(mpsc::RecvTimeoutError::Timeout) {
                        break;
                    }
                }
            })?;
        Ok(Self {
            stop,
            worker: Some(worker),
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
            }
        }
    }
}

fn tick(db: &mut Database) {
    if let Err(error) = automations::tick(db) {
        eprintln!("Scheduler tick deferred: {}", error.code);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let mode = args
        .next()
        .ok_or("Expected --stdio, --scheduler, or --version")?;
    if mode == "--version" && args.len() == 0 {
        println!(
            "{}",
            serde_json::to_string(&info()).map_err(|error| error.to_string())?
        );
        return Ok(());
    }
    if mode != "--stdio" && mode != "--scheduler" {
        return Err("Expected --stdio or --scheduler".into());
    }
    let path = match args.next() {
        Some(flag) if flag == "--data-dir" => {
            let path = PathBuf::from(args.next().ok_or("--data-dir requires a path")?);
            if !path.is_absolute() || args.len() != 0 {
                return Err("Expected one absolute --data-dir path".into());
            }
            path
        }
        None => storage::default_data_dir().map_err(|error| error.to_string())?,
        _ => return Err("Unknown runtime argument".into()),
    };
    let mut db =
        Database::open(&path).map_err(|error| format!("{}: {}", error.code, error.message))?;
    if mode == "--stdio" {
        return stdio(&mut db).map_err(|error| error.to_string());
    }
    loop {
        tick(&mut db);
        thread::sleep(TICK);
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
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
