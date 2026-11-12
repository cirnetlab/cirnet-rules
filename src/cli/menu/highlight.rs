use super::terminal::Tone;
use serde::de::IgnoredAny;
use std::{borrow::Cow, ops::Range};

// Keep dense punctuation and a large captured stream from growing the style index indefinitely.
const MAX_SPANS: usize = 64 * 1024;

pub(super) struct Span {
    pub(super) range: Range<usize>,
    pub(super) tone: Tone,
}

pub(super) fn highlight(input: &str, stderr: bool) -> Vec<Span> {
    let mut spans = Vec::new();
    if !stderr
        && let Some((header, body)) = input.split_once('\n')
        && let Some((command, status)) = report_header(header)
        && valid_json(body)
    {
        let command_at = "CNR · ".len();
        push(&mut spans, 0..3, Tone::Keyword);
        push(
            &mut spans,
            command_at..command_at + command.len(),
            Tone::Keyword,
        );
        let header_status = spans.len();
        push(
            &mut spans,
            header.len() - status.len()..header.len(),
            Tone::Keyword,
        );
        if let Some(record) = json_spans(body, header.len() + 1, &mut spans)
            && record.label == status
        {
            spans[header_status].tone = record.tone;
            spans[record.index].tone = record.tone;
        }
        return spans;
    }
    if valid_json(input) {
        json_record(input, 0, !stderr, &mut spans);
        return spans;
    }

    let mut offset = 0;
    let mut fence: Option<Fence> = None;
    for part in input.split_inclusive('\n') {
        let line = part.strip_suffix('\n').unwrap_or(part);
        let trimmed = line.trim_start();
        let start = offset + line.len() - trimmed.len();
        if let Some(open) = &mut fence {
            if fence_marker(trimmed).is_some_and(|(marker, count, rest)| {
                marker == open.marker && count >= open.count && rest.is_empty()
            }) {
                if open.language == Some(Language::Json) {
                    fenced_json(input, open.start..offset, &mut spans);
                }
                push(&mut spans, start..offset + line.len(), Tone::Punctuation);
                fence = None;
            } else if let Some(language) = open.language {
                match language {
                    Language::Config => {
                        config_line(trimmed, start, &mut spans);
                    }
                    Language::Json => {}
                    _ => code_line(line, offset, language, &mut open.code, &mut spans),
                }
            }
        } else if let Some((marker, count, info)) = fence_marker(trimmed)
            && (marker != b'\x60' || !info.contains('\x60'))
        {
            push(&mut spans, start..offset + line.len(), Tone::Punctuation);
            fence = Some(Fence {
                marker,
                count,
                start: offset + part.len(),
                language: info
                    .split_ascii_whitespace()
                    .next()
                    .and_then(Language::parse),
                code: CodeState::default(),
            });
        } else if valid_json(trimmed) {
            json_record(trimmed, start, !stderr, &mut spans);
        } else {
            plain_line(trimmed, start, stderr, &mut spans);
        }
        offset += part.len();
        if spans.len() >= MAX_SPANS {
            break;
        }
    }
    if let Some(open) = fence.filter(|open| open.language == Some(Language::Json)) {
        fenced_json(input, open.start..offset, &mut spans);
    }
    spans
}

fn push(spans: &mut Vec<Span>, range: Range<usize>, tone: Tone) {
    if !range.is_empty() && spans.len() < MAX_SPANS {
        spans.push(Span { range, tone });
    }
}

fn report_header(line: &str) -> Option<(&str, &str)> {
    let (command, status) = line.strip_prefix("CNR · ")?.split_once(" · ")?;
    (!command.is_empty()
        && command
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        && status_tone(status).is_some())
    .then_some((command, status))
}

fn valid_json(input: &str) -> bool {
    let input = input.trim_matches([' ', '\t', '\n', '\r']);
    if !matches!(
        input.as_bytes().first(),
        Some(b'{' | b'[' | b'"' | b'-' | b'0'..=b'9' | b't' | b'f' | b'n')
    ) {
        return false;
    }
    // IgnoredAny validates the structure without materializing a JSON tree.
    // It skips Unicode surrogate checks, which matter when the text uses escapes.
    if input.contains("\\u") {
        serde_json::from_str::<serde_json::Value>(input).is_ok()
    } else {
        serde_json::from_str::<IgnoredAny>(input).is_ok()
    }
}

fn fenced_json(input: &str, range: Range<usize>, spans: &mut Vec<Span>) {
    let body = &input[range.clone()];
    if valid_json(body) {
        json_record(body, range.start, false, spans);
    }
}

fn json_record(input: &str, offset: usize, report_status: bool, spans: &mut Vec<Span>) {
    if let Some(record) = json_spans(input, offset, spans)
        && report_status
    {
        spans[record.index].tone = record.tone;
    }
}

struct Status {
    index: usize,
    label: &'static str,
    tone: Tone,
}

fn status_tone(text: &str) -> Option<(&'static str, Tone)> {
    Some(match text {
        "PASS" => ("PASS", Tone::Success),
        "SUCCESS" => ("SUCCESS", Tone::Success),
        "FAIL" => ("FAIL", Tone::Error),
        "FAILED" => ("FAILED", Tone::Error),
        "ERROR" => ("ERROR", Tone::Error),
        "BLOCKED" => ("BLOCKED", Tone::Warning),
        "NOT_RUN" => ("NOT_RUN", Tone::Warning),
        "WARNING" => ("WARNING", Tone::Warning),
        "CANCELLED" => ("CANCELLED", Tone::Warning),
        _ => return None,
    })
}

fn unquoted(token: &str) -> Cow<'_, str> {
    let inner = &token[1..token.len() - 1];
    if inner.contains('\\') {
        serde_json::from_str::<String>(token).map_or(Cow::Borrowed(inner), Cow::Owned)
    } else {
        Cow::Borrowed(inner)
    }
}

// The input passed here has already been validated. Byte boundaries for tokens
// come from ASCII delimiters, so ranges also cover Unicode values losslessly.
fn json_spans(input: &str, offset: usize, spans: &mut Vec<Span>) -> Option<Status> {
    let root_object = input.trim_start().starts_with('{');
    let bytes = input.as_bytes();
    let mut cursor = 0;
    let mut depth = 0;
    let mut status_keys = 0;
    let mut status_pending = false;
    let mut status = None;
    while cursor < bytes.len() && spans.len() < MAX_SPANS {
        let start = cursor;
        let tone = match bytes[cursor] {
            b' ' | b'\t' | b'\n' | b'\r' => {
                cursor += 1;
                continue;
            }
            b'"' => {
                cursor = quote_end(bytes, cursor, b'"')?;
                let token = &input[start..cursor];
                let inner = &token[1..token.len() - 1];
                if input[cursor..].trim_start().starts_with(':') {
                    status_pending = root_object && depth == 1 && unquoted(token) == "status";
                    status_keys += usize::from(status_pending);
                    Tone::Key
                } else {
                    if status_pending {
                        status = status_tone(&unquoted(token)).map(|(label, tone)| Status {
                            index: spans.len(),
                            label,
                            tone,
                        });
                    }
                    status_pending = false;
                    string_tone(inner)
                }
            }
            b'{' | b'[' => {
                cursor += 1;
                depth += 1;
                status_pending = false;
                Tone::Punctuation
            }
            b'}' | b']' => {
                cursor += 1;
                depth -= 1;
                status_pending = false;
                Tone::Punctuation
            }
            b':' | b',' => {
                status_pending &= bytes[cursor] == b':';
                cursor += 1;
                Tone::Punctuation
            }
            b't' | b'f' | b'n' => {
                cursor += if bytes[cursor] == b'f' { 5 } else { 4 };
                status_pending = false;
                Tone::Literal
            }
            _ => {
                cursor += 1;
                while bytes.get(cursor).is_some_and(|byte| {
                    byte.is_ascii_digit() || matches!(*byte, b'.' | b'e' | b'E' | b'+' | b'-')
                }) {
                    cursor += 1;
                }
                status_pending = false;
                Tone::Number
            }
        };
        push(spans, offset + start..offset + cursor, tone);
    }
    (cursor == bytes.len() && status_keys == 1)
        .then_some(status)
        .flatten()
}

fn looks_like_path(text: &str) -> bool {
    text.starts_with('/')
        || text.starts_with(r"\\")
        || text.starts_with("./")
        || text.starts_with("../")
        || text.starts_with(r".\")
        || text.starts_with(r"..\")
        || text.as_bytes().get(..3).is_some_and(|first| {
            first[0].is_ascii_alphabetic() && first[1] == b':' && matches!(first[2], b'\\' | b'/')
        })
}

fn option(text: &str) -> bool {
    text.strip_prefix("--").is_some_and(|flag| {
        flag.as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
            && flag
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    })
}

fn string_tone(text: &str) -> Tone {
    if looks_like_path(text) {
        Tone::Path
    } else if option(text) {
        Tone::Option
    } else {
        Tone::String
    }
}

fn quote_end(bytes: &[u8], start: usize, delimiter: u8) -> Option<usize> {
    let mut cursor = start + 1;
    while let Some(&byte) = bytes.get(cursor) {
        cursor += 1;
        if byte == delimiter {
            return Some(cursor);
        }
        if byte == b'\\' {
            cursor += 1;
        }
    }
    None
}

fn plain_line(line: &str, offset: usize, stderr: bool, spans: &mut Vec<Span>) {
    if let Some((range, tone)) = severity(line) {
        push(spans, offset + range.start..offset + range.end, tone);
        if stderr {
            push(spans, offset + range.end..offset + line.len(), Tone::Stderr);
        }
    } else if command_line(line, offset, spans) || config_line(line, offset, spans) {
        // The format handlers supplied the recognized spans.
    } else if line.starts_with('|') && line.ends_with('|') {
        for (index, _) in line.match_indices('|') {
            push(spans, offset + index..offset + index + 1, Tone::Punctuation);
        }
    } else if line.starts_with('#') && line.trim_start_matches('#').starts_with(' ') {
        push(spans, offset..offset + line.len(), Tone::Keyword);
    } else if let Some((start, rest)) = line.split_once('\x60')
        && let Some((code, _)) = rest.split_once('\x60')
    {
        let begin = offset + start.len();
        push(spans, begin..begin + code.len() + 2, Tone::String);
    } else if stderr {
        push(spans, offset..offset + line.len(), Tone::Stderr);
    }
}

fn severity(line: &str) -> Option<(Range<usize>, Tone)> {
    let bracket = usize::from(line.starts_with('['));
    let word = &line[bracket..];
    let end = word
        .find(|ch: char| !ch.is_ascii_alphabetic())
        .unwrap_or(word.len());
    let name = &word[..end];
    let suffix = &word[end..];
    let tone = if ["error", "fatal"]
        .iter()
        .any(|level| name.eq_ignore_ascii_case(level))
    {
        Tone::Error
    } else if ["warning", "warn"]
        .iter()
        .any(|level| name.eq_ignore_ascii_case(level))
    {
        Tone::Warning
    } else if ["info", "debug", "trace"]
        .iter()
        .any(|level| name.eq_ignore_ascii_case(level))
    {
        Tone::Stderr
    } else {
        return None;
    };
    let delimited = if bracket == 1 {
        suffix.starts_with(']')
    } else {
        suffix.starts_with(':')
            || suffix.starts_with("[E") && suffix.contains("]:")
            || !suffix.is_empty()
                && name.bytes().all(|byte| byte.is_ascii_uppercase())
                && suffix.starts_with(char::is_whitespace)
    };
    delimited.then_some((bracket..bracket + end, tone))
}

fn command_line(line: &str, offset: usize, spans: &mut Vec<Span>) -> bool {
    let command = line
        .strip_prefix("$ ")
        .or_else(|| line.strip_prefix("> "))
        .unwrap_or(line);
    let Some(_) = command
        .strip_prefix("cnr")
        .filter(|rest| rest.starts_with(char::is_whitespace))
    else {
        return false;
    };
    let at = offset + line.len() - command.len();
    push(spans, at..at + 3, Tone::Keyword);
    let mut cursor = 3;
    let mut action = true;
    let mut options = true;
    while cursor < command.len() {
        let tail = command[cursor..].trim_start();
        cursor = command.len() - tail.len();
        let Some(first) = tail.as_bytes().first().copied() else {
            break;
        };
        let (length, tone) = if matches!(first, b'"' | b'\'') {
            let Some(end) = quote_end(tail.as_bytes(), 0, first) else {
                break;
            };
            (end, string_tone(&tail[1..end - 1]))
        } else {
            let end = tail.find(char::is_whitespace).unwrap_or(tail.len());
            let token = &tail[..end];
            let tone = if token == "--" {
                options = false;
                Tone::Punctuation
            } else if options && option(token) {
                Tone::Option
            } else if action
                && options
                && token
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
            {
                action = false;
                Tone::Keyword
            } else if looks_like_path(token) {
                Tone::Path
            } else {
                cursor += end;
                continue;
            };
            (end, tone)
        };
        push(spans, at + cursor..at + cursor + length, tone);
        cursor += length;
    }
    true
}

fn config_line(line: &str, offset: usize, spans: &mut Vec<Span>) -> bool {
    if let Some(section) = line
        .strip_prefix('[')
        .and_then(|tail| tail.strip_suffix(']'))
        && bare_key(section)
    {
        push(spans, offset + 1..offset + line.len() - 1, Tone::Key);
        return true;
    }
    let content = line.strip_prefix("- ").unwrap_or(line).trim_start();
    let start = offset + line.len() - content.len();
    let Some(at) = content.find([':', '=']) else {
        return false;
    };
    let key = content[..at].trim_end();
    let remainder = &content[at + 1..];
    if (content.as_bytes()[at] == b':'
        && !remainder.is_empty()
        && !remainder.starts_with(char::is_whitespace))
        || !(bare_key(key)
            || key.starts_with('"') && quote_end(key.as_bytes(), 0, b'"') == Some(key.len())
            || key.starts_with('\'') && quote_end(key.as_bytes(), 0, b'\'') == Some(key.len()))
    {
        return false;
    }
    push(spans, start..start + key.len(), Tone::Key);
    push(spans, start + at..start + at + 1, Tone::Punctuation);
    let value = remainder.trim_start();
    let value_at = start + content.len() - value.len();
    if let Some(delimiter @ (b'"' | b'\'')) = value.as_bytes().first().copied()
        && let Some(end) = quote_end(value.as_bytes(), 0, delimiter)
    {
        push(
            spans,
            value_at..value_at + end,
            string_tone(&value[1..end - 1]),
        );
    } else {
        let value = value.split(" #").next().unwrap_or(value).trim_end();
        if valid_json(value) {
            json_record(value, value_at, false, spans);
        } else if looks_like_path(value) {
            push(spans, value_at..value_at + value.len(), Tone::Path);
        }
    }
    true
}

fn bare_key(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '-' | '.'))
}

struct Fence {
    marker: u8,
    count: usize,
    start: usize,
    language: Option<Language>,
    code: CodeState,
}

fn fence_marker(line: &str) -> Option<(u8, usize, &str)> {
    let marker = *line.as_bytes().first()?;
    if !matches!(marker, b'\x60' | b'~') {
        return None;
    }
    let count = line.bytes().take_while(|byte| *byte == marker).count();
    (count >= 3).then_some((marker, count, line[count..].trim()))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Language {
    Rust,
    C,
    Cpp,
    Python,
    Js,
    Ts,
    Shell,
    Json,
    Config,
}

impl Language {
    fn parse(info: &str) -> Option<Self> {
        Some(match info {
            "rust" | "rs" => Self::Rust,
            "c" => Self::C,
            "cpp" | "c++" | "cxx" => Self::Cpp,
            "python" | "py" => Self::Python,
            "javascript" | "js" | "jsx" => Self::Js,
            "typescript" | "ts" | "tsx" => Self::Ts,
            "sh" | "bash" | "zsh" | "shell" | "powershell" | "pwsh" => Self::Shell,
            "json" => Self::Json,
            "yaml" | "yml" | "toml" => Self::Config,
            _ => return None,
        })
    }

    fn slash_comments(self) -> bool {
        matches!(self, Self::Rust | Self::C | Self::Cpp | Self::Js | Self::Ts)
    }

    fn word_tone(self, word: &str) -> Option<Tone> {
        if matches!(
            (self, word),
            (Self::Python, "True" | "False" | "None")
                | (Self::Js | Self::Ts, "true" | "false" | "null" | "undefined")
                | (
                    Self::Rust | Self::C | Self::Cpp,
                    "true" | "false" | "nullptr" | "NULL"
                )
        ) {
            return Some(Tone::Literal);
        }
        let common = matches!(
            word,
            "if" | "else" | "for" | "while" | "return" | "break" | "continue"
        );
        let keyword = common
            || match self {
                Self::Rust => matches!(
                    word,
                    "fn" | "let"
                        | "mut"
                        | "const"
                        | "static"
                        | "pub"
                        | "impl"
                        | "trait"
                        | "struct"
                        | "enum"
                        | "match"
                        | "loop"
                        | "use"
                        | "mod"
                        | "crate"
                        | "self"
                        | "super"
                        | "async"
                        | "await"
                        | "unsafe"
                        | "where"
                        | "type"
                        | "dyn"
                ),
                Self::C | Self::Cpp => {
                    matches!(
                        word,
                        "typedef"
                            | "struct"
                            | "union"
                            | "enum"
                            | "static"
                            | "const"
                            | "void"
                            | "char"
                            | "int"
                            | "double"
                            | "float"
                            | "unsigned"
                            | "long"
                            | "short"
                            | "switch"
                            | "case"
                            | "default"
                            | "sizeof"
                            | "include"
                    ) || self == Self::Cpp
                        && matches!(
                            word,
                            "class"
                                | "namespace"
                                | "template"
                                | "typename"
                                | "auto"
                                | "constexpr"
                                | "public"
                                | "private"
                                | "protected"
                                | "new"
                                | "delete"
                                | "using"
                        )
                }
                Self::Python => matches!(
                    word,
                    "def"
                        | "class"
                        | "import"
                        | "from"
                        | "as"
                        | "elif"
                        | "try"
                        | "except"
                        | "finally"
                        | "with"
                        | "yield"
                        | "async"
                        | "await"
                        | "lambda"
                        | "in"
                        | "is"
                        | "and"
                        | "or"
                        | "not"
                        | "pass"
                        | "raise"
                ),
                Self::Js | Self::Ts => {
                    matches!(
                        word,
                        "let"
                            | "const"
                            | "var"
                            | "function"
                            | "async"
                            | "await"
                            | "class"
                            | "new"
                            | "import"
                            | "export"
                            | "from"
                            | "default"
                            | "throw"
                            | "try"
                            | "catch"
                            | "finally"
                    ) || self == Self::Ts
                        && matches!(
                            word,
                            "interface"
                                | "type"
                                | "enum"
                                | "implements"
                                | "extends"
                                | "readonly"
                                | "public"
                                | "private"
                                | "string"
                                | "number"
                                | "boolean"
                        )
                }
                Self::Shell => matches!(
                    word,
                    "then"
                        | "fi"
                        | "do"
                        | "done"
                        | "case"
                        | "esac"
                        | "in"
                        | "function"
                        | "export"
                        | "local"
                        | "set"
                ),
                Self::Config | Self::Json => false,
            };
        keyword.then_some(Tone::Keyword)
    }
}

#[derive(Default)]
struct CodeState {
    comment: usize,
    quote: Option<&'static str>,
}

fn code_line(
    line: &str,
    offset: usize,
    language: Language,
    state: &mut CodeState,
    spans: &mut Vec<Span>,
) {
    let bytes = line.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() && spans.len() < MAX_SPANS {
        let start = cursor;
        let tail = &line[cursor..];
        let first = bytes[cursor];
        let tone = if state.comment != 0 || language.slash_comments() && tail.starts_with("/*") {
            if state.comment == 0 {
                cursor += 2;
                state.comment = 1;
            }
            cursor = comment_end(
                bytes,
                cursor,
                language == Language::Rust,
                &mut state.comment,
            );
            Tone::Stderr
        } else if let Some(quote) = state.quote {
            if let Some(end) = tail.find(quote) {
                cursor += end + quote.len();
                state.quote = None;
            } else {
                cursor = bytes.len();
            }
            Tone::String
        } else if language.slash_comments() && tail.starts_with("//")
            || language == Language::Python && first == b'#'
            || language == Language::Shell
                && first == b'#'
                && (cursor == 0 || bytes[cursor - 1].is_ascii_whitespace())
        {
            cursor = bytes.len();
            Tone::Stderr
        } else if language == Language::Python
            && (tail.starts_with("\"\"\"") || tail.starts_with("'''"))
        {
            let quote = if first == b'"' { "\"\"\"" } else { "'''" };
            cursor += quote.len();
            if let Some(end) = line[cursor..].find(quote) {
                cursor += end + quote.len();
            } else {
                state.quote = Some(quote);
                cursor = bytes.len();
            }
            Tone::String
        } else if matches!(first, b'"' | b'\'' | b'\x60')
            && let Some(end) = code_quote_end(line, cursor, first, language)
        {
            cursor = end;
            Tone::String
        } else if first.is_ascii_digit() {
            cursor += 1;
            while bytes
                .get(cursor)
                .is_some_and(|byte| code_word(*byte) || *byte == b'.')
            {
                cursor += 1;
            }
            Tone::Number
        } else if code_word(first) {
            cursor += 1;
            while bytes.get(cursor).is_some_and(|byte| code_word(*byte)) {
                cursor += 1;
            }
            let Some(tone) = language.word_tone(&line[start..cursor]) else {
                continue;
            };
            tone
        } else {
            cursor += tail.chars().next().unwrap().len_utf8();
            continue;
        };
        push(spans, offset + start..offset + cursor, tone);
    }
}

fn code_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || !byte.is_ascii()
}

fn code_quote_end(line: &str, start: usize, delimiter: u8, language: Language) -> Option<usize> {
    if language != Language::Rust || delimiter != b'\'' {
        return quote_end(line.as_bytes(), start, delimiter);
    }

    // In Rust an apostrophe can start a lifetime. Only style a character
    // literal when its closing delimiter follows one scalar or a short escape.
    let first = line[start + 1..].chars().next()?;
    if first == '\\' {
        return quote_end(
            &line.as_bytes()[..line.len().min(start + 16)],
            start,
            delimiter,
        );
    }
    let end = start + 1 + first.len_utf8();
    (first != '\'' && line.as_bytes().get(end) == Some(&delimiter)).then_some(end + 1)
}

fn comment_end(bytes: &[u8], mut cursor: usize, nested: bool, depth: &mut usize) -> usize {
    while let Some(pair) = bytes.get(cursor..cursor + 2) {
        match pair {
            b"*/" => {
                cursor += 2;
                *depth -= 1;
                if *depth == 0 {
                    return cursor;
                }
            }
            b"/*" if nested => {
                *depth += 1;
                cursor += 2;
            }
            _ => cursor += 1,
        }
    }
    bytes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(input: &str, stderr: bool, expected: &[(&str, Tone)]) {
        let spans = highlight(input, stderr);
        let mut end = 0;
        let actual: Vec<_> = spans
            .iter()
            .map(|span| {
                assert!(span.range.start >= end);
                assert!(span.range.start < span.range.end);
                end = span.range.end;
                (&input[span.range.clone()], span.tone)
            })
            .collect();
        for item in expected {
            assert!(actual.contains(item), "missing {item:?} in {actual:?}");
        }
    }

    #[test]
    fn highlights_multiline_json_and_only_the_root_status() {
        let input = r#"CNR · config · PASS
{
  "project": "E:\\Projects\\Việt",
  "nested": {"status": "PASS", "quote": "\"hello\""},
  "data": [12.5, true, null, false],
  "status": "PASS"
}"#;
        check(
            input,
            false,
            &[
                (r#""project""#, Tone::Key),
                (r#""E:\\Projects\\Việt""#, Tone::Path),
                (r#""\"hello\"""#, Tone::String),
                ("12.5", Tone::Number),
                ("true", Tone::Literal),
                ("null", Tone::Literal),
                ("false", Tone::Literal),
                ("PASS", Tone::Success),
                (r#""PASS""#, Tone::Success),
                (r#""PASS""#, Tone::String),
            ],
        );
        assert_eq!(
            highlight(input, false)
                .iter()
                .filter(|span| span.tone == Tone::Success)
                .count(),
            2
        );
    }

    #[test]
    fn rejects_unreliable_statuses() {
        for input in [
            r#"{"detail":"PASS"}"#,
            r#"{"status":"PASS","statu\u0073":"FAIL"}"#,
            r#"{"status":"PASS","broken":}"#,
            "CNR · config · PASS\n{\"status\":\"FAIL\"}",
            "PASS is a string in this message",
            "~~~json\n{\"status\":\"PASS\"}\n~~~",
        ] {
            assert!(
                !highlight(input, false)
                    .iter()
                    .any(|span| span.tone == Tone::Success),
                "{input:?}"
            );
        }
        assert!(
            !highlight(r#"{"status":"PASS"}"#, true)
                .iter()
                .any(|span| span.tone == Tone::Success)
        );
    }

    #[test]
    fn distinguishes_logs_config_cli_and_code_fences() {
        let input = "error[E0001]: compiler detail\n[WARN] retry\n\
            tools: true\nversion = \"1.99.1\"\n\
            cnr list --json\n\
            ~~~rust\nfn main() { let n = 12; }\n~~~\n\
            ~~~python\ndef run():\n    return 'text'\n~~~";
        check(
            input,
            true,
            &[
                ("error", Tone::Error),
                ("WARN", Tone::Warning),
                ("tools", Tone::Key),
                ("true", Tone::Literal),
                ("version", Tone::Key),
                (r#""1.99.1""#, Tone::String),
                ("cnr", Tone::Keyword),
                ("--json", Tone::Option),
                ("fn", Tone::Keyword),
                ("12", Tone::Number),
                ("def", Tone::Keyword),
                ("'text'", Tone::String),
            ],
        );
    }

    #[test]
    fn handles_ten_thousand_ndjson_records_without_losing_styles() {
        let input = "[12]\n".repeat(10_000);
        let spans = highlight(&input, false);
        let numbers = spans.iter().filter(|span| span.tone == Tone::Number);
        assert_eq!(numbers.count(), 10_000);
        assert!(
            spans
                .windows(2)
                .all(|pair| pair[0].range.end <= pair[1].range.start)
        );
        assert_eq!(&input[spans.last().unwrap().range.clone()], "]");
    }

    #[test]
    fn rust_lifetimes_do_not_consume_surrounding_code_as_a_string() {
        let input = "~~~rust\nfn borrow<'a>(text: &'a str) { let marker = 'ệ'; }\n~~~";
        let strings: Vec<_> = highlight(input, false)
            .into_iter()
            .filter(|span| span.tone == Tone::String)
            .map(|span| &input[span.range])
            .collect();
        assert_eq!(strings, ["'ệ'"]);
    }
}
