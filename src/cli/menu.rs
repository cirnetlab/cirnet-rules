mod completion;
mod highlight;
mod runner;
mod terminal;
mod view;
mod viewport;

use self::{
    completion::{Completer, Kind, Source, Suggestion},
    terminal::{Painter, Screen, Tone},
    view::Banner,
    viewport::Buffer,
};
use clap::ArgMatches;
use crossterm::{
    event::{
        self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
    },
    terminal as console,
};
use std::{
    io,
    ops::Range,
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};
use unicode_width::UnicodeWidthChar;

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

const COMMAND_LIMIT: usize = 8192;
const SEARCH_LIMIT: usize = 256;
const HISTORY_LIMIT: usize = 100;
const SUGGESTION_LIMIT: usize = 7;
const COMMAND_TOO_LONG: Localized = ["Command exceeds 8 KiB.", "Lệnh vượt quá 8 KiB."];

fn text_modifiers(key: KeyEvent) -> bool {
    !key.modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
}

fn control(key: KeyEvent, ch: char) -> bool {
    key.code == KeyCode::Char(ch) && key.modifiers == KeyModifiers::CONTROL
}

fn printable(key: KeyEvent) -> Option<char> {
    match key.code {
        KeyCode::Char(ch) if !ch.is_control() && text_modifiers(key) => Some(ch),
        _ => None,
    }
}

#[derive(Clone, Default)]
struct Input {
    value: String,
    cursor: usize,
}

impl Input {
    fn insert(&mut self, value: &str, limit: usize) -> bool {
        if value.is_empty() || self.value.len() + value.len() > limit {
            return false;
        }
        self.value.insert_str(self.cursor, value);
        self.cursor += value.len();
        true
    }

    fn edit(&mut self, key: KeyEvent, limit: usize) -> bool {
        let old_cursor = self.cursor;
        // Move/delete a base character together with its combining marks.
        let before = self.value[..self.cursor]
            .char_indices()
            .rev()
            .find(|(index, ch)| *index == 0 || ch.width() != Some(0))
            .map(|(index, _)| index);
        let after = (self.cursor < self.value.len()).then(|| {
            self.value[self.cursor..]
                .char_indices()
                .skip(1)
                .find(|(_, ch)| ch.width() != Some(0))
                .map_or(self.value.len(), |(index, _)| self.cursor + index)
        });
        match key.code {
            KeyCode::Char('u') if control(key, 'u') && self.cursor > 0 => {
                self.value.drain(..self.cursor);
                self.cursor = 0;
                return true;
            }
            KeyCode::Char('a') if control(key, 'a') => self.cursor = 0,
            KeyCode::Char('e') if control(key, 'e') => self.cursor = self.value.len(),
            _ if !text_modifiers(key) => return false,
            KeyCode::Char(ch) if printable(key).is_some() => {
                return self.insert(ch.encode_utf8(&mut [0; 4]), limit);
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
    completions: Vec<Suggestion>,
    edited: bool,
    accepted: Option<(usize, Input)>,
}

impl Prompt {
    fn changed(&mut self) {
        self.error = None;
        self.edited = true;
        self.accepted = None;
    }
}

struct OutputView {
    title: String,
    buffer: Buffer,
    scroll: usize,
    last_page: usize,
    page_size: usize,
    width: usize,
    follow: bool,
    started_at: Instant,
    running: bool,
    status: String,
    tone: Tone,
    find: Option<Input>,
    query: String,
    hit: Option<Range<usize>>,
}

impl OutputView {
    fn new(title: String, vi: bool) -> Self {
        Self {
            title,
            buffer: Buffer::default(),
            scroll: 0,
            last_page: 0,
            page_size: 0,
            width: 0,
            follow: false,
            started_at: Instant::now(),
            running: true,
            status: text(["Running…", "Đang chạy…"], vi).into(),
            tone: Tone::Accent,
            find: None,
            query: String::new(),
            hit: None,
        }
    }

    fn layout(&mut self, width: usize, height: usize) {
        let anchor = self.buffer.row_start(self.scroll);
        self.page_size = height;
        self.width = width;
        self.last_page = self.buffer.layout(width).saturating_sub(height.max(1));
        self.scroll = if self.follow {
            self.last_page
        } else {
            anchor
                .map_or(self.scroll, |offset| self.buffer.row_at(offset))
                .min(self.last_page)
        };
    }

    fn scroll_to(&mut self, row: usize, follow: bool) -> Effect {
        let row = row.min(self.last_page);
        self.follow = follow && row == self.last_page;
        if row == self.scroll {
            return Effect::Idle;
        }
        self.scroll = row;
        Effect::Redraw
    }

    fn scroll_by(&mut self, lines: isize) -> Effect {
        self.scroll_to(self.scroll.saturating_add_signed(lines), lines > 0)
    }

    fn search(&mut self, backwards: bool) {
        self.hit = self.buffer.find(&self.query, self.hit.as_ref(), backwards);
        if let Some(hit) = &self.hit {
            self.scroll_to(self.buffer.row_at(hit.start), false);
        }
    }

    fn finish(&mut self, result: Result<runner::RunOutput, String>) -> Option<Completer> {
        self.running = false;
        self.hit = None;
        match result {
            Ok(result) => {
                self.buffer = result.buffer;
                if !self.query.is_empty() {
                    self.last_page = self
                        .buffer
                        .layout(self.width)
                        .saturating_sub(self.page_size.max(1));
                    self.search(false);
                }
                self.completed_status(result.outcome, result.exit_code, result.elapsed_ms.into());
                result.completer
            }
            Err(error) => {
                self.buffer =
                    Buffer::from_streams(String::new(), &runner::sanitize(error.as_bytes()), None);
                self.completed_status(
                    runner::Outcome::Fail,
                    None,
                    self.started_at.elapsed().as_millis(),
                );
                None
            }
        }
    }

    fn completed_status(
        &mut self,
        outcome: runner::Outcome,
        exit_code: Option<i32>,
        elapsed_ms: u128,
    ) {
        let (symbol, tone) = match outcome {
            runner::Outcome::Pass => ("✓", Tone::Success),
            runner::Outcome::Fail => ("✗", Tone::Error),
            _ => ("!", Tone::Warning),
        };
        let code = exit_code.map_or_else(|| "?".into(), |code| code.to_string());
        self.tone = tone;
        self.status = format!(
            "{symbol} {} · Exit {code} · Duration {elapsed_ms} ms",
            outcome.label()
        );
    }
}

enum Effect {
    Idle,
    Redraw,
    Run(Vec<String>),
    Cancel,
    Exit,
}

struct Menu {
    vi: bool,
    items: Vec<&'static Entry>,
    selected: usize,
    first: usize,
    search: Option<Input>,
    prompt: Option<Prompt>,
    draft: Input,
    help: bool,
    output: Option<OutputView>,
    history: Vec<String>,
    history_index: Option<usize>,
    history_draft: Input,
    completer: Completer,
    banner: Banner,
}

impl Menu {
    fn new(vi: bool, project: Option<&Path>) -> Self {
        Self {
            vi,
            items: ENTRIES.iter().collect(),
            selected: 0,
            first: 0,
            search: None,
            prompt: None,
            draft: Input::default(),
            help: false,
            output: None,
            history: Vec::new(),
            history_index: None,
            history_draft: Input::default(),
            completer: Completer::new(project, vi),
            banner: Banner::default(),
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
        self.first = self
            .first
            .min(self.items.len().saturating_sub(visible))
            .clamp(
                self.selected.saturating_sub(visible.saturating_sub(1)),
                self.selected,
            );
    }

    fn select(&mut self, index: usize) -> Effect {
        let index = index.min(self.items.len().saturating_sub(1));
        let effect = if index == self.selected {
            Effect::Idle
        } else {
            Effect::Redraw
        };
        self.selected = index;
        effect
    }

    fn activate(&mut self) -> Effect {
        let Some(entry) = self.items.get(self.selected) else {
            return Effect::Idle;
        };
        if entry.command.is_empty() {
            return Effect::Exit;
        }
        if entry.prompt {
            self.open_prompt(Input::default());
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

    fn open_prompt(&mut self, input: Input) {
        self.prompt = Some(Prompt {
            input,
            edited: true,
            ..Prompt::default()
        });
        self.help = false;
        self.history_index = None;
    }

    fn close_main_prompt(&mut self) -> bool {
        let Some(prompt) = self.prompt.take() else {
            return false;
        };
        self.draft = if prompt.error == Some(COMMAND_TOO_LONG) {
            Input::default()
        } else {
            prompt.input
        };
        self.history_index = None;
        self.history_draft = Input::default();
        true
    }

    fn refresh_suggestions(&mut self) {
        let Some(prompt) = self.prompt.as_mut().filter(|prompt| prompt.edited) else {
            return;
        };
        prompt.edited = false;
        let input = &prompt.input;
        prompt.completions = if input.value.is_empty() {
            self.output
                .is_none()
                .then(|| self.items.get(self.selected))
                .flatten()
                .filter(|entry| !entry.command.is_empty())
                .map(|entry| Suggestion {
                    text: entry.command.into(),
                    insert_range: 0..0,
                    label: entry.command.into(),
                    kind: Kind::Command,
                    description: text(entry.detail, self.vi).into(),
                    source: Source::Menu,
                })
                .into_iter()
                .collect()
        } else {
            self.completer.suggestions(&input.value, input.cursor)
        };
        if input.cursor == input.value.len()
            && !input.value.is_empty()
            && !input.value.contains("-- ")
        {
            // The completer only falls back to fuzzy matches if no exact prefix
            // matches exist. Safe history takes precedence over those matches.
            let fuzzy = if prompt.completions.iter().all(|item| {
                !item
                    .text
                    .starts_with(&input.value[item.insert_range.start..input.cursor])
            }) {
                std::mem::take(&mut prompt.completions)
            } else {
                Vec::new()
            };
            for entry in self
                .history
                .iter()
                .rev()
                .filter(|entry| entry.starts_with(&input.value) && *entry != &input.value)
            {
                if prompt.completions.len() >= SUGGESTION_LIMIT {
                    break;
                }
                if prompt.completions.iter().any(|suggestion| {
                    let range = &suggestion.insert_range;
                    entry.len() == input.value.len() - range.len() + suggestion.text.len()
                        && entry.starts_with(&input.value[..range.start])
                        && entry.ends_with(&input.value[range.end..])
                        && entry[range.start..].starts_with(&suggestion.text)
                }) {
                    continue;
                }
                prompt.completions.push(Suggestion {
                    text: entry.clone(),
                    insert_range: 0..input.value.len(),
                    label: entry.clone(),
                    kind: Kind::History,
                    description: text(["Recent command", "Lệnh gần đây"], self.vi).into(),
                    source: Source::History,
                });
            }
            for suggestion in fuzzy {
                if prompt.completions.len() == SUGGESTION_LIMIT {
                    break;
                }
                if !prompt.completions.iter().any(|existing| {
                    existing.text == suggestion.text
                        && existing.insert_range == suggestion.insert_range
                }) {
                    prompt.completions.push(suggestion);
                }
            }
        }
    }

    fn complete(&mut self, backwards: bool) -> Effect {
        self.refresh_suggestions();
        let prompt = self.prompt.as_mut().expect("prompt is active");
        let count = prompt.completions.len();
        if count == 0 || prompt.error == Some(COMMAND_TOO_LONG) {
            return Effect::Idle;
        }
        let (index, base) = prompt.accepted.take().map_or_else(
            || (if backwards { count - 1 } else { 0 }, prompt.input.clone()),
            |(index, base)| {
                (
                    (index + if backwards { count - 1 } else { 1 }) % count,
                    base,
                )
            },
        );
        let completion = &prompt.completions[index];
        let range = completion.insert_range.clone();
        if base.value.len() - range.len() + completion.text.len() > COMMAND_LIMIT {
            return Effect::Idle;
        }
        prompt.input.value.clone_from(&base.value);
        prompt
            .input
            .value
            .replace_range(range.clone(), &completion.text);
        prompt.input.cursor = range.start + completion.text.len();
        prompt.accepted = Some((index, base));
        prompt.error = None;
        self.history_index = None;
        Effect::Redraw
    }

    fn history_move(&mut self, older: bool, prefix: bool) -> Effect {
        if self.history.is_empty() || (!older && self.history_index.is_none()) {
            return Effect::Idle;
        }
        let prompt = self.prompt.as_mut().expect("prompt is active");
        if self.history_index.is_none() {
            self.history_draft.clone_from(&prompt.input);
        }
        self.history_index = if older {
            let before = self.history_index.unwrap_or(self.history.len());
            let matches = |entry: &String| !prefix || entry.starts_with(&self.history_draft.value);
            let Some(index) = self.history[..before].iter().rposition(matches) else {
                return Effect::Idle;
            };
            Some(index)
        } else {
            self.history_index
                .and_then(|index| (index + 1 < self.history.len()).then_some(index + 1))
        };
        prompt.input = self.history_index.map_or_else(
            || self.history_draft.clone(),
            |index| Input {
                value: self.history[index].clone(),
                cursor: self.history[index].len(),
            },
        );
        prompt.changed();
        Effect::Redraw
    }

    fn submit_prompt(&mut self) -> Effect {
        let prompt = self.prompt.as_mut().expect("prompt is active");
        if prompt.input.value.trim().is_empty() || prompt.error == Some(COMMAND_TOO_LONG) {
            return Effect::Idle;
        }
        let Ok(mut args) = completion::arguments(&prompt.input.value) else {
            prompt.error = Some([
                "Close the quoted argument.",
                "Hãy đóng dấu nháy của đối số.",
            ]);
            return Effect::Redraw;
        };
        if args.first().is_some_and(|word| word == "cnr") {
            args.remove(0);
        }
        if args.len() == 1 && args[0] == "clear" {
            if let Some(output) = &mut self.output {
                output.buffer = Buffer::default();
                output.scroll = 0;
                output.follow = false;
                output.hit = None;
            }
            self.open_prompt(Input::default());
            return Effect::Redraw;
        }
        if args.is_empty() || args[0] == "tui" {
            prompt.error = Some(["Enter a CNR command.", "Nhập một lệnh CNR."]);
            return Effect::Redraw;
        }
        if safe_history(&args) {
            let command = args.join(" ");
            if self.history.last() != Some(&command) {
                self.history.push(command);
                if self.history.len() > HISTORY_LIMIT {
                    self.history.remove(0);
                }
            }
        }
        self.history_index = None;
        self.history_draft = Input::default();
        self.draft = Input::default();
        self.prompt = None;
        Effect::Run(args)
    }

    fn edit_prompt(&mut self, key: KeyEvent) -> Effect {
        match key.code {
            KeyCode::Enter
                if text_modifiers(key)
                    && self.output.as_ref().is_none_or(|output| !output.running) =>
            {
                return self.submit_prompt();
            }
            KeyCode::Tab | KeyCode::BackTab if text_modifiers(key) => {
                return self.complete(
                    key.code == KeyCode::BackTab || key.modifiers.contains(KeyModifiers::SHIFT),
                );
            }
            KeyCode::Up | KeyCode::Down if self.output.is_some() && text_modifiers(key) => {
                return self.history_move(key.code == KeyCode::Up, false);
            }
            _ if control(key, 'r') => return self.history_move(true, true),
            _ => {}
        }
        let prompt = self.prompt.as_mut().expect("prompt is active");
        if printable(key).is_some_and(|ch| prompt.input.value.len() + ch.len_utf8() > COMMAND_LIMIT)
        {
            prompt.error = Some(COMMAND_TOO_LONG);
            return Effect::Redraw;
        }
        let previous_len = prompt.input.value.len();
        if !prompt.input.edit(key, COMMAND_LIMIT) {
            return Effect::Idle;
        }
        if prompt.input.value.len() != previous_len || prompt.error != Some(COMMAND_TOO_LONG) {
            prompt.changed();
            self.history_index = None;
        }
        Effect::Redraw
    }

    fn handle_output(&mut self, key: KeyEvent) -> Effect {
        let output = self.output.as_mut().expect("output is active");
        if control(key, 'f') {
            output.find.get_or_insert_with(|| Input {
                value: output.query.clone(),
                cursor: output.query.len(),
            });
            return Effect::Redraw;
        }
        if key.code == KeyCode::F(3) && text_modifiers(key) {
            output.search(key.modifiers.contains(KeyModifiers::SHIFT));
            return Effect::Redraw;
        }
        let page = output.page_size.clamp(1, 10) as isize;
        match key.code {
            KeyCode::PageUp => return output.scroll_by(-page),
            KeyCode::PageDown => return output.scroll_by(page),
            KeyCode::Home if key.modifiers == KeyModifiers::CONTROL => {
                return output.scroll_to(0, false);
            }
            KeyCode::End if key.modifiers == KeyModifiers::CONTROL => {
                return output.scroll_to(output.last_page, true);
            }
            _ => {}
        }
        if let Some(find) = &mut output.find {
            return match key.code {
                KeyCode::Enter | KeyCode::Esc => {
                    output.find = None;
                    Effect::Redraw
                }
                _ if find.edit(key, SEARCH_LIMIT) => {
                    output.query.clone_from(&find.value);
                    output.hit = None;
                    output.search(false);
                    Effect::Redraw
                }
                _ => Effect::Idle,
            };
        }
        if key.code == KeyCode::Esc {
            if output.running {
                return Effect::Idle;
            }
            self.output = None;
            self.prompt = None;
            self.history_index = None;
            return Effect::Redraw;
        }
        self.edit_prompt(key)
    }

    fn handle(&mut self, key: KeyEvent) -> Effect {
        if key.kind == KeyEventKind::Release {
            return Effect::Idle;
        }
        if control(key, 'c') {
            if let Some(output) = self.output.as_mut().filter(|output| output.running) {
                output.status = text(["Cancelling…", "Đang hủy…"], self.vi).into();
                output.tone = Tone::Warning;
                return Effect::Cancel;
            }
            return Effect::Exit;
        }
        if self.output.is_some() {
            return self.handle_output(key);
        }
        if key.code == KeyCode::Esc {
            if self.close_main_prompt() {
                return Effect::Redraw;
            }
            if self.search.take().is_some() {
                self.refilter();
                return Effect::Redraw;
            }
            if std::mem::take(&mut self.help) {
                return Effect::Redraw;
            }
            return Effect::Exit;
        }
        if text_modifiers(key) {
            let index = match key.code {
                KeyCode::Up => Some(self.selected.saturating_sub(1)),
                KeyCode::Down => Some(self.selected + 1),
                KeyCode::Home => Some(0),
                KeyCode::End => Some(self.items.len().saturating_sub(1)),
                _ => None,
            };
            if let Some(index) = index {
                let closed = self.close_main_prompt();
                let selection = self.select(index);
                return if closed { Effect::Redraw } else { selection };
            }
        }
        if self.prompt.is_some() {
            return self.edit_prompt(key);
        }
        if control(key, 'k') {
            self.search = Some(Input::default());
            self.help = false;
            self.refilter();
            return Effect::Redraw;
        }
        if control(key, 'r') && !self.history.is_empty() {
            self.open_prompt(Input::default());
            return self.history_move(true, true);
        }
        if key.code == KeyCode::Enter && text_modifiers(key) {
            return self.activate();
        }
        if let Some(search) = &mut self.search {
            let previous = search.value.len();
            if !search.edit(key, SEARCH_LIMIT) {
                return Effect::Idle;
            }
            if search.value.len() != previous {
                self.refilter();
            }
            return Effect::Redraw;
        }
        match printable(key) {
            Some('/') => {
                self.search = Some(Input::default());
                self.help = false;
                Effect::Redraw
            }
            Some('?') => {
                self.help = !self.help;
                Effect::Redraw
            }
            Some(':') => {
                let draft = std::mem::take(&mut self.draft);
                self.open_prompt(draft);
                Effect::Redraw
            }
            Some(ch) if !ch.is_whitespace() && !self.help => {
                self.draft = Input::default();
                self.open_prompt(Input::default());
                self.edit_prompt(key)
            }
            _ => Effect::Idle,
        }
    }

    fn paste(&mut self, value: &str) -> Effect {
        if value.len() > COMMAND_LIMIT {
            if self.prompt.is_none() && self.search.is_none() && !self.help {
                self.open_prompt(Input::default());
            }
            if let Some(prompt) = &mut self.prompt {
                prompt.error = Some(COMMAND_TOO_LONG);
                return Effect::Redraw;
            }
            return Effect::Idle;
        }
        let sanitized = runner::sanitize(value.as_bytes()).replace('\n', " ");
        if let Some(output) = &mut self.output
            && let Some(find) = &mut output.find
        {
            if !find.insert(&sanitized, SEARCH_LIMIT) {
                return Effect::Idle;
            }
            output.query.clone_from(&find.value);
            output.hit = None;
            output.search(false);
            return Effect::Redraw;
        }
        if self.prompt.is_none() {
            if let Some(search) = &mut self.search {
                if !search.insert(&sanitized, SEARCH_LIMIT) {
                    return Effect::Idle;
                }
                self.refilter();
                return Effect::Redraw;
            }
            if self.help || sanitized.trim().is_empty() {
                return Effect::Idle;
            }
            self.draft = Input::default();
            self.open_prompt(Input::default());
        }
        let prompt = self.prompt.as_mut().expect("prompt is active");
        if prompt.input.value.len() + sanitized.len() > COMMAND_LIMIT {
            prompt.error = Some(COMMAND_TOO_LONG);
            return Effect::Redraw;
        }
        if prompt.input.insert(&sanitized, COMMAND_LIMIT) {
            prompt.changed();
            self.history_index = None;
            Effect::Redraw
        } else {
            Effect::Idle
        }
    }
}

fn safe_history(args: &[String]) -> bool {
    // Record known read-only command shapes. Free-form arguments may be secrets.
    let args = if args.last().is_some_and(|arg| arg == "--json") {
        &args[..args.len() - 1]
    } else {
        args
    };
    match args {
        [command] => matches!(command.as_str(), "list" | "doctor" | "--help" | "--version"),
        [command, action] => matches!(
            (command.as_str(), action.as_str()),
            ("env", "plan" | "profiles")
                | ("config", "show")
                | ("venv", "list")
                | ("workspace", "discover" | "health")
                | ("governance", "inspect" | "validate")
                | ("agent", "list")
                | ("audit", "list" | "tooling")
        ),
        _ => false,
    }
}

fn output_title(args: &[String]) -> String {
    if safe_history(args) {
        return format!("cnr {}", args.join(" "));
    }
    let command = args
        .iter()
        .take_while(|word| word.as_str() != "--")
        .find(|word| super::COMMANDS.iter().any(|(name, _, _)| name == *word));
    command.map_or_else(|| "cnr".into(), |command| format!("cnr {command}"))
}

fn read_events(batch: &mut Vec<Event>, running: bool) -> io::Result<()> {
    batch.clear();
    let timeout = if running {
        Some(Duration::from_millis(16))
    } else if cfg!(windows) {
        Some(Duration::from_millis(120))
    } else {
        None
    };
    if let Some(timeout) = timeout
        && !event::poll(timeout)?
    {
        return Ok(());
    }
    batch.push(event::read()?);
    while batch.len() < 128 && event::poll(Duration::ZERO)? {
        batch.push(event::read()?);
    }
    Ok(())
}

fn windows_paste(batch: &[Event], previous_text: usize) -> Option<String> {
    // The Windows console input API reports unbracketed paste as queued keys.
    // A newline in a queued text burst is input, never a submit.
    let mut value = String::new();
    let mut characters = previous_text;
    let mut newline = false;
    for event in batch {
        let Event::Key(key) = event else { return None };
        if key.kind == KeyEventKind::Release {
            continue;
        }
        if let Some(ch) = printable(*key) {
            characters += 1;
            value.push(ch);
        } else if key.code == KeyCode::Enter && text_modifiers(*key) {
            newline = true;
            value.push(' ');
        } else {
            return None;
        }
    }
    (newline && characters >= 2).then_some(value)
}

fn show(
    menu: &mut Menu,
    color: bool,
    banner: bool,
    project: Option<PathBuf>,
    catalog_project: Option<&Path>,
) -> io::Result<()> {
    // Restore the terminal before joining an owned worker on an I/O error.
    let mut running: Option<runner::Runner> = None;
    let mut screen = Screen::enter(io::BufWriter::with_capacity(64 * 1024, io::stdout().lock()))?;
    let mut painter = Painter::default();
    let mut size = console::size()?;
    let mut batch = Vec::with_capacity(128);
    let mut redraw = true;
    let mut last_size_check = Instant::now();
    let mut paste_continuation = 0;
    loop {
        if let Some(result) = running.as_mut().and_then(runner::Runner::poll) {
            if let Some(output) = &mut menu.output
                && let Some(completer) = output.finish(result)
            {
                menu.completer = completer;
                if let Some(prompt) = &mut menu.prompt {
                    prompt.changed();
                }
            }
            running = None;
            redraw = true;
        }
        if redraw {
            painter.draw(
                &mut screen.out,
                view::frame(menu, size, banner, color),
                color,
            )?;
        }
        read_events(&mut batch, running.is_some())?;
        redraw = false;
        if batch
            .iter()
            .any(|event| matches!(event, Event::Resize(_, _)))
            || (cfg!(windows) && last_size_check.elapsed() >= Duration::from_millis(120))
        {
            let new_size = console::size()?;
            last_size_check = Instant::now();
            if new_size.0 > 0 && new_size.1 > 0 && new_size != size {
                size = new_size;
                redraw = true;
            }
        }

        let pending = std::mem::take(&mut paste_continuation);
        if cfg!(windows) {
            if batch.len() == 128 {
                paste_continuation = pending + batch.iter().filter(|event| {
                    matches!(event, Event::Key(key) if key.kind != KeyEventKind::Release && printable(*key).is_some())
                }).count();
            }
            if let Some(paste) = windows_paste(&batch, pending) {
                redraw |= matches!(menu.paste(&paste), Effect::Redraw);
                continue;
            }
        }

        for input in batch.drain(..) {
            let effect = match input {
                Event::Key(key) => menu.handle(key),
                Event::Paste(value) => menu.paste(&value),
                Event::Mouse(mouse) if menu.output.is_some() => {
                    let output = menu.output.as_mut().expect("output is active");
                    match mouse.kind {
                        MouseEventKind::ScrollUp => output.scroll_by(-3),
                        MouseEventKind::ScrollDown => output.scroll_by(3),
                        _ => Effect::Idle,
                    }
                }
                Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                    if let Some(index) = painter.menu_hit(size, mouse.column, mouse.row)
                        && index < menu.items.len()
                    {
                        redraw |= menu.close_main_prompt();
                        redraw |= matches!(menu.select(index), Effect::Redraw);
                        menu.activate()
                    } else {
                        Effect::Idle
                    }
                }
                _ => Effect::Idle,
            };
            match effect {
                Effect::Idle => {}
                Effect::Redraw => redraw = true,
                Effect::Cancel => {
                    if let Some(worker) = &running {
                        worker.cancel();
                    }
                    redraw = true;
                }
                Effect::Run(args) => {
                    menu.output = Some(OutputView::new(output_title(&args), menu.vi));
                    menu.open_prompt(Input::default());
                    match runner::Runner::start(args, project.clone(), catalog_project, menu.vi) {
                        Ok(worker) => running = Some(worker),
                        Err(error) => {
                            menu.output
                                .as_mut()
                                .expect("output was created")
                                .finish(Err(error));
                        }
                    }
                    redraw = true;
                }
                Effect::Exit => return Ok(()),
            }
        }
    }
}

pub fn run(flags: &ArgMatches, vi: bool) -> ExitCode {
    let color = terminal::color_enabled();
    let banner = !flags.get_flag("no-banner");
    let catalog_project = super::project(flags, true).ok().flatten();
    let catalog_project = catalog_project
        .as_ref()
        .map(cirnetrules::environment::Project::root);
    let mut menu = Menu::new(vi, catalog_project);
    match show(
        &mut menu,
        color,
        banner,
        flags.get_one::<PathBuf>("project").cloned(),
        catalog_project,
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => super::emit(
            "tui",
            super::error(&format!("Terminal input failed: {error}")),
            false,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_status_uses_measured_milliseconds_and_the_actual_exit() {
        use runner::Outcome::*;
        let mut output = OutputView::new("fixture".into(), false);
        assert_eq!(output.status, "Running…");
        for (outcome, exit_code, elapsed_ms, expected, tone) in [
            (
                Pass,
                Some(0),
                850,
                "✓ PASS · Exit 0 · Duration 850 ms",
                Tone::Success,
            ),
            (
                Pass,
                Some(0),
                0,
                "✓ PASS · Exit 0 · Duration 0 ms",
                Tone::Success,
            ),
            (
                Pass,
                Some(0),
                1,
                "✓ PASS · Exit 0 · Duration 1 ms",
                Tone::Success,
            ),
            (
                Pass,
                Some(0),
                999,
                "✓ PASS · Exit 0 · Duration 999 ms",
                Tone::Success,
            ),
            (
                Pass,
                Some(0),
                1000,
                "✓ PASS · Exit 0 · Duration 1000 ms",
                Tone::Success,
            ),
            (
                Fail,
                Some(1),
                850,
                "✗ FAIL · Exit 1 · Duration 850 ms",
                Tone::Error,
            ),
            (
                Blocked,
                Some(2),
                850,
                "! BLOCKED · Exit 2 · Duration 850 ms",
                Tone::Warning,
            ),
            (
                NotRun,
                Some(2),
                1,
                "! NOT_RUN · Exit 2 · Duration 1 ms",
                Tone::Warning,
            ),
            (
                Cancelled,
                Some(130),
                4,
                "! CANCELLED · Exit 130 · Duration 4 ms",
                Tone::Warning,
            ),
            (
                TimedOut,
                None,
                2345,
                "! TIMED_OUT · Exit ? · Duration 2345 ms",
                Tone::Warning,
            ),
            (
                Fail,
                None,
                850,
                "✗ FAIL · Exit ? · Duration 850 ms",
                Tone::Error,
            ),
        ] {
            output.finish(Ok(runner::RunOutput {
                buffer: Buffer::default(),
                elapsed_ms,
                exit_code,
                outcome,
                completer: None,
            }));
            assert_eq!(output.status, expected);
            assert!(output.tone == tone);
            assert!(!output.running);
        }
    }

    #[test]
    fn runner_error_reports_failure_with_unknown_exit_and_measured_time() {
        let mut output = OutputView::new("fixture".into(), false);
        output.finish(Err("Unable to start the contained child".into()));
        let duration = output
            .status
            .strip_prefix("✗ FAIL · Exit ? · Duration ")
            .unwrap()
            .strip_suffix(" ms")
            .unwrap();
        assert!(duration.parse::<u64>().is_ok());
        assert!(output.tone == Tone::Error);
        assert!(!output.running);
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn press(menu: &mut Menu, code: KeyCode) -> Effect {
        menu.handle(key(code))
    }

    fn command(menu: &Menu) -> &str {
        &menu.prompt.as_ref().expect("command prompt").input.value
    }

    #[test]
    fn typing_opens_command_and_escape_restores_menu_and_draft() {
        for vi in [false, true] {
            let mut menu = Menu::new(vi, None);
            assert!(matches!(press(&mut menu, KeyCode::Char(' ')), Effect::Idle));
            assert!(matches!(
                menu.handle(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::ALT)),
                Effect::Idle
            ));
            assert!(menu.prompt.is_none());
            assert!(matches!(
                press(&mut menu, KeyCode::Char('l')),
                Effect::Redraw
            ));
            assert_eq!(command(&menu), "l");
            menu.refresh_suggestions();
            assert_eq!(menu.prompt.as_ref().unwrap().completions[0].text, "list");
            assert!(matches!(press(&mut menu, KeyCode::Esc), Effect::Redraw));
            assert!(menu.prompt.is_none());
            assert_eq!(menu.selected, 0);
            press(&mut menu, KeyCode::Char(':'));
            assert_eq!(command(&menu), "l");
            press(&mut menu, KeyCode::Esc);
            press(&mut menu, KeyCode::Char('s'));
            assert_eq!(command(&menu), "s");
            press(&mut menu, KeyCode::Esc);
            assert!(matches!(press(&mut menu, KeyCode::Esc), Effect::Exit));
        }
    }

    #[test]
    fn arrows_close_prompt_and_select_in_the_same_event_at_edges_and_in_filtered_menu() {
        let mut menu = Menu::new(false, None);
        press(&mut menu, KeyCode::Char('l'));
        assert!(matches!(press(&mut menu, KeyCode::Up), Effect::Redraw));
        assert!(menu.prompt.is_none());
        assert_eq!(menu.selected, 0);
        press(&mut menu, KeyCode::Char(':'));
        assert!(matches!(press(&mut menu, KeyCode::Down), Effect::Redraw));
        assert!(menu.prompt.is_none());
        assert_eq!(menu.selected, 1);
        press(&mut menu, KeyCode::Char('/'));
        menu.paste("tools");
        assert!(menu.items.len() > 1);
        menu.open_prompt(Input::default());
        assert!(matches!(press(&mut menu, KeyCode::Down), Effect::Redraw));
        assert!(menu.prompt.is_none());
        assert!(menu.search.is_some());
        assert_eq!(menu.selected, 1);
    }

    #[test]
    fn tab_fills_selected_suggestion_without_executing_it_and_cycles_both_ways() {
        let mut menu = Menu::new(false, None);
        press(&mut menu, KeyCode::Down);
        press(&mut menu, KeyCode::Enter);
        assert!(matches!(press(&mut menu, KeyCode::Enter), Effect::Idle));
        press(&mut menu, KeyCode::Tab);
        assert_eq!(command(&menu), "search");
        press(&mut menu, KeyCode::Esc);
        menu.paste("cnr workspace b --json");
        menu.prompt.as_mut().unwrap().input.cursor = "cnr workspace b".len();
        press(&mut menu, KeyCode::Tab);
        let first = command(&menu).to_owned();
        assert!(first.ends_with(" --json"));
        press(&mut menu, KeyCode::Tab);
        assert_ne!(command(&menu), first);
        press(&mut menu, KeyCode::BackTab);
        assert_eq!(command(&menu), first);
        match press(&mut menu, KeyCode::Enter) {
            Effect::Run(args) => assert_eq!(args.join(" "), first.strip_prefix("cnr ").unwrap()),
            _ => panic!("accepted command should run"),
        }
    }

    #[test]
    fn menu_shortcuts_are_literal_when_command_has_focus() {
        let mut menu = Menu::new(true, None);
        press(&mut menu, KeyCode::Char('?'));
        assert!(menu.help);
        press(&mut menu, KeyCode::Esc);
        press(&mut menu, KeyCode::Char('/'));
        assert!(menu.search.is_some());
        press(&mut menu, KeyCode::Esc);
        press(&mut menu, KeyCode::Char(':'));
        for ch in "/?:".chars() {
            press(&mut menu, KeyCode::Char(ch));
        }
        assert_eq!(command(&menu), "/?:");
    }

    #[test]
    fn preserves_quoted_paths_and_keeps_parse_errors_inline() {
        let mut menu = Menu::new(false, None);
        menu.paste(r#"cnr exec -- "C:\Program Files\Việt\tool.exe"#);
        assert!(matches!(press(&mut menu, KeyCode::Enter), Effect::Redraw));
        assert!(menu.prompt.as_ref().unwrap().error.is_some());
        menu.paste("\" \"\" 'two words'");
        match press(&mut menu, KeyCode::Enter) {
            Effect::Run(args) => assert_eq!(
                args,
                [
                    "exec",
                    "--",
                    r"C:\Program Files\Việt\tool.exe",
                    "",
                    "two words",
                ]
            ),
            _ => panic!("expected parsed command"),
        }
        assert!(menu.history.is_empty());
    }

    #[test]
    fn output_history_restores_the_draft_and_clear_keeps_the_prompt() {
        let mut menu = Menu::new(false, None);
        menu.paste("config show");
        assert!(matches!(press(&mut menu, KeyCode::Enter), Effect::Run(_)));
        let mut output = OutputView::new("cnr config show".into(), false);
        output.running = false;
        output.buffer = Buffer::from_streams("previous output".into(), "", None);
        menu.output = Some(output);
        menu.open_prompt(Input::default());
        menu.paste("clear");
        press(&mut menu, KeyCode::Up);
        assert_eq!(command(&menu), "config show");
        press(&mut menu, KeyCode::Down);
        assert_eq!(command(&menu), "clear");
        assert!(matches!(press(&mut menu, KeyCode::Enter), Effect::Redraw));
        assert_eq!(menu.output.as_mut().unwrap().buffer.layout(80), 0);
        assert_eq!(command(&menu), "");
        press(&mut menu, KeyCode::Esc);
        assert!(menu.output.is_none());
    }

    #[test]
    fn unicode_editing_keeps_a_base_and_combining_mark_together() {
        let mut input = Input::default();
        input.insert("a東京e\u{301}c", COMMAND_LIMIT);
        input.edit(key(KeyCode::Left), COMMAND_LIMIT);
        input.edit(key(KeyCode::Backspace), COMMAND_LIMIT);
        assert_eq!(input.value, "a東京c");
        assert_eq!(input.cursor, "a東京".len());
    }

    #[test]
    fn pasted_newlines_and_oversized_text_do_not_submit() {
        let mut menu = Menu::new(false, None);
        assert!(matches!(menu.paste("list\n"), Effect::Redraw));
        assert_eq!(command(&menu), "list ");
        menu.paste(&"x".repeat(COMMAND_LIMIT + 1));
        assert_eq!(command(&menu), "list ");
        assert!(matches!(press(&mut menu, KeyCode::Enter), Effect::Idle));
        assert!(windows_paste(&[Event::Key(key(KeyCode::Enter))], 0).is_none());
    }
}
