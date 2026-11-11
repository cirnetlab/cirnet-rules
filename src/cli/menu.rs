mod terminal;
mod view;

use self::terminal::{Painter, Screen};
use clap::ArgMatches;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal as console,
};
use std::{env, io, path::PathBuf, process::ExitCode, time::Duration};

type Localized = [&'static str; 2];

fn text(value: Localized, vi: bool) -> &'static str {
    value[usize::from(vi)]
}

struct Entry {
    label: Localized,
    detail: Localized,
    command: &'static str,
    prompt: bool,
}

const ENTRIES: &[Entry] = &[
    Entry {
        label: ["Tool inventory", "Danh mục công cụ"],
        detail: [
            "Inspect installed tools and available providers.",
            "Xem công cụ đã cài và các provider khả dụng.",
        ],
        command: "list",
        prompt: false,
    },
    Entry {
        label: ["Search tools", "Tìm công cụ"],
        detail: [
            "Search the local tool catalog by name or capability.",
            "Tìm theo tên hoặc khả năng trong danh mục cục bộ.",
        ],
        command: "search",
        prompt: true,
    },
    Entry {
        label: ["Manage tools", "Quản lý công cụ"],
        detail: [
            "Plan an install, update or removal for an exact tool version.",
            "Lập kế hoạch cài, cập nhật hoặc gỡ một phiên bản cụ thể.",
        ],
        command: "install",
        prompt: true,
    },
    Entry {
        label: ["Environment plan", "Kế hoạch môi trường"],
        detail: [
            "Review native requirements, version pins and conflicts.",
            "Xem yêu cầu native, phiên bản đã ghim và xung đột.",
        ],
        command: "env plan",
        prompt: false,
    },
    Entry {
        label: ["Virtual environments", "Môi trường ảo"],
        detail: [
            "List the virtual environments for the active project.",
            "Liệt kê môi trường ảo của dự án hiện tại.",
        ],
        command: "venv list",
        prompt: false,
    },
    Entry {
        label: ["Build, run and test", "Build, chạy và kiểm thử"],
        detail: [
            "Prepare a native build, run or test for the active project.",
            "Chuẩn bị build, chạy hoặc kiểm thử native cho dự án.",
        ],
        command: "build --dry-run",
        prompt: true,
    },
    Entry {
        label: ["Governance workspace", "Workspace governance"],
        detail: [
            "Find registered repositories and candidate worktrees.",
            "Tìm repository đã đăng ký và worktree ứng viên.",
        ],
        command: "workspace discover",
        prompt: false,
    },
    Entry {
        label: ["Configuration", "Cấu hình"],
        detail: [
            "Inspect settings for the active project.",
            "Xem cấu hình của dự án hiện tại.",
        ],
        command: "config show",
        prompt: false,
    },
    Entry {
        label: ["Diagnostics", "Chẩn đoán"],
        detail: [
            "Check local prerequisites and environment status.",
            "Kiểm tra điều kiện cục bộ và trạng thái môi trường.",
        ],
        command: "doctor",
        prompt: false,
    },
    Entry {
        label: ["Command reference", "Tham chiếu lệnh"],
        detail: [
            "Show CLI commands and supported options.",
            "Xem các lệnh CLI và tùy chọn được hỗ trợ.",
        ],
        command: "--help",
        prompt: false,
    },
    Entry {
        label: ["Exit", "Thoát"],
        detail: ["Return to the shell.", "Trở về shell."],
        command: "",
        prompt: false,
    },
];

#[derive(Default)]
struct Input {
    value: String,
    cursor: usize,
}

impl Input {
    fn edit(&mut self, key: KeyEvent) -> bool {
        let old_cursor = self.cursor;
        let before = self.value[..self.cursor]
            .char_indices()
            .next_back()
            .map(|(index, _)| index);
        let after = self.value[self.cursor..]
            .chars()
            .next()
            .map(|ch| self.cursor + ch.len_utf8());
        match key.code {
            KeyCode::Char(ch)
                if !ch.is_control()
                    && !key.modifiers.intersects(
                        KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER,
                    ) =>
            {
                self.value.insert(self.cursor, ch);
                self.cursor += ch.len_utf8();
                return true;
            }
            KeyCode::Backspace if let Some(before) = before => {
                self.value.drain(before..self.cursor);
                self.cursor = before;
                return true;
            }
            KeyCode::Delete if let Some(after) = after => {
                self.value.drain(self.cursor..after);
                return true;
            }
            KeyCode::Left if let Some(before) = before => self.cursor = before,
            KeyCode::Right if let Some(after) = after => self.cursor = after,
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.value.len(),
            _ => {}
        }
        self.cursor != old_cursor
    }
}

#[derive(Default)]
struct Prompt {
    input: Input,
    error: Option<Localized>,
}

enum Effect {
    Idle,
    Redraw,
    Run(Vec<String>),
    Exit,
}

struct Menu {
    vi: bool,
    items: Vec<&'static Entry>,
    selected: usize,
    first: usize,
    search: Option<Input>,
    prompt: Option<Prompt>,
    help: bool,
}

impl Menu {
    fn new(vi: bool) -> Self {
        Self {
            vi,
            items: ENTRIES.iter().collect(),
            selected: 0,
            first: 0,
            search: None,
            prompt: None,
            help: false,
        }
    }

    fn refilter(&mut self) {
        let query = self
            .search
            .as_ref()
            .map_or_else(String::new, |input| input.value.to_lowercase());
        self.items = ENTRIES
            .iter()
            .filter(|entry| {
                text(entry.label, self.vi).to_lowercase().contains(&query)
                    || text(entry.detail, self.vi).to_lowercase().contains(&query)
                    || entry.command.contains(&query)
            })
            .collect();
        self.selected = 0;
        self.first = 0;
    }

    fn reveal_selection(&mut self, visible: usize) {
        self.first = self.first.min(self.items.len().saturating_sub(visible));
        self.first = self.first.clamp(
            self.selected.saturating_sub(visible.saturating_sub(1)),
            self.selected,
        );
    }

    fn select(&mut self, index: usize) -> Effect {
        let index = index.min(self.items.len().saturating_sub(1));
        if index == self.selected {
            return Effect::Idle;
        }
        self.selected = index;
        Effect::Redraw
    }

    fn activate(&mut self) -> Effect {
        let Some(entry) = self.items.get(self.selected) else {
            return Effect::Idle;
        };
        if entry.command.is_empty() {
            return Effect::Exit;
        }
        if entry.prompt {
            self.prompt = Some(Prompt::default());
            self.help = false;
            return Effect::Redraw;
        }
        Effect::Run(
            entry
                .command
                .split_ascii_whitespace()
                .map(str::to_owned)
                .collect(),
        )
    }

    fn submit_prompt(&mut self) -> Effect {
        let prompt = self.prompt.as_mut().expect("prompt is active");
        let value = prompt.input.value.trim();
        let command = if value.is_empty() {
            self.items[self.selected].command
        } else {
            value
        };
        let Ok(mut args) = words(command) else {
            prompt.error = Some([
                "Close the quoted argument.",
                "Hãy đóng dấu nháy của đối số.",
            ]);
            return Effect::Redraw;
        };
        if args.first().is_some_and(|word| word == "cnr") {
            args.remove(0);
        }
        if args.is_empty() || args[0] == "tui" {
            prompt.error = Some(["Enter a CNR command.", "Nhập một lệnh CNR."]);
            return Effect::Redraw;
        }
        self.prompt = None;
        Effect::Run(args)
    }

    fn handle(&mut self, key: KeyEvent) -> Effect {
        if key.kind == KeyEventKind::Release {
            return Effect::Idle;
        }
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Effect::Exit;
        }
        if key.code == KeyCode::Esc {
            if self.prompt.take().is_some() {
                return Effect::Redraw;
            }
            if self.search.take().is_some() {
                self.refilter();
                return Effect::Redraw;
            }
            if self.help {
                self.help = false;
                return Effect::Redraw;
            }
            return Effect::Exit;
        }
        if let Some(prompt) = &mut self.prompt {
            if key.code == KeyCode::Enter {
                return self.submit_prompt();
            }
            if prompt.input.edit(key) {
                prompt.error = None;
                return Effect::Redraw;
            }
            return Effect::Idle;
        }
        match key.code {
            KeyCode::Up => self.select(self.selected.saturating_sub(1)),
            KeyCode::Down => self.select(self.selected + 1),
            KeyCode::Enter => self.activate(),
            _ if let Some(search) = &mut self.search => {
                if matches!(key.code, KeyCode::Char(ch) if search.value.len() + ch.len_utf8() > 256)
                    || !search.edit(key)
                {
                    return Effect::Idle;
                }
                let query_unchanged = matches!(
                    key.code,
                    KeyCode::Left | KeyCode::Right | KeyCode::Home | KeyCode::End
                );
                if !query_unchanged {
                    self.refilter();
                }
                Effect::Redraw
            }
            KeyCode::Char('/')
                if !key.modifiers.intersects(
                    KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER,
                ) =>
            {
                self.search = Some(Input::default());
                self.help = false;
                Effect::Redraw
            }
            KeyCode::Char('?') => {
                self.help = !self.help;
                Effect::Redraw
            }
            _ => Effect::Idle,
        }
    }
}

fn show(menu: &mut Menu, color: bool, banner: bool) -> io::Result<Option<Vec<String>>> {
    let _screen = Screen::enter()?;
    let mut out = io::BufWriter::new(io::stdout().lock());
    let mut painter = Painter::default();
    let mut size = console::size()?;
    let mut redraw = true;
    loop {
        if redraw {
            painter.draw(&mut out, view::frame(menu, size, banner, color), color)?;
        }

        // Windows pseudoconsoles can change size without a resize event.
        // Recheck the viewport on a blocking wait timeout, without drawing
        // or flushing anything while the menu and viewport are unchanged.
        let input = if cfg!(windows) && !event::poll(Duration::from_millis(120))? {
            None
        } else {
            Some(event::read()?)
        };
        let new_size = if cfg!(windows) || matches!(input, Some(Event::Resize(_, _))) {
            console::size()?
        } else {
            size
        };
        redraw = new_size != size;
        size = new_size;
        if let Some(Event::Key(key)) = input {
            match menu.handle(key) {
                Effect::Idle => {}
                Effect::Redraw => redraw = true,
                Effect::Run(args) => return Ok(Some(args)),
                Effect::Exit => return Ok(None),
            }
        }
    }
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
    let color = env::var_os("NO_COLOR").is_none();
    let banner = !flags.get_flag("no-banner");
    let mut menu = Menu::new(vi);
    loop {
        let mut args = match show(&mut menu, color, banner) {
            Ok(Some(args)) => args,
            Ok(None) => return ExitCode::SUCCESS,
            Err(error) => {
                return super::emit(
                    "tui",
                    super::error(&format!("Terminal input failed: {error}")),
                    false,
                );
            }
        };
        let mut prefix = vec!["--non-interactive".into()];
        if let Some(path) = flags.get_one::<PathBuf>("project") {
            prefix.extend(["--project".into(), path.to_string_lossy().into_owned()]);
        }
        if vi {
            prefix.extend(["--lang".into(), "vi".into()]);
        }
        prefix.append(&mut args);
        super::run(&prefix);
        println!(
            "\n{}",
            text(
                [
                    "Press Enter to return to the menu.",
                    "Nhấn Enter để quay lại menu."
                ],
                vi
            )
        );
        if io::stdin().read_line(&mut String::new()).unwrap_or(0) == 0 {
            return ExitCode::SUCCESS;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn command_prompt_preserves_quotes_and_keeps_errors_in_the_menu() {
        let mut menu = Menu::new(false);
        assert!(matches!(menu.handle(key(KeyCode::Down)), Effect::Redraw));
        assert!(matches!(menu.handle(key(KeyCode::Enter)), Effect::Redraw));
        assert!(menu.prompt.is_some());
        for ch in r#"cnr search "rust toolchain"#.chars() {
            menu.handle(key(KeyCode::Char(ch)));
        }
        assert!(matches!(menu.handle(key(KeyCode::Enter)), Effect::Redraw));
        assert!(menu.prompt.as_ref().unwrap().error.is_some());
        menu.handle(key(KeyCode::Char('"')));
        match menu.handle(key(KeyCode::Enter)) {
            Effect::Run(args) => assert_eq!(args, ["search", "rust toolchain"]),
            _ => panic!("expected parsed command"),
        }
        assert!(menu.prompt.is_none());
    }

    #[test]
    fn unicode_input_can_edit_in_the_middle() {
        let mut input = Input::default();
        for ch in "aệc".chars() {
            input.edit(key(KeyCode::Char(ch)));
        }
        assert!(input.edit(key(KeyCode::Left)));
        assert!(input.edit(key(KeyCode::Backspace)));
        assert!(input.edit(key(KeyCode::Char('b'))));
        assert_eq!(input.value, "abc");
        assert_eq!(input.cursor, 2);
    }

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
