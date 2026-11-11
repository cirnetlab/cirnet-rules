use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::{Attribute, Color, Print, SetAttribute, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};

const ORANGE: Color = Color::Rgb {
    r: 211,
    g: 133,
    b: 103,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Tone {
    Plain,
    Muted,
    Heading,
    Accent,
}

#[derive(Default, PartialEq, Eq)]
pub(super) struct Line(Vec<(Tone, String)>);

impl Line {
    pub(super) fn push(&mut self, tone: Tone, text: impl Into<String>) {
        let text = text.into();
        if !text.is_empty() {
            self.0.push((tone, text));
        }
    }

    #[cfg(test)]
    pub(super) fn text(&self) -> String {
        self.0.iter().map(|(_, text)| text.as_str()).collect()
    }
}

pub(super) struct Frame {
    pub(super) size: (u16, u16),
    pub(super) rows: Vec<Line>,
    pub(super) cursor: Option<(u16, u16)>,
}

impl Frame {
    pub(super) fn new(size: (u16, u16)) -> Self {
        Self {
            size,
            cursor: None,
            rows: (0..size.1).map(|_| Line::default()).collect(),
        }
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
                None => !line.0.is_empty(),
            })
            .map(|(y, line)| (y as u16, line))
    }
}

#[derive(Default)]
pub(super) struct Painter(Option<Frame>);

impl Painter {
    pub(super) fn draw(
        &mut self,
        out: &mut impl Write,
        frame: Frame,
        color: bool,
    ) -> io::Result<()> {
        let reset = self.0.as_ref().is_none_or(|old| old.size != frame.size);
        if reset {
            queue!(out, MoveTo(0, 0), Clear(ClearType::All))?;
        }
        let mut changed = reset;
        for (y, line) in frame.changed_rows(self.0.as_ref()) {
            queue!(out, MoveTo(0, y))?;
            let mut active = Tone::Plain;
            for (tone, text) in &line.0 {
                if color && *tone != active {
                    if active != Tone::Plain {
                        queue!(out, SetAttribute(Attribute::Reset))?;
                    }
                    match tone {
                        Tone::Plain => {}
                        Tone::Muted => queue!(out, SetAttribute(Attribute::Dim))?,
                        Tone::Heading => queue!(out, SetAttribute(Attribute::Bold))?,
                        Tone::Accent => queue!(out, SetForegroundColor(ORANGE))?,
                    }
                    active = *tone;
                }
                queue!(out, Print(text))?;
            }
            if active != Tone::Plain {
                queue!(out, SetAttribute(Attribute::Reset))?;
            }
            // Erase only stale text after the new content has been written.
            queue!(out, Clear(ClearType::UntilNewLine))?;
            changed = true;
        }
        let old_cursor = self.0.as_ref().and_then(|old| old.cursor);
        if changed || frame.cursor != old_cursor {
            if let Some((x, y)) = frame.cursor {
                queue!(out, MoveTo(x, y), Show)?;
            } else if old_cursor.is_some() {
                queue!(out, Hide)?;
            }
            out.flush()?;
            self.0 = Some(frame);
        }
        Ok(())
    }
}

pub(super) struct Screen;

impl Screen {
    pub(super) fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let screen = Self;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(screen)
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let _ = execute!(
            io::stdout(),
            SetAttribute(Attribute::Reset),
            Show,
            LeaveAlternateScreen
        );
        let _ = terminal::disable_raw_mode();
    }
}
