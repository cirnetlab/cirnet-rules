use super::{
    Input, Menu, Prompt,
    banner::BANNER_WIDTH,
    completion::Source,
    terminal::{Frame, Line, MenuRows, PalettePart, Tone},
    text,
};
use std::{borrow::Cow, sync::LazyLock};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const NAME: &str = "CIRNET RULES";
const MAX_COLUMNS: usize = 132;
#[cfg(test)]
use super::banner::{WORDMARK, WORDMARK_WIDTH};

fn clip(value: &str, width: usize) -> Cow<'_, str> {
    let mut remaining = width;
    let mut end = value.len();
    let mut control = false;
    for (index, ch) in value.char_indices() {
        let Some(columns) = ch.width() else {
            control = true;
            continue;
        };
        if columns > remaining {
            end = index;
            break;
        }
        remaining -= columns;
    }
    let visible = &value[..end];
    if control {
        Cow::Owned(visible.chars().filter(|ch| ch.width().is_some()).collect())
    } else {
        Cow::Borrowed(visible)
    }
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
    line.push(Tone::Plain, spaces(margin));
    line
}

fn spaces(count: usize) -> &'static str {
    const SPACES: &str = concat!(
        "                           ",
        "                           ",
        "                           ",
        "                           ",
        "                           "
    );
    &SPACES[..count]
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
    feedback: &str,
) {
    let mut line = indented(margin);
    let (cursor, _) = input_text(
        &mut line,
        width,
        prefix,
        Tone::Heading,
        Tone::Plain,
        input,
        feedback,
    );
    frame.cursor = Some(((margin + cursor) as u16, row));
    frame.rows[usize::from(row)] = line;
}

fn input_text(
    line: &mut Line,
    width: usize,
    prefix: &str,
    marker: Tone,
    body: Tone,
    input: &Input,
    feedback: &str,
) -> (usize, usize) {
    let prefix = clip(prefix, width.saturating_sub(1));
    let available = width - prefix.width();
    let (before, after) = input.value.split_at(input.cursor);
    let before = tail(before, available.saturating_sub(1));
    line.push(marker, &prefix);
    line.push(body, before);
    let (suffix, tone) = if after.is_empty() {
        (feedback, Tone::Muted)
    } else {
        (after, body)
    };
    let suffix = clip(suffix, available - before.width());
    line.push(tone, &suffix);
    let cursor = prefix.width() + before.width();
    (cursor, cursor + suffix.width())
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

const MENU_HINT: [&str; 2] = [
    "Tab complete · : command · / search · Esc back · Ctrl+C exit · ? help",
    "Tab gợi ý · : lệnh · / tìm · Esc quay lại · Ctrl+C thoát · ? trợ giúp",
];

fn menu_footer(frame: &mut Frame, margin: usize, width: usize, vi: bool) -> u16 {
    let height = frame.size.1;
    if height < 2 {
        return height;
    }
    let full = text(MENU_HINT, vi);
    let labeled = text(
        [
            "Tab · : cmd · / find · Esc · Ctrl+C · ?",
            "Tab · : lệnh · / tìm · Esc · Ctrl+C · ?",
        ],
        vi,
    );
    let keys = "Tab : / Esc Ctrl+C ?";
    let last = height - 1;
    for candidate in [full, labeled, keys] {
        if candidate.width() <= width {
            label(frame, last, margin, width, Tone::Muted, candidate);
            return last;
        }
    }
    if width >= 8 && height >= 4 {
        label(frame, last - 1, margin, width, Tone::Muted, "Tab : /");
        let rest = if width >= 12 {
            "Esc Ctrl+C ?"
        } else {
            "Esc ^C ?"
        };
        label(frame, last, margin, width, Tone::Muted, rest);
        return last - 1;
    }
    let brief = if width >= 5 {
        "Esc ?"
    } else if width >= 3 {
        "Esc"
    } else {
        "?"
    };
    label(frame, last, margin, width, Tone::Muted, brief);
    last
}

struct Palette<'a> {
    prompt: &'a Prompt,
    width: usize,
    choices: u16,
    label_width: usize,
    bordered: bool,
}

impl<'a> Palette<'a> {
    fn new(
        prompt: &'a Prompt,
        width: usize,
        available: u16,
        show_choices: bool,
        show_notice: bool,
    ) -> Self {
        let bordered = width >= 12 && available >= 4;
        let label_width = prompt
            .completions
            .iter()
            .map(|candidate| candidate.label.width())
            .max()
            .unwrap_or(0);
        let choices = if show_notice && prompt.error.is_some() {
            1
        } else if !show_choices
            || prompt.error.is_some()
            || prompt.input.value.trim().is_empty()
            || width < label_width.saturating_add(if bordered { 6 } else { 2 })
        {
            0
        } else {
            prompt.completions.len().min(5) as u16
        };
        Self {
            prompt,
            width,
            choices: choices.min(available.saturating_sub(if bordered { 3 } else { 1 })),
            label_width,
            bordered,
        }
    }

    fn rows(&self) -> u16 {
        1 + self.choices + 2 * u16::from(self.bordered)
    }

    fn draw_cap(&self, line: &mut Line, top: bool, color: bool) {
        static RULE: LazyLock<String> = LazyLock::new(|| "─".repeat(MAX_COLUMNS));

        let (left, right) = if top { ("╭", "╮") } else { ("╰", "╯") };
        let tone = if color {
            Tone::Palette(PalettePart::InputRim)
        } else {
            Tone::Plain
        };
        line.push(tone, left);
        line.push(tone, &RULE[..(self.width - 2) * '─'.len_utf8()]);
        line.push(tone, right);
    }

    fn draw(&self, line: &mut Line, row: u16, vi: bool, color: bool) -> Option<usize> {
        use PalettePart::*;
        if self.bordered && (row == 0 || row == self.rows() - 1) {
            self.draw_cap(line, row == 0, color);
            return None;
        }

        let input_row = row == u16::from(self.bordered);
        let body_tone = match (self.bordered, input_row) {
            (true, true) => Tone::Palette(Input),
            (true, false) => Tone::Palette(Detail),
            (false, true) => Tone::Plain,
            (false, false) => Tone::Muted,
        };
        let inset = usize::from(self.bordered) * 2;
        let inner = self.width - 2 * inset;
        let (left, right, edge_tone) = if color {
            ("│", "│", Tone::Palette(InputEdge))
        } else {
            ("│", "│", Tone::Plain)
        };
        if self.bordered {
            line.push(edge_tone, left);
            line.push(body_tone, " ");
        }
        let (cursor, used) = if input_row {
            let (cursor, used) = input_text(
                line,
                inner,
                "› ",
                body_tone,
                body_tone,
                &self.prompt.input,
                "",
            );
            (Some(inset + cursor), used)
        } else if let Some(error) = self.prompt.error {
            let value = clip(text(error, vi), inner);
            let used = value.width();
            line.push(body_tone, value);
            (None, used)
        } else {
            (None, self.draw_choice(line, row, inner, vi, color))
        };
        if self.bordered {
            line.push(body_tone, spaces(inner - used + 1));
            line.push(edge_tone, right);
        }
        cursor
    }

    fn draw_choice(&self, line: &mut Line, row: u16, width: usize, vi: bool, color: bool) -> usize {
        let detail_tone = if self.bordered {
            Tone::Palette(PalettePart::Detail)
        } else {
            Tone::Muted
        };
        let focus = self.prompt.accepted.as_ref().map_or(0, |(index, _)| *index);
        let offset = row - if self.bordered { 2 } else { 1 };
        let index = (focus + 1).saturating_sub(usize::from(self.choices)) + usize::from(offset);
        let item = &self.prompt.completions[index];
        let selected = index == focus;
        line.push(
            detail_tone,
            if selected && (!self.bordered || !color) {
                "› "
            } else {
                "  "
            },
        );
        let tone = match (self.bordered, selected) {
            (true, true) => Tone::Palette(PalettePart::Selected),
            (true, false) => Tone::Palette(PalettePart::Suggestion),
            (false, true) => Tone::Heading,
            (false, false) => Tone::Plain,
        };
        line.push(tone, &item.label);
        let label = item.label.width();
        let detail_width = width.saturating_sub(self.label_width + 5);
        if detail_width < 12 {
            return label + 2;
        }
        let detail = match item.source {
            Source::Pinned | Source::Installed => item.source.label(vi),
            _ if item.description.is_empty() => item.kind.label(vi),
            _ => &item.description,
        };
        let detail = clip(detail, detail_width);
        let used = self.label_width + 5 + detail.width();
        line.push(detail_tone, spaces(self.label_width - label + 3));
        line.push(detail_tone, detail);
        used
    }
}

fn output_frame(menu: &mut Menu, size: (u16, u16), color: bool) -> Frame {
    let mut frame = Frame::new(size);
    let columns = usize::from(size.0.saturating_sub(1)).min(MAX_COLUMNS);
    let height = size.1;
    if columns == 0 || height == 0 {
        return frame;
    }
    let inset = if columns >= 24 { "  " } else { "" };
    let margin = inset.len();
    let width = columns - 2 * margin;
    let output = menu.output.as_mut().expect("output mode");
    let find = output.find.as_ref();
    let separate_find = find.is_some() && height >= 3;
    let header = u16::from(height >= 4 + u16::from(separate_find));
    let has_status = height >= 2;
    let body = u16::from(height >= 3 + u16::from(separate_find));
    let status_gap = u16::from(height >= 16 && width >= 24 && menu.prompt.is_some());
    let palette = menu.prompt.as_ref().map(|prompt| {
        let available = height.saturating_sub(
            header + u16::from(has_status) + body + u16::from(separate_find) + status_gap,
        );
        Palette::new(
            prompt,
            width,
            available,
            find.is_none() && !output.running,
            find.is_none(),
        )
    });
    let command_row = height - palette.as_ref().map_or(0, Palette::rows);
    let status_row = has_status.then(|| command_row - status_gap - 1);
    let body_end = status_row
        .unwrap_or(command_row)
        .saturating_sub(u16::from(separate_find));
    let visible = body_end.saturating_sub(header);
    if header != 0 {
        label(
            &mut frame,
            0,
            margin,
            width,
            Tone::Accent,
            &format!("{NAME}  /  {}", output.title),
        );
    }
    output.layout(width, usize::from(visible));
    for row in 0..visible {
        if let Some(line) =
            output
                .buffer
                .line(output.scroll + usize::from(row), inset, output.hit.as_ref())
        {
            frame.rows[usize::from(header + row)] = line;
        }
    }
    if let Some(row) = status_row {
        label(&mut frame, row, margin, width, output.tone, &output.status);
    }
    if let Some(palette) = palette {
        for offset in 0..palette.rows() {
            let row = command_row + offset;
            let mut line = indented(margin);
            if let Some(cursor) = palette.draw(&mut line, offset, menu.vi, color) {
                frame.cursor = Some(((margin + cursor) as u16, row));
            }
            frame.rows[usize::from(row)] = line;
        }
    }
    if let Some(find) = &output.find {
        let feedback = if !find.value.is_empty() && output.hit.is_none() {
            text(["  (no matches)", "  (không có kết quả)"], menu.vi)
        } else {
            ""
        };
        let row = if separate_find { body_end } else { height - 1 };
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
        return output_frame(menu, size, color);
    }
    menu_frame(menu, size, banner, color)
}

fn menu_frame(menu: &mut Menu, size: (u16, u16), banner: bool, color: bool) -> Frame {
    let mut frame = Frame::new(size);
    let columns = usize::from(size.0.saturating_sub(1)).min(MAX_COLUMNS);
    let height = size.1;
    if columns == 0 || height == 0 {
        return frame;
    }
    let margin = if columns >= 24 { 2 } else { 0 };
    let width = columns - 2 * margin;
    let mut row = u16::from(height >= 16);
    if banner && columns >= BANNER_WIDTH && height >= 22 {
        let inset = if width >= BANNER_WIDTH { margin } else { 0 };
        let lines = menu
            .banner
            .lines(inset, color, columns.saturating_sub(inset));
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

    let mut end = menu_footer(&mut frame, margin, width, menu.vi);
    if menu.prompt.is_none()
        && let Some(search) = &menu.search
        && end > row
    {
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
    }

    let wide = width >= 80 && end.saturating_sub(row) >= 4;
    let left_width = if wide { 33 } else { width };
    let right_width = width.saturating_sub(left_width + 5);
    let mut preview = Vec::new();
    if menu.help && menu.prompt.is_none() {
        let help = text(["Command reference", "Tham chiếu lệnh"], menu.vi);
        if wide {
            preview.push((Tone::Heading, help.to_owned()));
            preview.push((Tone::Plain, "cnr --help".to_owned()));
        } else if end > row {
            label(
                &mut frame,
                row,
                margin,
                width,
                Tone::Heading,
                &format!("{help} · cnr --help"),
            );
            row += 1;
        }
    } else if wide && let Some(entry) = menu.items.get(menu.selected) {
        if !entry.command.is_empty() {
            preview.push((Tone::Accent, format!("cnr {}", entry.command)));
        }
        preview.extend(
            wrap(text(entry.detail, menu.vi), right_width)
                .into_iter()
                .map(|line| (Tone::Plain, line)),
        );
    }

    let mut wide_start = None;
    if let Some(prompt) = &menu.prompt {
        let available = end.saturating_sub(row);
        if wide {
            let start = (preview.len() as u16 + 1).max(5).min(available - 1);
            wide_start = Some(start);
        } else if available > 0 {
            let menu_rows = (available - 1).min(2);
            let palette = Palette::new(prompt, width, available - menu_rows, true, true);
            end -= palette.rows();
            for offset in 0..palette.rows() {
                let y = end + offset;
                let mut line = indented(margin);
                if let Some(cursor) = palette.draw(&mut line, offset, menu.vi, color) {
                    frame.cursor = Some(((margin + cursor) as u16, y));
                }
                frame.rows[usize::from(y)] = line;
            }
        }
    }

    let visible = usize::from(end.saturating_sub(row));
    if visible == 0 {
        return frame;
    }
    menu.reveal_selection(visible);
    let wide_palette = wide_start.zip(menu.prompt.as_ref()).map(|(start, prompt)| {
        (
            start,
            Palette::new(prompt, right_width, end - row - start, true, true),
        )
    });
    let items = &menu.items[menu.first..];
    frame.menu = Some(MenuRows {
        columns: margin as u16..(margin + left_width) as u16,
        rows: row..row + items.len().min(visible) as u16,
        first: menu.first,
    });
    let right_rows = wide_palette
        .as_ref()
        .map_or(0, |(start, palette)| usize::from(start + palette.rows()));
    let count = items
        .len()
        .max(1)
        .max(preview.len())
        .max(right_rows)
        .min(visible);
    for i in 0..count {
        let y = row + i as u16;
        let mut line = indented(margin);
        let mut used = 0;
        if let Some(entry) = items.get(i) {
            let selected = menu.first + i == menu.selected;
            let marker = clip(if selected { "› " } else { "  " }, left_width);
            let value = clip(text(entry.label, menu.vi), left_width - marker.width());
            let tone = if selected { Tone::Accent } else { Tone::Plain };
            used = marker.width() + value.width();
            line.push(tone, marker);
            line.push(tone, value);
        } else if i == 0 && items.is_empty() {
            let message = text(["No matching commands", "Không có lệnh phù hợp"], menu.vi);
            let value = clip(message, left_width);
            used = value.width();
            line.push(Tone::Muted, value);
        }
        if wide {
            line.push(Tone::Plain, spaces(left_width - used));
            line.push(Tone::Muted, "  │  ");
            if let Some((tone, value)) = preview
                .get(i)
                .filter(|_| wide_start.is_none_or(|start| i < usize::from(start)))
            {
                line.push(*tone, clip(value, right_width));
            } else if let Some((start, palette)) = &wide_palette
                && let Some(offset) = (i as u16).checked_sub(*start)
                && offset < palette.rows()
                && let Some(cursor) = palette.draw(&mut line, offset, menu.vi, color)
            {
                frame.cursor = Some(((margin + left_width + 5 + cursor) as u16, y));
            }
        }
        frame.rows[usize::from(y)] = line;
    }
    frame
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::{Effect, OutputView};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn press(menu: &mut Menu, code: KeyCode) -> Effect {
        menu.handle(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn rows(frame: &Frame) -> Vec<String> {
        frame.rows.iter().map(Line::text).collect()
    }

    #[test]
    fn command_input_has_rounded_border_and_attached_suggestions() {
        let mut menu = Menu::new(false, None);
        for ch in "env p".chars() {
            press(&mut menu, KeyCode::Char(ch));
        }
        for color in [true, false] {
            let rendered = frame(&mut menu, (120, 35), true, color);
            let lines = rows(&rendered);
            let (x, y) = rendered.cursor.unwrap();
            let row = usize::from(y);
            let left = usize::from(x) - "│ › env p".width();
            let panel = |index: usize| lines[index].chars().skip(left).collect::<String>();
            assert!(panel(row - 1).starts_with('╭'));
            assert!(panel(row - 1).ends_with('╮'));
            assert!(panel(row).starts_with('│'));
            assert!(panel(row + 1).contains("env plan"));
            assert!(panel(row + 2).contains("env profiles"));
            assert!(panel(row + 3).starts_with('╰'));
            assert!(panel(row + 3).ends_with('╯'));
            let panel_width = panel(row - 1).width();
            for index in row..=row + 2 {
                let body = panel(index);
                assert!(body.starts_with('│') && body.ends_with('│'));
                assert_eq!(body.width(), panel_width);
            }
            assert_eq!(panel(row + 3).width(), panel_width);
        }
    }

    #[test]
    fn palette_cursor_tracks_unicode_display_columns() {
        let mut menu = Menu::new(true, None);
        menu.open_prompt(Input::default());
        for ch in "搜索界".chars() {
            press(&mut menu, KeyCode::Char(ch));
        }
        let rendered = frame(&mut menu, (120, 35), true, true);
        let (x, y) = rendered.cursor.unwrap();
        let line = &rendered.rows[usize::from(y)].text();
        let end = line.find("搜索界").unwrap() + "搜索界".len();
        assert_eq!(usize::from(x), line[..end].width());
    }

    #[test]
    fn compact_command_panel_keeps_errors_visible() {
        let prompt = Prompt {
            error: Some([
                "Close the quoted argument.",
                "Hãy đóng dấu nháy của đối số.",
            ]),
            ..Prompt::default()
        };
        for available in [4, 5] {
            let palette = Palette::new(&prompt, 40, available, true, true);
            let mut displayed = Vec::new();
            for row in 0..palette.rows() {
                let mut line = Line::default();
                palette.draw(&mut line, row, false, true);
                displayed.push(line.text());
            }
            assert!(
                displayed
                    .iter()
                    .any(|line| line.contains("Close the quoted argument."))
            );
            assert!(palette.rows() <= available);
        }
    }

    #[test]
    fn command_panel_clips_caps_and_keeps_suggestions_below_input() {
        let mut menu = Menu::new(false, None);
        for ch in "env p".chars() {
            press(&mut menu, KeyCode::Char(ch));
        }
        let main = frame(&mut menu, (120, 35), true, true);
        let lines = rows(&main);
        let (x, y) = main.cursor.unwrap();
        let row = usize::from(y);
        let left = usize::from(x) - "│ › env p".width();
        let caps = |line: &str| line.chars().skip(left).collect::<String>();
        let top = caps(&lines[row - 1]);
        let bottom = caps(&lines[row + 3]);
        assert!(top.starts_with('╭') && top.ends_with('╮'));
        assert!(bottom.starts_with('╰') && bottom.ends_with('╯'));
        for line in &lines[row..row + 3] {
            let body = caps(line);
            assert!(body.starts_with('│') && body.ends_with('│'));
            assert_eq!(body.width(), top.width());
        }
        assert!(lines[row].contains("› env p"));
        assert!(lines[row + 1].contains("env plan"));
        assert!(lines[row + 1].contains("Review requirements, version pins and conflicts"));
        assert!(lines[row + 2].contains("env profiles"));
        assert!(lines[row + 2].contains("List environment profiles"));
    }

    #[test]
    fn result_input_has_a_closed_frame_across_the_viewport() {
        let mut menu = Menu::new(false, None);
        menu.open_prompt(Input::default());
        menu.output = Some(OutputView::new("cnr env plan".into(), false));
        let result = frame(&mut menu, (120, 35), true, true);
        let lines = rows(&result);
        let row = usize::from(result.cursor.unwrap().1);
        assert!(lines[row - 1].trim_start().starts_with('╭'));
        assert!(lines[row].trim_start().starts_with("│ › "));
        assert!(lines[row + 1].trim_start().starts_with('╰'));
        assert_eq!(lines[row].width(), 117);
        assert!(
            lines[..row - 1]
                .iter()
                .any(|line| line.trim() == "Running…")
        );

        let monochrome = frame(&mut menu, (120, 35), true, false);
        assert_eq!(monochrome.cursor, result.cursor);
        let lines = rows(&monochrome);
        assert!(lines[row - 1].trim_start().starts_with('╭'));
        assert!(lines[row].trim_start().starts_with("│ › "));
        assert!(lines[row + 1].trim_start().starts_with('╰'));
    }

    #[test]
    fn wide_menu_has_one_fixed_footer_and_the_palette_opens_downward() {
        let mut menu = Menu::new(false, None);
        let initial = frame(&mut menu, (120, 35), true, true);
        let expected = "Tab complete · : command · / search · Esc back · Ctrl+C exit · ? help";
        assert_eq!(initial.rows.last().unwrap().text().trim(), expected);
        assert!(initial.cursor.is_none());

        press(&mut menu, KeyCode::Char('l'));
        let opened = frame(&mut menu, (120, 35), true, true);
        let (column, input_row) = opened.cursor.unwrap();
        assert!(column >= 40);
        let lines = rows(&opened);
        assert!(lines[usize::from(input_row)].contains("› l"));
        assert!(lines[usize::from(input_row + 1)].contains("list"));
        assert_eq!(lines.last().unwrap().trim(), expected);
        assert_eq!(
            lines
                .iter()
                .filter(|line| line.contains("Ctrl+C exit"))
                .count(),
            1
        );
        assert!(
            !opened
                .changed_rows(Some(&initial))
                .any(|(row, _)| (1..=6).contains(&row))
        );
    }

    #[test]
    fn running_command_shows_notice_but_find_hides_it() {
        for size in [(80, 24), (26, 8)] {
            for color in [true, false] {
                let mut menu = Menu::new(false, None);
                menu.open_prompt(Input::default());
                menu.prompt.as_mut().unwrap().error = Some(["Wait for command", "Đợi lệnh"]);
                let mut output = OutputView::new("cnr list".into(), false);
                output.running = true;
                menu.output = Some(output);
                let running = frame(&mut menu, size, true, color);
                let running_rows = rows(&running).join("\n");
                assert!(running_rows.contains("Wait for command"));
                assert!(running.cursor.is_some());
                menu.output.as_mut().unwrap().find = Some(Input::default());
                let searching = frame(&mut menu, size, true, color);
                let search_rows = rows(&searching).join("\n");
                assert!(search_rows.contains("Find"));
                assert!(!search_rows.contains("Wait for command"));
                assert!(
                    rows(&searching)
                        .iter()
                        .all(|line| line.width() < usize::from(size.0))
                );
            }
        }
    }

    #[test]
    fn results_put_status_above_command_and_keep_suggestions_below_it_without_hints() {
        for (status, running) in [
            ("Running…", true),
            ("Cancelling…", true),
            ("✓ PASS · Exit 0 · Duration 850 ms", false),
            ("✗ FAIL · Exit 1 · Duration 850 ms", false),
            ("! BLOCKED · Exit 2 · Duration 850 ms", false),
            ("! NOT_RUN · Exit 2 · Duration 850 ms", false),
            ("! CANCELLED · Exit ? · Duration 850 ms", false),
            ("! TIMED_OUT · Exit ? · Duration 850 ms", false),
        ] {
            let mut menu = Menu::new(false, None);
            press(&mut menu, KeyCode::Char('l'));
            let mut output = OutputView::new("cnr list".into(), false);
            output.running = running;
            output.status = status.into();
            menu.output = Some(output);
            let result = frame(&mut menu, (80, 24), true, true);
            let lines = rows(&result);
            let input_row = usize::from(result.cursor.unwrap().1);
            assert_eq!(lines[input_row - 3].trim(), status);
            assert!(lines[input_row].contains("› l"));
            assert!(!lines.iter().any(|line| {
                ["PgUp/PgDn", "Ctrl+C", "Tab complete", "Esc back"]
                    .iter()
                    .any(|hint| line.contains(hint))
            }));
            assert_eq!(
                lines
                    .get(input_row + 1)
                    .is_some_and(|line| line.contains("list")),
                !running
            );
        }
    }

    #[test]
    fn responsive_frames_keep_status_command_and_footer_within_their_own_regions() {
        for vi in [false, true] {
            for color in [false, true] {
                for size in [
                    (120, 35),
                    (100, 25),
                    (80, 24),
                    (50, 15),
                    (26, 8),
                    (12, 4),
                    (2, 2),
                    (1, 1),
                    (0, 0),
                ] {
                    let mut menu = Menu::new(vi, None);
                    press(&mut menu, KeyCode::Char('l'));
                    let main = frame(&mut menu, size, true, color);
                    let main_rows = rows(&main);
                    let columns = usize::from(size.0.saturating_sub(1));
                    assert!(main_rows.iter().all(|line| line.width() <= columns));
                    if size.0 >= 26 {
                        let last = main.rows.last().unwrap();
                        assert!(last.has(Tone::Muted));
                        assert!(last.text().contains("Ctrl+C"));
                    }
                    if size.0 > 1 && size.1 > 0 {
                        let (x, y) = main.cursor.expect("input remains visible");
                        assert!(x < size.0 && y < size.1);
                        if size.1 >= 2 {
                            assert!(y < size.1 - 1);
                        }
                        if size.0 >= 26 {
                            assert!(
                                main_rows[usize::from(y + 1)..]
                                    .iter()
                                    .any(|line| line.contains("list"))
                            );
                        }
                    }

                    let mut output = OutputView::new("cnr list".into(), vi);
                    output.buffer = super::super::viewport::Buffer::from_streams(
                        "{\"count\": 42}".into(),
                        "",
                        None,
                    );
                    output.running = false;
                    output.status = "✓ PASS · Exit 0 · Duration 850 ms".into();
                    menu.output = Some(output);
                    let result = frame(&mut menu, size, true, color);
                    let result_rows = rows(&result);
                    assert!(result_rows.iter().all(|line| line.width() <= columns));
                    assert!(
                        !result_rows
                            .iter()
                            .any(|line| line.contains("Ctrl+C") || line.contains("PgUp/PgDn"))
                    );
                    if size.0 > 1 && size.1 > 0 {
                        let (x, y) = result.cursor.expect("input remains visible");
                        assert!(x < size.0 && y < size.1);
                        if size.1 >= 2 && size.0 >= 26 {
                            let y = usize::from(y);
                            assert!(
                                result_rows[y.saturating_sub(3)..y]
                                    .iter()
                                    .any(|line| line.contains("PASS"))
                            );
                            assert!(
                                result_rows[y + 1..]
                                    .iter()
                                    .any(|line| line.contains("list"))
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn result_palette_shrinks_and_restores_the_viewport_without_losing_scroll() {
        let mut menu = Menu::new(false, None);
        menu.open_prompt(Input::default());
        let mut output = OutputView::new("cnr list".into(), false);
        output.running = false;
        output.buffer = super::super::viewport::Buffer::from_streams(
            (0..20_000).map(|i| format!("row {i}\n")).collect(),
            "",
            None,
        );
        menu.output = Some(output);
        let size = (120, 35);
        let initial = frame(&mut menu, size, true, true);
        let output = menu.output.as_mut().unwrap();
        let _ = output.scroll_to(80, false);
        let scrolled = frame(&mut menu, size, true, true);
        press(&mut menu, KeyCode::Char('l'));
        let opened = frame(&mut menu, size, true, true);
        let command = opened.cursor.unwrap().1;
        assert!(command < initial.cursor.unwrap().1);
        assert!(
            opened.rows[usize::from(command + 1)]
                .text()
                .contains("list")
        );
        assert_eq!(opened.rows[1], scrolled.rows[1]);
        assert_eq!(menu.output.as_ref().unwrap().scroll, 80);

        press(&mut menu, KeyCode::Backspace);
        let closed = frame(&mut menu, size, true, true);
        assert_eq!(closed.cursor, scrolled.cursor);
        assert_eq!(closed.changed_rows(Some(&scrolled)).count(), 0);
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
        let art = frame(&mut menu, (100, 25), true, true);
        let lines = rows(&art);
        assert!(lines.iter().any(|line| line.contains('█')));
        assert!(art.rows.iter().any(|line| line.has(Tone::Shadow)));
    }

    #[test]
    fn monochrome_banner_keeps_letter_faces_without_shadow_noise() {
        let mut menu = Menu::new(false, None);
        let color = frame(&mut menu, (100, 25), true, true);
        let monochrome = frame(&mut menu, (100, 25), true, false);
        let colored_rows = rows(&color);
        let plain_rows = rows(&monochrome);
        assert!(color.rows.iter().any(|line| line.has(Tone::Shadow)));
        assert!(monochrome.rows.iter().all(|line| !line.has(Tone::Shadow)));
        for (colored, plain) in colored_rows.iter().zip(&plain_rows) {
            assert_eq!(colored.chars().count(), plain.chars().count());
            for (colored, plain) in colored.chars().zip(plain.chars()) {
                if colored != plain {
                    assert_eq!(plain, ' ');
                    assert!(matches!(colored, '⠛' | '⣤' | '⣿' | '\'' | '.' | ':'));
                }
            }
        }
    }

    #[test]
    fn navigation_changes_only_the_affected_rows() {
        let mut menu = Menu::new(false, None);
        let first = frame(&mut menu, (80, 25), true, true);
        assert!(matches!(press(&mut menu, KeyCode::Down), Effect::Redraw));
        let next = frame(&mut menu, (80, 25), true, true);
        let changed: Vec<_> = next
            .changed_rows(Some(&first))
            .map(|(_, line)| line.text())
            .collect();
        assert_eq!(changed, ["    Tool inventory", "  › Search tools"]);
        assert_eq!(next.changed_rows(Some(&next)).count(), 0);

        let wide = frame(&mut menu, (110, 30), true, true);
        assert!(matches!(press(&mut menu, KeyCode::Down), Effect::Redraw));
        let next = frame(&mut menu, (110, 30), true, true);
        for (_, line) in next.changed_rows(Some(&wide)) {
            assert!(!line.text().contains('█'));
        }
        assert!(
            next.changed_rows(Some(&wide))
                .any(|(_, line)| line.text().contains("cnr install"))
        );
    }

    #[test]
    fn help_and_search_keep_one_footer_at_the_same_position() {
        let mut menu = Menu::new(true, None);
        let initial = frame(&mut menu, (78, 24), true, true);
        assert!(
            rows(&initial)
                .iter()
                .all(|line| !line.contains("cnr --help"))
        );
        assert_eq!(initial.cursor, None);
        press(&mut menu, KeyCode::Char('?'));
        let help = frame(&mut menu, (78, 24), true, true);
        assert!(rows(&help).iter().any(|line| line.contains("cnr --help")));
        assert_eq!(help.rows.last(), initial.rows.last());

        press(&mut menu, KeyCode::Char('/'));
        for ch in "chẩn đoán".chars() {
            press(&mut menu, KeyCode::Char(ch));
        }
        let search = frame(&mut menu, (78, 24), true, true);
        assert!(search.rows[22].text().contains("Tìm kiếm  › chẩn đoán"));
        assert_eq!(search.cursor.map(|(_, y)| y), Some(22));
        assert!(
            rows(&search)
                .iter()
                .any(|line| line.contains("› Chẩn đoán"))
        );
        press(&mut menu, KeyCode::Esc);
        let closed = frame(&mut menu, (78, 24), true, true);
        assert!(closed.rows[22].text().is_empty());
        assert!(closed.changed_rows(Some(&search)).any(|(row, _)| row == 22));
    }

    #[test]
    fn resized_and_tiny_frames_fit_terminal_columns() {
        let mut menu = Menu::new(false, None);
        assert_eq!(
            WORDMARK.iter().map(|row| row.width()).max(),
            Some(WORDMARK_WIDTH)
        );
        let art = frame(&mut menu, (92, 22), true, true);
        assert!(art.rows[1].text().contains('█'));
        for size in [(92, 22), (93, 22), (94, 22), (100, 25)] {
            for color in [true, false] {
                let frame = frame(&mut menu, size, true, color);
                assert!(
                    rows(&frame)
                        .iter()
                        .all(|line| line.width() < usize::from(size.0))
                );
            }
        }
        for (size, banner) in [((91, 22), true), ((91, 21), true), ((110, 25), false)] {
            let compact = frame(&mut menu, size, banner, true);
            assert!(rows(&compact).iter().any(|line| line.trim() == NAME));
            assert!(compact.changed_rows(Some(&art)).count() > 2);
        }
        for size in [(0, 0), (1, 1), (2, 2), (10, 4), (26, 8)] {
            let small = frame(&mut menu, size, true, true);
            for line in rows(&small) {
                assert!(line.width() < usize::from(size.0).max(1));
                assert!(!line.contains('\u{1b}'));
            }
        }
        assert_eq!(clip("Việt 東京", 7), "Việt 東");
    }
}
