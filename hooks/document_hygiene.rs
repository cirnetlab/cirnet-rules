use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Write;
use std::path::{Path, PathBuf};

use crate::{markdown, path_text, sha256, stamp};

const MIN_WORDS: usize = 40;
const MIN_SHINGLES: usize = 24;
const NEAR_PERCENT: usize = 60;
const TOPIC_PERCENT: usize = 40;

pub(super) enum Diagnostic {
    Info(String),
    Warning(String),
    Error(String),
}

#[derive(PartialEq, Eq)]
struct Scope {
    language: &'static str,
    role: &'static str,
    preserve: bool,
}

fn scope(path: &Path, text: &str) -> Result<Scope, &'static str> {
    let mut parts = path.iter().filter_map(|part| part.to_str());
    let first = parts.next().unwrap_or_default();
    let language = match first {
        "en" => "en",
        "vi" => "vi",
        _ => "root",
    };
    let directories: Vec<_> = parts.collect();
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if (language == "root" || directories.len() == 1)
        && matches!(
            name.to_ascii_lowercase().as_str(),
            "agents.md" | "claude.md"
        )
    {
        return Err("required entrypoint, mirror or adapter");
    }
    if matches!(
        name.to_ascii_lowercase().as_str(),
        "license.md" | "copying.md" | "notice.md" | "third_party_notices.md"
    ) {
        return Err("license or attribution notice");
    }
    if directories.iter().any(|part| {
        matches!(
            part.to_ascii_lowercase().as_str(),
            "templates" | "samples" | "examples" | "fixtures" | "tests" | ".git" | ".superpowers"
        )
    }) {
        return Err("template, sample, fixture or test material");
    }
    if name.eq_ignore_ascii_case("README.md") && text.split_whitespace().count() < MIN_WORDS {
        return Err("short README or index");
    }

    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let name_words = words(stem);
    let has = |word: &str| name_words.iter().any(|token| token == word);
    let is_dated = name_words.windows(3).any(|parts| {
        parts[0].len() == 4
            && parts[0]
                .parse::<u16>()
                .is_ok_and(|year| (1900..=2100).contains(&year))
            && parts[1]
                .parse::<u8>()
                .is_ok_and(|month| (1..=12).contains(&month))
            && parts[2]
                .parse::<u8>()
                .is_ok_and(|day| (1..=31).contains(&day))
    });
    let reports = ["verification", "report", "rerun", "followup"];
    let archive = [
        "archive",
        "archives",
        "history",
        "historical",
        "evidence",
        "snapshot",
        "snapshots",
        "release",
        "releases",
    ];
    let role = if has("changelog") || name.eq_ignore_ascii_case("RELEASE_NOTES.md") {
        "release notes"
    } else if archive.iter().any(|word| {
        has(word)
            || directories
                .iter()
                .any(|part| part.eq_ignore_ascii_case(word))
    }) || is_dated
    {
        "historical evidence"
    } else if reports.iter().any(|word| has(word))
        || (has("follow") && has("up"))
        || directories
            .iter()
            .any(|part| matches!(*part, "reports" | "verification"))
    {
        "verification report"
    } else if language == "root" {
        "root documents"
    } else {
        match directories.first().copied() {
            Some("plans") => "plans",
            Some("specs") => "specs",
            Some("references") => "references",
            _ => "docs",
        }
    };
    Ok(Scope {
        language,
        role,
        preserve: matches!(
            role,
            "release notes" | "historical evidence" | "verification report"
        ),
    })
}

fn heading_only(text: &str) -> bool {
    let mut lines = text.lines().peekable();
    while let Some(raw) = lines.next() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let after_marks = line.trim_start_matches('#');
        if (1..=6).contains(&(line.len() - after_marks.len()))
            && after_marks.starts_with(char::is_whitespace)
        {
            continue;
        }
        let setext = lines.next_if(|underline| {
            let underline = underline.trim();
            underline.bytes().next().is_some_and(|marker| {
                matches!(marker, b'=' | b'-') && underline.bytes().all(|byte| byte == marker)
            })
        });
        if setext.is_none() {
            return false;
        }
    }
    true
}

fn normalized<'a>(
    path: &Path,
    text: &'a str,
    documents: &BTreeMap<PathBuf, String>,
) -> (Cow<'a, str>, Option<&'a str>) {
    // Retain the paired source hash so stripping two valid footers cannot
    // turn translations of different source material into a hard duplicate.
    let (body, source_hash) = path
        .strip_prefix("vi")
        .ok()
        .and_then(|relative| {
            let expected = Path::new("en").join(relative);
            let (source, hash) = stamp(text)?;
            let paired_text = documents.get(&expected)?;
            if source != path_text(&expected) || hash != sha256::hex(paired_text.as_bytes()) {
                return None;
            }
            let body = text.trim_end_matches('\n').rsplitn(3, '\n').nth(2)?;
            (!markdown::ends_in_fence(body)).then_some((body, Some(hash)))
        })
        .unwrap_or((text, None));
    let content = if body.contains("\r\n") {
        let mut lf = body.replace("\r\n", "\n");
        if !markdown::ends_in_fence(&lf) {
            lf.truncate(lf.trim_end_matches('\n').len());
        }
        Cow::Owned(lf)
    } else if markdown::ends_in_fence(body) {
        Cow::Borrowed(body)
    } else {
        Cow::Borrowed(body.trim_end_matches('\n'))
    };
    (content, source_hash)
}

fn words(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn revision_suffix(word: &str) -> bool {
    matches!(
        word,
        "fix" | "fixed" | "final" | "latest" | "updated" | "draft"
    ) || ["v", "rev"].iter().any(|prefix| {
        word.strip_prefix(prefix)
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
    })
}

fn file_family(path: &Path) -> (Vec<String>, bool) {
    let mut family = words(path.file_stem().and_then(|s| s.to_str()).unwrap_or(""));
    let mut revised = false;
    while family.len() > 1 && family.last().is_some_and(|word| revision_suffix(word)) {
        family.pop();
        revised = true;
    }
    (family, revised)
}

fn issues(text: &str) -> BTreeSet<String> {
    text.split(|ch: char| !(ch.is_ascii_alphanumeric() || matches!(ch, '#' | '-')))
        .filter_map(|token| {
            let id = token.strip_prefix('#').or_else(|| {
                let (label, number) = token.rsplit_once('-')?;
                label.rsplit('-').next().filter(|p| {
                    matches!(p.to_ascii_lowercase().as_str(), "issue" | "bug" | "ticket")
                })?;
                Some(number)
            })?;
            (!id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit())).then(|| id.to_owned())
        })
        .collect()
}

struct Document<'a> {
    path: &'a Path,
    raw: &'a str,
    normalized: Cow<'a, str>,
    source_hash: Option<&'a str>,
    scope: Scope,
    word_count: usize,
    shingles: BTreeSet<String>,
    blocks: BTreeMap<String, usize>,
    headings: HashSet<String>,
    title: BTreeSet<String>,
    family: Vec<String>,
    revised: bool,
    issues: BTreeSet<String>,
}

impl<'a> Document<'a> {
    fn new(
        path: &'a Path,
        raw: &'a str,
        normalized: Cow<'a, str>,
        source_hash: Option<&'a str>,
        scope: Scope,
    ) -> Self {
        let mut shingles = BTreeSet::new();
        let mut blocks = BTreeMap::new();
        let mut word_count = 0;
        // Use separate paragraphs so moving one does not turn its neighbors into matches.
        // Fenced examples do not count as prose. Equality still uses the full text.
        let prose = markdown::prose(&normalized, true);
        for block in prose.split("\n\n") {
            let block_words = words(block);
            word_count += block_words.len();
            let triples: BTreeSet<_> = block_words
                .windows(3)
                .map(|words| words.join(" "))
                .collect();
            if triples.len() >= MIN_SHINGLES {
                blocks.insert(block_words.join(" "), triples.len());
            }
            shingles.extend(triples);
        }
        let title_line = prose
            .lines()
            .find_map(|line| line.trim_start().strip_prefix("# "))
            .unwrap_or("");
        let title = words(title_line).into_iter().collect();
        let mut issue_ids = issues(title_line);
        issue_ids.extend(issues(
            path.file_stem().and_then(|s| s.to_str()).unwrap_or(""),
        ));
        let headings = markdown::anchors(&normalized);
        let (family, revised) = file_family(path);
        Self {
            path,
            raw,
            normalized,
            source_hash,
            scope,
            word_count,
            shingles,
            blocks,
            headings,
            title,
            family,
            revised,
            issues: issue_ids,
        }
    }

    fn topic(&self, other: &Self) -> Option<&'static str> {
        // A title or issue number can be shared by independently owned folders.
        if self.path.parent() != other.path.parent() {
            None
        } else if !self.issues.is_disjoint(&other.issues) {
            Some("shared issue number")
        } else if !self.family.is_empty()
            && self.family == other.family
            && (self.revised || other.revised)
        {
            Some("same folder and filename family after revision suffixes")
        } else {
            let shared = self.title.intersection(&other.title).count();
            let union = self.title.len() + other.title.len() - shared;
            (shared >= 2 && shared * 100 >= union * NEAR_PERCENT).then_some("similar titles")
        }
    }

    fn compare(&self, other: &Self) -> Option<(Match, usize, usize)> {
        if self.scope != other.scope
            || self.normalized.trim().is_empty()
            || other.normalized.trim().is_empty()
        {
            return None;
        }
        let shared = self.shingles.intersection(&other.shingles).count();
        let union = self.shingles.len() + other.shingles.len() - shared;
        if self.raw == other.raw {
            return Some((Match::Exact, shared, union));
        }
        if self.normalized == other.normalized {
            return Some((Match::Normalized, shared, union));
        }
        if self.word_count.min(other.word_count) < MIN_WORDS
            || self.shingles.len().min(other.shingles.len()) < MIN_SHINGLES
        {
            return None;
        }
        let topic = self.topic(other);
        let threshold = if topic.is_some() {
            TOPIC_PERCENT
        } else {
            NEAR_PERCENT
        };
        if shared * 100 < union * threshold {
            return None;
        }
        // With no topic agreement, one common block alone is weak boilerplate evidence.
        if topic.is_none()
            && self.blocks.iter().any(|(block, count)| {
                other.blocks.contains_key(block) && shared.saturating_sub(*count) < 3
            })
        {
            return None;
        }
        Some((Match::Near(topic), shared, union))
    }
}

#[derive(PartialEq, Eq)]
enum Match {
    Exact,
    Normalized,
    Near(Option<&'static str>),
}

fn path_label(path: &Path) -> String {
    let path = path_text(path);
    if path.chars().any(char::is_control) {
        format!("{path:?}")
    } else {
        path
    }
}

fn references(incoming: &BTreeMap<PathBuf, BTreeSet<PathBuf>>, path: &Path) -> String {
    let Some(sources) = incoming.get(path).filter(|sources| !sources.is_empty()) else {
        return "none in the checked Markdown".into();
    };
    let mut paths = sources
        .iter()
        .take(5)
        .map(|source| path_label(source))
        .collect::<Vec<_>>()
        .join(", ");
    if sources.len() > 5 {
        let _ = write!(paths, " (+{} more)", sources.len() - 5);
    }
    paths
}

pub(super) fn review(
    documents: &BTreeMap<PathBuf, String>,
    new_files: &BTreeSet<PathBuf>,
    incoming: &BTreeMap<PathBuf, BTreeSet<PathBuf>>,
) -> Vec<Diagnostic> {
    let mut findings = Vec::new();
    let mut excluded = BTreeMap::new();
    let profiles: Vec<_> = documents
        .iter()
        .filter_map(|(path, text)| {
            let scope = match scope(path, text) {
                Ok(scope) => scope,
                Err(reason) => {
                    excluded.insert(path.as_path(), reason);
                    return None;
                }
            };
            let (normalized, source_hash) = normalized(path, text, documents);
            if heading_only(&normalized) {
                excluded.insert(path.as_path(), "empty or heading-only material");
                return None;
            }
            Some(Document::new(path, text, normalized, source_hash, scope))
        })
        .collect();
    let incoming_count = |path: &Path| incoming.get(path).map_or(0, BTreeSet::len);
    let mut suggested = BTreeSet::new();
    for (i, a) in profiles.iter().enumerate() {
        for b in &profiles[i + 1..] {
            let Some((kind, overlap, union)) = a.compare(b) else {
                continue;
            };
            let new_a = new_files.contains(a.path);
            let new_b = new_files.contains(b.path);
            let preserve = if a.scope.preserve || b.scope.preserve {
                Some("Preserve separate evidence and review dates, revisions and outcomes.")
            } else if kind == Match::Normalized && a.source_hash != b.source_hash {
                Some("Paired English sources differ. Preserve provenance and review the source documents.")
            } else {
                None
            };
            let new_duplicate = (new_a || new_b)
                && preserve.is_none()
                && matches!(kind, Match::Exact | Match::Normalized);
            let has_topic_evidence = !matches!(kind, Match::Near(None));
            let (id, score, confidence, cause) = match kind {
                Match::Exact => ("DOC_EXACT_DUPLICATE", "100% (byte equality)".into(), "high (byte identity)", "identical bytes"),
                Match::Normalized => ("DOC_NORMALIZED_DUPLICATE", "100% (normalized equality)".into(), "high (normalized text identity)", "same content after line-ending, terminal blank line and paired footer normalization"),
                Match::Near(topic) => ("DOC_NEAR_DUPLICATE", format!("{:.1}% (3-word Jaccard on prose: {overlap}/{union})", 100.0 * overlap as f64 / union as f64), "medium (lexical overlap)", topic.unwrap_or("substantial shared content across passages")),
            };
            let id = if new_duplicate {
                "DOC_NEW_DUPLICATE"
            } else {
                id
            };
            let (action, canonical, direction) = if let Some(reason) = preserve {
                ("KEEP_SEPARATE", None, reason)
            } else {
                suggested.extend(
                    [(a.path, new_a), (b.path, new_b)]
                        .into_iter()
                        .filter_map(|(path, new)| new.then_some(path)),
                );
                let canonical = match (new_a, new_b) {
                    (true, false) => b,
                    (false, true) => a,
                    _ if (a.revised, std::cmp::Reverse(incoming_count(a.path)))
                        <= (b.revised, std::cmp::Reverse(incoming_count(b.path))) =>
                    {
                        a
                    }
                    _ => b,
                };
                let (action, direction) = if new_a != new_b && has_topic_evidence {
                    (
                        "UPDATE_EXISTING",
                        "Review the existing candidate before adding another file.",
                    )
                } else if new_a != new_b {
                    ("REVIEW_MERGE", "Check whether the new file and the existing candidate serve the same purpose.")
                } else if new_a && new_b {
                    (
                        "REVIEW_MERGE",
                        "Confirm whether both proposed files need distinct owners.",
                    )
                } else {
                    ("REVIEW_MERGE", "Confirm ownership, unique content and links before consolidating. The candidate favors a stable name and inbound links.")
                };
                (action, Some(canonical.path), direction)
            };
            let canonical = canonical.map_or_else(|| "none".into(), path_label);
            let heading_overlap = a.headings.intersection(&b.headings).count();
            let message = format!(
                "{id}\n  file_a: {}\n  file_b: {}\n  similarity: {score}\n  confidence: {confidence}\n  reason: same {}/{} comparison group, {cause}; {heading_overlap} shared heading(s)\n  unique_content: {}/{} unmatched headings, {}/{} unmatched prose shingles (A/B); review code blocks, commands, values, statuses and evidence\n  incoming_a: {}\n  incoming_b: {}\n  canonical_candidate: {canonical}\n  suggestion: {action} — {direction}",
                path_label(a.path), path_label(b.path), a.scope.language, a.scope.role,
                a.headings.len() - heading_overlap, b.headings.len() - heading_overlap,
                a.shingles.len() - overlap, b.shingles.len() - overlap,
                references(incoming, a.path), references(incoming, b.path),
            );
            findings.push(if new_duplicate {
                Diagnostic::Error(message)
            } else {
                Diagnostic::Warning(message)
            });
        }
    }
    for path in new_files {
        if !suggested.contains(path.as_path()) {
            let reason = if let Some(reason) = excluded.get(path.as_path()) {
                format!("KEEP_SEPARATE ({reason})")
            } else if profiles
                .iter()
                .any(|doc| doc.path == path && doc.scope.preserve)
            {
                "KEEP_SEPARATE (verification, historical or release record)".into()
            } else {
                "no viable existing candidate met the comparison rules".into()
            };
            findings.push(Diagnostic::Info(format!(
                "NEW_DOCUMENT_REVIEW\n  file: {}\n  reason: {reason}\n  suggestion: confirm purpose, owner and index links",
                path_label(path)
            )));
        }
    }
    findings
}
