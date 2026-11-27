use crate::{link_target, markdown, percent_decode, sha256, Checker};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const MANIFEST: &str = "docs/architecture-sources.tsv";
const REVISION_FIELD: &str = "# Source checkout revision:";
const LEGACY_DIRECTORY: &str = "docs/tai-lieu-kien-tuc-chuyen-biet";
const ECOSYSTEM_SECTION: &str = "## 4. Repository Responsibility and Boundary Matrix";
const LAB_FIRST_SECTION: &str = "## 2. CIRNETLAB Organization and Source-Code Ownership";
const ROUTES: [&str; 2] = [
    "en/references/architecture-routing.md",
    "vi/references/architecture-routing.md",
];
const SOURCES: [(&str, &str, &str); 6] = [
    (
        "ecosystem",
        "en",
        "docs/tai-lieu-kien-tuc-he-thong/cirnetlab-architecture-en.md",
    ),
    (
        "ecosystem",
        "vi",
        "docs/tai-lieu-kien-tuc-he-thong/cirnetlab-architecture-vi.md",
    ),
    (
        "lab-first",
        "en",
        "docs/tai-lieu-kien-tuc-he-thong/cirnet-architecture-lab-first-en.md",
    ),
    (
        "lab-first",
        "vi",
        "docs/tai-lieu-kien-tuc-he-thong/cirnet-architecture-lab-first-vi.md",
    ),
    ("fpga", "en", "docs/cirnetlab-FPGA-architecture-en.md"),
    ("fpga", "vi", "docs/cirnetlab-FPGA-architecture-vi.md"),
];

fn source_of_same_name(path: &Path) -> Option<&'static str> {
    let name = path.file_name()?;
    SOURCES.iter().find_map(|&(_, _, source)| {
        name.eq_ignore_ascii_case(Path::new(source).file_name()?)
            .then_some(source)
    })
}

pub(super) fn is_snapshot(path: &Path) -> bool {
    source_of_same_name(path).is_some_and(|source| path == Path::new(source))
}

pub(super) fn check_link(path: &Path) -> Result<(), String> {
    let legacy_file = path
        .parent()
        .is_some_and(|parent| super::path_text(parent).eq_ignore_ascii_case(LEGACY_DIRECTORY));
    if let Some(source) = source_of_same_name(path).filter(|_| legacy_file) {
        return Err(format!("superseded architecture path; use {source}"));
    }
    Ok(())
}

pub(super) fn check_github_link(destination: &str) -> Result<(), String> {
    let url = destination.split(['?', '#']).next().unwrap_or("");
    let url = percent_decode(url);
    let Some((scheme, rest)) = url.split_once("://") else {
        return Ok(());
    };
    if !scheme.eq_ignore_ascii_case("https") && !scheme.eq_ignore_ascii_case("http") {
        return Ok(());
    }
    let Some((host, path)) = rest.split_once('/') else {
        return Ok(());
    };
    let normalized_path = path.to_ascii_lowercase();
    let known_architecture_link = |repo: &str| {
        if host.eq_ignore_ascii_case("github.com") {
            ["blob", "tree"]
                .into_iter()
                .find_map(|kind| normalized_path.strip_prefix(&format!("cirnetlab/{repo}/{kind}/")))
        } else if host.eq_ignore_ascii_case("raw.githubusercontent.com") {
            normalized_path.strip_prefix(&format!("cirnetlab/{repo}/"))
        } else {
            None
        }
    };
    let is_pinned =
        |revision: &str| revision.len() == 40 && revision.bytes().all(|b| b.is_ascii_hexdigit());
    if let Some((revision, file)) =
        known_architecture_link("cirnet-remote").and_then(|rest| rest.split_once("/docs/"))
    {
        if !is_pinned(revision) && !file.contains('/') {
            if let Some(source) = source_of_same_name(Path::new(file)) {
                return Err(format!(
                    "removed Remote architecture path; use cirnet-rules/{source}"
                ));
            }
        }
    }
    if let Some((revision, file)) = known_architecture_link("about-cirnetlab")
        .and_then(|rest| rest.split_once("/architecture cirnetlab/"))
    {
        if !is_pinned(revision) {
            if let Some(source) = source_of_same_name(Path::new(file)) {
                return Err(format!(
                    "archival architecture path; use cirnet-rules/{source}"
                ));
            }
        }
    }
    let prefix = if host.eq_ignore_ascii_case("github.com") {
        "cirnetlab/cirnet-rules/blob/"
    } else if host.eq_ignore_ascii_case("raw.githubusercontent.com") {
        "cirnetlab/cirnet-rules/"
    } else {
        return Ok(());
    };
    if let Some((_, suffix)) = path
        .strip_prefix(prefix)
        .and_then(|rest| rest.split_once("/docs/"))
    {
        if let Ok(target) = link_target(Path::new(""), &format!("docs/{suffix}")) {
            check_link(&target)?;
        }
    }
    Ok(())
}

fn manifest_hashes<'a>(
    checker: &mut Checker,
    manifest: &'a str,
) -> BTreeMap<&'static str, &'a str> {
    let path = Path::new(MANIFEST);
    let mut hashes = BTreeMap::new();
    let mut revision_seen = false;
    for (index, row) in manifest.lines().enumerate() {
        let number = index + 1;
        if let Some(value) = row.strip_prefix(REVISION_FIELD) {
            if revision_seen {
                checker.fail(
                    path,
                    format!("line {number}: duplicate source checkout revision"),
                );
            } else if !value
                .strip_prefix(' ')
                .is_some_and(|rev| lowercase_hex(rev, 40))
            {
                checker.fail(
                    path,
                    format!(
                    "line {number}: expected a 40-character lowercase hex source checkout revision"
                ),
                );
            }
            revision_seen = true;
        }
        if row.trim().is_empty() || row.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = row.split('\t').collect();
        let [id, language, actual_path, hash] = fields.as_slice() else {
            checker.fail(
                path,
                format!(
                    "line {number}: expected source, language, path and SHA-256 separated by tabs"
                ),
            );
            continue;
        };
        let Some(&(_, _, expected_path)) = SOURCES
            .iter()
            .find(|(source, locale, _)| source == id && locale == language)
        else {
            checker.fail(
                path,
                format!("line {number}: unknown architecture source or language"),
            );
            continue;
        };
        if actual_path != &expected_path {
            checker.fail(
                path,
                format!("line {number}: architecture source path must be {expected_path}"),
            );
        } else if !lowercase_hex(hash, 64) {
            checker.fail(path, format!("line {number}: expected a lowercase SHA-256"));
        } else if hashes.insert(expected_path, *hash).is_some() {
            checker.fail(
                path,
                format!("line {number}: duplicate source row for {expected_path}"),
            );
        }
    }
    if !revision_seen {
        checker.fail(path, "missing source checkout revision");
    }
    for (id, language, source) in SOURCES {
        if !hashes.contains_key(source) {
            checker.fail(
                path,
                format!("missing architecture source row for {id} ({language})"),
            );
        }
    }
    hashes
}

fn lowercase_hex(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn source_repositories(text: &str, heading: &str) -> BTreeSet<String> {
    markdown::markup(text)
        .lines()
        .skip_while(|line| line.trim() != heading)
        .skip(1)
        .take_while(|line| !line.starts_with("## "))
        .filter_map(|line| {
            let row = line.strip_prefix("| [`")?;
            let (name, rest) = row.split_once("`](https://github.com/cirnetlab/")?;
            let (repo, _) = rest.split_once(')')?;
            (name == repo).then(|| name.to_owned())
        })
        .collect()
}

fn validate_routing(
    checker: &mut Checker,
    path: &Path,
    ecosystem: &BTreeSet<String>,
    lab_first: &BTreeSet<String>,
) {
    let Some(table) = checker.read(path).map(markdown::markup) else {
        return;
    };
    let mut routed = BTreeSet::new();
    for row in table.lines().filter(|line| line.starts_with("| `")) {
        let cells: Vec<_> = row.trim_matches('|').split('|').map(str::trim).collect();
        let [repo, selector, context, outputs, exchange, boundary] = cells.as_slice() else {
            checker.fail(path, "expected six fields in each architecture route");
            continue;
        };
        let Some(repo) = repo
            .strip_prefix('`')
            .and_then(|name| name.strip_suffix('`'))
        else {
            checker.fail(path, "expected a repository ID in an inline code span");
            continue;
        };
        if !routed.insert(repo) {
            checker.fail(path, format!("duplicate route for `{repo}`"));
        }
        if !ecosystem.contains(repo) {
            checker.fail(
                path,
                format!("repository `{repo}` is outside the Ecosystem matrix"),
            );
            continue;
        }
        let expected = if repo == "cirnet-embedded-hardware" {
            "`ecosystem`, `lab-first`, `fpga`"
        } else if lab_first.contains(repo) {
            "`ecosystem`, `lab-first`"
        } else {
            "`ecosystem`"
        };
        if *selector != expected {
            checker.fail(path, format!("repository `{repo}` must select {expected}"));
        }
        if [context, outputs, exchange, boundary]
            .iter()
            .any(|cell| cell.is_empty())
        {
            checker.fail(
                path,
                format!("repository `{repo}` needs context, outputs, exchanges and boundaries"),
            );
        }
    }
    for name in ecosystem
        .iter()
        .filter(|name| !routed.contains(name.as_str()))
    {
        checker.fail(path, format!("missing route for `{name}`"));
    }
}

fn validate_snapshot_links(checker: &mut Checker, path: &Path, text: &str) {
    let (links, undefined) = markdown::links(text);
    for destination in links {
        if let Err(error) = checker.validate_link(path, &destination, None) {
            checker.fail(path, format!("invalid link {destination:?}: {error}"));
        }
    }
    for label in undefined {
        checker.fail(path, format!("undefined link reference {label:?}"));
    }
}

fn check_variants(checker: &mut Checker, source_hashes: &BTreeMap<&str, String>) {
    let mut candidates = BTreeSet::new();
    let directory = Path::new("docs");
    checker.collect(directory, &mut candidates);
    candidates.extend(checker.documents.keys().cloned());
    for path in candidates {
        let Some(source) = source_of_same_name(&path).filter(|_| !is_snapshot(&path)) else {
            continue;
        };
        let Some(canonical_hash) = source_hashes.get(source) else {
            continue;
        };
        match checker.regular_file_text(&path, true) {
            Ok(text) if &sha256::hex(text.as_bytes()) != canonical_hash => checker.warn(
                &path,
                format!("divergent architecture copy; compare with {source} and preserve history"),
            ),
            Ok(_) => {}
            Err(error) => checker.fail(&path, error),
        }
    }
}

pub(super) fn check(checker: &mut Checker, is_toolkit: bool) {
    let path = Path::new(MANIFEST);
    if !is_toolkit
        && checker.metadata(path).is_err()
        && !ROUTES
            .iter()
            .any(|route| checker.documents.contains_key(Path::new(route)))
    {
        return;
    }
    let index = match checker.regular_file_text(path, true) {
        Ok(text) => Some(text),
        Err(error) => {
            checker.fail(path, error);
            None
        }
    };
    let manifest = index
        .as_deref()
        .map_or_else(BTreeMap::new, |text| manifest_hashes(checker, text));
    let mut ecosystem = BTreeSet::new();
    let mut lab_first = BTreeSet::new();
    let mut source_hashes = BTreeMap::new();
    for (id, locale, source) in SOURCES {
        let path = Path::new(source);
        let text = match checker.regular_file_text(path, true) {
            Ok(text) => text,
            Err(error) => {
                checker.fail(path, error);
                continue;
            }
        };
        let hash = sha256::hex(text.as_bytes());
        if manifest
            .get(source)
            .is_some_and(|expected| expected != &hash)
        {
            checker.fail(path, "architecture source SHA-256 mismatch");
        }
        source_hashes.insert(source, hash);
        validate_snapshot_links(checker, path, &text);
        let (owners, heading) = match (id, locale) {
            ("ecosystem", "en") => (&mut ecosystem, ECOSYSTEM_SECTION),
            ("lab-first", "en") => (&mut lab_first, LAB_FIRST_SECTION),
            _ => continue,
        };
        *owners = source_repositories(&text, heading);
        if owners.is_empty() {
            checker.fail(
                path,
                format!("could not read repository rows under {heading}"),
            );
        }
    }
    if !lab_first.is_subset(&ecosystem)
        || (!lab_first.is_empty() && !lab_first.contains("cirnet-fpga"))
    {
        checker.fail(
            path,
            "Lab-First owners must be in the Ecosystem matrix and include cirnet-fpga",
        );
    }
    // The pinned architecture snapshots record the original repository name.
    for owners in [&mut ecosystem, &mut lab_first] {
        if owners.remove("cirnet-fpga") {
            owners.insert("cirnet-embedded-hardware".into());
        }
    }
    for route in ROUTES {
        let path = Path::new(route);
        if ecosystem.is_empty() || lab_first.is_empty() {
            checker.read(path);
        } else {
            validate_routing(checker, path, &ecosystem, &lab_first);
        }
    }
    check_variants(checker, &source_hashes);
}
