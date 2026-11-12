use super::{completion::Completer, text, viewport::Buffer};
use cirnetrules::execution;
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
    Fail,
    Blocked,
    NotRun,
    Cancelled,
    #[serde(skip_deserializing)]
    TimedOut,
}

impl Outcome {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Blocked => "BLOCKED",
            Self::NotRun => "NOT_RUN",
            Self::Cancelled => "CANCELLED",
            Self::TimedOut => "TIMED_OUT",
        }
    }

    fn expected_exit(self) -> i32 {
        match self {
            Self::Pass => 0,
            Self::Fail => 1,
            Self::Blocked | Self::NotRun | Self::TimedOut => 2,
            Self::Cancelled => 130,
        }
    }
}

pub(super) struct RunOutput {
    pub(super) buffer: Buffer,
    pub(super) elapsed_ms: u64,
    pub(super) exit_code: Option<i32>,
    pub(super) outcome: Outcome,
    pub(super) completer: Option<Completer>,
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
        let refresh_catalog = check_command(&args, vi)?;
        let refresh = catalog_project
            .filter(|_| refresh_catalog)
            .map(PathBuf::from);
        let executable = std::env::current_exe().map_err(|_| "Cannot locate the CNR executable")?;
        let mut command = Command::new(executable);
        command.arg("--non-interactive");
        if let Some(project) = project {
            command.arg("--project").arg(project);
        }
        if vi {
            command.args(["--lang", "vi"]);
        }
        command.args(args);
        Ok(Self::spawn(command, refresh, vi, true))
    }

    fn spawn(mut command: Command, refresh: Option<PathBuf>, vi: bool, cnr_stdout: bool) -> Self {
        let (sender, receiver) = mpsc::sync_channel(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        let signal = Arc::clone(&cancelled);
        let worker = thread::spawn(move || {
            let result = execution::run_quiet_native(&mut command, TIMEOUT, || {
                signal.load(Ordering::Relaxed)
            })
            .map(|result| {
                let outcome = completed_outcome(&result, cnr_stdout);
                let notice = result
                    .output_truncated
                    .then(|| text(["[Output truncated]", "[Đầu ra đã được rút gọn]"], vi));
                let buffer = Buffer::from_streams(
                    sanitize(&result.output),
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
                }
            });
            let _ = sender.send(result);
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
    let report: CnrReport<'_> = serde_json::from_str(body).ok()?;
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

fn check_command(args: &[String], vi: bool) -> Result<bool, String> {
    let prefix = args.split(|arg| arg == "--").next().unwrap_or(&[]);
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
            Ok(false)
        };
    };
    let (name, sub) = parsed.subcommand().ok_or_else(interactive)?;
    if matches!(name, "tui" | "mcp") {
        return Err(interactive());
    }
    if parsed.get_flag("dry-run") {
        return Ok(false);
    }
    Ok(match name {
        "init" | "pin" | "use" | "install" | "update" | "remove" => true,
        "config" | "env" => sub
            .get_many::<String>("words")
            .and_then(|mut words| words.next())
            .is_some_and(|action| {
                matches!(
                    (name, action.as_str()),
                    ("config", "set" | "unset") | ("env", "sync" | "import" | "lock")
                )
            }),
        _ => false,
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
        Runner::spawn(command, None, false, false)
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
        assert!(check_command(&["mcp".into(), "serve".into()], false).is_err());
        assert!(check_command(&["--json".into(), "tui".into()], false).is_err());
        assert!(check_command(&["workspace".into(), "discover".into()], false).is_ok());
        assert!(
            check_command(
                &["exec".into(), "--".into(), "mcp".into(), "serve".into()],
                false
            )
            .is_ok()
        );
    }

    #[test]
    fn refreshes_catalog_after_actual_project_selection_changes() {
        let check = |args: &[&str]| {
            check_command(
                &args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>(),
                false,
            )
        };
        assert_eq!(check(&["config", "set", "tools.rust", "1.99.1"]), Ok(true));
        assert_eq!(check(&["env", "import", "selection.json"]), Ok(true));
        assert_eq!(check(&["search", "install"]), Ok(false));
        assert_eq!(check(&["install", "rust@1.99.1", "--dry-run"]), Ok(false));
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
