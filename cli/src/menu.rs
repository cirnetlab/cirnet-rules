use clap::ArgMatches;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    env,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

struct Entry {
    en: &'static str,
    vi: &'static str,
    command: &'static str,
    prompt: bool,
}

const ENTRIES: &[Entry] = &[
    Entry {
        en: "Tools / inventory",
        vi: "Công cụ / danh mục",
        command: "list",
        prompt: false,
    },
    Entry {
        en: "Tools / search",
        vi: "Công cụ / tìm kiếm",
        command: "search",
        prompt: true,
    },
    Entry {
        en: "Tools / install, update, remove",
        vi: "Công cụ / cài, cập nhật, gỡ",
        command: "install",
        prompt: true,
    },
    Entry {
        en: "Environment / plan",
        vi: "Môi trường / kế hoạch",
        command: "env plan",
        prompt: false,
    },
    Entry {
        en: "Environment / virtual environments",
        vi: "Môi trường / môi trường ảo",
        command: "venv list",
        prompt: false,
    },
    Entry {
        en: "Build / Run / Test",
        vi: "Biên dịch / Chạy / Kiểm thử",
        command: "build --dry-run",
        prompt: true,
    },
    Entry {
        en: "Governance / workspace",
        vi: "Governance / workspace",
        command: "workspace discover",
        prompt: false,
    },
    Entry {
        en: "Settings / config",
        vi: "Cài đặt / cấu hình",
        command: "config show",
        prompt: false,
    },
    Entry {
        en: "Settings / doctor",
        vi: "Cài đặt / chẩn đoán",
        command: "doctor",
        prompt: false,
    },
    Entry {
        en: "Help / commands",
        vi: "Trợ giúp / lệnh",
        command: "--help",
        prompt: false,
    },
    Entry {
        en: "Exit",
        vi: "Thoát",
        command: "",
        prompt: false,
    },
];

const ART: [&str; 5] = [
    " █████  ██   ██  ██████ ",
    "██      ███  ██  ██   ██",
    "██      ██ █ ██  ██████ ",
    "██      ██  ███  ██  ██ ",
    " █████  ██   ██  ██   ██",
];

struct Screen;

impl Screen {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let screen = Self;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(screen)
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), ResetColor, Show, LeaveAlternateScreen);
    }
}

fn show(title: &str, color: bool, banner: bool, vi: bool) -> io::Result<Option<&'static Entry>> {
    let _screen = Screen::enter()?;
    let mut filter = String::new();
    let mut editing = false;
    let mut selected = 0usize;
    loop {
        let (width, height) = terminal::size()?;
        let needle = filter.to_lowercase();
        let matches: Vec<_> = ENTRIES
            .iter()
            .filter(|entry| {
                let label = if vi { entry.vi } else { entry.en };
                label.to_lowercase().contains(&needle) || entry.command.contains(&needle)
            })
            .collect();
        selected = selected.min(matches.len().saturating_sub(1));
        let mut out = io::stdout().lock();
        let mut row = 0u16;
        queue!(out, MoveTo(0, 0), Clear(ClearType::All))?;
        if color {
            queue!(
                out,
                SetForegroundColor(Color::Rgb {
                    r: 211,
                    g: 133,
                    b: 103
                })
            )?;
        }
        if banner && width >= 36 && height >= 18 {
            for line in ART {
                queue!(out, MoveTo(0, row), Print(line))?;
                row += 1;
            }
        } else {
            queue!(out, Print("CNR"))?;
            row += 1;
        }
        if color {
            queue!(out, ResetColor)?;
        }
        if height > row + 5 {
            let subtitle = if vi {
                "CIRNET RULES — Runtime, SDK và công cụ phát triển"
            } else {
                "CIRNET RULES — Runtime, SDK & Development Tools"
            };
            row = print_line(&mut out, row, width, subtitle)?;
            row = print_line(&mut out, row, width, title)?;
        }
        let navigation = if vi {
            "↑/↓ Chọn · Enter Mở · / Tìm · Esc Thoát"
        } else {
            "↑/↓ Select · Enter Open · / Search · Esc Exit"
        };
        if row + 3 < height {
            row = print_line(&mut out, row, width, navigation)?;
            if editing || !filter.is_empty() {
                row = print_line(&mut out, row, width, &format!("/ {filter}"))?;
            }
        }
        let visible = usize::from(height.saturating_sub(row)).max(1);
        let offset = selected.saturating_sub(visible - 1);
        if matches.is_empty() && row < height {
            print_line(
                &mut out,
                row,
                width,
                if vi {
                    "Không có kết quả"
                } else {
                    "No matches"
                },
            )?;
        }
        for (index, entry) in matches.iter().enumerate().skip(offset).take(visible) {
            if row >= height {
                break;
            }
            if color && selected == index {
                queue!(
                    out,
                    SetForegroundColor(Color::Rgb {
                        r: 235,
                        g: 163,
                        b: 133
                    })
                )?;
            }
            let marker = if selected == index { "> " } else { "  " };
            let label = if vi { entry.vi } else { entry.en };
            row = print_line(&mut out, row, width, &format!("{marker}{label}"))?;
            if color {
                queue!(out, ResetColor)?;
            }
        }
        out.flush()?;
        drop(out);
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind == KeyEventKind::Release {
            continue;
        }
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(None),
            KeyCode::Esc if editing || !filter.is_empty() => {
                filter.clear();
                editing = false;
                selected = 0;
            }
            KeyCode::Esc => return Ok(None),
            KeyCode::Up => selected = selected.saturating_sub(1),
            KeyCode::Down => selected = (selected + 1).min(matches.len().saturating_sub(1)),
            KeyCode::Enter => {
                return Ok(matches
                    .get(selected)
                    .copied()
                    .filter(|entry| !entry.command.is_empty()));
            }
            KeyCode::Char('/') if !editing => editing = true,
            KeyCode::Backspace if editing => {
                filter.pop();
                selected = 0;
            }
            KeyCode::Char(ch) if editing && !ch.is_control() && filter.len() < 256 => {
                filter.push(ch);
                selected = 0;
            }
            _ => {}
        }
    }
}

fn print_line(out: &mut impl Write, row: u16, width: u16, text: &str) -> io::Result<u16> {
    let clipped: String = text
        .chars()
        .filter(|ch| !ch.is_control())
        .take(usize::from(width.saturating_sub(1)))
        .collect();
    queue!(out, MoveTo(0, row), Print(clipped))?;
    Ok(row + 1)
}

fn words(text: &str) -> Result<Vec<String>, &'static str> {
    let mut result = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut started = false;
    for ch in text.trim_end_matches(['\r', '\n']).chars() {
        if let Some(delimiter) = quote {
            if ch == delimiter {
                quote = None;
            } else {
                word.push(ch);
            }
        } else if matches!(ch, '\'' | '"') {
            quote = Some(ch);
            started = true;
        } else if ch.is_whitespace() {
            if started {
                result.push(std::mem::take(&mut word));
                started = false;
            }
        } else {
            word.push(ch);
            started = true;
        }
    }
    if quote.is_some() {
        return Err("Unclosed quote");
    }
    if started {
        result.push(word);
    }
    Ok(result)
}

pub fn run(flags: &ArgMatches, vi: bool) -> ExitCode {
    let path = super::project(flags, false)
        .ok()
        .flatten()
        .map(|project| project.root().to_path_buf())
        .or_else(|| flags.get_one::<PathBuf>("project").cloned())
        .or_else(|| env::current_dir().ok());
    let project = path.as_deref().map_or_else(
        || "-".into(),
        |path| {
            path.file_name()
                .unwrap_or(path.as_os_str())
                .to_string_lossy()
        },
    );
    let env_state = match path {
        Some(ref path) if path.join("cnr.lock").is_file() => {
            if vi {
                "có tệp khóa"
            } else {
                "lock file present"
            }
        }
        Some(ref path) if path.join("cnr.toml").is_file() => {
            if vi {
                "đã cấu hình"
            } else {
                "configured"
            }
        }
        _ => {
            if vi {
                "chưa cấu hình"
            } else {
                "unconfigured"
            }
        }
    };
    let title = format!(
        "{} · {}/{} · {} · {}",
        super::VERSION,
        env::consts::OS,
        env::consts::ARCH,
        env_state,
        project
    );
    let color = env::var_os("NO_COLOR").is_none();
    let banner = !flags.get_flag("no-banner");
    loop {
        let entry = match show(&title, color, banner, vi) {
            Ok(Some(entry)) => entry,
            Ok(None) => return ExitCode::SUCCESS,
            Err(error) => {
                return super::emit(
                    "tui",
                    super::error(&format!("Terminal input failed: {error}")),
                    false,
                );
            }
        };
        let mut input = String::new();
        if entry.prompt {
            println!(
                "{}",
                if vi {
                    "Nhập lệnh CNR hoặc nhấn Enter để dùng gợi ý."
                } else {
                    "Enter a CNR command, or press Enter to use the example."
                }
            );
            println!("  cnr {}", entry.command);
            if io::stdin().read_line(&mut input).unwrap_or(0) == 0 {
                return ExitCode::SUCCESS;
            }
        }
        let command = if input.trim().is_empty() {
            entry.command
        } else {
            input.trim()
        };
        match words(command) {
            Ok(mut args) => {
                if args.first().is_some_and(|word| word == "cnr") {
                    args.remove(0);
                }
                if !args.is_empty() && args[0] != "tui" {
                    let mut prefix = vec!["--non-interactive".into()];
                    if let Some(path) = flags.get_one::<PathBuf>("project") {
                        prefix.extend(["--project".into(), path.to_string_lossy().into_owned()]);
                    }
                    if vi {
                        prefix.extend(["--lang".into(), "vi".into()]);
                    }
                    prefix.append(&mut args);
                    super::run(&prefix);
                }
            }
            Err(error) => {
                eprintln!("{error}");
            }
        }
        println!(
            "{}",
            if vi {
                "Nhấn Enter để quay lại menu."
            } else {
                "Press Enter to return to the menu."
            }
        );
        if io::stdin().read_line(&mut input).unwrap_or(0) == 0 {
            return ExitCode::SUCCESS;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::words;

    #[test]
    fn preserves_quoted_unicode_paths_and_empty_argv_values() {
        assert_eq!(
            words(r#"exec -- "C:\Program Files\Việt\tool.exe" "" 'two words'"#).unwrap(),
            [
                "exec",
                "--",
                r"C:\Program Files\Việt\tool.exe",
                "",
                "two words"
            ]
        );
        assert!(words("exec -- \"unclosed").is_err());
    }
}
