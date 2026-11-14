use std::collections::{BTreeMap, BTreeSet};

use crate::markdown;

pub(super) struct Size {
    pub bytes: usize,
    pub words: usize,
    pub lines: usize,
}

impl Size {
    pub fn of(text: &str) -> Self {
        Self {
            bytes: text.len(),
            words: text.split_whitespace().count(),
            lines: text.lines().count(),
        }
    }
}

impl std::fmt::Display for Size {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            output,
            "{} bytes, {} words, {} lines",
            self.bytes, self.words, self.lines
        )
    }
}

// Find only stand-alone, unquoted @imports, outside code and comments. These
// are patterns for maintainers to review; this does not model client behavior.
pub(super) fn startup_imports(text: &str) -> Vec<String> {
    markdown::prose(text, false)
        .lines()
        .filter_map(|line| {
            let import = line.trim().strip_prefix('@')?;
            if import.is_empty() || import.contains(char::is_whitespace) {
                return None;
            }
            Some(
                import
                    .replace('\\', "/")
                    .trim_start_matches("./")
                    .trim_end_matches('/')
                    .to_owned(),
            )
        })
        .collect()
}

pub(super) fn suspicious_prose(text: &str) -> Vec<String> {
    let prose = markdown::prose(text, false);
    let mut warnings = Vec::new();
    let mut paragraphs = BTreeSet::new();
    for paragraph in prose.split("\n\n") {
        if paragraph.trim_start().starts_with(['#', '>'])
            || paragraph.trim_start().starts_with("Example:")
        {
            continue;
        }
        let words: Vec<_> = paragraph.split_whitespace().collect();
        if words.len() >= 24 && !paragraphs.insert(words.join(" ")) {
            warnings.push("repeated prose block with at least 24 words".to_owned());
            break;
        }
    }
    let mut rules = BTreeMap::new();
    for line in prose.lines() {
        let line = line.trim().trim_start_matches("- ");
        let Some((verb, rest)) = line.split_once(' ') else {
            continue;
        };
        let positive = match verb {
            "Always" => true,
            "Never" => false,
            _ => continue,
        };
        let rest = rest.trim().trim_end_matches('.');
        if !rest.is_empty() && rules.insert(rest, positive) == Some(!positive) {
            warnings.push(format!("possible conflicting Always/Never rules: {rest}"));
        }
    }
    warnings
}
