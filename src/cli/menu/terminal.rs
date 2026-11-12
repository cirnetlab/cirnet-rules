#[cfg(not(windows))]
use crossterm::event::{DisableBracketedPaste, EnableBracketedPaste};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, queue,
    style::{Attribute, Color, Print, SetAttribute, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    env,
    io::{self, Write},
    ops::Range,
    rc::Rc,
};
use unicode_width::UnicodeWidthStr;

const ORANGE: Color = Color::Rgb {
    r: 211,
    g: 133,
    b: 103,
};

const SHADOW: Color = Color::Rgb {
    r: 57,
    g: 45,
    b: 50,
};

const BANNER_COLORS: [Color; 4] = [
    Color::Rgb {
        r: 225,
        g: 158,
        b: 119,
    },
    Color::Rgb {
        r: 219,
        g: 145,
        b: 110,
    },
    Color::Rgb {
        r: 211,
        g: 133,
        b: 103,
    },
    Color::Rgb {
        r: 200,
        g: 119,
        b: 101,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Tone {
    Plain,
    Muted,
    Heading,
    Accent,
    Shadow,
    Banner(usize),
    Key,
    String,
    Number,
    Literal,
    Punctuation,
    Keyword,
    Option,
    Path,
    Success,
    Warning,
    Error,
    Stderr,
    Match,
}

pub(super) fn color_enabled() -> bool {
    env::var_os("NO_COLOR").is_none() && env::var("TERM").as_deref() != Ok("dumb")
}

#[derive(Clone, Copy)]
enum SemanticDepth {
    Rgb,
    Indexed,
    Basic,
}

impl SemanticDepth {
    fn detect() -> Self {
        let term = env::var("TERM").unwrap_or_default();
        if env::var("COLORTERM").is_ok_and(|value| matches!(value.as_str(), "truecolor" | "24bit"))
            || term.ends_with("-direct")
            || env::var_os("WT_SESSION").is_some_and(|value| !value.is_empty())
        {
            Self::Rgb
        } else if term.contains("256color") {
            Self::Indexed
        } else {
            #[cfg(windows)]
            if term.is_empty() && crossterm::ansi_support::supports_ansi() {
                return Self::Rgb;
            }
            Self::Basic
        }
    }

    fn color(self, rgb: (u8, u8, u8), index: u8, basic: Color) -> Color {
        match self {
            Self::Rgb => Color::Rgb {
                r: rgb.0,
                g: rgb.1,
                b: rgb.2,
            },
            Self::Indexed => Color::AnsiValue(index),
            Self::Basic => basic,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Style {
    color: Option<Color>,
    attribute: Option<Attribute>,
}

impl Style {
    fn for_tone(tone: Tone, depth: SemanticDepth) -> Self {
        use Color::*;
        let color = match tone {
            Tone::Plain => return Self::default(),
            Tone::Muted | Tone::Heading | Tone::Match => {
                return Self {
                    attribute: Some(match tone {
                        Tone::Muted => Attribute::Dim,
                        Tone::Heading => Attribute::Bold,
                        _ => Attribute::Reverse,
                    }),
                    ..Self::default()
                };
            }
            Tone::Accent => ORANGE,
            Tone::Shadow => SHADOW,
            Tone::Banner(index) => BANNER_COLORS[index],
            Tone::Key => depth.color((125, 207, 222), 117, Cyan),
            Tone::String => depth.color((223, 198, 145), 222, DarkYellow),
            Tone::Number => depth.color((238, 170, 117), 216, Yellow),
            Tone::Literal => depth.color((130, 204, 181), 115, Green),
            Tone::Punctuation => depth.color((167, 166, 177), 248, Grey),
            Tone::Keyword => depth.color((202, 170, 234), 183, Magenta),
            Tone::Option => depth.color((231, 159, 126), 216, DarkYellow),
            Tone::Path => depth.color((154, 189, 237), 111, Blue),
            Tone::Success => depth.color((143, 207, 149), 114, Green),
            Tone::Warning => depth.color((242, 199, 105), 221, Yellow),
            Tone::Error => depth.color((244, 137, 130), 210, Red),
            Tone::Stderr => depth.color((178, 176, 187), 249, Grey),
        };
        Self {
            color: Some(color),
            attribute: None,
        }
    }

    fn write(self, out: &mut impl Write, previous: Self) -> io::Result<()> {
        if self.attribute != previous.attribute {
            if let Some(attribute) = previous.attribute {
                let reset = if attribute == Attribute::Reverse {
                    Attribute::NoReverse
                } else {
                    Attribute::NormalIntensity
                };
                queue!(out, SetAttribute(reset))?;
            }
            if let Some(attribute) = self.attribute {
                queue!(out, SetAttribute(attribute))?;
            }
        }
        if self.color != previous.color {
            queue!(out, SetForegroundColor(self.color.unwrap_or(Color::Reset)))?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Line(Option<Rc<LineData>>);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct LineData {
    text: String,
    ends: Vec<(Tone, usize)>,
}

impl Line {
    pub(super) fn push(&mut self, tone: Tone, text: impl AsRef<str>) {
        let text = text.as_ref();
        if text.is_empty() {
            return;
        }
        let data = Rc::make_mut(self.0.get_or_insert_with(Rc::default));
        data.text.push_str(text);
        let end = data.text.len();
        match data.ends.last_mut() {
            Some((last, previous_end)) if *last == tone => *previous_end = end,
            _ => data.ends.push((tone, end)),
        }
    }

    fn contents(&self) -> &str {
        self.0.as_ref().map_or("", |data| data.text.as_str())
    }

    fn segments(&self) -> impl DoubleEndedIterator<Item = (Tone, &str)> {
        self.0.iter().flat_map(|data| {
            data.ends.iter().enumerate().map(|(index, &(tone, end))| {
                let start = index.checked_sub(1).map_or(0, |i| data.ends[i].1);
                (tone, &data.text[start..end])
            })
        })
    }

    fn damage(&self, previous: Option<&Self>) -> Range<usize> {
        let text = self.contents();
        let len = text.len();
        // A byte offset is also a screen column only for ASCII here.
        let Some(previous) = previous.filter(|old| text.is_ascii() && old.contents().is_ascii())
        else {
            return 0..len;
        };
        let start = self
            .segments()
            .zip(previous.segments())
            .take_while(|(new, old)| new == old)
            .map(|((_, text), _)| text.len())
            .sum();
        let end = if previous.contents().len() == len {
            let suffix: usize = self
                .segments()
                .rev()
                .zip(previous.segments().rev())
                .take_while(|(new, old)| new == old)
                .map(|((_, text), _)| text.len())
                .sum();
            len.saturating_sub(suffix).max(start)
        } else {
            len
        };
        start..end
    }

    #[cfg(test)]
    pub(super) fn text(&self) -> String {
        self.contents().to_owned()
    }

    #[cfg(test)]
    pub(super) fn has(&self, tone: Tone) -> bool {
        self.segments().any(|(actual, _)| actual == tone)
    }

    #[cfg(test)]
    pub(super) fn shares_storage(&self, other: &Self) -> bool {
        matches!((&self.0, &other.0), (Some(a), Some(b)) if Rc::ptr_eq(a, b))
    }
}

pub(super) struct MenuRows {
    pub(super) columns: Range<u16>,
    pub(super) rows: Range<u16>,
    pub(super) first: usize,
}

pub(super) struct Frame {
    pub(super) size: (u16, u16),
    pub(super) rows: Vec<Line>,
    pub(super) cursor: Option<(u16, u16)>,
    pub(super) menu: Option<MenuRows>,
}

impl Frame {
    pub(super) fn new(size: (u16, u16)) -> Self {
        Self {
            size,
            cursor: None,
            rows: vec![Line::default(); usize::from(size.1)],
            menu: None,
        }
    }

    pub(super) fn menu_hit(&self, x: u16, y: u16) -> Option<usize> {
        let menu = self.menu.as_ref()?;
        (menu.columns.contains(&x) && menu.rows.contains(&y))
            .then(|| menu.first + usize::from(y - menu.rows.start))
    }

    pub(super) fn changed_rows<'a>(
        &'a self,
        previous: Option<&'a Self>,
    ) -> impl Iterator<Item = (u16, &'a Line)> {
        let previous = previous.filter(|old| old.size == self.size);
        self.rows
            .iter()
            .enumerate()
            .filter(move |(y, line)| match previous {
                Some(old) => &old.rows[*y] != *line,
                None => line.0.is_some(),
            })
            .map(|(y, line)| (y as u16, line))
    }
}

pub(super) struct Painter {
    previous: Option<Frame>,
    depth: SemanticDepth,
}

impl Default for Painter {
    fn default() -> Self {
        Self {
            previous: None,
            depth: SemanticDepth::detect(),
        }
    }
}

impl Painter {
    pub(super) fn menu_hit(&self, size: (u16, u16), x: u16, y: u16) -> Option<usize> {
        self.previous
            .as_ref()
            .filter(|frame| frame.size == size)?
            .menu_hit(x, y)
    }

    pub(super) fn draw(
        &mut self,
        out: &mut impl Write,
        frame: Frame,
        color: bool,
    ) -> io::Result<()> {
        let previous = self.previous.as_ref();
        let reset = previous.is_none_or(|old| old.size != frame.size);
        if reset {
            queue!(out, MoveTo(0, 0), Clear(ClearType::All))?;
        }
        let mut changed = reset;
        let mut active = Style::default();
        for (y, line) in frame.changed_rows(previous) {
            let old = previous
                .filter(|_| !reset)
                .map(|old| &old.rows[usize::from(y)]);
            let dirty = line.damage(old);
            queue!(out, MoveTo(dirty.start as u16, y))?;
            if color {
                let mut offset = 0;
                for (tone, text) in line.segments() {
                    let end = offset + text.len();
                    if offset >= dirty.end {
                        break;
                    }
                    if end > dirty.start {
                        let next = Style::for_tone(tone, self.depth);
                        next.write(out, active)?;
                        queue!(out, Print(text))?;
                        active = next;
                    }
                    offset = end;
                }
            } else {
                queue!(out, Print(&line.contents()[dirty]))?;
            }
            if old.is_some_and(|old| old.contents().width() > line.contents().width()) {
                let plain = Style::default();
                plain.write(out, active)?;
                active = plain;
                queue!(out, Clear(ClearType::UntilNewLine))?;
            }
            changed = true;
        }
        Style::default().write(out, active)?;
        let old_cursor = previous.and_then(|old| old.cursor);
        if let Some((x, y)) = frame.cursor {
            if changed || frame.cursor != old_cursor {
                queue!(out, MoveTo(x, y))?;
            }
            if old_cursor.is_none() {
                queue!(out, Show)?;
            }
        } else if old_cursor.is_some() {
            queue!(out, Hide)?;
        }
        if changed || frame.cursor != old_cursor {
            out.flush()?;
        }
        // A frame can change its hit regions without changing any pixels.
        self.previous = Some(frame);
        Ok(())
    }
}

pub(super) struct Screen<W: Write> {
    pub(super) out: W,
    raw: bool,
    mouse: bool,
    #[cfg(not(windows))]
    paste: bool,
}

impl<W: Write> Screen<W> {
    pub(super) fn enter(out: W) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Self::activate(out, true)
    }

    fn activate(out: W, raw: bool) -> io::Result<Self> {
        let mut screen = Self {
            out,
            raw,
            mouse: false,
            #[cfg(not(windows))]
            paste: false,
        };
        // A flush can fail after a command has already reached the terminal.
        execute!(screen.out, EnterAlternateScreen, Hide)?;
        screen.mouse = true;
        execute!(screen.out, EnableMouseCapture)?;
        #[cfg(not(windows))]
        {
            screen.paste = true;
            execute!(screen.out, EnableBracketedPaste)?;
        }
        Ok(screen)
    }
}

impl<W: Write> Drop for Screen<W> {
    fn drop(&mut self) {
        let _ = execute!(self.out, SetAttribute(Attribute::Reset));
        #[cfg(not(windows))]
        if self.paste {
            let _ = execute!(self.out, DisableBracketedPaste);
        }
        if self.mouse {
            let _ = execute!(self.out, DisableMouseCapture);
        }
        let _ = execute!(self.out, Show);
        let _ = execute!(self.out, LeaveAlternateScreen);
        if self.raw {
            let _ = terminal::disable_raw_mode();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn ran_in_ansi_fixture(test: &str) -> bool {
        const FIXTURE: &str = "CNR_TERMINAL_TEST_ANSI";
        if env::var_os(FIXTURE).is_some() || (!cfg!(windows) && env::var_os("NO_COLOR").is_none()) {
            return false;
        }
        // Crossterm selects ANSI or Win32 once per process.
        let (_, module) = module_path!().split_once("::").unwrap();
        let output = std::process::Command::new(env::current_exe().unwrap())
            .args(["--exact", &format!("{module}::{test}"), "--nocapture"])
            .env("TERM", "xterm-256color")
            .env_remove("NO_COLOR")
            .env(FIXTURE, "1")
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        true
    }

    #[test]
    fn painter_updates_a_changed_row_without_toggling_the_cursor() {
        if ran_in_ansi_fixture("painter_updates_a_changed_row_without_toggling_the_cursor") {
            return;
        }

        let frame = |text| {
            let mut frame = Frame::new((50, 8));
            frame.rows[0].push(Tone::Banner(0), "CIRNET RULES");
            frame.rows[1].push(Tone::Plain, text);
            frame.cursor = Some((2, 5));
            frame
        };
        let mut bytes = Vec::new();
        let mut painter = Painter::default();
        painter.draw(&mut bytes, frame("first"), true).unwrap();
        bytes.clear();
        painter.draw(&mut bytes, frame("newer"), true).unwrap();
        let delta = String::from_utf8_lossy(&bytes);
        assert!(delta.contains("newer"));
        assert!(!delta.contains("CIRNET RULES"));
        assert!(!delta.contains("\x1b[2J"));
        assert!(
            !delta.contains("\x1b[?25h") && !delta.contains("\x1b[?25l"),
            "{delta:?}"
        );
        assert!(!delta.contains("\x1b[K"), "{delta:?}");
        let length = bytes.len();
        painter.draw(&mut bytes, frame("newer"), true).unwrap();
        assert_eq!(bytes.len(), length);
        bytes.clear();
        painter.draw(&mut bytes, frame("new"), true).unwrap();
        assert!(String::from_utf8_lossy(&bytes).contains("new\x1b[K"));
    }

    #[test]
    fn styled_ascii_damage_rewrites_only_the_changed_run() {
        let line = |number| {
            let mut row = Line::default();
            row.push(Tone::Plain, "  ");
            row.push(Tone::Punctuation, "[");
            row.push(Tone::Number, number);
            row.push(Tone::Punctuation, "]");
            row
        };
        let old = line("99");
        assert_eq!(line("88").damage(Some(&old)), 3..5);
        assert_eq!(line("1").damage(Some(&old)), 3..5);
        assert_eq!(line("東京").damage(Some(&old)), 0..10);
        let old_copy = old.clone();
        assert!(old.shares_storage(&old_copy));
    }

    #[test]
    fn same_color_is_emitted_once_across_updated_rows() {
        if ran_in_ansi_fixture("same_color_is_emitted_once_across_updated_rows") {
            return;
        }

        let frame = |number| {
            let mut frame = Frame::new((20, 6));
            for line in &mut frame.rows[..2] {
                line.push(Tone::Punctuation, "[");
                line.push(Tone::Number, number);
                line.push(Tone::Punctuation, "]");
            }
            frame
        };
        let mut painter = Painter {
            previous: None,
            depth: SemanticDepth::Rgb,
        };
        let mut bytes = Vec::new();
        painter.draw(&mut bytes, frame("1"), true).unwrap();
        bytes.clear();
        painter.draw(&mut bytes, frame("2"), true).unwrap();
        let delta = String::from_utf8_lossy(&bytes);
        assert_eq!(
            delta.matches("\x1b[38;2;238;170;117m").count(),
            1,
            "{delta:?}"
        );
        assert!(delta.ends_with("\x1b[39m"));
    }

    struct FailOnce {
        bytes: Rc<RefCell<Vec<u8>>>,
        failed: bool,
    }

    impl Write for FailOnce {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.bytes.borrow_mut().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            if std::mem::replace(&mut self.failed, true) {
                Ok(())
            } else {
                Err(io::Error::other(
                    "first flush fails after entering alternate screen",
                ))
            }
        }
    }

    #[test]
    fn entering_alternate_screen_restores_cursor_and_screen_after_flush_error() {
        if ran_in_ansi_fixture(
            "entering_alternate_screen_restores_cursor_and_screen_after_flush_error",
        ) {
            return;
        }

        let bytes = Rc::new(RefCell::new(Vec::new()));
        let writer = FailOnce {
            bytes: Rc::clone(&bytes),
            failed: false,
        };
        assert!(Screen::activate(writer, false).is_err());
        let bytes = bytes.borrow();
        let trace = String::from_utf8_lossy(&bytes);
        assert!(trace.contains("\x1b[?1049h"));
        assert!(trace.contains("\x1b[?25h"));
        assert!(trace.contains("\x1b[?1049l"));
    }
}
