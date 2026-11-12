use super::terminal::{Line, Tone};

pub(super) const WORDMARK_WIDTH: usize = 90;
pub(super) const BANNER_WIDTH: usize = WORDMARK_WIDTH + 1;
pub(super) const WORDMARK: [&str; 5] = [
    "▄██████ ███████ ██████▄ ██▄  ██ ███████ ███████    ██████▄ ██   ██ ██      ███████ ▄██████",
    "██        ███   ██   ██ ███▄ ██ ██        ███      ██   ██ ██   ██ ██      ██      ██",
    "██        ███   ████▀▀  ██ ████ ██▀▀▀▀    ███      ████▀▀  ██   ██ ██      ██▀▀▀▀   ▀▀▀▀█▄",
    "██▄▄▄▄▄ ▄▄███▄▄ ██ ▀█▄  ██  ███ ██▄▄▄▄▄   ███      ██ ▀█▄  ██▄▄▄██ ██▄▄▄▄▄ ██▄▄▄▄▄ ▄▄▄▄▄██",
    " ▀▀▀▀▀▀ ▀▀▀▀▀▀▀ ▀▀   ▀▀ ▀▀   ▀▀ ▀▀▀▀▀▀▀   ▀▀▀      ▀▀   ▀▀  ▀▀▀▀▀  ▀▀▀▀▀▀▀ ▀▀▀▀▀▀▀ ▀▀▀▀▀▀",
];

#[derive(Default)]
pub(super) struct Banner {
    cached: Option<(usize, bool, Vec<Line>)>,
}

impl Banner {
    pub(super) fn lines(&mut self, inset: usize, color: bool) -> &[Line] {
        if self
            .cached
            .as_ref()
            .is_none_or(|(x, colors, _)| *x != inset || *colors != color)
        {
            self.cached = Some((
                inset,
                color,
                compose(&WORDMARK, WORDMARK_WIDTH, inset, color),
            ));
        }
        &self.cached.as_ref().expect("banner initialized").2
    }
}

fn compose(art: &[&str], width: usize, inset: usize, color: bool) -> Vec<Line> {
    let art: Vec<Vec<char>> = art.iter().map(|row| row.chars().collect()).collect();
    let mut lines = Vec::with_capacity(art.len() + 1);
    for y in 0..=art.len() {
        let mut line = Line::default();
        line.push(Tone::Plain, " ".repeat(inset));
        for x in 0..=width {
            let face = x
                .checked_sub(1)
                .and_then(|x| art.get(y).and_then(|row| row.get(x)))
                .copied()
                .unwrap_or(' ');
            let shadow = y
                .checked_sub(1)
                .and_then(|y| art.get(y).and_then(|row| row.get(x)))
                .copied()
                .unwrap_or(' ');
            if face != ' ' {
                let gradient = (x.saturating_sub(1) * 4 / width.max(1)).min(3);
                line.push(Tone::Banner(gradient), face.encode_utf8(&mut [0; 4]));
            } else if color {
                // Each block encodes two vertical pixels. Preserve their mask
                // rather than shading an entire cell for a half-height stroke.
                let shaded = match shadow {
                    '█' | '▀' | '▄' => shadow,
                    _ => ' ',
                };
                let tone = if shaded == ' ' {
                    Tone::Plain
                } else {
                    Tone::Shadow
                };
                line.push(tone, shaded.encode_utf8(&mut [0; 4]));
            } else {
                line.push(Tone::Plain, " ");
            }
        }
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::UnicodeWidthStr;

    fn text(lines: &[Line]) -> Vec<String> {
        lines.iter().map(Line::text).collect()
    }

    #[test]
    fn shadows_follow_the_occupied_halves_of_i_and_s() {
        let i = text(&compose(&["███", " ▄ ", " ▀ ", "███"], 3, 0, true));
        assert_eq!(i, [" ███", "██▄ ", " ▄▀ ", " ███", "███ "]);
        let s = text(&compose(&["███", "▄  ", " ▀▀", "  ▄"], 3, 0, true));
        assert_eq!(s[2], "▄ ▀▀");
        assert_eq!(s[4], "  ▄ ");
    }

    #[test]
    fn face_wins_in_shared_cells_and_monochrome_omits_shadow() {
        let colored = text(&compose(&[" ▄", "▀ "], 2, 0, true));
        assert_eq!(colored, ["  ▄", " ▀ ", "▀  "]);
        let plain = text(&compose(&["███", " ▄ ", " ▀ ", "███"], 3, 0, false));
        assert!(plain.iter().all(|row| !row.contains('░')));
        assert_eq!(plain, [" ███", "  ▄ ", "  ▀ ", " ███", "    "]);
    }

    #[test]
    fn real_banner_preserves_width_and_cache_in_both_modes() {
        let mut banner = Banner::default();
        for color in [true, false] {
            let lines = banner.lines(2, color);
            assert_eq!(lines.len(), WORDMARK.len() + 1);
            assert!(
                text(lines)
                    .iter()
                    .all(|row| row.width() == BANNER_WIDTH + 2)
            );
            assert!(text(lines).iter().all(|row| !row.contains('░')));
        }
    }
}
