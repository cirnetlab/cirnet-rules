use super::{
    Input, Menu, Prompt,
    completion::Source,
    terminal::{Frame, Line, MenuRows, Tone},
    text,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const NAME: &str = "CIRNET RULES";
const WORDMARK_WIDTH: usize = 90;
const BANNER_WIDTH: usize = WORDMARK_WIDTH + 1;
// Each letter uses the same seven-pixel width and nine-pixel height.
// Half blocks keep the pixels proportional in a terminal cell.
const WORDMARK: [&str; 5] = [
    "▄██████ ███████ ██████▄ ██▄  ██ ███████ ███████    ██████▄ ██   ██ ██      ███████ ▄██████",
    "██        ███   ██   ██ ███▄ ██ ██        ███      ██   ██ ██   ██ ██      ██      ██",
    "██        ███   ████▀▀  ██ ████ ██▀▀▀▀    ███      ████▀▀  ██   ██ ██      ██▀▀▀▀   ▀▀▀▀█▄",
    "██▄▄▄▄▄ ▄▄███▄▄ ██ ▀█▄  ██  ███ ██▄▄▄▄▄   ███      ██ ▀█▄  ██▄▄▄██ ██▄▄▄▄▄ ██▄▄▄▄▄ ▄▄▄▄▄██",
    " ▀▀▀▀▀▀ ▀▀▀▀▀▀▀ ▀▀   ▀▀ ▀▀   ▀▀ ▀▀▀▀▀▀▀   ▀▀▀      ▀▀   ▀▀  ▀▀▀▀▀  ▀▀▀▀▀▀▀ ▀▀▀▀▀▀▀ ▀▀▀▀▀▀",
];

fn clip(value: &str, width: usize) -> String {
    let mut remaining = width;
    let mut result = String::new();
    for ch in value.chars() {
        let Some(columns) = ch.width() else {
            continue;
        };
        if columns > remaining {
            break;
        }
        result.push(ch);
        remaining -= columns;
    }
    result
}

fn tail(value: &str, width: usize) -> &str {
    let mut remaining = width;
    let mut start = value.len();
    for (index, ch) in value.char_indices().rev() {
        let columns = ch.width().unwrap_or(0);
        if columns > remaining {
            break;
        }
        if columns > 0 {
            start = index;
            remaining -= columns;
        }
    }
    &value[start..]
}

fn indented(margin: usize) -> Line {
    let mut line = Line::default();
    line.push(Tone::Plain, " ".repeat(margin));
    line
}

fn label(frame: &mut Frame, row: u16, margin: usize, width: usize, tone: Tone, value: &str) {
    let mut line = indented(margin);
    line.push(tone, clip(value, width));
    frame.rows[usize::from(row)] = line;
}

fn input_line(
    frame: &mut Frame,
    row: u16,
    margin: usize,
    width: usize,
    prefix: &str,
    input: &Input,
    ghost: &str,
) {
    let tone = if COMMAND.contains(&prefix) {
        Tone::Accent
    } else {
        Tone::Heading
    };
    let prefix = clip(prefix, width.saturating_sub(1));
    let available = width - prefix.width();
    let (before, after) = input.value.split_at(input.cursor);
    let before = tail(before, available.saturating_sub(1));
    let mut line = indented(margin);
    line.push(tone, &prefix);
    line.push(Tone::Plain, before);
    line.push(Tone::Plain, clip(after, available - before.width()));
    if after.is_empty() {
        line.push(Tone::Muted, clip(ghost, available - before.width()));
    }
    let cursor = margin + prefix.width() + before.width();
    frame.cursor = Some((cursor as u16, row));
    frame.rows[usize::from(row)] = line;
}

fn wrap(value: &str, width: usize) -> Vec<String> {
    let mut rows = Vec::new();
    let mut line = String::new();
    for word in value.split_whitespace() {
        if !line.is_empty() && line.width() + word.width() + 1 > width {
            rows.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        rows.push(line);
    }
    rows
}

const COMMAND: [&str; 2] = ["Command  › ", "Lệnh  › "];
const MENU_HINT: [&str; 2] = [
    "↑↓ / click select  ·  Enter open  ·  : command  ·  / search  ·  ? help",
    "↑↓ / click chọn  ·  Enter mở  ·  : lệnh  ·  / tìm  ·  ? trợ giúp",
];
const OUTPUT_HINT: [&str; 2] = [
    "PgUp/PgDn scroll  ·  Tab complete  ·  Esc back  ·  Ctrl+C exit",
    "PgUp/PgDn cuộn  ·  Tab gợi ý  ·  Esc quay lại  ·  Ctrl+C thoát",
];
const RUNNING_HINT: [&str; 2] = [
    "PgUp/PgDn scroll  ·  Tab complete  ·  Ctrl+C cancel",
    "PgUp/PgDn cuộn  ·  Tab gợi ý  ·  Ctrl+C hủy",
];

#[derive(Default)]
pub(super) struct Banner {
    cached: Option<(usize, bool, Vec<Line>)>,
}

impl Banner {
    fn lines(&mut self, inset: usize, color: bool) -> &[Line] {
        if self
            .cached
            .as_ref()
            .is_none_or(|(x, colors, _)| *x != inset || *colors != color)
        {
            let art: Vec<Vec<char>> = WORDMARK.iter().map(|line| line.chars().collect()).collect();
            let mut lines = Vec::with_capacity(art.len() + 1);
            for y in 0..=art.len() {
                let mut line = indented(inset);
                for x in 0..BANNER_WIDTH {
                    let face = x
                        .checked_sub(1)
                        .and_then(|x| art.get(y).and_then(|line| line.get(x)))
                        .copied()
                        .unwrap_or(' ');
                    let shadow = y
                        .checked_sub(1)
                        .and_then(|y| art.get(y).and_then(|line| line.get(x)))
                        .is_some_and(|ch| *ch != ' ');
                    if face != ' ' {
                        line.push(
                            Tone::Banner((x.saturating_sub(1) * 4 / WORDMARK_WIDTH).min(3)),
                            face.encode_utf8(&mut [0; 4]),
                        );
                    } else if shadow && color {
                        line.push(Tone::Shadow, "░");
                    } else {
                        line.push(Tone::Plain, " ");
                    }
                }
                lines.push(line);
            }
            self.cached = Some((inset, color, lines));
        }
        &self.cached.as_ref().expect("banner initialized").2
    }
}

fn ghost(prompt: &Prompt) -> &str {
    if prompt.error.is_some()
        || prompt.accepted.is_some()
        || prompt.input.cursor != prompt.input.value.len()
    {
        return "";
    }
    prompt
        .completions
        .first()
        .filter(|item| item.insert_range.end == prompt.input.cursor)
        .and_then(|item| {
            item.text
                .strip_prefix(&prompt.input.value[item.insert_range.clone()])
        })
        .unwrap_or_default()
}

fn popup_rows(prompt: Option<&Prompt>, available: u16) -> u16 {
    let count = prompt.map_or(0, |p| {
        if p.error.is_some() {
            1
        } else {
            p.completions.len()
        }
    });
    (count.min(5) as u16).min(available)
}

fn popup(
    frame: &mut Frame,
    prompt: &Prompt,
    rows: std::ops::Range<u16>,
    margin: usize,
    width: usize,
    vi: bool,
) {
    if rows.is_empty() {
        return;
    }
    if let Some(error) = prompt.error {
        label(
            frame,
            rows.end - 1,
            margin,
            width,
            Tone::Accent,
            text(error, vi),
        );
        return;
    }
    let focus = prompt.accepted.as_ref().map_or(0, |(index, _)| *index);
    let start = (focus + 1).saturating_sub(rows.len());
    for (row, (index, candidate)) in rows.zip(prompt.completions.iter().enumerate().skip(start)) {
        let mut line = indented(margin);
        let selected = index == focus;
        let tone = if selected { Tone::Accent } else { Tone::Plain };
        let name = clip(
            &format!("{}{}", if selected { "› " } else { "  " }, candidate.label),
            width.min(27),
        );
        let used = name.width();
        line.push(tone, name);
        if width.saturating_sub(used) >= 12 {
            let source = if matches!(candidate.source, Source::Pinned | Source::Installed) {
                candidate.source.label(vi)
            } else {
                candidate.kind.label(vi)
            };
            line.push(
                Tone::Muted,
                clip(
                    &format!("  {source}  ·  {}", candidate.description),
                    width - used,
                ),
            );
        }
        frame.rows[usize::from(row)] = line;
    }
}

fn output_frame(menu: &mut Menu, size: (u16, u16)) -> Frame {
    let mut frame = Frame::new(size);
    let columns = usize::from(size.0.saturating_sub(1)).min(108);
    let height = size.1;
    if columns == 0 || height == 0 {
        return frame;
    }
    let inset = if columns >= 24 { "  " } else { "" };
    let margin = inset.len();
    let width = columns - 2 * margin;
    let output = menu.output.as_mut().expect("output mode");
    let hint = (height >= 6).then_some(height - 1);
    let command = hint.map_or(height - 1, |row| row - 1);
    let status = (height >= 4).then(|| command - 1);
    let first = u16::from(height >= 2);
    let mut end = status.unwrap_or(command);
    if first != 0 {
        label(
            &mut frame,
            0,
            margin,
            width,
            Tone::Accent,
            &format!("{NAME}  /  {}", output.title),
        );
    }
    let find_row = output.find.as_ref().map(|_| {
        if end > first + 1 {
            end -= 1;
            end
        } else {
            command
        }
    });
    let suggestions = if find_row.is_none() {
        popup_rows(menu.prompt.as_ref(), end.saturating_sub(first + 1))
    } else {
        0
    };
    let visible = end.saturating_sub(first + suggestions);
    output.layout(width, usize::from(visible));
    for row in 0..visible {
        if let Some(line) =
            output
                .buffer
                .line(output.scroll + usize::from(row), inset, output.hit.as_ref())
        {
            frame.rows[usize::from(first + row)] = line;
        }
    }
    if let Some(row) = status {
        label(&mut frame, row, margin, width, output.tone, &output.status);
    }
    if let Some(prompt) = &menu.prompt {
        popup(
            &mut frame,
            prompt,
            end - suggestions..end,
            margin,
            width,
            menu.vi,
        );
        input_line(
            &mut frame,
            command,
            margin,
            width,
            text(COMMAND, menu.vi),
            &prompt.input,
            ghost(prompt),
        );
    }
    if let Some(row) = hint {
        label(
            &mut frame,
            row,
            margin,
            width,
            Tone::Muted,
            text(
                if output.running {
                    RUNNING_HINT
                } else {
                    OUTPUT_HINT
                },
                menu.vi,
            ),
        );
    }
    if let (Some(row), Some(find)) = (find_row, &output.find) {
        let feedback = if !find.value.is_empty() && output.hit.is_none() {
            text(["  (no matches)", "  (không có kết quả)"], menu.vi)
        } else {
            ""
        };
        input_line(
            &mut frame,
            row,
            margin,
            width,
            text(["Find  › ", "Tìm  › "], menu.vi),
            find,
            feedback,
        );
    }
    frame
}
pub(super) fn frame(menu: &mut Menu, size: (u16, u16), banner: bool, color: bool) -> Frame {
    menu.refresh_suggestions();
    if menu.output.is_some() {
        return output_frame(menu, size);
    }
    frame_with_status(menu, size, "", banner, color)
}

fn frame_with_status(
    menu: &mut Menu,
    size: (u16, u16),
    status: &str,
    banner: bool,
    color: bool,
) -> Frame {
    let mut frame = Frame::new(size);
    let columns = usize::from(size.0.saturating_sub(1)).min(108);
    let height = size.1;
    if columns == 0 || height == 0 {
        return frame;
    }
    let margin = if columns >= 24 { 2 } else { 0 };
    let width = columns - 2 * margin;
    let mut row = u16::from(height >= 16);
    if banner && columns >= BANNER_WIDTH && height >= 22 {
        let inset = if width >= BANNER_WIDTH { margin } else { 0 };
        let lines = menu.banner.lines(inset, color);
        let start = usize::from(row);
        frame.rows[start..start + lines.len()].clone_from_slice(lines);
        row += lines.len() as u16 + 1;
    } else if height >= 3 {
        let name = if width >= NAME.len() { NAME } else { "CNR" };
        label(&mut frame, row, margin, width, Tone::Accent, name);
        row += 1 + u16::from(height >= 12);
    }
    if height >= 12 {
        label(
            &mut frame,
            row,
            margin,
            width,
            Tone::Heading,
            text(["Commands", "Lệnh"], menu.vi),
        );
        row += 1;
    }

    let mut end = height;
    if height >= 8 {
        end -= 1;
        if status.is_empty() {
            label(
                &mut frame,
                end,
                margin,
                width,
                Tone::Muted,
                text(MENU_HINT, menu.vi),
            );
        }
    }
    if let Some(prompt) = &menu.prompt {
        end -= 1;
        input_line(
            &mut frame,
            end,
            margin,
            width,
            text(COMMAND, menu.vi),
            &prompt.input,
            ghost(prompt),
        );
        let suggestions = popup_rows(Some(prompt), end.saturating_sub(row + 3));
        popup(
            &mut frame,
            prompt,
            end - suggestions..end,
            margin,
            width,
            menu.vi,
        );
        end -= suggestions;
    } else {
        if height >= 18 {
            end -= 1;
        }
        if height >= 8 && !status.is_empty() {
            label(&mut frame, end, margin, width, Tone::Muted, status);
        }
        if end > row + 1 {
            if let Some(search) = &menu.search {
                end -= 1;
                input_line(
                    &mut frame,
                    end,
                    margin,
                    width,
                    text(["Search  › ", "Tìm kiếm  › "], menu.vi),
                    search,
                    "",
                );
            } else if menu.help {
                end -= 1;
                label(
                    &mut frame,
                    end,
                    margin,
                    width,
                    Tone::Muted,
                    text(
                        [
                            "/  Search commands   ·   cnr --help  Full reference",
                            "/  Tìm lệnh   ·   cnr --help  Xem tham chiếu",
                        ],
                        menu.vi,
                    ),
                );
            }
        }
    }

    let visible = usize::from(end.saturating_sub(row));
    if visible == 0 {
        return frame;
    }
    menu.reveal_selection(visible);
    if menu.items.is_empty() {
        label(
            &mut frame,
            row,
            margin,
            width,
            Tone::Muted,
            text(["No matching commands", "Không có lệnh phù hợp"], menu.vi),
        );
        return frame;
    }

    let left_width = if width >= 80 && visible >= 4 {
        33
    } else {
        width
    };
    let mut preview = Vec::new();
    if left_width < width {
        let entry = menu.items[menu.selected];
        if !entry.command.is_empty() {
            preview.push((Tone::Accent, format!("cnr {}", entry.command)));
        }
        preview.extend(
            wrap(text(entry.detail, menu.vi), width - left_width - 5)
                .into_iter()
                .map(|line| (Tone::Plain, line)),
        );
    }
    let items = &menu.items[menu.first..];
    frame.menu = Some(MenuRows {
        columns: margin as u16..(margin + left_width) as u16,
        rows: row..row + items.len().min(visible) as u16,
        first: menu.first,
    });
    let count = items.len().max(preview.len()).min(visible);
    for i in 0..count {
        let mut line = indented(margin);
        let mut used = 0;
        if let Some(entry) = items.get(i) {
            let selected = menu.first + i == menu.selected;
            let marker = if selected { "› " } else { "  " };
            let value = clip(
                &format!("{marker}{}", text(entry.label, menu.vi)),
                left_width,
            );
            used = value.width();
            line.push(if selected { Tone::Accent } else { Tone::Plain }, value);
        }
        if left_width < width {
            line.push(Tone::Plain, " ".repeat(left_width - used));
            line.push(Tone::Muted, "  │  ");
            if let Some((tone, value)) = preview.get(i) {
                line.push(*tone, clip(value, width - left_width - 5));
            }
        }
        frame.rows[usize::from(row) + i] = line;
    }
    frame
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::{Effect, OutputView, Prompt};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    const STATUS: &str = "workspace  ·  Configured  ·  v0.1.0";

    fn press(menu: &mut Menu, code: KeyCode) -> Effect {
        menu.handle(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn rows(frame: &Frame) -> Vec<String> {
        frame.rows.iter().map(Line::text).collect()
    }

    #[test]
    fn command_and_suggestions_stay_above_the_muted_hint() {
        for vi in [false, true] {
            for size in [(120, 32), (100, 25), (80, 24), (40, 12), (20, 6)] {
                let mut menu = Menu::new(vi, None);
                let initial = frame(&mut menu, size, true, true);
                assert!(initial.cursor.is_none());
                press(&mut menu, KeyCode::Char('l'));
                let active = frame(&mut menu, size, true, true);
                let command = usize::from(active.cursor.unwrap().1);
                assert!(rows(&active)[command].contains(text(COMMAND, vi)));
                assert!(
                    rows(&active)[..command]
                        .iter()
                        .any(|line| line.contains("› list"))
                );
                if size.1 >= 8 {
                    assert_eq!(
                        rows(&active)[command + 1].trim(),
                        clip(
                            text(MENU_HINT, vi),
                            usize::from(size.0.saturating_sub(1)).min(108) - 4
                        )
                        .trim()
                    );
                    assert!(active.rows[command + 1].has(Tone::Muted));
                }
                if size.0 >= 100 {
                    assert!(
                        !active
                            .changed_rows(Some(&initial))
                            .any(|(row, _)| (1..=6).contains(&row))
                    );
                }

                let mut output = OutputView::new("cnr list".into(), vi);
                output.buffer = super::super::viewport::Buffer::from_streams(
                    "{\"count\": 42}".into(),
                    "",
                    None,
                );
                output.running = false;
                output.status = text(["Exit code: 0", "Mã thoát: 0"], vi).into();
                menu.output = Some(output);
                menu.prompt = Some(Prompt::default());
                let result = frame(&mut menu, size, true, true);
                let lines = rows(&result);
                let command = usize::from(result.cursor.unwrap().1);
                assert!(lines[command - 1].contains(text(["Exit code:", "Mã thoát:"], vi)));
                assert!(lines[command].contains(text(COMMAND, vi)));
                assert!(lines[command + 1].contains("PgUp/PgDn"));
                assert!(result.rows[command + 1].has(Tone::Muted));
                menu.output.as_mut().unwrap().running = true;
                let running = frame(&mut menu, (120, 32), true, true);
                assert!(
                    rows(&running)
                        .last()
                        .unwrap()
                        .contains(text(RUNNING_HINT, vi))
                );
            }
        }
    }

    #[test]
    fn mouse_hit_matches_the_displayed_menu_while_command_is_open() {
        let mut menu = Menu::new(false, None);
        press(&mut menu, KeyCode::Char('l'));
        let frame = frame(&mut menu, (110, 30), true, true);
        let row = rows(&frame)
            .iter()
            .position(|line| line.contains("› Tool inventory"))
            .unwrap() as u16;
        assert_eq!(frame.menu_hit(4, row), Some(0));
        assert_eq!(frame.menu_hit(4, row + 1), Some(1));
        assert_eq!(frame.menu_hit(40, row), None);
        assert_eq!(frame.menu_hit(4, row - 1), None);
    }

    #[test]
    fn wide_banner_has_a_visible_offset_shadow() {
        let mut menu = Menu::new(false, None);
        let art = frame_with_status(&mut menu, (100, 25), STATUS, true, true);
        let lines = rows(&art);
        assert!(lines.iter().any(|line| line.contains('█')));
        assert!(lines.iter().any(|line| line.contains('░')));
    }

    #[test]
    fn monochrome_banner_keeps_letter_faces_without_shadow_noise() {
        let mut menu = Menu::new(false, None);
        let color = frame(&mut menu, (100, 25), true, true);
        let monochrome = frame(&mut menu, (100, 25), true, false);
        let colored_rows = rows(&color);
        let plain_rows = rows(&monochrome);
        assert!(colored_rows.iter().any(|line| line.contains('░')));
        assert!(plain_rows.iter().all(|line| !line.contains('░')));
        for (colored, plain) in colored_rows.iter().zip(&plain_rows) {
            assert_eq!(colored.replace('░', " "), *plain);
        }
    }

    #[test]
    fn navigation_changes_only_the_affected_rows() {
        let mut menu = Menu::new(false, None);
        let first = frame_with_status(&mut menu, (80, 25), STATUS, true, true);
        assert!(matches!(press(&mut menu, KeyCode::Down), Effect::Redraw));
        let next = frame_with_status(&mut menu, (80, 25), STATUS, true, true);
        let changed: Vec<_> = next
            .changed_rows(Some(&first))
            .map(|(_, line)| line.text())
            .collect();
        assert_eq!(changed, ["    Tool inventory", "  › Search tools"]);
        assert_eq!(next.changed_rows(Some(&next)).count(), 0);

        let wide = frame_with_status(&mut menu, (110, 30), STATUS, true, true);
        assert!(matches!(press(&mut menu, KeyCode::Down), Effect::Redraw));
        let next = frame_with_status(&mut menu, (110, 30), STATUS, true, true);
        for (_, line) in next.changed_rows(Some(&wide)) {
            assert!(!line.text().contains('█'));
            assert!(!line.text().contains(STATUS));
        }
        assert!(
            next.changed_rows(Some(&wide))
                .any(|(_, line)| line.text().contains("cnr install"))
        );
    }

    #[test]
    fn help_and_search_are_at_the_bottom_only_when_active() {
        let mut menu = Menu::new(true, None);
        let initial = frame_with_status(&mut menu, (78, 24), STATUS, true, true);
        assert!(
            rows(&initial)
                .iter()
                .all(|line| !line.contains("cnr --help"))
        );
        assert_eq!(initial.cursor, None);
        press(&mut menu, KeyCode::Char('?'));
        let help = frame_with_status(&mut menu, (78, 24), STATUS, true, true);
        assert!(help.rows[21].text().contains("cnr --help"));
        assert!(help.rows[22].text().contains(STATUS));

        press(&mut menu, KeyCode::Char('/'));
        for ch in "chẩn đoán".chars() {
            press(&mut menu, KeyCode::Char(ch));
        }
        let search = frame_with_status(&mut menu, (78, 24), STATUS, true, true);
        assert!(search.rows[21].text().contains("Tìm kiếm  › chẩn đoán"));
        assert_eq!(search.cursor.map(|(_, y)| y), Some(21));
        assert!(
            rows(&search)
                .iter()
                .any(|line| line.contains("› Chẩn đoán"))
        );
        press(&mut menu, KeyCode::Esc);
        let closed = frame_with_status(&mut menu, (78, 24), STATUS, true, true);
        assert!(closed.rows[21].text().is_empty());
        assert!(closed.changed_rows(Some(&search)).any(|(row, _)| row == 21));
    }

    #[test]
    fn resized_and_tiny_frames_fit_terminal_columns() {
        let mut menu = Menu::new(false, None);
        assert_eq!(
            WORDMARK.iter().map(|row| row.width()).max(),
            Some(WORDMARK_WIDTH)
        );
        let art = frame_with_status(&mut menu, (92, 22), "Dự án 東京", true, true);
        assert!(art.rows[1].text().contains('█'));
        for (size, banner) in [((91, 22), true), ((91, 21), true), ((110, 25), false)] {
            let compact = frame_with_status(&mut menu, size, STATUS, banner, true);
            assert!(rows(&compact).iter().any(|line| line.trim() == NAME));
            assert!(compact.changed_rows(Some(&art)).count() > 2);
        }
        for size in [(0, 0), (1, 1), (2, 2), (10, 4), (26, 8)] {
            let small = frame_with_status(&mut menu, size, "Dự án 東京\u{1b}[31m", true, true);
            for line in rows(&small) {
                assert!(line.width() < usize::from(size.0).max(1));
                assert!(!line.contains('\u{1b}'));
            }
        }
        assert_eq!(clip("Việt 東京", 7), "Việt 東");
    }
}
