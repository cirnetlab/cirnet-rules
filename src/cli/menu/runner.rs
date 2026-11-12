use super::{completion::Completer, text, viewport::Buffer};
use crate::operator::{CheckRequest, check_request};
use cirnetrules::{
    execution,
    operator::{CheckReport, Status},
};
use serde::Deserialize;
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError},
    },
    thread,
    time::Duration,
};

const TIMEOUT: Duration = Duration::from_secs(30 * 60);

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(super) enum Outcome {
    Pass,
    Warning,
    Fail,
    Error,
    Blocked,
    NotRun,
    Unsupported,
    Skipped,
    Cancelled,
    #[serde(skip_deserializing)]
    TimedOut,
}

impl Outcome {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warning => "WARNING",
            Self::Fail => "FAIL",
            Self::Error => "ERROR",
            Self::Blocked => "BLOCKED",
            Self::NotRun => "NOT_RUN",
            Self::Unsupported => "UNSUPPORTED",
            Self::Skipped => "SKIPPED",
            Self::Cancelled => "CANCELLED",
            Self::TimedOut => "TIMED_OUT",
        }
    }

    fn expected_exit(self) -> i32 {
        match self {
            Self::Pass | Self::Warning | Self::Skipped => 0,
            Self::Fail | Self::Error => 1,
            Self::Blocked | Self::NotRun | Self::Unsupported | Self::TimedOut => 2,
            Self::Cancelled => 130,
        }
    }
}

impl From<Status> for Outcome {
    fn from(status: Status) -> Self {
        match status {
            Status::Pass => Self::Pass,
            Status::Warning => Self::Warning,
            Status::Error => Self::Error,
            Status::Blocked => Self::Blocked,
            Status::NotRun => Self::NotRun,
            Status::Unsupported => Self::Unsupported,
            Status::Skipped => Self::Skipped,
            Status::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Clone, Copy, Default)]
enum OutputContract {
    #[cfg(test)]
    External,
    #[default]
    Cnr,
    Check {
        summary: bool,
        pretty: bool,
    },
}

pub(super) struct RunOutput {
    pub(super) buffer: Buffer,
    pub(super) elapsed_ms: u64,
    pub(super) exit_code: Option<i32>,
    pub(super) outcome: Outcome,
    pub(super) completer: Option<Completer>,
    pub(super) check_report: Option<CheckReport>,
}

pub(super) struct Runner {
    receiver: Receiver<Result<RunOutput, String>>,
    cancelled: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
    finished: bool,
}

impl Runner {
    pub(super) fn start(
        args: Vec<String>,
        project: Option<PathBuf>,
        catalog_project: Option<&Path>,
        vi: bool,
    ) -> Result<Self, String> {
        let mut invocation = prepare(&args, vi)?;
        if let Some(mut check) = invocation.check.take() {
            if check.scope.workspace.is_none() {
                check.scope.workspace = project;
            }
            return Ok(Self::spawn_check(check));
        }
        let refresh = catalog_project
            .filter(|_| invocation.refresh_catalog)
            .map(PathBuf::from);
        let executable = std::env::current_exe().map_err(|_| "Cannot locate the CNR executable")?;
        let mut command = Command::new(executable);
        if !invocation.non_interactive {
            command.arg("--non-interactive");
        }
        if let Some(project) = project.filter(|_| invocation.inherit_project) {
            command.arg("--project").arg(project);
        }
        if vi && !invocation.explicit_lang {
            command.args(["--lang", "vi"]);
        }
        if matches!(invocation.output, OutputContract::Check { .. }) && !invocation.explicit_json {
            command.arg("--json");
        }
        command.args(args);
        Ok(Self::spawn(command, refresh, vi, invocation.output))
    }

    fn spawn(
        mut command: Command,
        refresh: Option<PathBuf>,
        vi: bool,
        contract: OutputContract,
    ) -> Self {
        Self::worker(move |signal| {
            execution::run_quiet_native(&mut command, TIMEOUT, || signal.load(Ordering::Relaxed))
                .map(|result| {
                    let (outcome, rendered) = match contract {
                        #[cfg(test)]
                        OutputContract::External => (completed_outcome(&result, false), None),
                        OutputContract::Cnr => (completed_outcome(&result, true), None),
                        OutputContract::Check { summary, pretty } => {
                            checked_report(&result, summary, pretty)
                        }
                    };
                    let notice = result
                        .output_truncated
                        .then(|| text(["[Output truncated]", "[Đầu ra đã được rút gọn]"], vi));
                    let buffer = Buffer::from_streams(
                        sanitize(
                            rendered
                                .as_ref()
                                .map_or(result.output.as_slice(), String::as_bytes),
                        ),
                        &sanitize(&result.error),
                        notice,
                    );
                    let completer = refresh
                        .filter(|_| outcome == Outcome::Pass)
                        .map(|project| Completer::new(Some(&project), vi));
                    RunOutput {
                        buffer,
                        elapsed_ms: result.elapsed_ms,
                        exit_code: result.exit_code,
                        outcome,
                        completer,
                        check_report: None,
                    }
                })
        })
    }

    fn spawn_check(check: CheckRequest) -> Self {
        Self::worker(move |signal| {
            let report = check.finalize(cirnetrules::operator::check(
                &check.target,
                &check.scope,
                check.strict,
                || signal.load(Ordering::Relaxed),
            ));
            let exit = i32::from(report.exit_code);
            let outcome = check_outcome(&report, Some(exit))
                .ok_or("Check report and exit status disagree.")?;
            let rendered = if check.machine {
                serde_json::to_string(&report).map_err(|_| "Cannot encode check report")?
            } else if check.verbose {
                serde_json::to_string_pretty(&report).map_err(|_| "Cannot encode check report")?
            } else {
                report.render()
            };
            Ok(RunOutput {
                buffer: Buffer::from_streams(sanitize(rendered.as_bytes()), "", None),
                elapsed_ms: report.duration_ms,
                exit_code: Some(exit),
                outcome,
                completer: None,
                check_report: Some(report),
            })
        })
    }

    fn worker(run: impl FnOnce(&AtomicBool) -> Result<RunOutput, String> + Send + 'static) -> Self {
        let (sender, receiver) = mpsc::sync_channel(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        let signal = Arc::clone(&cancelled);
        let worker = thread::spawn(move || {
            let _ = sender.send(run(&signal));
        });
        Self {
            receiver,
            cancelled,
            worker: Some(worker),
            finished: false,
        }
    }

    pub(super) fn poll(&mut self) -> Option<Result<RunOutput, String>> {
        if self.finished {
            return None;
        }
        match self.receiver.try_recv() {
            Ok(result) => {
                self.finished = true;
                Some(result)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.finished = true;
                Some(Err("Command worker stopped unexpectedly".into()))
            }
        }
    }

    pub(super) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        self.cancel();
        // Do not leave an owned command running when the TUI is closed.
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[derive(Deserialize)]
struct CnrReport<'a> {
    status: Outcome,
    #[serde(borrow)]
    command: Option<&'a str>,
    #[serde(default)]
    timed_out: bool,
    #[serde(default)]
    cancelled: bool,
}

impl CnrReport<'_> {
    fn blocked_outcome(&self) -> Outcome {
        if self.cancelled {
            Outcome::Cancelled
        } else if self.timed_out {
            Outcome::TimedOut
        } else {
            Outcome::Blocked
        }
    }
}

#[derive(Deserialize)]
struct GovernedStepReport<'a> {
    #[serde(borrow)]
    result: CnrReport<'a>,
}

fn cnr_outcome(bytes: &[u8], exit_code: Option<i32>) -> Option<Outcome> {
    let stdout = std::str::from_utf8(bytes).ok()?;
    let (header_status, body) = if let Some(report) = stdout.strip_prefix("CNR · ") {
        let (header, body) = report.split_once('\n')?;
        let (name, status) = header.trim_end_matches('\r').split_once(" · ")?;
        if name != "cnr" && !super::super::COMMANDS.iter().any(|(key, ..)| *key == name) {
            return None;
        }
        (Some(status), body)
    } else {
        (None, stdout)
    };
    if !body.trim_start().starts_with('{') {
        return None;
    }
    // Deserialize only top-level report fields. This avoids retaining a second
    // copy of captured output and rejects duplicate status keys.
    let report: CnrReport<'_> = match serde_json::from_str(body) {
        Ok(report) => report,
        Err(_) => return (exit_code == Some(0)).then_some(Outcome::Fail),
    };
    if header_status.is_some_and(|header| header != report.status.label())
        || exit_code != Some(report.status.expected_exit())
    {
        return Some(Outcome::Fail);
    }
    if report.status != Outcome::Blocked {
        return Some(report.status);
    }
    // Governance run-step retains a blocked exit code for interrupted steps.
    // Only this envelope has a nested process receipt that can refine BLOCKED.
    if header_status.is_none()
        && report.command == Some("run-step")
        && let Ok(step) = serde_json::from_str::<GovernedStepReport<'_>>(body)
        && step.result.status == Outcome::Blocked
    {
        return Some(step.result.blocked_outcome());
    }
    Some(report.blocked_outcome())
}

fn completed_outcome(result: &execution::NativeResult, cnr_stdout: bool) -> Outcome {
    if result.cancelled {
        return Outcome::Cancelled;
    }
    if result.timed_out {
        return Outcome::TimedOut;
    }
    if cnr_stdout
        && !result.output_truncated
        && let Some(outcome) = cnr_outcome(&result.output, result.exit_code)
    {
        return outcome;
    }
    match result.exit_code {
        Some(0) => Outcome::Pass,
        Some(2) => Outcome::Blocked,
        Some(130) => Outcome::Cancelled,
        _ => Outcome::Fail,
    }
}

fn check_outcome(report: &CheckReport, exit_code: Option<i32>) -> Option<Outcome> {
    let outcome = Outcome::from(report.overall_status);
    (report.schema_version == 1
        && exit_code == Some(i32::from(report.exit_code))
        && i32::from(report.exit_code) == outcome.expected_exit())
    .then_some(outcome)
}

fn checked_report(
    result: &execution::NativeResult,
    summary: bool,
    pretty: bool,
) -> (Outcome, Option<String>) {
    if result.cancelled || result.timed_out {
        return (completed_outcome(result, true), None);
    }
    if !result.output_truncated
        && let Ok(report) = serde_json::from_slice::<CheckReport>(&result.output)
    {
        let Some(outcome) = check_outcome(&report, result.exit_code) else {
            return (
                Outcome::Error,
                Some("Check report and measured exit status disagree.".into()),
            );
        };
        return (
            outcome,
            if summary {
                Some(report.render())
            } else if pretty {
                serde_json::to_string_pretty(&report).ok()
            } else {
                None
            },
        );
    }
    // Parser diagnostics still use the legacy failure envelope. A successful
    // process without a complete versioned check report cannot establish PASS.
    if !result.output_truncated
        && let Some(Outcome::Fail | Outcome::Error) = cnr_outcome(&result.output, result.exit_code)
    {
        return (Outcome::Error, None);
    }
    (
        Outcome::Error,
        Some("Check did not return a complete versioned report.".into()),
    )
}

#[derive(Default)]
struct Invocation {
    inherit_project: bool,
    refresh_catalog: bool,
    explicit_json: bool,
    explicit_lang: bool,
    non_interactive: bool,
    output: OutputContract,
    check: Option<CheckRequest>,
}

fn prepare(args: &[String], vi: bool) -> Result<Invocation, String> {
    let boundary = args
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or(args.len());
    let (prefix, trailing) = args.split_at(boundary);
    let interactive =
        || "Interactive TUI and MCP servers cannot run inside the command view".to_owned();
    let parsed = super::super::parser(vi)
        .try_get_matches_from(std::iter::once("cnr").chain(prefix.iter().map(String::as_str)));
    let Ok(parsed) = parsed else {
        // The CLI still produces the diagnostic for ordinary invalid commands.
        return if prefix.first().is_some_and(|arg| arg == "tui")
            || prefix.windows(2).any(|pair| pair == ["mcp", "serve"])
        {
            Err(interactive())
        } else {
            Ok(Invocation::default())
        };
    };
    let (name, sub) = parsed.subcommand().ok_or_else(interactive)?;
    if matches!(name, "tui" | "mcp") {
        return Err(interactive());
    }
    let words: Vec<_> = sub
        .get_many::<String>("words")
        .into_iter()
        .flatten()
        .cloned()
        .collect();
    let maintenance = matches!(
        name,
        "workspace" | "task" | "governance" | "agent" | "audit" | "ci" | "check" | "setup"
    );
    let has_workspace = sub
        .try_get_one::<String>("workspace")
        .is_ok_and(|value| value.is_some())
        || maintenance && trailing.iter().any(|arg| arg == "--workspace");
    let explicit_project = parsed.get_one::<PathBuf>("project").is_some();
    let inherit_project = !explicit_project
        && !has_workspace
        && super::super::command_catalog::accepts_inherited_project(name, &words);
    let refresh_catalog = !explicit_project
        && !parsed.get_flag("dry-run")
        && match name {
            "init" | "pin" | "use" | "install" | "update" | "remove" => true,
            "config" | "env" => words.first().is_some_and(|action| {
                matches!(
                    (name, action.as_str()),
                    ("config", "set" | "unset") | ("env", "sync" | "import" | "lock")
                )
            }),
            _ => false,
        };
    let explicit_json = parsed.get_flag("json");
    let check = if name == "check"
        && trailing.is_empty()
        && super::super::validate_flags(name, &[], &parsed, false).is_ok()
        && super::super::command_catalog::preflight(args).is_none()
    {
        check_request(sub, &parsed).ok()
    } else {
        None
    };
    let output = if name == "check" && trailing.is_empty() {
        let pretty = sub.get_flag("verbose");
        OutputContract::Check {
            summary: !explicit_json && !pretty,
            pretty: pretty && !explicit_json,
        }
    } else {
        OutputContract::Cnr
    };
    Ok(Invocation {
        inherit_project,
        refresh_catalog,
        explicit_json,
        explicit_lang: parsed.value_source("lang") == Some(clap::parser::ValueSource::CommandLine),
        non_interactive: parsed.get_flag("non-interactive"),
        output,
        check,
    })
}

// A child may emit arbitrary terminal control sequences. Strip them before
// putting the captured text through the TUI painter.
pub(super) fn sanitize(bytes: &[u8]) -> String {
    enum State {
        Text,
        Escape,
        Intermediate,
        Csi,
        ControlString { osc: bool, escape: bool },
    }
    let control_string = |osc| State::ControlString { osc, escape: false };
    let chars = bytes.utf8_chunks().flat_map(|chunk| {
        let invalid = match chunk.invalid() {
            [] => None,
            [control @ 0x80..=0x9f] => Some(char::from(*control)),
            _ => Some('\u{fffd}'),
        };
        chunk.valid().chars().chain(invalid)
    });
    let mut output = String::with_capacity(bytes.len());
    let mut state = State::Text;
    let mut progress = false;
    for ch in chars {
        state = match (state, ch) {
            (State::ControlString { osc, escape }, ch) => {
                if ch == '\u{9c}' || (escape && ch == '\\') || (osc && ch == '\x07') {
                    State::Text
                } else {
                    State::ControlString {
                        osc,
                        escape: ch == '\x1b',
                    }
                }
            }
            (_, '\x1b') => State::Escape,
            (State::Escape, '[') | (_, '\u{9b}') => State::Csi,
            (State::Escape, ']') | (_, '\u{9d}') => control_string(true),
            (State::Escape, 'P' | 'X' | '^' | '_')
            | (_, '\u{90}' | '\u{98}' | '\u{9e}' | '\u{9f}') => control_string(false),
            (State::Escape | State::Intermediate, '\x20'..='\x2f') => State::Intermediate,
            (State::Escape | State::Intermediate, _) => State::Text,
            (State::Csi, '@'..='~') => State::Text,
            (State::Csi, _) => State::Csi,
            (State::Text, '\r') => {
                progress = true;
                State::Text
            }
            (State::Text, ch) => {
                if matches!(ch, '\n' | '\t') || !ch.is_control() {
                    if std::mem::take(&mut progress) && ch != '\n' {
                        output.push('\n');
                    }
                    if ch == '\t' {
                        output.push_str("    ");
                    } else {
                        output.push(ch);
                    }
                }
                State::Text
            }
        };
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn wait(runner: &mut Runner) -> RunOutput {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(result) = runner.poll() {
                return result.expect("worker should complete");
            }
            assert!(Instant::now() < deadline, "worker did not finish");
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn child_fixture() {
        match std::env::var("CNR_RUNNER_TEST_FIXTURE").as_deref() {
            Ok("output") => {
                println!("fixture \x1b[31mred\x1b[0m");
                eprintln!("fixture stderr");
            }
            Ok("wait") => thread::sleep(Duration::from_secs(30)),
            _ => {}
        }
    }

    fn fixture(mode: &str) -> Runner {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.args([
            "--exact",
            "menu::runner::tests::child_fixture",
            "--nocapture",
        ]);
        command.env("CNR_RUNNER_TEST_FIXTURE", mode);
        Runner::spawn(command, None, false, OutputContract::External)
    }

    #[test]
    fn worker_captures_and_sanitizes_both_streams() {
        let mut runner = fixture("output");
        let mut output = wait(&mut runner);
        assert_eq!(output.exit_code, Some(0));
        let count = output.buffer.layout(80);
        let captured: Vec<_> = (0..count)
            .map(|row| output.buffer.line(row, "", None).unwrap().text())
            .collect();
        let captured = captured.join("\n");
        assert!(captured.contains("fixture red"));
        assert!(captured.contains("[stderr]\nfixture stderr"));
        assert!(!captured.contains('\x1b'));
        assert!(!captured.contains("[Output truncated]"));
        assert_eq!(output.outcome, Outcome::Pass);
        assert!(runner.poll().is_none());
    }

    #[test]
    fn cancellation_stops_the_owned_process() {
        let mut runner = fixture("wait");
        assert!(runner.poll().is_none());
        runner.cancel();
        let output = wait(&mut runner);
        assert_eq!(output.outcome, Outcome::Cancelled);
    }

    fn captured_result(stdout: &str, exit_code: Option<i32>) -> execution::NativeResult {
        execution::NativeResult {
            output: stdout.as_bytes().to_vec(),
            error: Vec::new(),
            output_truncated: false,
            exit_code,
            timed_out: false,
            cancelled: false,
            elapsed_ms: 850,
        }
    }

    #[test]
    fn complete_cnr_report_agrees_with_measured_exit_or_fails_closed() {
        for (report, code, outcome) in [
            (
                "CNR · env · PASS\r\n{\"status\":\"PASS\"}\r\n",
                Some(0),
                Outcome::Pass,
            ),
            (
                r#"{"command":"task-show","status":"NOT_RUN","result":{"status":"NOT_FOUND"}}"#,
                Some(2),
                Outcome::NotRun,
            ),
            (
                r#"{"command":"run-step","status":"BLOCKED","result":{"status":"BLOCKED","timed_out":true,"cancelled":false}}"#,
                Some(2),
                Outcome::TimedOut,
            ),
            (
                r#"{"command":"run-step","status":"BLOCKED","result":{"status":"BLOCKED","timed_out":false,"cancelled":true}}"#,
                Some(2),
                Outcome::Cancelled,
            ),
            (
                r#"{"status":"BLOCKED","timed_out":true}"#,
                Some(2),
                Outcome::TimedOut,
            ),
            (r#"{"status":"BLOCKED"}"#, Some(2), Outcome::Blocked),
            (r#"{"status":"CANCELLED"}"#, Some(130), Outcome::Cancelled),
            (r#"{"status":"WARNING"}"#, Some(0), Outcome::Warning),
            (
                r#"{"status":"ERROR","message":"PASS"}"#,
                Some(1),
                Outcome::Error,
            ),
            (
                r#"{"status":"ERROR","message":"PASS"}"#,
                Some(0),
                Outcome::Fail,
            ),
            (r#"{"status":"FAIL"}"#, Some(0), Outcome::Fail),
            (r#"{"status":"PASS"}"#, Some(2), Outcome::Fail),
            (
                "CNR · env · PASS\n{\"status\":\"FAIL\"}\n",
                Some(0),
                Outcome::Fail,
            ),
            (r#"{"status":"PASS"}"#, None, Outcome::Fail),
        ] {
            assert_eq!(
                completed_outcome(&captured_result(report, code), true),
                outcome,
                "{report}"
            );
        }
    }

    #[test]
    fn status_uses_owned_stdout_and_ignores_embedded_or_incomplete_reports() {
        for report in [
            r#"{"result":{"status":"NOT_RUN"}}"#,
            r#"{"command":"task-show","status":"BLOCKED","result":{"status":"BLOCKED","timed_out":true}}"#,
            r#"{"command":"run-step","status":"BLOCKED","result":{"status":"PASS","timed_out":true}}"#,
            r#"[{"status":"NOT_RUN"}]"#,
            r#"{"status":"NOT_RUN","status":"NOT_RUN"}"#,
            r#"debug output {"status":"NOT_RUN"}"#,
            "CNR · unknown · NOT_RUN\n{\"status\":\"NOT_RUN\"}",
            "{\"status\":\"NOT_RUN\"}\nnon-report output",
        ] {
            assert_eq!(
                completed_outcome(&captured_result(report, Some(2)), true),
                Outcome::Blocked,
                "{report}"
            );
        }
        let mut result = captured_result(r#"{"status":"NOT_RUN"}"#, Some(2));
        assert_eq!(completed_outcome(&result, false), Outcome::Blocked);
        result.output_truncated = true;
        assert_eq!(completed_outcome(&result, true), Outcome::Blocked);

        for malformed in [
            r#"{"status":"UNRECOGNIZED","message":"PASS"}"#,
            r#"{"status":"PASS","status":"PASS"}"#,
        ] {
            assert_eq!(
                completed_outcome(&captured_result(malformed, Some(0)), true),
                Outcome::Fail
            );
        }

        let mut result = captured_result("", Some(0));
        result.error = br#"{"status":"FAIL"}"#.to_vec();
        assert_eq!(completed_outcome(&result, true), Outcome::Pass);
        result.timed_out = true;
        assert_eq!(completed_outcome(&result, true), Outcome::TimedOut);
        result.cancelled = true;
        assert_eq!(completed_outcome(&result, true), Outcome::Cancelled);
    }

    #[test]
    fn rejects_unbounded_protocol_server() {
        assert!(prepare(&["mcp".into(), "serve".into()], false).is_err());
        assert!(prepare(&["--json".into(), "tui".into()], false).is_err());
        assert!(prepare(&["workspace".into(), "discover".into()], false).is_ok());
        assert!(
            prepare(
                &["exec".into(), "--".into(), "mcp".into(), "serve".into()],
                false
            )
            .is_ok()
        );
    }

    #[test]
    fn refreshes_catalog_after_actual_project_selection_changes() {
        let check = |args: &[&str]| {
            prepare(
                &args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>(),
                false,
            )
            .map(|command| command.refresh_catalog)
        };
        assert_eq!(check(&["config", "set", "tools.rust", "1.99.1"]), Ok(true));
        assert_eq!(check(&["env", "import", "selection.json"]), Ok(true));
        assert_eq!(check(&["search", "install"]), Ok(false));
        assert_eq!(check(&["install", "rust@1.99.1", "--dry-run"]), Ok(false));
    }

    #[test]
    fn explicit_context_takes_precedence_over_the_terminal_project() {
        let context = |args: &[&str]| {
            let args = args
                .iter()
                .map(|arg| (*arg).into())
                .collect::<Vec<String>>();
            prepare(&args, false).unwrap().inherit_project
        };
        assert!(context(&["env", "plan"]));
        assert!(!context(&["env", "plan", "--project=other"]));
        assert!(!context(&["agent", "sync", "--workspace", "other"]));
        assert!(!context(&["agent", "sync", "--", "--workspace", "other"]));
        assert!(!context(&[
            "agent",
            "mcp-config",
            "check",
            "--bundle",
            "bundle"
        ]));
        assert!(context(&[
            "agent",
            "mcp-config",
            "export",
            "--binary",
            "cnr"
        ]));
        assert!(context(&["run", "--", "--workspace", "child-option"]));
    }

    #[test]
    fn typed_check_reports_require_schema_status_and_process_exit_agreement() {
        let report = CheckReport {
            schema_version: 1,
            run_id: "fixture".into(),
            requested_scope: serde_json::json!({"target":"governance"}),
            resolved_context: serde_json::json!({}),
            started_at: 0,
            duration_ms: 7,
            completion: "COMPLETE".into(),
            overall_status: Status::Warning,
            findings: Vec::new(),
            counts_by_status: Default::default(),
            coverage: serde_json::json!({}),
            exit_code: 0,
        };
        let mut captured = captured_result(&serde_json::to_string(&report).unwrap(), Some(0));
        let (outcome, summary) = checked_report(&captured, true, false);
        assert_eq!(outcome, Outcome::Warning);
        assert!(summary.unwrap().contains("WARNING"));
        captured.exit_code = Some(1);
        assert_eq!(checked_report(&captured, true, false).0, Outcome::Error);
        let fake = captured_result(r#"{"status":"PASS","message":"PASS"}"#, Some(0));
        assert_eq!(checked_report(&fake, true, false).0, Outcome::Error);
        captured.cancelled = true;
        assert_eq!(checked_report(&captured, true, false).0, Outcome::Cancelled);
    }

    #[test]
    fn sanitizes_terminal_controls_without_losing_text() {
        assert_eq!(
            sanitize(b"one\x1b[31m red\x1b[0m\n\x1b]0;title\x07two\tend\r"),
            "one red\ntwo    end"
        );
        assert_eq!(
            sanitize(
                "Việt 東京\u{1b}]8;;https://example.test\u{1b}\\label\u{1b}]8;;\u{1b}\\".as_bytes()
            ),
            "Việt 東京label"
        );
    }

    #[test]
    fn strips_control_string_payloads_and_preserves_progress_rows() {
        assert_eq!(
            sanitize(b"first\x1bPignored\x1b\\\x1b(Bsecond\x9b31m\rthird\r\nfourth"),
            "firstsecond\nthird\nfourth"
        );
        assert_eq!(
            sanitize("one\u{9d}title\u{9c}two\u{1b}_ignored\u{1b}\\three".as_bytes()),
            "onetwothree"
        );
        assert_eq!(sanitize(b"safe\x1bPunterminated"), "safe");
        assert_eq!(sanitize(b"a\xffb"), "a\u{fffd}b");
    }
}
