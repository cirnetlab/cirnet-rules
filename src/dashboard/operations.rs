//! Bounded storage jobs. Each job owns its SQLite connection on one worker.
use super::{
    Diagnostics,
    dashboard::storage::{self, AppError, Empty, decode},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap},
    io,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const MAX_ACTIVE: usize = 3;
const MAX_RETAINED: usize = 32;
const JOB_LIMIT: Duration = Duration::from_secs(60);
const JOIN_LIMIT: Duration = Duration::from_secs(2);

pub(crate) fn limits() -> Value {
    json!({"workers":1,"active_limit":MAX_ACTIVE,"retained_limit":MAX_RETAINED,
        "deadline_seconds":JOB_LIMIT.as_secs()})
}

#[derive(Clone, Copy)]
struct Progress {
    phase: &'static str,
    completed: u64,
    total: Option<u64>,
}

pub(crate) struct JobControl {
    cancelled: AtomicBool,
    running: AtomicBool,
    progress: Mutex<Progress>,
    deadline: Instant,
}

impl JobControl {
    fn new() -> Self {
        Self {
            cancelled: AtomicBool::new(false),
            running: AtomicBool::new(false),
            progress: Mutex::new(Progress {
                phase: "QUEUED",
                completed: 0,
                total: None,
            }),
            deadline: Instant::now() + JOB_LIMIT,
        }
    }

    fn begin(&self) {
        self.running.store(true, Ordering::Release);
    }

    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    fn cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub(crate) fn check(&self) -> Result<(), AppError> {
        if self.cancelled() {
            return Err(AppError::new("CANCELLED", "The operation was cancelled."));
        }
        if Instant::now() >= self.deadline {
            return Err(AppError::new(
                "OPERATION_TIMEOUT",
                "The storage job exceeded its time budget.",
            ));
        }
        Ok(())
    }

    pub(crate) fn update(&self, phase: &'static str, completed: u64, total: Option<u64>) {
        *self.progress.lock().unwrap() = Progress {
            phase,
            completed,
            total,
        };
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Verify {
    backup: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Compact {
    pub expected_page_count: u64,
    pub expected_free_pages: u64,
}

enum Job {
    ChatUsage,
    Scan,
    Backup,
    Verify(Option<String>),
    Compact(Compact),
}

impl Job {
    fn parse(method: &str, params: Value) -> Result<Self, AppError> {
        match method {
            "storage.chats" => {
                let _: Empty = decode(params)?;
                Ok(Self::ChatUsage)
            }
            "storage.scan" => {
                let _: Empty = decode(params)?;
                Ok(Self::Scan)
            }
            "storage.backup" | "storage.backup.start" => {
                let _: Empty = decode(params)?;
                Ok(Self::Backup)
            }
            "storage.verify" | "storage.verify.start" => {
                let input: Verify = decode(params)?;
                if let Some(name) = &input.backup {
                    storage::validate_backup_name(name)?;
                }
                Ok(Self::Verify(input.backup))
            }
            "storage.compact.start" => Ok(Self::Compact(decode(params)?)),
            _ => Err(AppError::new("METHOD_NOT_FOUND", "Unknown storage job.")),
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Self::ChatUsage => "CHAT_USAGE",
            Self::Scan => "SCAN",
            Self::Backup => "BACKUP",
            Self::Verify(_) => "VERIFY",
            Self::Compact(_) => "COMPACT",
        }
    }

    fn run(self, data: &std::path::Path, control: Arc<JobControl>) -> Result<Value, AppError> {
        control.begin();
        control.check()?;
        let outcome = match self {
            Self::ChatUsage => storage::chat_usage(data, &control),
            Self::Scan => storage::inventory(data, &control),
            Self::Backup => storage::online_backup(data, &control),
            Self::Verify(backup) => storage::verify_job(data, backup.as_deref(), &control),
            Self::Compact(input) => storage::compact(data, input, &control),
        };
        match outcome {
            Err(error) if error.code == "SQLITE_INTERRUPTED" => control.check().and(Err(error)),
            outcome => outcome,
        }
    }
}

struct Work {
    id: u64,
    job: Job,
    control: Arc<JobControl>,
}

type Outcome = Result<Value, AppError>;

struct Record {
    kind: &'static str,
    control: Arc<JobControl>,
    outcome: Option<Outcome>,
}

impl Record {
    fn view(&self, operation_id: &str) -> Value {
        let progress = *self.control.progress.lock().unwrap();
        let state = match &self.outcome {
            Some(Ok(_)) => "SUCCEEDED",
            Some(Err(error)) if error.code == "CANCELLED" => "CANCELLED",
            Some(Err(_)) => "FAILED",
            None if self.control.cancelled() => "CANCEL_REQUESTED",
            None if self.control.running.load(Ordering::Acquire) => "RUNNING",
            None => "QUEUED",
        };
        json!({
            "operation_id": operation_id,
            "kind": self.kind,
            "state": state,
            "progress": {"phase":progress.phase,"completed":progress.completed,"total":progress.total},
            "result": self.outcome.as_ref().and_then(|result| result.as_ref().ok()),
            "error": self.outcome.as_ref().and_then(|result| result.as_ref().err()).map(AppError::wire),
        })
    }
}

pub(crate) struct Operations {
    prefix: String,
    next_id: u64,
    work: Option<mpsc::SyncSender<Work>>,
    completed: mpsc::Receiver<(u64, Outcome)>,
    thread: Option<JoinHandle<()>>,
    records: BTreeMap<u64, Record>,
    deferred: HashMap<u64, String>,
    diagnostics: Diagnostics,
}

impl Operations {
    pub(crate) fn new(data: PathBuf, nonce: &[u8], diagnostics: Diagnostics) -> io::Result<Self> {
        let (work, requests) = mpsc::sync_channel::<Work>(MAX_ACTIVE);
        let (results, completed) = mpsc::sync_channel(MAX_ACTIVE);
        let thread = thread::Builder::new()
            .name("cirnetrules-storage".into())
            .spawn(move || {
                for work in requests {
                    let outcome = work.job.run(&data, work.control);
                    if results.send((work.id, outcome)).is_err() {
                        break;
                    }
                }
            })?;
        Ok(Self {
            prefix: format!(
                "operation-{}-",
                nonce
                    .iter()
                    .take(8)
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            ),
            next_id: 0,
            work: Some(work),
            completed,
            thread: Some(thread),
            records: BTreeMap::new(),
            deferred: HashMap::new(),
            diagnostics,
        })
    }

    pub(crate) fn busy(&self) -> bool {
        self.records.values().any(|record| record.outcome.is_none())
    }

    pub(crate) fn pending(&self) -> usize {
        self.records
            .values()
            .filter(|record| record.outcome.is_none())
            .count()
    }

    pub(crate) fn take_deferred(&mut self) -> impl Iterator<Item = String> {
        std::mem::take(&mut self.deferred).into_values()
    }

    pub(crate) fn start(
        &mut self,
        method: &str,
        params: Value,
        deferred: Option<&str>,
    ) -> Result<Value, AppError> {
        let job = Job::parse(method, params)?;
        let kind = job.kind();
        if self.pending() >= MAX_ACTIVE
            || (kind == "COMPACT" && self.busy())
            || self
                .records
                .values()
                .any(|record| record.kind == "COMPACT" && record.outcome.is_none())
        {
            return Err(AppError::new(
                "OPERATION_BUSY",
                "Storage maintenance is busy. Retry after current work finishes.",
            ));
        }
        while self.records.len() >= MAX_RETAINED {
            let Some((&old, _)) = self
                .records
                .iter()
                .find(|(_, record)| record.outcome.is_some())
            else {
                return Err(AppError::new(
                    "OPERATION_BUSY",
                    "The storage operation budget is full.",
                ));
            };
            self.records.remove(&old);
        }
        self.next_id += 1;
        let id = self.next_id;
        let control = Arc::new(JobControl::new());
        self.work
            .as_ref()
            .ok_or_else(|| {
                AppError::new("OPERATION_UNAVAILABLE", "The storage worker is stopped.")
            })?
            .try_send(Work {
                id,
                job,
                control: control.clone(),
            })
            .map_err(|_| {
                AppError::new(
                    "OPERATION_UNAVAILABLE",
                    "The storage worker is unavailable.",
                )
            })?;
        let record = Record {
            kind,
            control,
            outcome: None,
        };
        let result = record.view(&format!("{}{id}", self.prefix));
        self.records.insert(id, record);
        if let Some(request) = deferred {
            self.deferred.insert(id, request.to_owned());
        }
        Ok(result)
    }

    pub(crate) fn inspect(&self, params: Value, cancel: bool) -> Result<Value, AppError> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Input {
            operation_id: String,
        }
        let input: Input = decode(params)?;
        let id = input
            .operation_id
            .strip_prefix(&self.prefix)
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|id| format!("{}{id}", self.prefix) == input.operation_id)
            .ok_or_else(|| {
                AppError::new("INVALID_INPUT", "The operation identifier is invalid.")
            })?;
        let record = self
            .records
            .get(&id)
            .ok_or_else(|| AppError::new("NOT_FOUND", "This operation is no longer available."))?;
        if cancel && record.outcome.is_none() {
            record.control.cancel();
        }
        Ok(record.view(&input.operation_id))
    }

    pub(crate) fn reap(&mut self) -> Vec<(String, Outcome)> {
        let mut responses = Vec::new();
        while let Ok((id, outcome)) = self.completed.try_recv() {
            if let Some(request) = self.deferred.remove(&id) {
                responses.push((request, outcome.clone()));
            }
            if let Some(record) = self.records.get_mut(&id) {
                record.outcome = Some(outcome);
            }
        }
        responses
    }
}

impl Drop for Operations {
    fn drop(&mut self) {
        for record in self.records.values() {
            if record.outcome.is_none() {
                record.control.cancel();
            }
        }
        self.work.take();
        let until = Instant::now() + JOIN_LIMIT;
        if let Some(thread) = self.thread.take() {
            while !thread.is_finished() && Instant::now() < until {
                thread::sleep(Duration::from_millis(10));
            }
            if thread.is_finished() {
                let _ = thread.join();
            } else {
                self.diagnostics.line(format_args!(
                    "Storage worker did not finish before process exit"
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_queued_cancel_stops_before_opening_storage() {
        let control = Arc::new(JobControl::new());
        control.cancel();
        let outcome = Job::Scan.run(std::path::Path::new("not-a-storage-root"), control);
        assert_eq!(outcome.unwrap_err().code, "CANCELLED");
    }

    #[test]
    fn active_operation_admission_is_bounded() {
        let mut operations = Operations::new(
            PathBuf::from("not-a-storage-root"),
            &[1; 8],
            Diagnostics::default(),
        )
        .unwrap();
        for _ in 0..MAX_ACTIVE {
            operations.start("storage.scan", json!({}), None).unwrap();
        }
        assert_eq!(operations.pending(), MAX_ACTIVE);
        assert_eq!(
            operations
                .start("storage.scan", json!({}), None)
                .unwrap_err()
                .code,
            "OPERATION_BUSY"
        );
        assert_eq!(
            operations
                .inspect(
                    json!({"operation_id":"operation-0202020202020202-1"}),
                    false
                )
                .unwrap_err()
                .code,
            "INVALID_INPUT"
        );
    }

    #[test]
    fn cancellation_interrupts_active_sqlite_work() {
        let control = Arc::new(JobControl::new());
        control.begin();
        let worker_control = Arc::clone(&control);
        let (progressed, progress) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || {
            let conn = rusqlite::Connection::open_in_memory().unwrap();
            conn.progress_handler(
                1000,
                Some(move || {
                    let _ = progressed.try_send(());
                    worker_control.check().is_err()
                }),
            );
            conn.query_row(
                "WITH RECURSIVE numbers(n) AS (
                    VALUES(0) UNION ALL SELECT n+1 FROM numbers WHERE n<100000000
                ) SELECT sum(n) FROM numbers",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(AppError::from)
        });
        progress.recv_timeout(Duration::from_secs(2)).unwrap();
        control.cancel();
        assert_eq!(
            worker.join().unwrap().unwrap_err().code,
            "SQLITE_INTERRUPTED"
        );
    }
}
