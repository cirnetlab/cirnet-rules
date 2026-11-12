use crate::{docs, workspace};
use pulldown_cmark::{Event, Parser};
use std::ops::Range;

const BEGIN: &str = "<!-- CIRNETRULES:BEGIN managed-context -->";
const END: &str = "<!-- CIRNETRULES:END managed-context -->";
const RESERVED: &str = "CIRNETRULES:";

pub(super) fn is_entry(path: &str) -> bool {
    matches!(path, "AGENTS.md" | "CLAUDE.md")
}

pub(super) fn user_text(bytes: &[u8]) -> Result<&str, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "Managed entry requires UTF-8 text")?;
    if text
        .chars()
        .any(|ch| ch.is_control() && !matches!(ch, '\r' | '\n' | '\t'))
        || workspace::contains_known_secrets(text)
    {
        return Err("Managed entry contains unsafe text or possible credentials".into());
    }
    Ok(text)
}

fn block(text: &str) -> Result<Option<Range<usize>>, String> {
    let mut start = None;
    let mut found = None;
    let bom = text.len() - text.trim_start_matches('\u{feff}').len();
    let mut offset = bom;
    for line in text[bom..].split_inclusive('\n') {
        match line.trim_end_matches(['\r', '\n']) {
            BEGIN if start.is_none() && found.is_none() => start = Some(offset),
            END if start.is_some() => {
                found = Some(start.take().expect("managed marker start")..offset + line.len());
            }
            value if value.contains(RESERVED) => {
                return Err("Managed entry markers are malformed, nested or duplicated".into());
            }
            _ => {}
        }
        offset += line.len();
    }
    if start.is_some() {
        return Err("Managed entry block has no end marker".into());
    }
    Ok(found)
}

pub(super) fn verify(text: &str, expected: Option<&str>) -> Result<Option<Range<usize>>, String> {
    let found = block(text)?;
    let actual = found
        .as_ref()
        .map(|span| docs::sha256(&text.as_bytes()[span.clone()]));
    if actual.as_deref() != expected {
        return Err(if expected.is_some() {
            "Managed entry block changed or is missing. Review local edits and the latest receipt."
        } else {
            "Entry contains a managed marker without matching ownership. Review it before adoption."
        }
        .into());
    }
    Ok(found)
}

fn line_ending(text: &str) -> &str {
    if text
        .find('\n')
        .is_some_and(|index| text.as_bytes()[..index].ends_with(b"\r"))
    {
        "\r\n"
    } else {
        "\n"
    }
}

pub(super) fn imports(text: &str, name: &str) -> bool {
    text.split_whitespace().any(|part| {
        part.trim_matches(['`', '\'', '"', '(', ')', ',', ';'])
            .strip_prefix('@')
            .and_then(|path| path.rsplit(['/', '\\']).next())
            .is_some_and(|path| path.eq_ignore_ascii_case(name))
    })
}

pub(super) struct Replacement {
    pub(super) bytes: Vec<u8>,
    pub(super) change: super::BlockChange,
    pub(super) patch: String,
    pub(super) previous_patch: Option<String>,
    pub(super) separator: Option<String>,
}

pub(super) fn reconcile(
    current: &str,
    desired: &str,
    owned: Option<&str>,
    full_file_owned: bool,
) -> Result<Replacement, String> {
    let span = verify(current, owned)?;
    if desired.contains(RESERVED) {
        return Err("Generated entry contains a reserved managed marker".into());
    }
    let eol = line_ending(
        span.as_ref()
            .map_or(current, |range| &current[range.clone()]),
    );
    let body = desired.replace("\r\n", "\n").replace('\n', eol);
    let patch = format!(
        "{BEGIN}{eol}{}{eol}{END}{eol}",
        body.trim_end_matches(['\r', '\n'])
    );
    let previous_patch = span
        .as_ref()
        .map(|range| &current[range.clone()])
        .or_else(|| full_file_owned.then_some(current))
        .and_then(|text| {
            super::super::render::guarded(text.as_bytes())
                .ok()
                .map(str::to_owned)
        });
    let (rendered, separator) = if let Some(span) = span {
        (
            format!("{}{patch}{}", &current[..span.start], &current[span.end..]),
            None,
        )
    } else if full_file_owned {
        (patch.clone(), None)
    } else {
        let separator = if current.trim_start_matches('\u{feff}').is_empty() {
            String::new()
        } else if current.ends_with('\n') {
            eol.into()
        } else {
            eol.repeat(2)
        };
        (format!("{current}{separator}{patch}"), Some(separator))
    };
    if rendered.len() as u64 > super::paths::MAX_FILE {
        return Err("Managed entry exceeds its file size limit".into());
    }
    let markers = Parser::new(&rendered)
        .filter_map(|event| match event {
            Event::Html(value) => match value.trim_end_matches(['\r', '\n']) {
                BEGIN => Some(BEGIN),
                END => Some(END),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();
    if markers != [BEGIN, END] {
        return Err("Managed markers would be hidden by the existing Markdown structure".into());
    }
    Ok(Replacement {
        bytes: rendered.into_bytes(),
        change: super::BlockChange {
            before: owned.map(str::to_owned),
            after: docs::sha256(patch.as_bytes()),
        },
        patch,
        previous_patch,
        separator,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_markers_are_never_adopted_implicitly() {
        for text in [
            format!("{BEGIN}\nuser content\n{END}\n"),
            format!("{BEGIN}\n{BEGIN}\n{END}\n"),
            format!("{BEGIN}\n{END}\n{BEGIN}\n{END}\n"),
            format!("{END}\n"),
            format!("`{BEGIN}`\n"),
        ] {
            assert!(reconcile(&text, "Managed instructions\n", None, false).is_err());
        }
    }

    #[test]
    fn an_open_markdown_fence_blocks_adoption() {
        assert!(reconcile("```text\nunclosed", "Managed instructions\n", None, false).is_err());
    }
}
