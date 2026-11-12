use super::terminal::{Line, Tone};

pub(super) const WORDMARK_WIDTH: usize = 90;
// The face stays at its original one-column inset; the extrusion may extend
// two additional columns. They can be clipped without clipping the face.
pub(super) const BANNER_WIDTH: usize = WORDMARK_WIDTH + 1;
const FULL_BANNER_WIDTH: usize = WORDMARK_WIDTH + 3;
pub(super) const WORDMARK: [&str; 5] = [
    "▄██████ ███████ ██████▄ ██▄  ██ ███████ ███████    ██████▄ ██   ██ ██      ███████ ▄██████",
    "██        ███   ██   ██ ███▄ ██ ██        ███      ██   ██ ██   ██ ██      ██      ██",
    "██        ███   ████▀▀  ██ ████ ██▀▀▀▀    ███      ████▀▀  ██   ██ ██      ██▀▀▀▀   ▀▀▀▀█▄",
    "██▄▄▄▄▄ ▄▄███▄▄ ██ ▀█▄  ██  ███ ██▄▄▄▄▄   ███      ██ ▀█▄  ██▄▄▄██ ██▄▄▄▄▄ ██▄▄▄▄▄ ▄▄▄▄▄██",
    " ▀▀▀▀▀▀ ▀▀▀▀▀▀▀ ▀▀   ▀▀ ▀▀   ▀▀ ▀▀▀▀▀▀▀   ▀▀▀      ▀▀   ▀▀  ▀▀▀▀▀  ▀▀▀▀▀▀▀ ▀▀▀▀▀▀▀ ▀▀▀▀▀▀",
];

#[derive(Default)]
pub(super) struct Banner {
    cached: Option<(usize, bool, usize, Vec<Line>)>,
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
                compose_with(&WORDMARK, WORDMARK_WIDTH, inset, color, available),
            ));
        }
        &self.cached.as_ref().expect("banner initialized").3
    }
}

#[cfg(test)]
fn compose(art: &[&str], width: usize, inset: usize, color: bool) -> Vec<Line> {
    compose_with(art, width, inset, color, width + 3)
}

fn pixels(ch: char) -> u8 {
    match ch {
        '█' => 0b11,
        '▀' => 0b01,
        '▄' => 0b10,
        _ => 0,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pixel {
    Empty,
    Face,
    Edge,
    Soft,
}

// Query the bitmap after a rightward / downward translation. One y unit is
// half a terminal row; x remains measured in full terminal columns.
fn shifted(art: &[Vec<char>], x: usize, y: usize, dx: usize, dy: usize) -> bool {
    let Some(x) = x.checked_sub(dx + 1) else {
        return false;
    };
    let Some(y) = y.checked_sub(dy) else {
        return false;
    };
    art.get(y / 2)
        .and_then(|row| row.get(x))
        .is_some_and(|ch| pixels(*ch) & (1 << (y % 2)) != 0)
}

fn classify(art: &[Vec<char>], x: usize, y: usize, color: bool) -> Pixel {
    if shifted(art, x, y, 0, 0) {
        Pixel::Face
    } else if !color {
        Pixel::Empty
    } else if shifted(art, x, y, 1, 0) || shifted(art, x, y, 1, 1) {
        Pixel::Edge
    } else if shifted(art, x, y, 2, 1) || shifted(art, x, y, 2, 2) {
        Pixel::Soft
    } else {
        Pixel::Empty
    }
}

// Full and half block glyphs encode both physical pixels in a cell. The
// background carries the second tone where a cell straddles two layers.
fn cell(top: Pixel, bottom: Pixel, gradient: usize) -> (Tone, char) {
    use Pixel::*;
    match (top, bottom) {
        (Empty, Empty) => (Tone::Plain, ' '),
        (Face, Face) => (Tone::Banner(gradient), '█'),
        (Face, Empty) => (Tone::Banner(gradient), '▀'),
        (Empty, Face) => (Tone::Banner(gradient), '▄'),
        (Face, Edge) => (Tone::BannerOnShadow(gradient, true), '▀'),
        (Edge, Face) => (Tone::BannerOnShadow(gradient, true), '▄'),
        (Face, Soft) => (Tone::BannerOnShadow(gradient, false), '▀'),
        (Soft, Face) => (Tone::BannerOnShadow(gradient, false), '▄'),
        (Edge, Edge) => (Tone::Shadow, '█'),
        (Edge, Empty) => (Tone::Shadow, '▀'),
        (Empty, Edge) => (Tone::Shadow, '▄'),
        (Soft, Soft) => (Tone::ShadowSoft, '█'),
        (Soft, Empty) => (Tone::ShadowSoft, '▀'),
        (Empty, Soft) => (Tone::ShadowSoft, '▄'),
        (Edge, Soft) => (Tone::ShadowOverSoft, '▀'),
        (Soft, Edge) => (Tone::SoftOverShadow, '▀'),
    }
}

fn compose_with(
    art: &[&str],
    width: usize,
    inset: usize,
    color: bool,
    available: usize,
) -> Vec<Line> {
    let art: Vec<Vec<char>> = art.iter().map(|row| row.chars().collect()).collect();
    let mut lines = Vec::with_capacity(art.len() + 1);
    for y in 0..=art.len() {
        let mut line = Line::default();
        line.push(Tone::Plain, " ".repeat(inset));
        for x in 0..available.min(width + 3) {
            let gradient = (x.saturating_sub(1) * 4 / width.max(1)).min(3);
            let (tone, glyph) = cell(
                classify(&art, x, y * 2, color),
                classify(&art, x, y * 2 + 1, color),
                gradient,
            );
            line.push(tone, glyph.encode_utf8(&mut [0; 4]));
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
    fn continuous_extrusion_covers_both_half_pixels_at_two_depths() {
        let rows = text(&compose(&["▄", "▀"], 1, 0, true));
        assert_eq!(rows, [" ▄▄ ", " ▀██", "   ▀"]);
        assert!(rows.iter().all(|row| !row.contains(['⠛', '⣤', '⣿', '░'])));
    }

    #[test]
    fn face_and_extrusion_in_one_cell_keep_both_layers() {
        assert_eq!(
            cell(Pixel::Face, Pixel::Edge, 2),
            (Tone::BannerOnShadow(2, true), '▀')
        );
        assert_eq!(
            cell(Pixel::Soft, Pixel::Face, 1),
            (Tone::BannerOnShadow(1, false), '▄')
        );
        assert_eq!(
            cell(Pixel::Edge, Pixel::Soft, 0),
            (Tone::ShadowOverSoft, '▀')
        );
        assert_eq!(
            cell(Pixel::Soft, Pixel::Edge, 0),
            (Tone::SoftOverShadow, '▀')
        );
        let lines = compose(&["▀▄"], 2, 0, true);
        assert!(lines[0].has(Tone::BannerOnShadow(2, true)));
    }

    #[test]
    fn monochrome_omits_both_shadow_layers() {
        let plain = text(&compose(&["███", " ▄ ", " ▀ ", "███"], 3, 0, false));
        assert_eq!(plain, [" ███  ", "  ▄   ", "  ▀   ", " ███  ", "      "]);
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
                    .all(|row| row.width() == FULL_BANNER_WIDTH + 2)
            );
            assert!(
                text(lines)
                    .iter()
                    .all(|row| !row.contains(['⠛', '⣤', '⣿', '░']))
            );
            if !color {
                for line in lines {
                    assert!(!line.has(Tone::Shadow));
                    assert!(!line.has(Tone::ShadowSoft));
                    for index in 0..4 {
                        assert!(!line.has(Tone::BannerOnShadow(index, true)));
                        assert!(!line.has(Tone::BannerOnShadow(index, false)));
                    }
                }
            }
        }
        let first = banner.lines(2, true, FULL_BANNER_WIDTH)[0].clone();
        let second = &banner.lines(2, true, FULL_BANNER_WIDTH)[0];
        assert!(first.shares_storage(second));
        let clipped = &banner.lines(2, true, BANNER_WIDTH)[0];
        assert!(!first.shares_storage(clipped));
    }

    #[test]
    fn all_real_face_cells_keep_their_positions_and_shadow_clips_at_edge() {
        let full = text(&compose_with(
            &WORDMARK,
            WORDMARK_WIDTH,
            0,
            true,
            FULL_BANNER_WIDTH,
        ));
        let clipped = text(&compose_with(
            &WORDMARK,
            WORDMARK_WIDTH,
            0,
            true,
            BANNER_WIDTH,
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
    fn masks_of_i_r_and_s_are_extruded_without_stray_pixels() {
        use std::collections::HashSet;
        for source in [
            vec!["███", " █ ", " █ ", "███"],
            vec!["██▄", "█ █", "██▀", "█ ▀"],
            vec!["▄██", "█  ", " ▀▄", "██▀"],
        ] {
            let art: Vec<Vec<char>> = source.iter().map(|row| row.chars().collect()).collect();
            let mut face = HashSet::new();
            for (row, value) in source.iter().enumerate() {
                for (column, ch) in value.chars().enumerate() {
                    for half in 0..2 {
                        if pixels(ch) & (1 << half) != 0 {
                            face.insert((column + 1, row * 2 + half));
                        }
                    }
                }
            }
            let edge: HashSet<_> = face
                .iter()
                .flat_map(|&(x, y)| [(x + 1, y), (x + 1, y + 1)])
                .collect();
            let soft: HashSet<_> = face
                .iter()
                .flat_map(|&(x, y)| [(x + 2, y + 1), (x + 2, y + 2)])
                .collect();
            for y in 0..(source.len() + 1) * 2 {
                for x in 0..6 {
                    let expected = if face.contains(&(x, y)) {
                        Pixel::Face
                    } else if edge.contains(&(x, y)) {
                        Pixel::Edge
                    } else if soft.contains(&(x, y)) {
                        Pixel::Soft
                    } else {
                        Pixel::Empty
                    };
                    assert_eq!(
                        classify(&art, x, y, true),
                        expected,
                        "x={x} y={y}, source={source:?}"
                    );
                    assert_eq!(
                        classify(&art, x, y, false),
                        if expected == Pixel::Face {
                            Pixel::Face
                        } else {
                            Pixel::Empty
                        }
                    );
                }
            }
        }
    }
}
