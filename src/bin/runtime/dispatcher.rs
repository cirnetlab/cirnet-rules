use super::{
    API_NAMESPACE, AppError, Database, Diagnostics, LocalScheduler, MAX_FRAME, PROTOCOL, Request,
    TICK, Trace, dispatch, frame, info, valid_id,
};
use crate::{
    dashboard::{
        integrations::ProbeQueue,
        request_budget::{self, Budget},
        storage::decode,
    },
    operations::Operations,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, VecDeque},
    io::{self, Write},
    process::ExitCode,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const EVENT_CAPACITY: usize = 16;
const LANE_QUEUE: usize = 8;
const REGULAR_OUTSTANDING: usize = 32;
const REJECTION_RESERVE: usize = 4;
const CONTROL_RESERVE: usize = 8;
const REQUEST_BUDGET: Duration = Duration::from_secs(12);
const PIPE_STALL: Duration = Duration::from_secs(2);
const SHUTDOWN_BUDGET: Duration = Duration::from_secs(3);

type Outcome = Result<Value, AppError>;

#[derive(Clone, Copy, PartialEq)]
enum Lane {
    Write,
    Interactive,
    SlowRead,
}

impl Lane {
    fn name(self) -> &'static str {
        match self {
            Self::Write => "writer",
            Self::Interactive => "interactive",
            Self::SlowRead => "slow-read",
        }
    }

    fn is_read(self) -> bool {
        self != Self::Write
    }
}

#[derive(Clone, Copy)]
enum Route {
    Control,
    Deferred,
    Storage,
    Worker(Lane),
}

// New methods require an explicit scheduling decision. In particular, names
// ending in "preview" or "review" do not determine whether they write.
fn route(method: &str) -> Option<Route> {
    use Lane::{Interactive, SlowRead, Write};
    use Route::{Control, Deferred, Storage, Worker};
    Some(match method {
        "runtime.info" | "request.cancel" | "operation.get" | "operation.cancel" => Control,
        "storage.backup" | "storage.verify" | "integration.probe" => Deferred,
        "storage.chats"
        | "storage.scan"
        | "storage.backup.start"
        | "storage.verify.start"
        | "storage.compact.start" => Storage,

        "conversation.get"
        | "conversation.list"
        | "conversation.events"
        | "preferences.get"
        | "integration.list" => Worker(Interactive),

        "conversation.resume"
        | "repo.list"
        | "repo.preview"
        | "repo.health"
        | "legacy.preview"
        | "legacy.inspect"
        | "automation.list"
        | "automation.runs"
        | "automation.events"
        | "automation.templates"
        | "automation.preview"
        | "governance.asset.list"
        | "governance.asset.get"
        | "governance.revision.validate"
        | "governance.revision.compare"
        | "governance.revision.get"
        | "governance.change.history"
        | "governance.revision.history"
        | "governance.assignment.list"
        | "governance.profile.list"
        | "governance.effective.preview"
        | "governance.asset.export"
        | "governance.asset.import.preview"
        | "integration.import_preview" => Worker(SlowRead),

        "conversation.create"
        | "conversation.send"
        | "conversation.rename"
        | "conversation.archive"
        | "conversation.unarchive"
        | "conversation.delete.preview"
        | "conversation.delete"
        | "conversation.cancel"
        | "message.delete.preview"
        | "message.delete"
        | "repo.add"
        | "repo.select"
        | "repo.relink"
        | "repo.rename"
        | "repo.archive"
        | "preferences.set"
        | "legacy.apply"
        | "legacy.link"
        | "automation.create"
        | "automation.update"
        | "automation.pause"
        | "automation.delete"
        | "automation.run"
        | "automation.cancel"
        | "governance.asset.create"
        | "governance.asset.rename"
        | "governance.asset.duplicate"
        | "governance.draft.save"
        | "governance.draft.discard"
        | "governance.change.review"
        | "governance.change.publish"
        | "governance.change.rollback"
        | "governance.asset.archive"
        | "governance.asset.restore"
        | "governance.asset.delete"
        | "governance.assignment.update"
        | "governance.profile.default"
        | "governance.asset.import.apply" => Worker(Write),

        _ => return None,
    })
}

enum Event {
    Input(Instant, Vec<u8>),
    InputClosed(io::Result<()>),
    Completed(Lane, Response),
    Written(io::Result<Instant>),
}

struct Work {
    request: Request,
    enqueued: Instant,
    budget: Budget,
    storage_pending: usize,
}

struct Running {
    id: String,
    budget: Budget,
}

struct Worker {
    lane: Lane,
    commands: Option<mpsc::SyncSender<Work>>,
    thread: thread::JoinHandle<()>,
    queue: VecDeque<Work>,
    running: Option<Running>,
}

impl Worker {
    fn start(lane: Lane, mut db: Database, events: mpsc::SyncSender<Event>) -> io::Result<Self> {
        let (commands, jobs) = mpsc::sync_channel::<Work>(1);
        let thread = thread::Builder::new()
            .name(format!("cirnetrules-{}", lane.name()))
            .spawn(move || {
                for job in jobs {
                    let started = Instant::now();
                    let Request {
                        id, method, params, ..
                    } = job.request;
                    let execute = |db: &mut Database| {
                        job.budget.check()?;
                        request_budget::scoped(&job.budget, || dispatch(db, &method, params))
                    };
                    let result = if lane.is_read() {
                        let budget = job.budget.clone();
                        let result = db.with_read_cancel(move || budget.check().is_err(), execute);
                        job.budget.finish_read().and(result)
                    } else {
                        // Once a mutation has begun, always return the handler's
                        // actual result. A commit cannot be cancelled retroactively.
                        execute(&mut db)
                    };
                    let timing = Timing::new(method, job.enqueued, started, job.storage_pending);
                    let response = Response::new(id, result, Some(timing));
                    if events.send(Event::Completed(lane, response)).is_err() {
                        break;
                    }
                }
            })?;
        Ok(Self {
            lane,
            commands: Some(commands),
            thread,
            queue: VecDeque::new(),
            running: None,
        })
    }

    fn pump(&mut self) -> io::Result<()> {
        if self.thread.is_finished() {
            return Err(io::Error::other("The runtime worker stopped"));
        }
        if self.running.is_none()
            && let Some(job) = self.queue.pop_front()
        {
            let running = Running {
                id: job.request.id.clone(),
                budget: job.budget.clone(),
            };
            self.commands
                .as_ref()
                .expect("worker commands available during admission")
                .try_send(job)
                .map_err(|_| io::Error::other("The runtime worker stopped"))?;
            self.running = Some(running);
        }
        Ok(())
    }

    fn stop(&mut self) {
        self.queue.clear();
        if let Some(job) = &self.running {
            job.budget.cancel();
        }
        self.commands.take();
    }
}

pub(super) struct Timing {
    pub(super) method: String,
    pub(super) queued: Duration,
    pub(super) execution: Duration,
    pub(super) serialization: Duration,
    pub(super) storage_pending: usize,
}

impl Timing {
    fn new(method: String, enqueued: Instant, started: Instant, storage_pending: usize) -> Self {
        Self {
            method,
            queued: started.duration_since(enqueued),
            execution: started.elapsed(),
            serialization: Duration::ZERO,
            storage_pending,
        }
    }
}

struct Response {
    id: String,
    bytes: Vec<u8>,
    ready: Instant,
    timing: Option<Timing>,
}

impl Response {
    fn new(id: String, outcome: Outcome, mut timing: Option<Timing>) -> Self {
        let started = Instant::now();
        let bytes = encode(&id, outcome).unwrap_or_else(|_| {
            encode(
                &id,
                Err(AppError::new(
                    "RESPONSE_LIMIT",
                    "Use pagination or a narrower request",
                )),
            )
            .expect("response limit error fits in a frame")
        });
        if let Some(timing) = &mut timing {
            timing.serialization = started.elapsed();
        }
        Self {
            id,
            bytes,
            timing,
            ready: Instant::now(),
        }
    }
}

fn encode(id: &str, outcome: Outcome) -> serde_json::Result<Vec<u8>> {
    struct FrameBuffer(Vec<u8>);

    impl Write for FrameBuffer {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if bytes.len() > MAX_FRAME - self.0.len() {
                return Err(io::Error::other("Response frame exceeds the limit"));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[derive(Serialize)]
    struct Envelope<'a, T> {
        v: u8,
        id: &'a str,
        ok: bool,
        #[serde(flatten)]
        outcome: T,
    }

    #[derive(Serialize)]
    struct Success {
        result: Value,
    }

    #[derive(Serialize)]
    struct Failure {
        error: Value,
    }

    let mut buffer = FrameBuffer(Vec::new());
    match outcome {
        Ok(result) => serde_json::to_writer(
            &mut buffer,
            &Envelope {
                v: PROTOCOL,
                id,
                ok: true,
                outcome: Success { result },
            },
        )?,
        Err(issue) => serde_json::to_writer(
            &mut buffer,
            &Envelope {
                v: PROTOCOL,
                id,
                ok: false,
                outcome: Failure {
                    error: issue.wire(),
                },
            },
        )?,
    }
    buffer.0.push(b'\n');
    Ok(buffer.0)
}

struct Writing {
    id: String,
    control: bool,
    ready: Instant,
    started: Instant,
    size: usize,
    timing: Option<Timing>,
}

struct Coordinator {
    workers: [Worker; 3],
    operations: Operations,
    probes: ProbeQueue,
    trace: Trace,
    output: mpsc::SyncSender<Vec<u8>>,
    outgoing: VecDeque<Response>,
    writing: Option<Writing>,
    outstanding: usize,
    regular: usize,
    ids: HashMap<String, bool>,
    input_closed: Option<Instant>,
}

impl Coordinator {
    fn run(&mut self, events: &mpsc::Receiver<Event>) -> io::Result<()> {
        loop {
            self.reap_deferred();
            for worker in &mut self.workers {
                while let Some(work) = worker.queue.front()
                    && let Err(error) = work.budget.check()
                {
                    let work = worker.queue.pop_front().expect("expired queued job");
                    let timing = Timing::new(
                        work.request.method,
                        work.enqueued,
                        Instant::now(),
                        work.storage_pending,
                    );
                    self.outgoing.push_back(Response::new(
                        work.request.id,
                        Err(error),
                        Some(timing),
                    ));
                }
                worker.pump()?;
            }
            self.deliver()?;
            if let Some(closed) = self.input_closed {
                if self.outstanding == 0 {
                    return Ok(());
                }
                if closed.elapsed() >= REQUEST_BUDGET + SHUTDOWN_BUDGET {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "Runtime work exceeded the input drain deadline",
                    ));
                }
            }
            let poll = if self.operations.busy() || self.probes.busy() {
                Duration::from_millis(5)
            } else {
                TICK
            };
            let wait = self.writing.as_ref().map_or(poll, |frame| {
                poll.min(PIPE_STALL.saturating_sub(frame.started.elapsed()))
            });
            match events.recv_timeout(wait) {
                Ok(Event::Input(enqueued, bytes)) => self.accept(enqueued, &bytes)?,
                Ok(Event::InputClosed(result)) => {
                    result?;
                    self.input_closed = Some(Instant::now());
                }
                Ok(Event::Completed(lane, response)) => {
                    let worker = self.worker(lane);
                    worker.running.take();
                    self.outgoing.push_back(response);
                }
                Ok(Event::Written(result)) => self.written(result?)?,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        "Operator channel closed",
                    ));
                }
            }
            if self
                .writing
                .as_ref()
                .is_some_and(|frame| frame.started.elapsed() >= PIPE_STALL)
            {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Operator response pipe stalled",
                ));
            }
        }
    }

    fn worker(&mut self, lane: Lane) -> &mut Worker {
        &mut self.workers[lane as usize]
    }

    fn accept(&mut self, enqueued: Instant, bytes: &[u8]) -> io::Result<()> {
        let started = Instant::now();
        let request = match parse(bytes) {
            Ok(request) => request,
            Err((id, issue)) => {
                self.track(&id, false)?;
                self.outgoing.push_back(Response::new(id, Err(issue), None));
                return Ok(());
            }
        };
        let Request {
            id, method, params, ..
        } = request;
        let route = route(&method);
        let control = matches!(route, Some(Route::Control));
        self.track(&id, control)?;
        let Some(route) = route else {
            let code = if matches!(
                method.split_once('.'),
                Some(("repo" | "legacy" | "storage" | "preferences", _))
            ) {
                "UNKNOWN_METHOD"
            } else {
                "METHOD_NOT_FOUND"
            };
            self.outgoing.push_back(Response::new(
                id,
                Err(AppError::new(code, "Unknown local runtime operation")),
                None,
            ));
            return Ok(());
        };
        let storage_pending = self.operations.pending();
        let timing = |method| Timing::new(method, enqueued, started, storage_pending);
        if self.regular > REGULAR_OUTSTANDING && !control {
            self.outgoing
                .push_back(Response::new(id, Err(overloaded()), Some(timing(method))));
            return Ok(());
        }
        match route {
            Route::Worker(lane) => {
                let worker = self.worker(lane);
                if worker.queue.len() == LANE_QUEUE {
                    self.outgoing.push_back(Response::new(
                        id,
                        Err(overloaded()),
                        Some(timing(method)),
                    ));
                } else {
                    worker.queue.push_back(Work {
                        request: Request {
                            v: PROTOCOL,
                            id,
                            method,
                            params,
                        },
                        enqueued,
                        budget: Budget::new(enqueued + REQUEST_BUDGET),
                        storage_pending,
                    });
                }
            }
            Route::Control => {
                let result = match method.as_str() {
                    "runtime.info" => Ok(info()),
                    "request.cancel" => self.cancel(params),
                    _ => self
                        .operations
                        .inspect(params, method == "operation.cancel"),
                };
                self.outgoing
                    .push_back(Response::new(id, result, Some(timing(method))));
            }
            Route::Deferred | Route::Storage => {
                let deferred = matches!(route, Route::Deferred);
                let result = if method == "integration.probe" {
                    self.probes.submit(&id, params).map(|()| Value::Null)
                } else {
                    self.operations
                        .start(&method, params, deferred.then_some(id.as_str()))
                };
                if !deferred || result.is_err() {
                    self.outgoing
                        .push_back(Response::new(id, result, Some(timing(method))));
                }
            }
        }
        Ok(())
    }

    fn track(&mut self, id: &str, control: bool) -> io::Result<()> {
        let regular_capacity = REGULAR_OUTSTANDING + REJECTION_RESERVE;
        if self.outstanding >= regular_capacity + CONTROL_RESERVE
            || (!control && self.regular >= regular_capacity)
        {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Operator response budget exhausted",
            ));
        }
        if !id.is_empty() && self.ids.insert(id.to_owned(), control).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Duplicate pending request ID",
            ));
        }
        self.outstanding += 1;
        self.regular += usize::from(!control);
        Ok(())
    }

    fn cancel(&mut self, params: Value) -> Outcome {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Input {
            id: String,
        }
        let input: Input = decode(params)?;
        if !valid_id(&input.id) {
            return Err(AppError::new(
                "INVALID_INPUT",
                "Target request ID is invalid",
            ));
        }
        for worker in self
            .workers
            .iter_mut()
            .filter(|worker| worker.lane.is_read())
        {
            if let Some(position) = worker
                .queue
                .iter()
                .position(|job| job.request.id == input.id)
            {
                let job = worker.queue.remove(position).expect("queued request");
                let timing = Timing::new(
                    job.request.method,
                    job.enqueued,
                    Instant::now(),
                    job.storage_pending,
                );
                self.outgoing.push_back(Response::new(
                    job.request.id,
                    Err(request_budget::cancelled()),
                    Some(timing),
                ));
                return Ok(json!({"state":"CANCELLED"}));
            }
            if let Some(job) = &worker.running
                && job.id == input.id
                && job.budget.cancel()
            {
                return Ok(json!({"state":"CANCELLATION_REQUESTED"}));
            }
        }
        Ok(json!({"state":"NOT_PENDING"}))
    }

    fn reap_deferred(&mut self) {
        for (id, outcome) in self.operations.reap() {
            self.outgoing.push_back(Response::new(id, outcome, None));
        }
        if let Some((id, outcome)) = self.probes.reap() {
            self.outgoing.push_back(Response::new(id, outcome, None));
        }
        if self
            .input_closed
            .is_some_and(|closed| closed.elapsed() >= PIPE_STALL)
        {
            for id in self.operations.take_deferred() {
                let outcome = Err(AppError::new(
                    "OPERATION_OUTCOME_UNKNOWN",
                    "The operator channel closed before the storage outcome was confirmed.",
                ));
                self.outgoing.push_back(Response::new(id, outcome, None));
            }
        }
    }

    fn deliver(&mut self) -> io::Result<()> {
        if self.writing.is_none()
            && let Some(response) = self.outgoing.pop_front()
        {
            // A peer can receive the whole frame before the writer's local
            // acknowledgment arrives. Free the ID at handoff while keeping
            // the response capacity charged until the physical write ends.
            let control = self.ids.remove(&response.id).unwrap_or(false);
            let size = response.bytes.len();
            self.output.try_send(response.bytes).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Operator response writer stopped",
                )
            })?;
            self.writing = Some(Writing {
                id: response.id,
                control,
                ready: response.ready,
                started: Instant::now(),
                size,
                timing: response.timing,
            });
        }
        Ok(())
    }

    fn written(&mut self, delivered: Instant) -> io::Result<()> {
        let written = self
            .writing
            .take()
            .ok_or_else(|| io::Error::other("Unexpected operator response acknowledgment"))?;
        self.outstanding -= 1;
        self.regular -= usize::from(!written.control);
        if let Some(timing) = written.timing {
            self.trace.request(
                &written.id,
                timing,
                delivered.duration_since(written.ready),
                written.size,
            );
        }
        Ok(())
    }
}

fn parse(bytes: &[u8]) -> Result<Request, (String, AppError)> {
    let mut request: Request = serde_json::from_slice(bytes).map_err(|_| {
        (
            String::new(),
            AppError::new("INVALID_REQUEST", "Expected one valid JSON request frame"),
        )
    })?;
    if !valid_id(&request.id) {
        return Err((
            String::new(),
            AppError::new("INVALID_REQUEST", "Request id is invalid"),
        ));
    }
    if request.v != PROTOCOL {
        return Err((
            request.id,
            AppError::new("PROTOCOL_VERSION", "Unsupported local protocol version"),
        ));
    }
    if !valid_id(&request.method) || !request.params.is_object() {
        return Err((
            request.id,
            AppError::new("INVALID_REQUEST", "Expected a method and object parameters"),
        ));
    }
    let Some(method) = request.method.strip_prefix(API_NAMESPACE) else {
        return Err((
            request.id,
            AppError::new(
                "METHOD_NOT_FOUND",
                "Unexpected local runtime method namespace",
            ),
        ));
    };
    request.method = method.to_owned();
    Ok(request)
}

fn overloaded() -> AppError {
    AppError::new(
        "REQUEST_LIMIT",
        "The runtime request queue is full. Try again when work finishes.",
    )
}

pub(super) fn stdio(db: Database) -> ExitCode {
    let diagnostics = Diagnostics::start();
    match run_stdio(db, &diagnostics) {
        Ok(status) => status,
        Err(error) => {
            diagnostics.line(format_args!("{error}"));
            ExitCode::FAILURE
        }
    }
}

fn run_stdio(db: Database, diagnostics: &Diagnostics) -> io::Result<ExitCode> {
    let factory = db.connection_factory();
    let nonce = db.operation_nonce().map_err(io::Error::other)?;
    let operations = Operations::new(db.data_dir().to_owned(), &nonce, diagnostics.clone())?;
    let probes = ProbeQueue::new()?;
    let scheduler = LocalScheduler::start(
        factory.open_write().map_err(io::Error::other)?,
        diagnostics.clone(),
    )?;
    let (events, incoming) = mpsc::sync_channel(EVENT_CAPACITY);
    let workers = [
        Worker::start(Lane::Write, db, events.clone())?,
        Worker::start(
            Lane::Interactive,
            factory.open_read().map_err(io::Error::other)?,
            events.clone(),
        )?,
        Worker::start(
            Lane::SlowRead,
            factory.open_read().map_err(io::Error::other)?,
            events.clone(),
        )?,
    ];

    let input_events = events.clone();
    let input = thread::Builder::new()
        .name("cirnetrules-stdin".into())
        .spawn(move || {
            let stdin = io::stdin();
            let mut input = stdin.lock();
            let result = loop {
                match frame(&mut input) {
                    Ok(Some(bytes)) => {
                        if input_events
                            .send(Event::Input(Instant::now(), bytes))
                            .is_err()
                        {
                            return;
                        }
                    }
                    Ok(None) => break Ok(()),
                    Err(issue) => break Err(issue),
                }
            };
            let _ = input_events.send(Event::InputClosed(result));
        })?;

    let (output, outgoing) = mpsc::sync_channel::<Vec<u8>>(1);
    let writer = thread::Builder::new()
        .name("cirnetrules-stdout".into())
        .spawn(move || {
            let stdout = io::stdout();
            let mut output = stdout.lock();
            for bytes in outgoing {
                let result = output
                    .write_all(&bytes)
                    .and_then(|()| output.flush())
                    .map(|()| Instant::now());
                let failed = result.is_err();
                if events.send(Event::Written(result)).is_err() || failed {
                    break;
                }
            }
        })?;

    let mut coordinator = Coordinator {
        workers,
        operations,
        probes,
        trace: Trace::new(diagnostics.clone()),
        output,
        outgoing: VecDeque::new(),
        writing: None,
        outstanding: 0,
        regular: 0,
        ids: HashMap::new(),
        input_closed: None,
    };
    let result = coordinator.run(&incoming);
    if let Err(error) = &result {
        diagnostics.line(format_args!("{error}"));
    }
    drop(incoming); // Unblock producers even when stdout is stuck.
    for worker in &mut coordinator.workers {
        worker.stop();
    }
    let until = Instant::now() + SHUTDOWN_BUDGET;
    drop(scheduler);
    let Coordinator {
        workers, output, ..
    } = coordinator;
    drop(output);
    for worker in workers {
        while !worker.thread.is_finished() && Instant::now() < until {
            thread::sleep(Duration::from_millis(5));
        }
        if worker.thread.is_finished() {
            let _ = worker.thread.join();
        } else {
            diagnostics.line(format_args!(
                "Runtime {} worker did not finish before process exit",
                worker.lane.name()
            ));
        }
    }
    let status = if result.is_ok() {
        // EOF and every write acknowledged. Both stdio threads can now exit.
        let _ = input.join();
        let _ = writer.join();
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    };
    // On pipe failure the OS may still block one stdio thread. The inherited
    // handles close at process exit, independently of worker cancellation.
    Ok(status)
}
