use super::{printable, runner, text_modifiers};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{collections::VecDeque, ops::Range};
use unicode_segmentation::GraphemeCursor;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const EDIT_LIMIT: usize = 64;

#[derive(Clone)]
struct Snapshot {
    value: String,
    cursor: usize,
    anchor: Option<usize>,
}

#[derive(Clone, Default)]
pub(super) struct Input {
    pub(super) value: String,
    pub(super) cursor: usize,
    anchor: Option<usize>,
    undo: VecDeque<Snapshot>,
    redo: VecDeque<Snapshot>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum PasteError {
    TooLong,
}

impl Input {
    pub(super) fn at(value: String, cursor: usize) -> Self {
        Self {
            cursor: snap(&value, cursor),
            value,
            ..Self::default()
        }
    }

    pub(super) fn at_end(value: String) -> Self {
        Self::at(value, usize::MAX)
    }

    pub(super) fn selection(&self) -> Option<Range<usize>> {
        self.anchor
            .filter(|&anchor| anchor != self.cursor)
            .map(|anchor| anchor.min(self.cursor)..anchor.max(self.cursor))
    }

    pub(super) fn selected_text(&self) -> Option<&str> {
        self.selection().map(|range| &self.value[range])
    }

    pub(super) fn clear_selection(&mut self) -> bool {
        self.anchor
            .take()
            .is_some_and(|anchor| anchor != self.cursor)
    }

    pub(super) fn select_all(&mut self) -> bool {
        if self.value.is_empty() {
            return false;
        }
        let changed = self.anchor != Some(0) || self.cursor != self.value.len();
        self.anchor = Some(0);
        self.cursor = self.value.len();
        changed
    }

    pub(super) fn move_to(&mut self, offset: usize, extend: bool) -> bool {
        let previous = (self.cursor, self.anchor);
        let offset = snap(&self.value, offset);
        self.anchor = if extend {
            Some(self.anchor.unwrap_or(self.cursor))
        } else {
            None
        };
        self.cursor = offset;
        (self.cursor, self.anchor) != previous
    }

    pub(super) fn offset_at_column(&self, start: usize, column: usize) -> usize {
        let start = snap(&self.value, start);
        start + byte_at_column(&self.value[start..], column)
    }

    pub(super) fn fits_replacement(&self, bytes: usize, limit: usize) -> bool {
        let removed = self.selection().map_or(0, |range| range.len());
        fits(self.value.len() - removed, bytes, limit)
    }

    pub(super) fn insert(&mut self, value: &str, limit: usize) -> bool {
        if value.is_empty() || !self.fits_replacement(value.len(), limit) {
            return false;
        }
        let range = self.selection().unwrap_or(self.cursor..self.cursor);
        self.replace(range, value)
    }

    pub(super) fn paste_sanitized(
        &mut self,
        value: &str,
        limit: usize,
    ) -> Result<bool, PasteError> {
        if value.len() > limit {
            return Err(PasteError::TooLong);
        }
        let sanitized = runner::sanitize(value.as_bytes()).replace('\n', " ");
        if !self.fits_replacement(sanitized.len(), limit) {
            return Err(PasteError::TooLong);
        }
        Ok(self.insert(&sanitized, limit))
    }

    pub(super) fn replace_from(
        &mut self,
        base: &Self,
        range: Range<usize>,
        value: &str,
        limit: usize,
    ) -> bool {
        if range.start > range.end
            || snap(&base.value, range.start) != range.start
            || snap(&base.value, range.end) != range.end
            || !fits(base.value.len() - range.len(), value.len(), limit)
        {
            return false;
        }
        self.clone_from(base);
        self.replace(range, value)
    }

    pub(super) fn delete_selection(&mut self) -> bool {
        self.selection()
            .is_some_and(|range| self.replace(range, ""))
    }

    pub(super) fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub(super) fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub(super) fn undo(&mut self) -> bool {
        let Some(previous) = self.undo.pop_back() else {
            return false;
        };
        let current = self.restore(previous);
        self.redo.push_back(current);
        true
    }

    pub(super) fn redo(&mut self) -> bool {
        let Some(next) = self.redo.pop_back() else {
            return false;
        };
        let current = self.restore(next);
        self.undo.push_back(current);
        true
    }

    pub(super) fn edit(&mut self, key: KeyEvent, limit: usize) -> bool {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        if let KeyCode::Char(ch) = key.code
            && matches!(key.modifiers - KeyModifiers::SHIFT, KeyModifiers::CONTROL)
        {
            return match (ch.to_ascii_lowercase(), shift) {
                ('z', false) => self.undo(),
                ('y', false) | ('z', true) => self.redo(),
                ('a', false) => self.select_all(),
                ('e', false) => self.move_to(self.value.len(), false),
                ('u', false) => {
                    let range = self.selection().unwrap_or(0..self.cursor);
                    self.replace(range, "")
                }
                _ => false,
            };
        }
        if let Some(ch) = printable(key) {
            return self.insert(ch.encode_utf8(&mut [0; 4]), limit);
        }
        match key.code {
            KeyCode::Home | KeyCode::End
                if (key.modifiers - KeyModifiers::SHIFT - KeyModifiers::CONTROL).is_empty() =>
            {
                let offset = if key.code == KeyCode::Home {
                    0
                } else {
                    self.value.len()
                };
                self.move_to(offset, shift)
            }
            _ if !text_modifiers(key) => false,
            KeyCode::Left => self.move_horizontally(false, shift),
            KeyCode::Right => self.move_horizontally(true, shift),
            KeyCode::Backspace => {
                let range = self
                    .selection()
                    .unwrap_or_else(|| self.adjacent(false)..self.cursor);
                self.replace(range, "")
            }
            KeyCode::Delete => {
                let range = self
                    .selection()
                    .unwrap_or_else(|| self.cursor..self.adjacent(true));
                self.replace(range, "")
            }
            _ => false,
        }
    }

    fn adjacent(&self, forward: bool) -> usize {
        let mut cursor = GraphemeCursor::new(self.cursor, self.value.len(), true);
        let boundary = if forward {
            cursor.next_boundary(&self.value, 0)
        } else {
            cursor.prev_boundary(&self.value, 0)
        };
        boundary
            .expect("entire field available")
            .unwrap_or(self.cursor)
    }

    fn move_horizontally(&mut self, forward: bool, extend: bool) -> bool {
        let selected_edge = self
            .selection()
            .filter(|_| !extend)
            .map(|range| if forward { range.end } else { range.start });
        self.move_to(
            selected_edge.unwrap_or_else(|| self.adjacent(forward)),
            extend,
        )
    }

    fn replace(&mut self, range: Range<usize>, value: &str) -> bool {
        let offset = range.start + value.len();
        if self.value[range.clone()] == *value {
            return self.move_to(offset, false);
        }
        let previous = Snapshot {
            value: self.value.clone(),
            cursor: self.cursor,
            anchor: self.anchor,
        };
        self.value.replace_range(range, value);
        // Combining input can extend the surrounding grapheme past the inserted bytes.
        let mut cursor = GraphemeCursor::new(offset, self.value.len(), true);
        self.cursor = if cursor
            .is_boundary(&self.value, 0)
            .expect("entire field available")
        {
            offset
        } else {
            cursor
                .next_boundary(&self.value, 0)
                .expect("entire field available")
                .unwrap_or(self.value.len())
        };
        self.anchor = None;
        self.redo.clear();
        if self.undo.len() == EDIT_LIMIT {
            self.undo.pop_front();
        }
        self.undo.push_back(previous);
        true
    }

    fn restore(&mut self, snapshot: Snapshot) -> Snapshot {
        Snapshot {
            value: std::mem::replace(&mut self.value, snapshot.value),
            cursor: std::mem::replace(&mut self.cursor, snapshot.cursor),
            anchor: std::mem::replace(&mut self.anchor, snapshot.anchor),
        }
    }
}

fn fits(remaining: usize, inserted: usize, limit: usize) -> bool {
    remaining
        .checked_add(inserted)
        .is_some_and(|total| total <= limit)
}

fn snap(text: &str, offset: usize) -> usize {
    let offset = text.floor_char_boundary(offset.min(text.len()));
    let mut cursor = GraphemeCursor::new(offset, text.len(), true);
    if cursor.is_boundary(text, 0).expect("entire field available") {
        offset
    } else {
        cursor
            .prev_boundary(text, 0)
            .expect("entire field available")
            .unwrap_or(0)
    }
}

/// Map a display column to the nearest grapheme boundary in visible text.
pub(super) fn byte_at_column(text: &str, column: usize) -> usize {
    let mut cells = 0;
    for (offset, grapheme) in text.grapheme_indices(true) {
        let width = grapheme.width();
        if column < cells + width {
            return if column - cells < width.div_ceil(2) {
                offset
            } else {
                offset + grapheme.len()
            };
        }
        cells += width;
    }
    text.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(input: &mut Input, code: KeyCode, modifiers: KeyModifiers) -> bool {
        input.edit(KeyEvent::new(code, modifiers), 32)
    }

    #[test]
    fn ctrl_a_then_delete_clears_the_entire_field_and_history_restores_it() {
        let mut input = Input::at_end("abc".into());
        assert!(press(&mut input, KeyCode::Char('a'), KeyModifiers::CONTROL));
        assert_eq!(input.selected_text(), Some("abc"));
        assert!(press(&mut input, KeyCode::Delete, KeyModifiers::NONE));
        assert_eq!(input.value, "");
        assert!(press(&mut input, KeyCode::Char('z'), KeyModifiers::CONTROL));
        assert_eq!(input.selected_text(), Some("abc"));
        assert!(press(
            &mut input,
            KeyCode::Char('Z'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT
        ));
        assert_eq!(input.value, "");
    }

    #[test]
    fn backspace_removes_one_extended_grapheme() {
        let mut input = Input::at_end("a👩🏽‍💻".into());
        assert!(press(&mut input, KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(input.value, "a");
    }

    #[test]
    fn shift_selection_uses_extended_graphemes_and_moves_to_the_selected_edge() {
        let mut input = Input::at_end("a\u{302}\u{301}🇻🇳界".into());
        assert!(press(&mut input, KeyCode::Left, KeyModifiers::SHIFT));
        assert!(press(&mut input, KeyCode::Left, KeyModifiers::SHIFT));
        assert_eq!(input.selected_text(), Some("🇻🇳界"));
        assert!(press(&mut input, KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(input.cursor, input.value.len());
        assert_eq!(input.selected_text(), None);
        assert!(press(&mut input, KeyCode::Home, KeyModifiers::SHIFT));
        assert_eq!(input.selected_text(), Some(input.value.as_str()));
    }

    #[test]
    fn composing_characters_preserves_whole_graphemes() {
        let mut input = Input::default();
        for ch in ['a', '\u{302}', '\u{301}', 'n'] {
            assert!(press(&mut input, KeyCode::Char(ch), KeyModifiers::NONE));
        }
        assert!(press(&mut input, KeyCode::Left, KeyModifiers::NONE));
        assert!(press(&mut input, KeyCode::Left, KeyModifiers::SHIFT));
        assert_eq!(input.selected_text(), Some("a\u{302}\u{301}"));
    }

    #[test]
    fn paste_replaces_selection_as_one_edit_and_clears_stale_redo() {
        let mut input = Input::at_end("abc".into());
        assert!(press(&mut input, KeyCode::Left, KeyModifiers::SHIFT));
        assert!(press(&mut input, KeyCode::Left, KeyModifiers::SHIFT));
        assert_eq!(input.selected_text(), Some("bc"));
        assert_eq!(input.paste_sanitized("ệ\r\n界", 8), Ok(true));
        assert_eq!(input.value, "aệ 界");
        assert!(input.can_undo());
        assert!(input.undo());
        assert_eq!(input.value, "abc");
        assert_eq!(input.selected_text(), Some("bc"));
        assert!(input.can_redo());
        assert!(press(&mut input, KeyCode::Char('y'), KeyModifiers::CONTROL));
        assert_eq!(input.value, "aệ 界");
        assert!(input.undo());
        assert!(input.delete_selection());
        assert_eq!(input.value, "a");
        assert!(!input.can_redo());
    }

    #[test]
    fn paste_limits_and_control_strings_preserve_draft_and_selection() {
        let mut input = Input::at_end("12345678".into());
        assert_eq!(input.paste_sanitized("x", 8), Err(PasteError::TooLong));
        assert_eq!(input.value, "12345678");
        assert!(!input.can_undo());
        assert!(input.select_all());
        assert_eq!(
            input.paste_sanitized("123456789", 8),
            Err(PasteError::TooLong)
        );
        assert_eq!(input.selected_text(), Some("12345678"));
        assert_eq!(input.paste_sanitized("\x1b[31mx", 8), Ok(true));
        assert_eq!(input.value, "x");
        assert!(input.undo());
        assert!(input.clear_selection());
        assert!(!input.clear_selection());
    }

    #[test]
    fn completion_cycles_as_one_edit_and_rejects_an_invalid_range() {
        let base = Input::at_end("t".into());
        let mut input = base.clone();
        assert!(input.replace_from(&base, 0..1, "task", 8));
        assert!(input.replace_from(&base, 0..1, "test", 8));
        assert_eq!(input.value, "test");
        assert!(input.undo());
        assert_eq!(input.value, "t");
        assert!(!input.can_undo());
        assert!(!input.replace_from(&base, 0..5, "bad", 8));
        assert_eq!(input.value, "t");
    }

    #[test]
    fn undo_history_is_bounded_to_recent_edits() {
        let mut input = Input::default();
        for _ in 0..80 {
            assert!(input.insert("x", 80));
        }
        while input.undo() {}
        assert_eq!(input.value, "x".repeat(16));
    }

    #[test]
    fn mouse_columns_and_invalid_offsets_snap_to_grapheme_edges() {
        let input = Input::at("e\u{301}界👩🏽‍💻".into(), 2);
        assert_eq!(input.cursor, 0);
        assert_eq!(byte_at_column(&input.value, 0), 0);
        assert_eq!(byte_at_column(&input.value, 1), 3);
        assert_eq!(byte_at_column(&input.value, 2), 6);
        assert_eq!(input.offset_at_column(3, 1), 6);
        assert_eq!(input.offset_at_column(6, 1), input.value.len());
        assert_eq!(byte_at_column(&input.value, 20), input.value.len());
    }
}
