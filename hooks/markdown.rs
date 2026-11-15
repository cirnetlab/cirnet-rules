use std::collections::{HashMap, HashSet};

fn unindent(line: &str) -> Option<&str> {
    let content = line.trim_start_matches(' ');
    (line.len() - content.len() <= 3).then_some(content)
}

fn scan_fences(text: &str, mut visit: impl FnMut(&str, bool)) -> bool {
    let mut open_fence: Option<(char, usize)> = None;
    for line in text.split_inclusive('\n') {
        let marker = unindent(line).and_then(|content| {
            let mark = content
                .starts_with('`')
                .then_some('`')
                .or_else(|| content.starts_with('~').then_some('~'))?;
            let length = content.len() - content.trim_start_matches(mark).len();
            (length >= 3).then_some((mark, length, &content[length..]))
        });
        if let Some((open_mark, open_length)) = open_fence {
            if let Some((mark, length, rest)) = marker {
                if mark == open_mark && length >= open_length && rest.trim().is_empty() {
                    open_fence = None;
                }
            }
        } else if let Some((mark, length, _)) = marker {
            open_fence = Some((mark, length));
        } else {
            visit(line, true);
            continue;
        }
        visit(line, false);
    }
    open_fence.is_some()
}

pub(super) fn ends_in_fence(text: &str) -> bool {
    scan_fences(text, |_, _| {})
}

fn without_fences(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    scan_fences(text, |line, keep| {
        if keep {
            output.push_str(line);
        } else {
            output.push('\n');
        }
    });
    output
}

fn preserve_newlines(output: &mut String, removed: &str) {
    for byte in removed.bytes() {
        if byte == b'\n' {
            output.push('\n');
        }
    }
}

fn closing_ticks(mut text: &str, length: usize) -> Option<usize> {
    let total = text.len();
    while let Some(index) = text.find('`') {
        let run = &text[index..];
        let rest = run.trim_start_matches('`');
        if run.len() - rest.len() == length {
            return Some(total - run.len());
        }
        text = rest;
    }
    None
}

pub(super) fn prose(text: &str, keep_inline_code: bool) -> String {
    let lines = without_fences(text);
    let mut output = String::with_capacity(lines.len());
    let mut text = lines.as_str();
    while let Some(index) = text.find(['`', '<']) {
        output.push_str(&text[..index]);
        text = &text[index..];
        if let Some(comment) = text.strip_prefix("<!--") {
            if let Some(end) = comment.find("-->") {
                preserve_newlines(&mut output, &comment[..end]);
                text = &comment[end + 3..];
                continue;
            }
        }
        if text.starts_with('`') {
            let content = text.trim_start_matches('`');
            let length = text.len() - content.len();
            if let Some(end) = closing_ticks(content, length) {
                if keep_inline_code {
                    output.push_str(&content[..end]);
                } else {
                    preserve_newlines(&mut output, &content[..end]);
                }
                text = &content[end + length..];
                continue;
            }
            output.push_str(&text[..length]);
            text = content;
        } else {
            output.push('<');
            text = &text[1..];
        }
    }
    output.push_str(text);
    output
}

fn brackets(text: &str) -> Option<(&str, &str)> {
    let content = text.strip_prefix('[')?;
    let (label, end) = content.split_at(content.find([']', '\n'])?);
    Some((label, end.strip_prefix(']')?))
}

fn destination(text: &str) -> Option<(&str, &str)> {
    if let Some(angle) = text.strip_prefix('<') {
        if let Some(end) = angle.find('>') {
            if end > 0 && !angle[..end].contains('\n') {
                return Some(text.split_at(end + 2));
            }
        }
    }
    let end = text
        .find(|ch: char| ch.is_whitespace() || ch == ')')
        .unwrap_or(text.len());
    (end > 0).then(|| text.split_at(end))
}

fn inline_link(text: &str) -> Option<(&str, &str)> {
    let (_, rest) = brackets(text)?;
    let (target, rest) = destination(rest.strip_prefix('(')?.trim_start())?;
    let after_space = rest.trim_start();
    if let Some(rest) = after_space.strip_prefix(')') {
        return Some((target, rest));
    }
    if after_space.len() == rest.len() {
        return None;
    }
    let quote = after_space
        .chars()
        .next()
        .filter(|&ch| ch == '\'' || ch == '"')?;
    let (title, rest) = after_space[1..].split_once(quote)?;
    if title.contains('\n') {
        return None;
    }
    Some((target, rest.trim_start().strip_prefix(')')?))
}

fn definition(line: &str) -> Option<(&str, &str, &str)> {
    let (label, rest) = brackets(unindent(line)?)?;
    if label.is_empty() {
        return None;
    }
    let (target, rest) = destination(rest.strip_prefix(':')?.trim_start())?;
    Some((label, target, rest))
}

fn normalize(label: &str) -> String {
    let mut folded = String::new();
    for word in label.split_whitespace() {
        if !folded.is_empty() {
            folded.push(' ');
        }
        for ch in word.chars() {
            match ch {
                // Default case folding preserves dotless i and uses uppercase for Cherokee.
                '\u{0131}' => folded.push(ch),
                '\u{13a0}'..='\u{13f5}' | '\u{13f8}'..='\u{13fd}' | '\u{ab70}'..='\u{abbf}' => {
                    folded.extend(ch.to_uppercase());
                }
                _ => folded.extend(
                    ch.to_lowercase()
                        .flat_map(char::to_uppercase)
                        .flat_map(char::to_lowercase),
                ),
            }
        }
    }
    folded
}

fn replace_brackets<'a>(
    mut text: &'a str,
    mut replacement: impl FnMut(&'a str) -> Option<(&'a str, &'a str)>,
) -> String {
    let mut output = String::with_capacity(text.len());
    while let Some(index) = text.find('[') {
        let (leading, candidate) = text.split_at(index);
        if let Some((value, rest)) = replacement(candidate) {
            output.push_str(leading.strip_suffix('!').unwrap_or(leading));
            output.push_str(value);
            text = rest;
        } else {
            output.push_str(leading);
            output.push('[');
            text = &candidate[1..];
        }
    }
    output.push_str(text);
    output
}

pub(super) fn links(text: &str) -> (Vec<String>, Vec<String>) {
    let clean = prose(text, false);
    let mut targets = Vec::new();
    let mut definitions = HashSet::new();
    let mut body = String::with_capacity(clean.len());
    for line in clean.split_inclusive('\n') {
        if let Some((label, target, rest)) = definition(line) {
            if definitions.insert(normalize(label)) {
                targets.push(target.to_owned());
            }
            body.push_str(rest);
        } else {
            body.push_str(line);
        }
    }
    let body = replace_brackets(&body, |candidate| {
        let (target, rest) = inline_link(candidate)?;
        targets.push(target.to_owned());
        Some(("", rest))
    });

    let mut undefined = Vec::new();
    let mut text = body.as_str();
    while let Some(index) = text.find('[') {
        let candidate = &text[index..];
        if let Some((label, rest)) = brackets(candidate) {
            if let Some((reference, rest)) = brackets(rest).filter(|_| !label.is_empty()) {
                let label = normalize(if reference.is_empty() {
                    label
                } else {
                    reference
                });
                if !definitions.contains(&label) {
                    undefined.push(label);
                }
                text = rest;
                continue;
            }
        }
        text = &candidate[1..];
    }
    (targets, undefined)
}

pub(super) fn frontmatter(text: &str) -> Option<(&str, &str)> {
    let text = text.strip_prefix("---\n")?;
    text.match_indices("\n---").find_map(|(index, delim)| {
        let rest = &text[index + delim.len()..];
        if rest.is_empty() {
            Some((&text[..index], rest))
        } else {
            Some((&text[..index], rest.strip_prefix('\n')?))
        }
    })
}

fn atx_title(line: &str) -> Option<&str> {
    let line = unindent(line)?;
    let rest = line.trim_start_matches('#');
    let marks = line.len() - rest.len();
    if !(1..=6).contains(&marks) || !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let title = rest.trim();
    (!title.is_empty()).then(|| title.trim_end_matches('#').trim_end())
}

pub(super) fn first_level_two_heading(text: &str) -> Option<String> {
    prose(text, false).lines().find_map(|line| {
        let title = unindent(line)?.strip_prefix("##")?;
        if !title.starts_with(char::is_whitespace) {
            return None;
        }
        atx_title(line).map(str::to_owned)
    })
}

fn setext_underline(line: &str) -> bool {
    unindent(line).is_some_and(|line| {
        let line = line.trim_end();
        line.starts_with('=') && line.bytes().all(|byte| byte == b'=')
            || line.starts_with('-') && line.bytes().all(|byte| byte == b'-')
    })
}

fn without_tags(mut text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    while let Some(start) = text.find('<') {
        let tag = &text[start + 1..];
        let Some(end) = tag.find('>') else { break };
        if end == 0 {
            output.push_str(&text[..start + 2]);
        } else {
            output.push_str(&text[..start]);
        }
        text = &tag[end + 1..];
    }
    output.push_str(text);
    output
}

pub(super) fn anchors(text: &str) -> HashSet<String> {
    let text = frontmatter(text).map_or(text, |(_, body)| body);
    let clean = prose(text, true);
    let mut lines = clean.lines().peekable();
    let mut counts = HashMap::new();
    let mut slugs = HashSet::new();
    while let Some(line) = lines.next() {
        let title = atx_title(line).or_else(|| {
            let title = line.trim();
            (!title.is_empty() && lines.peek().is_some_and(|next| setext_underline(next)))
                .then_some(title)
        });
        let Some(title) = title else { continue };
        let title = replace_brackets(title, |candidate| {
            let (label, rest) = brackets(candidate).filter(|(label, _)| !label.is_empty())?;
            let (_, rest) = rest.strip_prefix('(')?.split_once(')')?;
            Some((label, rest))
        });
        let slug: String = without_tags(&title)
            .to_lowercase()
            .chars()
            .filter(|&ch| ch.is_alphanumeric() || ch.is_whitespace() || matches!(ch, '-' | '_'))
            .map(|ch| if ch == ' ' { '-' } else { ch })
            .collect();
        let count = counts.entry(slug.clone()).or_insert(0);
        slugs.insert(if *count == 0 {
            slug
        } else {
            format!("{slug}-{count}")
        });
        *count += 1;
    }
    slugs
}

fn plain_scalar(value: &str) -> bool {
    let Some(first) = value.chars().next() else {
        return false;
    };
    if value.trim() != value
        || "[]{}'\"&*!|>@`#%".contains(first)
        || value.contains(char::is_control)
        || value.contains(": ")
        || value.ends_with(':')
        || value.contains(" #")
        || matches!(value, "-" | "?")
        || (matches!(first, '-' | '?' | ':')
            && value.chars().nth(1).is_some_and(char::is_whitespace))
        || matches!(
            value.to_ascii_lowercase().as_str(),
            "true"
                | "false"
                | "yes"
                | "no"
                | "on"
                | "off"
                | "null"
                | "~"
                | ".nan"
                | ".inf"
                | "+.inf"
                | "-.inf"
        )
    {
        return false;
    }
    let number = value.replace('_', "");
    if number.parse::<f64>().is_ok() {
        return false;
    }
    let number = number.strip_prefix(['-', '+']).unwrap_or(&number);
    ![("0b", 2), ("0o", 8), ("0x", 16)]
        .iter()
        .any(|(prefix, radix)| {
            number.strip_prefix(prefix).is_some_and(|digits| {
                !digits.is_empty() && digits.chars().all(|digit| digit.is_digit(*radix))
            })
        })
}

fn skill_scalar(value: &str) -> Option<&str> {
    // Support literal single-quoted strings without escapes as well as plain
    // scalars. In particular a digits-only name needs quoting to be a string.
    if let Some(value) = value.strip_prefix('\'') {
        return value
            .strip_suffix('\'')
            .filter(|value| !value.contains('\'') && !value.contains(char::is_control));
    }
    plain_scalar(value).then_some(value)
}

pub(super) fn skill_metadata(text: &str) -> Result<(&str, &str, &str), String> {
    let (metadata, body) = frontmatter(text).ok_or("missing --- delimited frontmatter")?;
    let mut name = None;
    let mut description = None;
    for line in metadata.lines() {
        let (key, value) = line
            .split_once(": ")
            .ok_or("expected an unindented key: value frontmatter line")?;
        let field = match key {
            "name" => &mut name,
            "description" => &mut description,
            _ => return Err(format!("unsupported frontmatter field {key:?}")),
        };
        let value = skill_scalar(value)
            .ok_or_else(|| format!("{key} must be a supported single-line string"))?;
        if field.replace(value).is_some() {
            return Err(format!("duplicate frontmatter field {key:?}"));
        }
    }
    Ok((
        name.ok_or("missing frontmatter name")?,
        description.ok_or("missing frontmatter description")?,
        body,
    ))
}
