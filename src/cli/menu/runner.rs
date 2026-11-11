use cirnetrules::execution;
use std::{
    path::PathBuf,
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
    pub(super) stdout: String,
    pub(super) stderr: String,
    pub(super) exit_code: Option<i32>,
    pub(super) timed_out: bool,
    pub(super) cancelled: bool,
    pub(super) truncated: bool,
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
        vi: bool,
    ) -> Result<Self, String> {
        reject_command(&args)?;
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
        Ok(Self::spawn(command))
    }

    fn spawn(mut command: Command) -> Self {
        let (sender, receiver) = mpsc::sync_channel(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        let signal = Arc::clone(&cancelled);
        let worker = thread::spawn(move || {
            let result = execution::run_quiet_native(&mut command, TIMEOUT, || {
                signal.load(Ordering::Relaxed)
            })
            .map(|result| RunOutput {
                stdout: sanitize(&result.output),
                stderr: sanitize(&result.error),
                exit_code: result.exit_code,
                timed_out: result.timed_out,
                cancelled: result.cancelled,
                truncated: result.output_truncated,
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

fn reject_command(args: &[String]) -> Result<(), String> {
    let prefix = args.split(|arg| arg == "--").next().unwrap_or(&[]);
    if prefix.first().is_some_and(|arg| arg == "tui")
        || prefix
            .windows(2)
            .any(|pair| pair[0] == "mcp" && pair[1] == "serve")
    {
        return Err("Interactive TUI and MCP servers cannot run inside the command view".into());
    }
    Ok(())
}

// A child may emit arbitrary terminal control sequences. Strip them before
// putting the captured text through the TUI painter.
fn sanitize(bytes: &[u8]) -> String {
    let input = String::from_utf8_lossy(bytes);
    let mut chars = input.chars().peekable();
    let mut output = String::new();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            match chars.next() {
                Some('[') => {
                    for ch in chars.by_ref() {
                        if ('@'..='~').contains(&ch) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    while let Some(ch) = chars.next() {
                        if ch == '\u{7}' {
                            break;
                        }
                        if ch == '\u{1b}' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => {}
            }
        } else if ch == '\n' {
            output.push('\n');
        } else if ch == '\t' {
            output.push_str("    ");
        } else if !ch.is_control() {
            output.push(ch);
        }
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
        Runner::spawn(command)
    }

    #[test]
    fn worker_captures_and_sanitizes_both_streams() {
        let mut runner = fixture("output");
        let output = wait(&mut runner);
        assert_eq!(output.exit_code, Some(0));
        assert!(output.stdout.contains("fixture red"));
        assert!(output.stderr.contains("fixture stderr"));
        assert!(!output.stdout.contains('\x1b'));
        assert!(!output.truncated && !output.cancelled && !output.timed_out);
        assert!(runner.poll().is_none());
    }

    #[test]
    fn cancellation_stops_the_owned_process() {
        let mut runner = fixture("wait");
        runner.cancel();
        let output = wait(&mut runner);
        assert!(output.cancelled);
        assert!(!output.timed_out);
    }

    #[test]
    fn rejects_unbounded_protocol_server() {
        assert!(reject_command(&["mcp".into(), "serve".into()]).is_err());
        assert!(reject_command(&["workspace".into(), "discover".into()]).is_ok());
        assert!(
            reject_command(&["exec".into(), "--".into(), "mcp".into(), "serve".into()]).is_ok()
        );
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
}
