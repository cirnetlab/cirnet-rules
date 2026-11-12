use super::terminal::{Line, Tone};

pub(super) const WORDMARK_WIDTH: usize = 90;
// The face stays at its original one-column inset; the shadow may extend one
// more column. The latter can be clipped without clipping the face.
pub(super) const BANNER_WIDTH: usize = WORDMARK_WIDTH + 1;
const FULL_BANNER_WIDTH: usize = WORDMARK_WIDTH + 2;
pub(super) const WORDMARK: [&str; 5] = [
    "▄██████ ███████ ██████▄ ██▄  ██ ███████ ███████    ██████▄ ██   ██ ██      ███████ ▄██████",
    "██        ███   ██   ██ ███▄ ██ ██        ███      ██   ██ ██   ██ ██      ██      ██",
    "██        ███   ████▀▀  ██ ████ ██▀▀▀▀    ███      ████▀▀  ██   ██ ██      ██▀▀▀▀   ▀▀▀▀█▄",
    "██▄▄▄▄▄ ▄▄███▄▄ ██ ▀█▄  ██  ███ ██▄▄▄▄▄   ███      ██ ▀█▄  ██▄▄▄██ ██▄▄▄▄▄ ██▄▄▄▄▄ ▄▄▄▄▄██",
    " ▀▀▀▀▀▀ ▀▀▀▀▀▀▀ ▀▀   ▀▀ ▀▀   ▀▀ ▀▀▀▀▀▀▀   ▀▀▀      ▀▀   ▀▀  ▀▀▀▀▀  ▀▀▀▀▀▀▀ ▀▀▀▀▀▀▀ ▀▀▀▀▀▀",
];

pub(super) struct Banner {
    glyphs: ShadowGlyphs,
    cached: Option<(usize, bool, usize, Vec<Line>)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShadowGlyphs {
    Braille,
    Ascii,
}

impl Default for Banner {
    fn default() -> Self {
        // A terminal cannot reliably report glyph coverage for its selected
        // font. Permit an explicit ASCII fallback and use it for Linux's
        // traditional console, whose fonts often omit Braille patterns.
        let ascii = std::env::var("CNR_SHADOW").is_ok_and(|s| s.eq_ignore_ascii_case("ascii"))
            || std::env::var("TERM").is_ok_and(|s| s == "linux");
        Self {
            glyphs: if ascii {
                ShadowGlyphs::Ascii
            } else {
                ShadowGlyphs::Braille
            },
            cached: None,
        }
    }
}

impl Banner {
    pub(super) fn lines(&mut self, inset: usize, color: bool, available: usize) -> &[Line] {
        let available = available.min(FULL_BANNER_WIDTH);
        if self.cached.as_ref().is_none_or(|(x, colors, columns, _)| {
            *x != inset || *colors != color || *columns != available
        }) {
            self.cached = Some((
                inset,
                color,
                available,
                compose_with(
                    &WORDMARK,
                    WORDMARK_WIDTH,
                    inset,
                    color,
                    available,
                    self.glyphs,
                ),
            ));
        }
        &self.cached.as_ref().expect("banner initialized").3
    }
}

#[cfg(test)]
fn compose(art: &[&str], width: usize, inset: usize, color: bool) -> Vec<Line> {
    compose_with(art, width, inset, color, width + 2, ShadowGlyphs::Braille)
}

fn pixels(ch: char) -> u8 {
    match ch {
        '█' => 0b11,
        '▀' => 0b01,
        '▄' => 0b10,
        _ => 0,
    }
}

fn dot(mask: u8, glyphs: ShadowGlyphs) -> char {
    match (glyphs, mask) {
        (ShadowGlyphs::Braille, 0b01) => '⠛',
        (ShadowGlyphs::Braille, 0b10) => '⣤',
        (ShadowGlyphs::Braille, 0b11) => '⣿',
        (ShadowGlyphs::Ascii, 0b01) => '\'',
        (ShadowGlyphs::Ascii, 0b10) => '.',
        (ShadowGlyphs::Ascii, 0b11) => ':',
        _ => ' ',
    }
}

fn compose_with(
    art: &[&str],
    width: usize,
    inset: usize,
    color: bool,
    available: usize,
    glyphs: ShadowGlyphs,
) -> Vec<Line> {
    let art: Vec<Vec<char>> = art.iter().map(|row| row.chars().collect()).collect();
    let mut lines = Vec::with_capacity(art.len() + 1);
    for y in 0..=art.len() {
        let mut line = Line::default();
        line.push(Tone::Plain, " ".repeat(inset));
        for x in 0..available.min(width + 2) {
            let face = x
                .checked_sub(1)
                .and_then(|x| art.get(y).and_then(|row| row.get(x)))
                .copied()
                .unwrap_or(' ');
            let shadow = y
                .checked_sub(1)
                .and_then(|y| {
                    x.checked_sub(2)
                        .and_then(|x| art.get(y).and_then(|row| row.get(x)))
                })
                .copied()
                .unwrap_or(' ');
            if face != ' ' {
                let gradient = (x.saturating_sub(1) * 4 / width.max(1)).min(3);
                line.push(Tone::Banner(gradient), face.encode_utf8(&mut [0; 4]));
            } else if color {
                // The visible shadow is the translated half-cell mask with
                // every face pixel excluded. If face occupies either half of a
                // cell, it wins: a terminal cannot combine its orange block
                // with a differently coloured dotted glyph in the same cell.
                let shaded = dot(pixels(shadow) & !pixels(face), glyphs);
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
        assert_eq!(i, [" ███ ", "  ▄⣿⣿", "  ▀⣤ ", " ███ ", "  ⣿⣿⣿"]);
        let s = text(&compose(&["███", "▄  ", " ▀▀", "  ▄"], 3, 0, true));
        assert_eq!(s, [" ███ ", " ▄⣿⣿⣿", "  ▀▀ ", "   ▄⠛", "    ⣤"]);
    }

    #[test]
    fn face_wins_in_shared_cells_and_monochrome_omits_shadow() {
        let colored = text(&compose(&[" ▄", "▀ "], 2, 0, true));
        assert_eq!(colored, ["  ▄ ", " ▀ ⣤", "  ⠛ "]);
        let plain = text(&compose(&["███", " ▄ ", " ▀ ", "███"], 3, 0, false));
        assert!(plain.iter().all(|row| !row.contains('░')));
        assert_eq!(plain, [" ███ ", "  ▄  ", "  ▀  ", " ███ ", "     "]);
    }

    #[test]
    fn real_banner_preserves_width_and_cache_in_both_modes() {
        let mut banner = Banner::default();
        for color in [true, false] {
            let lines = banner.lines(2, color, FULL_BANNER_WIDTH);
            assert_eq!(lines.len(), WORDMARK.len() + 1);
            assert!(
                text(lines)
                    .iter()
                    .all(|row| row.width() == BANNER_WIDTH + 3)
            );
            assert!(text(lines).iter().all(|row| !row.contains('░')));
        }
    }

    #[test]
    fn all_real_face_cells_keep_their_positions_and_shadow_clips_at_edge() {
        let full = text(&compose_with(
            &WORDMARK,
            WORDMARK_WIDTH,
            0,
            true,
            FULL_BANNER_WIDTH,
            ShadowGlyphs::Braille,
        ));
        let clipped = text(&compose_with(
            &WORDMARK,
            WORDMARK_WIDTH,
            0,
            true,
            BANNER_WIDTH,
            ShadowGlyphs::Braille,
        ));
        for (y, original) in WORDMARK.iter().enumerate() {
            let actual: Vec<_> = full[y].chars().collect();
            for (x, ch) in original.chars().enumerate() {
                if ch != ' ' {
                    assert_eq!(actual[x + 1], ch, "row {y}, column {x}");
                }
            }
        }
        for (full, clipped) in full.iter().zip(&clipped) {
            assert_eq!(
                clipped,
                &full.chars().take(BANNER_WIDTH).collect::<String>()
            );
            assert_eq!(clipped.width(), BANNER_WIDTH);
        }
    }

    #[test]
    fn ascii_fallback_keeps_half_masks_and_never_uses_block_shading() {
        let result = text(&compose_with(
            &["███", " ▄ ", " ▀ ", "███"],
            3,
            0,
            true,
            5,
            ShadowGlyphs::Ascii,
        ));
        assert_eq!(result, [" ███ ", "  ▄::", "  ▀. ", " ███ ", "  :::"]);
        let no_color = text(&compose_with(
            &["███", " ▄ ", " ▀ ", "███"],
            3,
            0,
            false,
            5,
            ShadowGlyphs::Ascii,
        ));
        assert_eq!(no_color, [" ███ ", "  ▄  ", "  ▀  ", " ███ ", "     "]);
    }
}
