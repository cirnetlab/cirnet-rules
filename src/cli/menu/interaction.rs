use super::{
    COMMAND_LIMIT, COMMAND_TOO_LONG, Effect, Input, Menu, SEARCH_LIMIT,
    clipboard::{Clipboard, ClipboardError},
    control,
    editor::byte_at_column,
    terminal::{Frame, InputHit, Line, Tone},
    text,
};
use cirnetrules::operator::presentation;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Editor {
    Prompt,
    MenuSearch,
    OutputSearch,
    CheckSearch,
}

impl Editor {
    fn limit(self) -> usize {
        if self == Self::Prompt {
            COMMAND_LIMIT
        } else {
            SEARCH_LIMIT
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Action {
    Copy,
    Cut,
    Paste,
    Delete,
    SelectAll,
    Undo,
    Redo,
    CopyLine,
    CopyValue,
    CopyFinding,
    CopyReport,
}

impl Action {
    fn label(self, vi: bool) -> &'static str {
        text(
            match self {
                Self::Copy => ["Copy", "Sao chép"],
                Self::Cut => ["Cut", "Cắt"],
                Self::Paste => ["Paste", "Dán"],
                Self::Delete => ["Delete", "Xóa"],
                Self::SelectAll => ["Select all", "Chọn tất cả"],
                Self::Undo => ["Undo", "Hoàn tác"],
                Self::Redo => ["Redo", "Làm lại"],
                Self::CopyLine => ["Copy line", "Sao chép dòng"],
                Self::CopyValue => ["Copy value", "Sao chép giá trị"],
                Self::CopyFinding => ["Copy finding as text", "Sao chép mục dạng văn bản"],
                Self::CopyReport => ["Copy report as text", "Sao chép báo cáo dạng văn bản"],
            },
            vi,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
struct Point {
    row: usize,
    byte: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Selection {
    anchor: Point,
    end: Point,
}

impl Selection {
    fn bounds(self) -> (Point, Point) {
        (self.anchor.min(self.end), self.anchor.max(self.end))
    }

    fn active(self) -> bool {
        self.anchor != self.end
    }

    fn row(self, index: usize, len: usize) -> Option<Range<usize>> {
        let (start, end) = self.bounds();
        (start.row..=end.row).contains(&index).then(|| {
            let left = if index == start.row { start.byte } else { 0 };
            let right = if index == end.row { end.byte } else { len };
            left.min(len)..right.min(len)
        })
    }
}

#[derive(Default, PartialEq, Eq)]
struct Visible {
    first: u16,
    lines: Vec<String>,
}

impl Visible {
    fn capture(frame: &Frame) -> Self {
        let Some(rows) = &frame.readable else {
            return Self::default();
        };
        let mut bytes = 0;
        let lines = rows
            .clone()
            .take(512)
            .filter_map(|row| frame.rows.get(usize::from(row)))
            .map(Line::contents)
            .take_while(|line| {
                bytes += line.len();
                bytes <= super::clipboard::MAX_BYTES
            })
            .map(str::to_owned)
            .collect();
        Self {
            first: rows.start,
            lines,
        }
    }

    fn point(&self, column: u16, row: u16, clamp: bool) -> Option<Point> {
        let last = self.lines.len().checked_sub(1)?;
        let index = if clamp {
            usize::from(row.saturating_sub(self.first)).min(last)
        } else {
            usize::from(row.checked_sub(self.first)?)
        };
        let value = self.lines.get(index)?;
        Some(Point {
            row: index,
            byte: byte_at_column(value, usize::from(column)),
        })
    }

    fn all(&self) -> Option<Selection> {
        let row = self.lines.len().checked_sub(1)?;
        Some(Selection {
            anchor: Point::default(),
            end: Point {
                row,
                byte: self.lines[row].len(),
            },
        })
    }

    fn selected_text(&self, selection: Selection) -> Option<String> {
        selection.active().then(|| {
            self.lines
                .iter()
                .enumerate()
                .filter_map(|(index, line)| {
                    let range = selection.row(index, line.len())?;
                    Some(presentation::text(line.get(range)?))
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
    }
}

#[derive(Clone, Copy)]
enum Press {
    Input(Editor),
    Read { column: u16, row: u16 },
    Menu(usize),
    Popup(usize),
}

struct Popup {
    items: Vec<Action>,
    selected: usize,
    position: (u16, u16),
}

impl Popup {
    fn bounds(&self, size: (u16, u16), vi: bool) -> (u16, u16, u16, usize, usize) {
        let width = (self
            .items
            .iter()
            .map(|action| action.label(vi).width())
            .max()
            .unwrap_or(0)
            + 4)
        .min(usize::from(size.0.saturating_sub(1))) as u16;
        let count = self.items.len().min(usize::from(size.1));
        let first = self.selected.saturating_sub(count.saturating_sub(1));
        let left = self.position.0.min(size.0.saturating_sub(width + 1));
        let top = self.position.1.min(size.1.saturating_sub(count as u16));
        (left, top, width, first, count)
    }

    fn hit(&self, size: (u16, u16), vi: bool, column: u16, row: u16) -> Option<usize> {
        let (left, top, width, first, count) = self.bounds(size, vi);
        ((left..left + width).contains(&column) && (top..top + count as u16).contains(&row))
            .then(|| first + usize::from(row - top))
    }

    fn step(&mut self, offset: isize) {
        self.selected = self
            .selected
            .saturating_add_signed(offset)
            .min(self.items.len() - 1);
    }

    fn draw(&self, frame: &mut Frame, vi: bool) {
        let (left, top, width, first, count) = self.bounds(frame.size, vi);
        if width < 4 {
            return;
        }
        frame.cursor = None;
        for (index, action) in self.items.iter().enumerate().skip(first).take(count) {
            let row = usize::from(top) + index - first;
            let old = frame.rows[row].contents();
            let mut line = Line::default();
            let prefix = fit(old, usize::from(left));
            line.push(Tone::Plain, prefix);
            line.push(Tone::Plain, " ".repeat(usize::from(left) - prefix.width()));
            let tone = if index == self.selected {
                Tone::Selection
            } else {
                Tone::Heading
            };
            let label = fit(action.label(vi), usize::from(width) - 4);
            line.push(tone, if index == self.selected { "> " } else { "| " });
            line.push(tone, label);
            line.push(tone, " ".repeat(usize::from(width) - 3 - label.width()));
            line.push(tone, "|");
            frame.rows[row] = line;
        }
    }
}

fn fit(value: &str, width: usize) -> &str {
    let mut used = 0;
    for (offset, grapheme) in value.grapheme_indices(true) {
        used += grapheme.width();
        if used > width {
            return &value[..offset];
        }
    }
    value
}

#[derive(Default)]
pub(super) struct Interaction {
    visible: Visible,
    selection: Option<Selection>,
    line: Option<usize>,
    press: Option<Press>,
    popup: Option<Popup>,
    feedback: Option<String>,
    pub(super) read_focus: bool,
    last_cursor: Option<(u16, u16)>,
}

impl Interaction {
    pub(super) fn reset_output(&mut self) {
        *self = Self::default();
    }

    pub(super) fn decorate(&mut self, frame: &mut Frame, vi: bool) {
        self.last_cursor = frame.cursor;
        let visible = Visible::capture(frame);
        if visible != self.visible {
            self.visible = visible;
            self.selection = None;
            self.line = None;
        }
        if let Some(selection) = self.selection.filter(|selection| selection.active()) {
            for (index, value) in self.visible.lines.iter().enumerate() {
                if let Some(range) = selection.row(index, value.len()) {
                    frame.rows[usize::from(self.visible.first) + index].select(range);
                }
            }
        }
        if self.read_focus && frame.input.is_some() {
            frame.cursor = None;
        }
        if let Some(feedback) = &self.feedback
            && frame.size.1 > 1
        {
            let mut row = frame.size.1 - 1;
            if frame.input.as_ref().is_some_and(|input| input.row == row) {
                row -= 1;
            }
            let mut line = Line::default();
            line.push(
                Tone::Warning,
                fit(feedback, usize::from(frame.size.0.saturating_sub(1))),
            );
            frame.rows[usize::from(row)] = line;
        }
        if let Some(popup) = &self.popup {
            popup.draw(frame, vi);
        }
    }
}

pub(super) fn clipboard_safe_output(value: &str) -> bool {
    // Inspect the full source before allowing selection, so a partial token
    // or one of its wrapped rows cannot evade the check by losing its label.
    let lower = value.to_ascii_lowercase();
    !lower.contains("-----begin")
        && !lower.contains("bearer ")
        && !lower.contains("session_token")
        && !lower.contains("capability_token")
        && !lower.contains("\"session\"")
        && !lower.contains("\"capability\"")
        && !lower.contains("\"token\"")
        && !lower.contains("\"password\"")
        && !value
            .lines()
            .any(|line| presentation::text(line) == "[redacted]")
}

impl Menu {
    fn editor_focus(&self) -> Option<Editor> {
        if self.check_visible() {
            return self
                .output
                .as_ref()?
                .check
                .as_ref()?
                .searching()
                .then_some(Editor::CheckSearch);
        }
        if self
            .output
            .as_ref()
            .is_some_and(|output| output.find.is_some())
        {
            return Some(Editor::OutputSearch);
        }
        if self
            .wizard
            .as_ref()
            .is_some_and(|wizard| !wizard.suspended && !wizard.editing_text())
            || self.interaction.read_focus
        {
            return None;
        }
        if self.prompt.is_some() {
            Some(Editor::Prompt)
        } else if self.search.is_some() {
            Some(Editor::MenuSearch)
        } else {
            None
        }
    }

    fn editor_mut(&mut self, editor: Editor) -> Option<&mut Input> {
        match editor {
            Editor::Prompt => self.prompt.as_mut().map(|prompt| &mut prompt.input),
            Editor::MenuSearch => self.search.as_mut(),
            Editor::OutputSearch => self.output.as_mut()?.find.as_mut(),
            Editor::CheckSearch => self.output.as_mut()?.check.as_mut()?.search_mut(),
        }
    }

    fn editor_changed(&mut self, editor: Editor) {
        match editor {
            Editor::Prompt => {
                if let Some(prompt) = &mut self.prompt {
                    prompt.changed();
                }
                self.history_index = None;
            }
            Editor::MenuSearch => self.refilter(),
            Editor::OutputSearch => {
                if let Some(output) = &mut self.output
                    && let Some(find) = &output.find
                {
                    output.query.clone_from(&find.value);
                    output.hit = None;
                    output.search(false);
                }
            }
            Editor::CheckSearch => {
                if let Some(output) = &mut self.output
                    && let (Some(check), Some(report)) = (&mut output.check, &output.check_report)
                {
                    check.search_changed(report);
                }
            }
        }
    }

    pub(super) fn paste_editor(&mut self, value: &str) -> Effect {
        if self.interaction.popup.is_some() {
            return Effect::Idle;
        }
        let Some(editor) = self.editor_focus() else {
            return Effect::Idle;
        };
        let Some(input) = self.editor_mut(editor) else {
            return Effect::Idle;
        };
        match input.paste_sanitized(value, editor.limit()) {
            Ok(true) => {
                self.editor_changed(editor);
                Effect::Redraw
            }
            Ok(false) => Effect::Idle,
            Err(_) => {
                if editor == Editor::Prompt
                    && let Some(prompt) = &mut self.prompt
                {
                    prompt.error = Some(COMMAND_TOO_LONG);
                } else {
                    self.interaction.feedback = Some(
                        text(
                            [
                                "Search text exceeds its limit.",
                                "Văn bản tìm kiếm vượt giới hạn.",
                            ],
                            self.vi,
                        )
                        .into(),
                    );
                }
                Effect::Redraw
            }
        }
    }

    fn clipboard_error(&mut self, error: ClipboardError) -> Effect {
        self.interaction.feedback = Some(format!(
            "{} · {}",
            error.code(),
            text(
                [
                    "Use the terminal's copy or paste shortcut.",
                    "Dùng phím sao chép hoặc dán của terminal."
                ],
                self.vi
            )
        ));
        Effect::Redraw
    }

    fn write_clipboard(&mut self, clipboard: &mut impl Clipboard, value: &str) -> bool {
        if value.is_empty() {
            return false;
        }
        if let Err(error) = clipboard.set_text(value) {
            self.clipboard_error(error);
            return false;
        }
        self.interaction.feedback = Some(text(["Copied", "Đã sao chép"], self.vi).into());
        true
    }

    fn clipboard_action(&mut self, action: Action, clipboard: &mut impl Clipboard) -> Effect {
        if let Some(editor) = self.editor_focus() {
            if action == Action::Paste {
                return match clipboard.get_text() {
                    Ok(value) => self.paste_editor(&value),
                    Err(error) => self.clipboard_error(error),
                };
            }
            let Some(input) = self.editor_mut(editor) else {
                return Effect::Idle;
            };
            if matches!(action, Action::Copy | Action::Cut) {
                let Some(value) = input.selected_text().map(str::to_owned) else {
                    return Effect::Idle;
                };
                if !clipboard_safe_output(&input.value) {
                    self.interaction.feedback = Some(
                        text(
                            [
                                "Sensitive text was withheld.",
                                "Đã giữ lại văn bản nhạy cảm.",
                            ],
                            self.vi,
                        )
                        .into(),
                    );
                    return Effect::Redraw;
                }
                if self.write_clipboard(clipboard, &value)
                    && action == Action::Cut
                    && self.editor_mut(editor).is_some_and(Input::delete_selection)
                {
                    self.editor_changed(editor);
                }
                return Effect::Redraw;
            }
            let changed = match action {
                Action::Delete => input.delete_selection(),
                Action::SelectAll => {
                    return if input.select_all() {
                        Effect::Redraw
                    } else {
                        Effect::Idle
                    };
                }
                Action::Undo => input.undo(),
                Action::Redo => input.redo(),
                _ => false,
            };
            return if changed {
                self.editor_changed(editor);
                Effect::Redraw
            } else {
                Effect::Idle
            };
        }

        if self.interaction.visible.lines.is_empty() {
            return Effect::Idle;
        }
        let visible = &self.interaction.visible;
        let value = match action {
            Action::Copy => self
                .interaction
                .selection
                .and_then(|selection| visible.selected_text(selection)),
            Action::CopyLine => visible
                .lines
                .get(self.interaction.line.unwrap_or(0))
                .map(|line| presentation::text(line.trim())),
            Action::CopyValue | Action::CopyFinding | Action::CopyReport => self
                .output
                .as_ref()
                .and_then(|output| output.check.as_ref().zip(output.check_report.as_ref()))
                .filter(|(check, _)| self.check_visible() && !check.is_json())
                .and_then(|(check, report)| match action {
                    Action::CopyValue => check.copy_value(report),
                    Action::CopyFinding => check.finding_text(report, self.vi),
                    _ => Some(report.render_for(self.vi)),
                }),
            Action::SelectAll => {
                self.interaction.selection = visible.all();
                return Effect::Redraw;
            }
            _ => None,
        };
        if let Some(value) = value {
            self.write_clipboard(clipboard, &value);
            Effect::Redraw
        } else {
            Effect::Idle
        }
    }

    fn open_context_menu(&mut self, position: Option<(u16, u16)>) -> Effect {
        let mut items = Vec::new();
        if let Some(input) = self
            .editor_focus()
            .and_then(|editor| self.editor_mut(editor))
        {
            if input.selection().is_some() {
                items.extend([Action::Copy, Action::Cut, Action::Paste, Action::Delete]);
            } else {
                items.extend([Action::Paste, Action::SelectAll]);
                if input.can_undo() {
                    items.push(Action::Undo);
                }
                if input.can_redo() {
                    items.push(Action::Redo);
                }
            }
        } else if !self.interaction.visible.lines.is_empty() {
            if self.interaction.selection.is_some_and(Selection::active) {
                items.push(Action::Copy);
            }
            items.push(Action::CopyLine);
            if self.check_visible() {
                items.extend([Action::CopyValue, Action::CopyFinding, Action::CopyReport]);
            }
            items.push(Action::SelectAll);
        }
        if items.is_empty() {
            return Effect::Idle;
        }
        self.interaction.press = None;
        self.interaction.popup = Some(Popup {
            items,
            selected: 0,
            position: position.or(self.interaction.last_cursor).unwrap_or((2, 2)),
        });
        Effect::Redraw
    }

    fn choose_context_action(&mut self, index: usize, clipboard: &mut impl Clipboard) -> Effect {
        if let Some(action) = self
            .interaction
            .popup
            .take()
            .and_then(|popup| popup.items.get(index).copied())
        {
            self.clipboard_action(action, clipboard);
        }
        Effect::Redraw
    }

    pub(super) fn interaction_key(
        &mut self,
        key: KeyEvent,
        clipboard: &mut impl Clipboard,
    ) -> Option<Effect> {
        self.interaction.feedback = None;
        if let Some(popup) = &mut self.interaction.popup {
            return Some(match key.code {
                KeyCode::Esc => {
                    self.interaction.popup = None;
                    Effect::Redraw
                }
                KeyCode::Up => {
                    popup.step(-1);
                    Effect::Redraw
                }
                KeyCode::Down => {
                    popup.step(1);
                    Effect::Redraw
                }
                KeyCode::Enter => {
                    let index = popup.selected;
                    self.choose_context_action(index, clipboard)
                }
                _ => Effect::Idle,
            });
        }
        if key.code == KeyCode::F(10) && key.modifiers == KeyModifiers::SHIFT
            || key.code == KeyCode::Menu
        {
            return Some(self.open_context_menu(None));
        }
        let editor = self.editor_focus();
        if key.code == KeyCode::F(6)
            && self.output.is_some()
            && !self.check_visible()
            && self
                .output
                .as_ref()
                .is_some_and(|output| output.find.is_none())
        {
            self.interaction.read_focus = !self.interaction.read_focus;
            self.interaction.selection = None;
            return Some(Effect::Redraw);
        }
        if key.code == KeyCode::Esc {
            if editor
                .and_then(|editor| self.editor_mut(editor))
                .is_some_and(Input::clear_selection)
            {
                return Some(Effect::Redraw);
            }
            if self
                .interaction
                .selection
                .take()
                .is_some_and(Selection::active)
            {
                return Some(Effect::Redraw);
            }
        }
        let selection = if let Some(editor) = editor {
            self.editor_mut(editor)
                .is_some_and(|input| input.selection().is_some())
        } else {
            self.interaction.selection.is_some_and(Selection::active)
        };
        let action = if control(key, 'c') && selection {
            Action::Copy
        } else if control(key, 'v') {
            Action::Paste
        } else if control(key, 'x') {
            Action::Cut
        } else if control(key, 'a') && editor.is_none() {
            Action::SelectAll
        } else {
            return None;
        };
        Some(self.clipboard_action(action, clipboard))
    }

    fn check_click(&mut self, column: u16, row: u16) -> bool {
        if self.check_visible()
            && let Some(output) = &mut self.output
            && let (Some(check), Some(report)) = (&mut output.check, &output.check_report)
        {
            return check.click(report, column, row);
        }
        false
    }

    fn wheel(&mut self, lines: isize) -> Effect {
        self.interaction.selection = None;
        self.interaction.press = None;
        if self.check_visible()
            && let Some(output) = &mut self.output
            && let (Some(check), Some(report)) = (&mut output.check, &output.check_report)
        {
            return if check.wheel(report, lines) {
                Effect::Redraw
            } else {
                Effect::Idle
            };
        }
        self.output
            .as_mut()
            .map_or(Effect::Idle, |output| output.scroll_by(lines))
    }

    fn move_input(&mut self, editor: Editor, hit: &InputHit, column: u16, extend: bool) -> bool {
        let column = column
            .clamp(hit.columns.start, hit.columns.end)
            .saturating_sub(hit.columns.start);
        let Some(input) = self.editor_mut(editor) else {
            return false;
        };
        let offset = input.offset_at_column(hit.offset, usize::from(column));
        let changed = input.move_to(offset, extend);
        if changed && editor == Editor::Prompt {
            self.editor_changed(editor);
        }
        changed
    }

    fn focus_input_at(&mut self, frame: &Frame, column: u16, row: u16) -> Option<Editor> {
        if frame
            .input
            .as_ref()
            .is_some_and(|hit| hit.row == row && hit.columns.contains(&column))
        {
            self.interaction.read_focus = false;
            return self.editor_focus();
        }
        None
    }

    pub(super) fn mouse(
        &mut self,
        event: MouseEvent,
        frame: Option<&Frame>,
        clipboard: &mut impl Clipboard,
    ) -> Effect {
        self.interaction.feedback = None;
        let MouseEvent {
            kind, column, row, ..
        } = event;
        if let Some(popup) = &mut self.interaction.popup {
            let hit = frame.and_then(|frame| popup.hit(frame.size, self.vi, column, row));
            return match kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    if let Some(index) = hit {
                        popup.selected = index;
                        self.interaction.press = Some(Press::Popup(index));
                    } else {
                        self.interaction.popup = None;
                        self.interaction.press = None;
                    }
                    Effect::Redraw
                }
                MouseEventKind::Up(MouseButton::Left) => {
                    if let Some(Press::Popup(index)) = self.interaction.press.take()
                        && hit == Some(index)
                    {
                        self.choose_context_action(index, clipboard)
                    } else {
                        Effect::Idle
                    }
                }
                MouseEventKind::Drag(MouseButton::Left) => {
                    self.interaction.press = None;
                    Effect::Idle
                }
                _ => Effect::Idle,
            };
        }
        match kind {
            MouseEventKind::ScrollUp => return self.wheel(-3),
            MouseEventKind::ScrollDown => return self.wheel(3),
            _ => {}
        }
        let Some(frame) = frame else {
            self.interaction.press = None;
            return Effect::Idle;
        };
        match kind {
            MouseEventKind::Down(MouseButton::Right) => {
                let editor = self.focus_input_at(frame, column, row);
                if editor.is_none() {
                    let Some(point) = self.interaction.visible.point(column, row, false) else {
                        return Effect::Idle;
                    };
                    self.interaction.read_focus = !self.check_visible();
                    self.interaction.line = Some(point.row);
                }
                self.open_context_menu(Some((column, row)))
            }
            MouseEventKind::Down(MouseButton::Left) => {
                self.interaction.press = None;
                self.interaction.selection = None;
                if let Some(editor) = self.focus_input_at(frame, column, row)
                    && let Some(hit) = &frame.input
                {
                    self.move_input(editor, hit, column, false);
                    self.interaction.press = Some(Press::Input(editor));
                    return Effect::Redraw;
                }
                if let Some(point) = self.interaction.visible.point(column, row, false) {
                    self.interaction.read_focus = !self.check_visible();
                    self.interaction.line = Some(point.row);
                    self.interaction.selection = Some(Selection {
                        anchor: point,
                        end: point,
                    });
                    self.interaction.press = Some(Press::Read { column, row });
                    return Effect::Redraw;
                }
                if self.check_visible() {
                    self.interaction.press = Some(Press::Read { column, row });
                } else if let Some(index) = frame.menu_hit(column, row) {
                    self.interaction.press = Some(Press::Menu(index));
                }
                Effect::Redraw
            }
            MouseEventKind::Drag(MouseButton::Left) => match self.interaction.press {
                Some(Press::Input(editor)) if self.editor_focus() == Some(editor) => {
                    let changed = frame
                        .input
                        .as_ref()
                        .is_some_and(|hit| self.move_input(editor, hit, column, true));
                    if changed {
                        Effect::Redraw
                    } else {
                        Effect::Idle
                    }
                }
                Some(Press::Read { .. }) => self.extend_output(column, row),
                _ => {
                    self.interaction.press = None;
                    Effect::Idle
                }
            },
            MouseEventKind::Up(MouseButton::Left) => match self.interaction.press.take() {
                Some(Press::Menu(index))
                    if frame.menu_hit(column, row) == Some(index) && index < self.items.len() =>
                {
                    self.close_main_prompt();
                    self.select(index);
                    self.activate()
                }
                Some(Press::Read {
                    column: origin_x,
                    row: origin_y,
                }) if (origin_x, origin_y) == (column, row)
                    && !self.interaction.selection.is_some_and(Selection::active) =>
                {
                    if self.check_click(column, row) {
                        Effect::Redraw
                    } else {
                        Effect::Idle
                    }
                }
                Some(Press::Read { .. }) => self.extend_output(column, row),
                Some(Press::Input(editor))
                    if self.editor_focus() == Some(editor)
                        && self
                            .editor_mut(editor)
                            .is_some_and(|input| input.selection().is_some()) =>
                {
                    if let Some(hit) = &frame.input {
                        self.move_input(editor, hit, column, true);
                    }
                    Effect::Redraw
                }
                _ => Effect::Idle,
            },
            _ => Effect::Idle,
        }
    }

    fn extend_output(&mut self, column: u16, row: u16) -> Effect {
        if let Some(selection) = &mut self.interaction.selection
            && let Some(point) = self.interaction.visible.point(column, row, true)
        {
            selection.end = point;
            Effect::Redraw
        } else {
            Effect::Idle
        }
    }

    pub(super) fn focus_lost(&mut self) -> Effect {
        self.interaction.popup = None;
        self.interaction.selection = None;
        self.interaction.press = None;
        if let Some(editor) = self.editor_focus()
            && let Some(input) = self.editor_mut(editor)
        {
            input.clear_selection();
        }
        Effect::Redraw
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::{view, viewport::Buffer};

    #[derive(Default)]
    struct MemoryClipboard {
        text: String,
        reads: usize,
        writes: usize,
        deny: bool,
    }

    impl Clipboard for MemoryClipboard {
        fn get_text(&mut self) -> Result<String, ClipboardError> {
            self.reads += 1;
            if self.deny {
                Err(ClipboardError::Unavailable)
            } else {
                Ok(self.text.clone())
            }
        }

        fn set_text(&mut self, text: &str) -> Result<(), ClipboardError> {
            self.writes += 1;
            if self.deny {
                return Err(ClipboardError::PermissionDenied);
            }
            self.text = text.into();
            Ok(())
        }
    }

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    fn ctrl(ch: char) -> KeyEvent {
        key(KeyCode::Char(ch), KeyModifiers::CONTROL)
    }

    fn mouse(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn selection_copies_before_cancel_and_cut_paste_undo_operate_on_one_edit() {
        let mut menu = Menu::new(false, None);
        let mut clipboard = MemoryClipboard::default();
        menu.begin_output(&["check".into()]);
        menu.open_prompt(Input::at_end("abc".into()));
        for _ in 0..2 {
            menu.handle_with_clipboard(key(KeyCode::Left, KeyModifiers::SHIFT), &mut clipboard);
        }
        assert!(matches!(
            menu.handle_with_clipboard(ctrl('c'), &mut clipboard),
            Effect::Redraw
        ));
        assert_eq!(clipboard.text, "bc");
        menu.handle_with_clipboard(ctrl('x'), &mut clipboard);
        assert_eq!(menu.prompt.as_ref().unwrap().input.value, "a");
        clipboard.text = "b\r\nc".into();
        menu.handle_with_clipboard(ctrl('v'), &mut clipboard);
        assert_eq!(menu.prompt.as_ref().unwrap().input.value, "ab c");
        menu.handle_with_clipboard(ctrl('z'), &mut clipboard);
        assert_eq!(menu.prompt.as_ref().unwrap().input.value, "a");
        menu.handle_with_clipboard(ctrl('y'), &mut clipboard);
        assert_eq!(menu.prompt.as_ref().unwrap().input.value, "ab c");
        assert!(matches!(
            menu.handle_with_clipboard(ctrl('c'), &mut clipboard),
            Effect::Cancel
        ));
        assert!(matches!(
            menu.handle_with_clipboard(key(KeyCode::Esc, KeyModifiers::NONE), &mut clipboard),
            Effect::Cancel
        ));
    }

    #[test]
    fn failed_cut_and_sensitive_selection_preserve_the_input() {
        let mut menu = Menu::new(false, None);
        let mut clipboard = MemoryClipboard {
            deny: true,
            ..Default::default()
        };
        menu.open_prompt(Input::at_end("Việt 👩🏽‍💻".into()));
        menu.handle_with_clipboard(ctrl('a'), &mut clipboard);
        menu.handle_with_clipboard(ctrl('x'), &mut clipboard);
        assert_eq!(menu.prompt.as_ref().unwrap().input.value, "Việt 👩🏽‍💻");
        assert!(
            menu.interaction
                .feedback
                .as_deref()
                .unwrap()
                .contains("CLIPBOARD_PERMISSION_DENIED")
        );
        let secret = "OPENAI_API_KEY=fixture-only";
        menu.open_prompt(Input::at_end(secret.into()));
        menu.handle_with_clipboard(ctrl('a'), &mut clipboard);
        menu.handle_with_clipboard(ctrl('c'), &mut clipboard);
        assert_eq!(clipboard.writes, 1);
        assert!(clipboard.text.is_empty());
    }

    #[test]
    fn right_click_and_drag_select_graphemes_without_activating_a_command() {
        let mut menu = Menu::new(false, None);
        let mut clipboard = MemoryClipboard::default();
        menu.open_prompt(Input::at_end("a東京e\u{301}".into()));
        let frame = view::frame(&mut menu, (80, 24), false, false);
        let hit = frame.input.as_ref().unwrap();
        let x = hit.columns.start;
        let y = hit.row;
        assert!(matches!(
            menu.mouse(
                mouse(MouseEventKind::Down(MouseButton::Left), x + 1, y),
                Some(&frame),
                &mut clipboard
            ),
            Effect::Redraw
        ));
        menu.mouse(
            mouse(MouseEventKind::Drag(MouseButton::Left), x + 5, y),
            Some(&frame),
            &mut clipboard,
        );
        menu.mouse(
            mouse(MouseEventKind::Up(MouseButton::Left), x + 5, y),
            Some(&frame),
            &mut clipboard,
        );
        assert_eq!(
            menu.prompt.as_ref().unwrap().input.selected_text(),
            Some("東京")
        );
        menu.mouse(
            mouse(MouseEventKind::Down(MouseButton::Right), x + 2, y),
            Some(&frame),
            &mut clipboard,
        );
        assert_eq!(
            menu.interaction.popup.as_ref().unwrap().items,
            [Action::Copy, Action::Cut, Action::Paste, Action::Delete]
        );
        assert!(matches!(
            menu.handle_with_clipboard(key(KeyCode::Enter, KeyModifiers::NONE), &mut clipboard),
            Effect::Redraw
        ));
        assert_eq!(clipboard.text, "東京");
        assert_eq!(menu.prompt.as_ref().unwrap().input.value, "a東京e\u{301}");
    }

    #[test]
    fn read_only_selection_excludes_output_paste_and_token_bearing_buffers() {
        let mut menu = Menu::new(false, None);
        let mut clipboard = MemoryClipboard::default();
        menu.begin_output(&["task".into()]);
        let output = menu.output.as_mut().unwrap();
        output.running = false;
        output.buffer = Buffer::from_streams("result for 東京e\u{301}\nnext line".into(), "", None);
        let frame = view::frame(&mut menu, (40, 12), false, false);
        let row = frame.readable.as_ref().unwrap().start;
        menu.mouse(
            mouse(MouseEventKind::Down(MouseButton::Left), 2, row),
            Some(&frame),
            &mut clipboard,
        );
        menu.mouse(
            mouse(MouseEventKind::Drag(MouseButton::Left), 24, row),
            Some(&frame),
            &mut clipboard,
        );
        menu.mouse(
            mouse(MouseEventKind::Up(MouseButton::Left), 24, row),
            Some(&frame),
            &mut clipboard,
        );
        menu.handle_with_clipboard(ctrl('c'), &mut clipboard);
        assert!(clipboard.text.contains("東京e\u{301}"));
        menu.handle_with_clipboard(ctrl('v'), &mut clipboard);
        menu.handle_with_clipboard(ctrl('x'), &mut clipboard);
        assert_eq!(clipboard.reads, 0);
        assert_eq!(clipboard.writes, 1);
        assert!(menu.prompt.as_ref().unwrap().input.value.is_empty());

        menu.output.as_mut().unwrap().buffer =
            Buffer::from_streams("{\"session_token\": \"fixture-only\"}".into(), "", None);
        let frame = view::frame(&mut menu, (40, 12), false, false);
        assert!(frame.readable.is_none());
        assert!(menu.interaction.selection.is_none());
    }

    #[test]
    fn a_menu_drag_and_wizard_action_paste_have_no_effects() {
        let mut menu = Menu::new(false, None);
        let mut clipboard = MemoryClipboard {
            text: "YES\n".into(),
            ..Default::default()
        };
        let frame = view::frame(&mut menu, (80, 24), false, false);
        let region = frame.menu.as_ref().unwrap();
        let (x, y) = (region.columns.start, region.rows.start);
        assert!(matches!(
            menu.mouse(
                mouse(MouseEventKind::Down(MouseButton::Left), x, y),
                Some(&frame),
                &mut clipboard
            ),
            Effect::Redraw
        ));
        menu.mouse(
            mouse(MouseEventKind::Drag(MouseButton::Left), x + 1, y),
            Some(&frame),
            &mut clipboard,
        );
        assert!(matches!(
            menu.mouse(
                mouse(MouseEventKind::Up(MouseButton::Left), x + 1, y),
                Some(&frame),
                &mut clipboard
            ),
            Effect::Idle
        ));

        menu.open_wizard(cirnetrules::operator::SetupInput {
            scope: cirnetrules::operator::Scope {
                governance_root: Some("missing-fixture-authority".into()),
                agent: Some("codex".into()),
                repo: Some("cirnet-rules".into()),
                ..Default::default()
            },
            ..Default::default()
        });
        assert!(matches!(
            menu.handle_with_clipboard(ctrl('v'), &mut clipboard),
            Effect::Idle
        ));
        assert!(matches!(menu.paste("YES\r\n"), Effect::Idle));
        assert_eq!(clipboard.reads, 0);
        assert!(menu.prompt.as_ref().unwrap().input.value.is_empty());
    }
}
