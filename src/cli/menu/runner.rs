use super::{completion::Completer, text, viewport::Buffer};
use cirnetrules::execution;
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

pub(super) struct RunOutput {
    pub(super) buffer: Buffer,
    pub(super) elapsed_ms: u64,
    pub(super) exit_code: Option<i32>,
    pub(super) timed_out: bool,
    pub(super) cancelled: bool,
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
        Ok(Self::spawn(command, refresh, vi))
    }

    fn spawn(mut command: Command, refresh: Option<PathBuf>, vi: bool) -> Self {
        let (sender, receiver) = mpsc::sync_channel(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        let signal = Arc::clone(&cancelled);
        let worker = thread::spawn(move || {
            let result = execution::run_quiet_native(&mut command, TIMEOUT, || {
                signal.load(Ordering::Relaxed)
            })
            .map(|result| {
                let notice = result
                    .output_truncated
                    .then(|| text(["[Output truncated]", "[Đầu ra đã được rút gọn]"], vi));
                let buffer = Buffer::from_streams(
                    sanitize(&result.output),
                    &sanitize(&result.error),
                    notice,
                );
                let completer = refresh
                    .filter(|_| result.success())
                    .map(|project| Completer::new(Some(&project), vi));
                RunOutput {
                    buffer,
                    elapsed_ms: result.elapsed_ms,
                    exit_code: result.exit_code,
                    timed_out: result.timed_out,
                    cancelled: result.cancelled,
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
        Runner::spawn(command, None, false)
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
        assert!(!output.cancelled && !output.timed_out);
        assert!(runner.poll().is_none());
    }

    #[test]
    fn cancellation_stops_the_owned_process() {
        let mut runner = fixture("wait");
        assert!(runner.poll().is_none());
        runner.cancel();
        let output = wait(&mut runner);
        assert!(output.cancelled);
        assert!(!output.timed_out);
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
