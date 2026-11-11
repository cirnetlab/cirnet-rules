use super::{
    Input, Menu,
    terminal::{Frame, Line, Tone},
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
        start = index;
        remaining -= columns;
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
    placeholder: &str,
) {
    let prefix = clip(prefix, width.saturating_sub(1));
    let available = width - prefix.width();
    let (before, after) = input.value.split_at(input.cursor);
    let before = tail(before, available.saturating_sub(1));
    let mut line = indented(margin);
    line.push(Tone::Heading, &prefix);
    line.push(Tone::Plain, before);
    if input.value.is_empty() {
        line.push(Tone::Muted, clip(placeholder, available));
    } else {
        line.push(Tone::Plain, clip(after, available - before.width()));
        if after.is_empty()
            && let Some(suffix) = placeholder.strip_prefix(&input.value)
        {
            line.push(Tone::Muted, clip(suffix, available - before.width()));
        }
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

// Preserve indentation and wrap long output without emitting terminal controls.
fn output_rows(lines: &[String], width: usize) -> Vec<String> {
    let mut rows = Vec::new();
    if width == 0 {
        return rows;
    }
    for value in lines {
        let mut row = String::new();
        let mut used = 0;
        for ch in value.chars() {
            let Some(columns) = ch.width() else { continue };
            if columns > width {
                continue;
            }
            if used + columns > width {
                rows.push(std::mem::take(&mut row));
                used = 0;
            }
            row.push(ch);
            used += columns;
        }
        rows.push(row);
    }
    rows
}

fn output_frame(menu: &mut Menu, size: (u16, u16)) -> Frame {
    let mut frame = Frame::new(size);
    let columns = usize::from(size.0.saturating_sub(1)).min(108);
    if columns == 0 || size.1 == 0 {
        return frame;
    }
    let margin = if columns >= 24 { 2 } else { 0 };
    let width = columns - 2 * margin;
    let suggestion = menu.suggestion();
    let output = menu.output.as_mut().expect("output mode");
    let prompt_row = size.1 - 1;
    let status_row = (size.1 >= 4).then_some(size.1.saturating_sub(2));
    let hint_row = (size.1 >= 6).then_some(size.1.saturating_sub(3));
    let first = u16::from(size.1 >= 2);
    let end = hint_row.or(status_row).unwrap_or(prompt_row);
    if size.1 >= 2 {
        label(
            &mut frame,
            0,
            margin,
            width,
            Tone::Accent,
            &format!("{NAME}  /  {}", output.title),
        );
    }
    let lines = output_rows(&output.lines, width);
    let available = usize::from(end.saturating_sub(first));
    output.scroll = output.scroll.min(lines.len().saturating_sub(available));
    for (offset, line) in lines.iter().skip(output.scroll).take(available).enumerate() {
        label(
            &mut frame,
            first + offset as u16,
            margin,
            width,
            Tone::Plain,
            line,
        );
    }
    if let Some(row) = status_row {
        label(&mut frame, row, margin, width, Tone::Accent, &output.status);
    }
    if let Some(row) = hint_row {
        let error = menu.prompt.as_ref().and_then(|prompt| prompt.error);
        let hint = error
            .map(|error| text(error, menu.vi).to_owned())
            .or_else(|| {
                suggestion
                    .filter(|_| {
                        menu.prompt
                            .as_ref()
                            .is_some_and(|p| !p.input.value.is_empty())
                    })
                    .map(|value| format!("Tab: {value}"))
            })
            .unwrap_or_else(|| {
                if output.running {
                    text(
                        [
                            "PgUp/PgDn scroll  ·  Tab complete  ·  Ctrl+C cancel",
                            "PgUp/PgDn cuộn  ·  Tab gợi ý  ·  Ctrl+C hủy",
                        ],
                        menu.vi,
                    )
                } else {
                    text(
                        [
                            "PgUp/PgDn scroll  ·  Tab complete  ·  Esc back  ·  Ctrl+C exit",
                            "PgUp/PgDn cuộn  ·  Tab gợi ý  ·  Esc quay lại  ·  Ctrl+C thoát",
                        ],
                        menu.vi,
                    )
                }
                .to_owned()
            });
        label(
            &mut frame,
            row,
            margin,
            width,
            if error.is_some() {
                Tone::Accent
            } else {
                Tone::Muted
            },
            &hint,
        );
    }
    if let Some(prompt) = &menu.prompt {
        input_line(
            &mut frame,
            prompt_row,
            margin,
            width,
            text(["Command  › ", "Lệnh  › "], menu.vi),
            &prompt.input,
            "",
        );
    }
    frame
}

// Coordinates refer to the ordinary menu, before any prompt or overlay is opened.
pub(super) fn menu_hit(size: (u16, u16), banner: bool, x: u16, y: u16) -> Option<usize> {
    let columns = usize::from(size.0.saturating_sub(1)).min(108);
    if columns == 0 || size.1 == 0 {
        return None;
    }
    let margin = if columns >= 24 { 2 } else { 0 };
    let width = columns - 2 * margin;
    let height = size.1;
    let mut row = u16::from(height >= 16);
    if banner && columns >= BANNER_WIDTH && height >= 22 {
        row += WORDMARK.len() as u16 + 2;
    } else if height >= 3 {
        row += 1 + u16::from(height >= 12);
    }
    row += u16::from(height >= 12);
    let end = if height >= 8 {
        height - 1 - u16::from(height >= 18)
    } else {
        height
    };
    let visible = usize::from(end.saturating_sub(row));
    let left_width = if width >= 80 && visible >= 4 {
        33
    } else {
        width
    };
    if usize::from(x) < margin || usize::from(x) >= margin + left_width || y < row || y >= end {
        return None;
    }
    Some(usize::from(y - row))
}

pub(super) fn frame(menu: &mut Menu, size: (u16, u16), banner: bool, color: bool) -> Frame {
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
        let art: Vec<Vec<char>> = WORDMARK.iter().map(|line| line.chars().collect()).collect();
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
                        face.to_string(),
                    );
                } else if shadow && color {
                    line.push(Tone::Shadow, "░");
                } else {
                    line.push(Tone::Plain, " ");
                }
            }
            frame.rows[usize::from(row)] = line;
            row += 1;
        }
        row += 1;
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
        end -= 1 + u16::from(height >= 18);
        label(&mut frame, end, margin, width, Tone::Muted, status);
        if status.is_empty() {
            let hint = if menu.prompt.is_some() {
                menu.suggestion().map(|value| format!("Tab: {value}"))
            } else {
                None
            }
            .unwrap_or_else(|| {
                text(
                    [
                        "↑↓ / click select  ·  Enter open  ·  : command  ·  / search  ·  ? help",
                        "↑↓ / click chọn  ·  Enter mở  ·  : lệnh  ·  / tìm  ·  ? trợ giúp",
                    ],
                    menu.vi,
                )
                .to_owned()
            });
            label(&mut frame, height - 1, margin, width, Tone::Muted, &hint);
        }
    }
    let menu_minimum = u16::from(height > 1);
    if end > row + menu_minimum {
        if let Some(prompt) = &menu.prompt {
            end -= 1;
            let example = menu.suggestion().unwrap_or_default();
            input_line(
                &mut frame,
                end,
                margin,
                width,
                text(["Command  › ", "Lệnh  › "], menu.vi),
                &prompt.input,
                &example,
            );
            if let Some(error) = prompt.error
                && end > row + menu_minimum
            {
                end -= 1;
                label(
                    &mut frame,
                    end,
                    margin,
                    width,
                    Tone::Accent,
                    text(error, menu.vi),
                );
            }
        } else if let Some(search) = &menu.search {
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
            let help = text(
                [
                    "/  Search commands   ·   cnr --help  Full reference",
                    "/  Tìm lệnh   ·   cnr --help  Xem tham chiếu",
                ],
                menu.vi,
            );
            label(&mut frame, end, margin, width, Tone::Muted, help);
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
    fn command_results_remain_in_the_interface_with_an_editable_prompt() {
        let mut menu = Menu::new(false);
        menu.prompt = Some(Prompt::default());
        menu.output = Some(OutputView {
            title: "cnr list".into(),
            lines: vec!["first result".into(), "second result".into()],
            scroll: 0,
            running: false,
            status: "PASS".into(),
        });
        let output = frame(&mut menu, (50, 12), true, true);
        let text = rows(&output).join("\n");
        assert!(text.contains("CIRNET RULES"));
        assert!(text.contains("cnr list"));
        assert!(text.contains("first result"));
        assert!(text.contains("PASS"));
        assert!(text.contains("Command"));
        assert!(output.cursor.is_some());
        let idle = frame(&mut menu, (100, 12), true, true);
        assert!(rows(&idle).join("\n").contains("Ctrl+C exit"));
        menu.output.as_mut().unwrap().running = true;
        let running = frame(&mut menu, (100, 12), true, true);
        let hint = rows(&running).join("\n");
        assert!(hint.contains("Ctrl+C cancel"));
        assert!(!hint.contains("Esc back"));
        for size in [(0, 0), (1, 1), (2, 2), (10, 4), (26, 8)] {
            let small = frame(&mut menu, size, true, true);
            for line in rows(&small) {
                assert!(line.width() < usize::from(size.0).max(1));
            }
        }
    }

    #[test]
    fn output_wraps_without_losing_indentation_and_scroll_is_clamped() {
        assert_eq!(
            output_rows(&["  abcdef".into(), "x\u{1b}y".into()], 4),
            ["  ab", "cdef", "xy"]
        );
        let mut menu = Menu::new(false);
        menu.prompt = Some(Prompt::default());
        menu.output = Some(OutputView {
            title: "list".into(),
            lines: (0..20).map(|n| format!("result {n}")).collect(),
            scroll: usize::MAX,
            running: false,
            status: "PASS".into(),
        });
        let output = frame(&mut menu, (40, 10), true, true);
        assert!(rows(&output).iter().any(|line| line.contains("result 19")));
        assert_ne!(menu.output.as_ref().unwrap().scroll, usize::MAX);
    }

    #[test]
    fn mouse_hit_only_selects_visible_menu_rows() {
        assert_eq!(menu_hit((110, 30), true, 4, 9), Some(0));
        assert_eq!(menu_hit((110, 30), true, 4, 10), Some(1));
        assert_eq!(menu_hit((110, 30), true, 40, 9), None);
        assert_eq!(menu_hit((110, 30), true, 4, 8), None);
        assert_eq!(menu_hit((30, 12), true, 4, 3), Some(0));
        assert_eq!(menu_hit((0, 0), true, 0, 0), None);
    }

    #[test]
    fn wide_banner_has_a_visible_offset_shadow() {
        let mut menu = Menu::new(false);
        let art = frame_with_status(&mut menu, (100, 25), STATUS, true, true);
        let lines = rows(&art);
        assert!(lines.iter().any(|line| line.contains('█')));
        assert!(lines.iter().any(|line| line.contains('░')));
    }

    #[test]
    fn monochrome_banner_keeps_letter_faces_without_shadow_noise() {
        let mut menu = Menu::new(false);
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
        let mut menu = Menu::new(false);
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
        let mut menu = Menu::new(true);
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
        let mut menu = Menu::new(false);
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
