use super::{
    Input, Menu,
    terminal::{Frame, Line, Tone},
    text,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const NAME: &str = "CIRNET RULES";
const WORDMARK_WIDTH: usize = 90;
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

pub(super) fn frame(menu: &mut Menu, size: (u16, u16), status: &str, banner: bool) -> Frame {
    let mut frame = Frame::new(size);
    let columns = usize::from(size.0.saturating_sub(1)).min(108);
    let height = size.1;
    if columns == 0 || height == 0 {
        return frame;
    }
    let margin = if columns >= 24 { 2 } else { 0 };
    let width = columns - 2 * margin;
    let mut row = u16::from(height >= 16);
    if banner && columns >= WORDMARK_WIDTH && height >= 22 {
        let inset = if width >= WORDMARK_WIDTH { margin } else { 0 };
        for line in WORDMARK {
            label(&mut frame, row, inset, WORDMARK_WIDTH, Tone::Accent, line);
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
    }
    let menu_minimum = u16::from(height > 1);
    if end > row + menu_minimum {
        if let Some(prompt) = &menu.prompt {
            end -= 1;
            let example = format!("cnr {}", menu.items[menu.selected].command);
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
    use crate::menu::Effect;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    const STATUS: &str = "workspace  ·  Configured  ·  v0.1.0";

    fn press(menu: &mut Menu, code: KeyCode) -> Effect {
        menu.handle(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn rows(frame: &Frame) -> Vec<String> {
        frame.rows.iter().map(Line::text).collect()
    }

    #[test]
    fn navigation_changes_only_the_affected_rows() {
        let mut menu = Menu::new(false);
        let first = frame(&mut menu, (80, 25), STATUS, true);
        assert!(matches!(press(&mut menu, KeyCode::Down), Effect::Redraw));
        let next = frame(&mut menu, (80, 25), STATUS, true);
        let changed: Vec<_> = next
            .changed_rows(Some(&first))
            .map(|(_, line)| line.text())
            .collect();
        assert_eq!(changed, ["    Tool inventory", "  › Search tools"]);
        assert_eq!(next.changed_rows(Some(&next)).count(), 0);

        let wide = frame(&mut menu, (110, 30), STATUS, true);
        assert!(matches!(press(&mut menu, KeyCode::Down), Effect::Redraw));
        let next = frame(&mut menu, (110, 30), STATUS, true);
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
        let initial = frame(&mut menu, (78, 24), STATUS, true);
        assert!(
            rows(&initial)
                .iter()
                .all(|line| !line.contains("cnr --help"))
        );
        assert_eq!(initial.cursor, None);
        press(&mut menu, KeyCode::Char('?'));
        let help = frame(&mut menu, (78, 24), STATUS, true);
        assert!(help.rows[21].text().contains("cnr --help"));
        assert!(help.rows[22].text().contains(STATUS));

        press(&mut menu, KeyCode::Char('/'));
        for ch in "chẩn đoán".chars() {
            press(&mut menu, KeyCode::Char(ch));
        }
        let search = frame(&mut menu, (78, 24), STATUS, true);
        assert!(search.rows[21].text().contains("Tìm kiếm  › chẩn đoán"));
        assert_eq!(search.cursor.map(|(_, y)| y), Some(21));
        assert!(
            rows(&search)
                .iter()
                .any(|line| line.contains("› Chẩn đoán"))
        );
        press(&mut menu, KeyCode::Esc);
        let closed = frame(&mut menu, (78, 24), STATUS, true);
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
        let art = frame(&mut menu, (91, 22), "Dự án 東京", true);
        assert!(art.rows[1].text().contains('█'));
        for (size, banner) in [((90, 22), true), ((91, 21), true), ((110, 25), false)] {
            let compact = frame(&mut menu, size, STATUS, banner);
            assert!(rows(&compact).iter().any(|line| line.trim() == NAME));
            assert!(compact.changed_rows(Some(&art)).count() > 2);
        }
        for size in [(0, 0), (1, 1), (2, 2), (10, 4), (26, 8)] {
            let small = frame(&mut menu, size, "Dự án 東京\u{1b}[31m", true);
            for line in rows(&small) {
                assert!(line.width() < usize::from(size.0).max(1));
                assert!(!line.contains('\u{1b}'));
            }
        }
        assert_eq!(clip("Việt 東京", 7), "Việt 東");
    }
}
