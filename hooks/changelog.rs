use crate::{markdown, Checker};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const INDEX: &str = "docs/changelogs/README.md";
const ROOT: &str = "CHANGELOG.md";
const CATEGORIES: [&str; 7] = [
    "Added",
    "Changed",
    "Fixed",
    "Deprecated",
    "Removed",
    "Security",
    "Breaking Changes",
];

fn number(text: &str) -> Option<u64> {
    if text.bytes().all(|b| b.is_ascii_digit()) && (text == "0" || !text.starts_with('0')) {
        text.parse().ok()
    } else {
        None
    }
}

#[derive(Clone, Copy)]
struct Version<'a> {
    core: [u64; 3],
    pre: Option<&'a str>,
}

fn identifiers(text: &str, prerelease: bool) -> bool {
    text.split('.').all(|id| {
        !id.is_empty()
            && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            && !(prerelease
                && id.len() > 1
                && id.starts_with('0')
                && id.bytes().all(|b| b.is_ascii_digit()))
    })
}

impl<'a> Version<'a> {
    fn parse(text: &'a str) -> Option<Self> {
        let (text, build) = text
            .split_once('+')
            .map_or((text, None), |(a, b)| (a, Some(b)));
        let (core, pre) = text
            .split_once('-')
            .map_or((text, None), |(a, b)| (a, Some(b)));
        if build.is_some_and(|b| !identifiers(b, false))
            || pre.is_some_and(|p| !identifiers(p, true))
        {
            return None;
        }
        let mut parts = core.split('.');
        let core = [
            number(parts.next()?)?,
            number(parts.next()?)?,
            number(parts.next()?)?,
        ];
        parts.next().is_none().then_some(Self { core, pre })
    }

    fn precedence(self, other: Self) -> Ordering {
        self.core
            .cmp(&other.core)
            .then_with(|| match (self.pre, other.pre) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(left), Some(right)) => {
                    let mut right = right.split('.');
                    for a in left.split('.') {
                        let Some(b) = right.next() else {
                            return Ordering::Greater;
                        };
                        let numeric = |id: &str| id.bytes().all(|b| b.is_ascii_digit());
                        let comparison = match (numeric(a), numeric(b)) {
                            (true, true) => a.len().cmp(&b.len()).then_with(|| a.cmp(b)),
                            (true, false) => Ordering::Less,
                            (false, true) => Ordering::Greater,
                            (false, false) => a.cmp(b),
                        };
                        if !comparison.is_eq() {
                            return comparison;
                        }
                    }
                    if right.next().is_some() {
                        Ordering::Less
                    } else {
                        Ordering::Equal
                    }
                }
            })
    }
}

fn translation_marker(text: &str) -> bool {
    // Recognize the previously used translation structure. Review prose language separately.
    let body = markdown::markup(text);
    body.lines().any(|line| {
        let line = line.trim();
        line.starts_with("- VI ·")
            || (line.starts_with('#')
                && (line.contains("Tiếng Việt") || line.contains("Nhật ký thay đổi")))
    })
}

pub(super) fn check(
    checker: &mut Checker,
    files: &BTreeSet<PathBuf>,
    incoming: &BTreeMap<PathBuf, BTreeSet<PathBuf>>,
    toolkit_version: Option<&str>,
) {
    let mut findings = Vec::new();
    let mut fail = |path: &Path, message: String| findings.push((path.to_path_buf(), message));
    let index = Path::new(INDEX);
    let root = Path::new(ROOT);
    let linked_from = |target: &Path, source: &Path| {
        incoming
            .get(target)
            .is_some_and(|paths| paths.contains(source))
    };
    if !linked_from(index, root) {
        fail(root, format!("missing index link to {INDEX}"));
    }
    for path in checker.documents.keys() {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if path != root
            && !files.contains(path)
            && (name.eq_ignore_ascii_case("changelog.md")
                || name.to_ascii_uppercase().starts_with("CHANGELOG_V"))
        {
            fail(path, format!("use the shared English changelog in {INDEX}"));
        }
    }
    let mut major_files = BTreeMap::new();
    for path in files.iter().filter(|path| path.as_path() != index) {
        let major = path
            .strip_prefix("docs/changelogs")
            .ok()
            .and_then(|path| path.to_str())
            .and_then(|name| name.strip_prefix("CHANGELOG_V"))
            .and_then(|name| name.strip_suffix(".md"))
            .and_then(number);
        if let Some(major) = major {
            major_files.insert(major, path);
        } else {
            fail(path, "use CHANGELOG_V<major>.md, one file per major".into());
        }
    }
    if major_files.is_empty() {
        fail(
            index,
            "expected at least one major-version changelog".into(),
        );
    }
    for path in files.iter().map(PathBuf::as_path).chain([root]) {
        if checker
            .documents
            .get(path)
            .is_some_and(|text| translation_marker(text))
        {
            fail(
                path,
                "remove the changelog translation marker and retain the English history".into(),
            );
        }
    }
    for path in [root, index] {
        if checker
            .documents
            .get(path)
            .is_some_and(|text| !markdown::level_headings(text, 2).is_empty())
        {
            fail(
                path,
                "keep changelog navigation here and put version entries in the major file".into(),
            );
        }
    }
    let newest = major_files.last_key_value().map(|(&major, _)| major);
    let mut newest_entry = None;
    for (&major, path) in &major_files {
        if !linked_from(path, index) {
            fail(path, format!("missing index link from {INDEX}"));
        }
        let Some(text) = checker.documents.get(*path) else {
            continue;
        };
        let entries = markdown::level_headings(text, 2);
        let unreleased = entries
            .iter()
            .filter(|heading| heading.as_str() == "Unreleased")
            .count();
        if (Some(major) == newest
            && (unreleased != 1 || entries.first().map(String::as_str) != Some("Unreleased")))
            || (Some(major) != newest && unreleased != 0)
        {
            fail(
                path,
                "put one Unreleased section first in the latest major file".into(),
            );
        }
        let mut seen = BTreeSet::new();
        let mut previous: Option<Version<'_>> = None;
        for entry in entries
            .iter()
            .filter(|heading| heading.as_str() != "Unreleased")
        {
            let Some((heading, label)) = entry.split_once(" — ") else {
                fail(
                    path,
                    format!("expected a full version and status or date in {entry:?}"),
                );
                continue;
            };
            let id = heading.strip_prefix('v').unwrap_or("");
            let Some(version) = Version::parse(id) else {
                fail(path, format!("invalid Semantic Version in {entry:?}"));
                continue;
            };
            if label.is_empty() {
                fail(path, format!("missing status or date for v{id}"));
            }
            if version.core[0] != major {
                fail(path, format!("v{id} belongs to the wrong major file"));
            }
            if !seen.insert(id) {
                fail(path, format!("duplicate version v{id}"));
            }
            if previous.is_none() && Some(major) == newest {
                newest_entry = Some(id.to_owned());
            }
            if previous.is_some_and(|old| old.precedence(version).is_lt()) {
                fail(path, format!("version v{id} is out of newest-first order"));
            }
            previous = Some(version);
        }
        if seen.is_empty() {
            fail(path, "expected a version entry in the major file".into());
        }
        let mut bullets = BTreeSet::new();
        for line in markdown::markup(text).lines() {
            if line.starts_with("## ") {
                bullets.clear();
            } else if let Some(bullet) = line.strip_prefix("- ") {
                if !bullets.insert(bullet) {
                    fail(path, format!("duplicate changelog bullet {bullet:?}"));
                }
            }
        }
        for category in markdown::level_headings(text, 3) {
            if !CATEGORIES.contains(&category.as_str()) {
                fail(path, format!("unsupported changelog category {category:?}"));
            }
        }
    }
    if let Some(version) = toolkit_version {
        if newest_entry.as_deref() != Some(version) {
            fail(
                index,
                format!("latest version entry must match VERSION ({version})"),
            );
        }
    }
    for (path, message) in findings {
        checker.fail(&path, message);
    }
}
