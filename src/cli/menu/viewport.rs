use super::{
    highlight::{Span, highlight},
    terminal::{Line, Tone},
};
use std::ops::Range;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Default)]
pub(super) struct Buffer {
    text: String,
    styles: Vec<Span>,
    wrap: Option<Wrap>,
}

struct Wrap {
    width: usize,
    rows: Vec<Range<usize>>,
}

impl Buffer {
    pub(super) fn from_streams(mut stdout: String, stderr: &str, notice: Option<&str>) -> Self {
        let mut styles = highlight(&stdout, false);
        if !stderr.is_empty() {
            let start = separate(&mut stdout);
            stdout.push_str("[stderr]\n");
            styles.push(Span {
                range: start..stdout.len() - 1,
                tone: Tone::Stderr,
            });
            let offset = stdout.len();
            stdout.push_str(stderr);
            styles.extend(highlight(stderr, true).into_iter().map(|span| Span {
                range: (offset + span.range.start)..(offset + span.range.end),
                tone: span.tone,
            }));
        }
        if let Some(notice) = notice.filter(|text| !text.is_empty()) {
            let start = separate(&mut stdout);
            stdout.push_str(notice);
            styles.push(Span {
                range: start..stdout.len(),
                tone: Tone::Warning,
            });
        }
        Self {
            text: stdout,
            styles,
            wrap: None,
        }
    }

    pub(super) fn layout(&mut self, width: usize) -> usize {
        if self.wrap.as_ref().is_none_or(|wrap| wrap.width != width) {
            let mut rows = Vec::new();
            if width > 0 {
                let mut offset = 0;
                for value in self.text.split_inclusive('\n') {
                    let line = value.strip_suffix('\n').unwrap_or(value);
                    let mut start = 0;
                    let mut used = 0;
                    for (index, ch) in line.char_indices() {
                        let columns = ch.width().unwrap_or(0).min(width);
                        if used + columns > width {
                            rows.push(offset + start..offset + index);
                            start = index;
                            used = 0;
                        }
                        used += columns;
                    }
                    rows.push(offset + start..offset + line.len());
                    offset += value.len();
                }
            }
            self.wrap = Some(Wrap { width, rows });
        }
        self.wrap.as_ref().map_or(0, |wrap| wrap.rows.len())
    }

    pub(super) fn row_start(&self, index: usize) -> Option<usize> {
        self.wrap.as_ref()?.rows.get(index).map(|row| row.start)
    }

    pub(super) fn row_at(&self, byte: usize) -> usize {
        self.wrap.as_ref().map_or(0, |wrap| {
            wrap.rows
                .partition_point(|row| row.start <= byte)
                .saturating_sub(1)
        })
    }

    pub(super) fn find(
        &self,
        query: &str,
        previous: Option<&Range<usize>>,
        backwards: bool,
    ) -> Option<Range<usize>> {
        if query.is_empty() {
            return None;
        }
        let start = if backwards {
            let end = previous.map_or(self.text.len(), |hit| hit.start);
            self.text
                .get(..end)
                .and_then(|text| text.rfind(query))
                .or_else(|| self.text.rfind(query))
        } else {
            let start = previous.map_or(0, |hit| hit.end);
            self.text
                .get(start..)
                .and_then(|text| text.find(query))
                .map(|index| index + start)
                .or_else(|| self.text.find(query))
        }?;
        Some(start..start + query.len())
    }

    pub(super) fn line(
        &self,
        index: usize,
        inset: &str,
        hit: Option<&Range<usize>>,
    ) -> Option<Line> {
        let wrap = self.wrap.as_ref()?;
        let row = wrap.rows.get(index)?;
        let mut line = Line::default();
        line.push(Tone::Plain, inset);
        if wrap.width == 1 && self.text[row.clone()].width() > 1 {
            // Retain the original bytes for search and for the next resize.
            line.push(Tone::Plain, "�");
            return Some(line);
        }
        let mut start = row.start;
        let mut styles = self.styles[self.styles.partition_point(|span| span.range.end <= start)..]
            .iter()
            .peekable();
        let hit = hit.filter(|hit| {
            hit.start < hit.end
                && self.text.is_char_boundary(hit.start)
                && self.text.is_char_boundary(hit.end)
        });
        while start < row.end {
            let (end, tone) = match styles.peek() {
                Some(span) if span.range.start <= start => {
                    let styled = (span.range.end.min(row.end), span.tone);
                    styles.next();
                    styled
                }
                Some(span) => (span.range.start.min(row.end), Tone::Plain),
                None => (row.end, Tone::Plain),
            };
            if let Some(hit) = hit.filter(|hit| hit.start < end && hit.end > start) {
                let matched_start = start.max(hit.start);
                let matched_end = end.min(hit.end);
                line.push(tone, &self.text[start..matched_start]);
                line.push(Tone::Match, &self.text[matched_start..matched_end]);
                line.push(tone, &self.text[matched_end..end]);
            } else {
                line.push(tone, &self.text[start..end]);
            }
            start = end;
        }
        Some(line)
    }
}

fn separate(text: &mut String) -> usize {
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styled_wrapping_preserves_indentation_and_unicode() {
        let input = "  {\"東京\": 12, \"ready\": true}\n\n a\u{301}";
        let mut buffer = Buffer::from_streams(input.into(), "", None);
        let count = buffer.layout(8);
        assert!(count > 4);
        let rows = (0..count)
            .map(|index| buffer.line(index, "", None).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(rows[0].text(), "  {\"東京");
        assert!(rows.iter().any(|row| row.has(Tone::Number)));
        assert_eq!(
            rows.iter().map(Line::text).collect::<String>(),
            input.replace('\n', "")
        );
    }

    #[test]
    fn large_output_reuses_wrapped_offsets_and_materializes_only_requested_rows() {
        let mut buffer = Buffer::from_streams("[1]\n".repeat(10_000), "", None);
        assert_eq!(buffer.layout(100), 10_000);
        let previous = buffer.wrap.as_ref().unwrap().rows.as_ptr();
        assert_eq!(buffer.layout(100), 10_000);
        assert_eq!(previous, buffer.wrap.as_ref().unwrap().rows.as_ptr());
        assert_eq!(buffer.line(9_999, "", None).unwrap().text(), "[1]");
        assert!(buffer.line(10_000, "", None).is_none());
        assert_eq!(buffer.layout(2), 20_000);
    }

    #[test]
    fn search_wraps_and_separates_streams() {
        let mut buffer = Buffer::from_streams(
            "first needle\n".into(),
            "second needle\n",
            Some("truncated"),
        );
        assert_eq!(buffer.layout(20), 4);
        let first = buffer.find("needle", None, false).unwrap();
        let second = buffer.find("needle", Some(&first), false).unwrap();
        assert_ne!(first, second);
        assert_eq!(
            buffer.find("needle", Some(&second), false),
            Some(first.clone())
        );
        assert_eq!(
            buffer.find("needle", Some(&first), true),
            Some(second.clone())
        );
        let row = buffer.row_at(second.start);
        assert_eq!(row, 2);
        let line = buffer.line(row, "", Some(&second)).unwrap();
        assert_eq!(line.text(), "second needle");
        assert!(line.has(Tone::Match));
        assert_eq!(buffer.line(1, "", None).unwrap().text(), "[stderr]");
    }
}
