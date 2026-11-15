//! Run the checker against isolated trees below the ignored fixture output directory.

#[path = "../hooks/sha256.rs"]
mod sha256;

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::sync::atomic::{AtomicUsize, Ordering};

const GUIDE: &str = "# Guide\n\n## First heading\n";
static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

struct Fixture {
    output: PathBuf,
    scratch: PathBuf,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let checkout = Path::new(env!("CARGO_MANIFEST_DIR"));
        let superpowers = fixture_directory(checkout, ".superpowers");
        let output = fixture_directory(&superpowers, "bilingual-tests");
        let scratch = loop {
            let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
            let path = output.join(format!("check-{}-{id}", process::id()));
            match fs::create_dir(&path) {
                Ok(()) => break path,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("could not create fixture {}: {error}", path.display()),
            }
        };
        assert_eq!(
            scratch.canonicalize().unwrap(),
            output
                .canonicalize()
                .unwrap()
                .join(scratch.file_name().unwrap()),
            "fixture escaped the output directory"
        );
        let root = scratch.join("repo");
        let fixture = Self {
            output,
            scratch,
            root,
        };
        fixture.write("AGENTS.md", "# Agent rules\n");
        fixture.write("CLAUDE.md", "@AGENTS.md\n");
        fixture.write("LICENSE", "Fixture license with no rewrite needed.\n");
        fixture.write("VERSION", "2.4.0\n");
        fixture.write(
            "Cargo.toml",
            "[package]\nname = \"cirnet-rules\"\nversion = \"2.4.0\"\n",
        );
        fixture.write(
            "README.md",
            "# Index\n\n[Guide](en/guide.md#first-heading)\n",
        );
        fixture.pair("AGENTS.md", "# Agent rules\n");
        fixture.pair("guide.md", GUIDE);
        fixture.pair("docs/changelog.md", "# Changelog\n\n## 2.4.0 (candidate)\n");
        fixture.write(
            "vi/docs/changelog.md",
            format!(
                "# Nhật ký thay đổi\n\n## 2.4.0 (candidate)\n\n<!-- source: en/docs/changelog.md -->\n<!-- source-sha256: {} -->\n",
                sha256::hex(b"# Changelog\n\n## 2.4.0 (candidate)\n")
            ),
        );
        for name in ["implementation", "debugging", "verification"] {
            fixture.pair(
                &format!("skills/{name}/SKILL.md"),
                format!(
                    "---\nname: {name}\ndescription: Use when work needs {name}.\n---\n\n# Skill\n"
                ),
            );
        }
        fixture
    }

    fn write(&self, name: &str, content: impl AsRef<[u8]>) {
        let path = self.root.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn pair(&self, name: &str, source: impl AsRef<[u8]>) {
        self.write(&format!("en/{name}"), source.as_ref());
        self.write(
            &format!("vi/{name}"),
            format!(
                "# Bản dịch\n\n<!-- source: en/{name} -->\n<!-- source-sha256: {} -->\n",
                sha256::hex(source.as_ref())
            ),
        );
    }

    #[track_caller]
    fn check(&self, expected: i32) -> String {
        self.check_with(expected, &[])
    }

    #[track_caller]
    fn check_with(&self, expected: i32, args: &[&str]) -> String {
        Self::check_root(&self.root, expected, args)
    }

    #[track_caller]
    fn check_root(root: &Path, expected: i32, args: &[&str]) -> String {
        let result = Command::new(env!("CARGO_BIN_EXE_check-docs"))
            .arg("--root")
            .arg(root)
            .args(args)
            .output()
            .expect("could not run check-docs");
        assert_eq!(
            result.status.code(),
            Some(expected),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        String::from_utf8(result.stdout).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let cleanup = || -> io::Result<()> {
            let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).canonicalize()?;
            let output = self.output.canonicalize()?;
            let scratch = self.scratch.canonicalize()?;
            if output != checkout.join(".superpowers").join("bilingual-tests")
                || scratch != output.join(self.scratch.file_name().unwrap())
            {
                return Err(io::Error::other("unsafe fixture cleanup path"));
            }
            fs::remove_dir_all(scratch)
        };
        if let Err(error) = cleanup() {
            let message = format!(
                "could not clean fixture {}: {error}",
                self.scratch.display()
            );
            if std::thread::panicking() {
                eprintln!("{message}");
            } else {
                panic!("{message}");
            }
        }
    }
}

fn fixture_directory(parent: &Path, name: &str) -> PathBuf {
    let resolved_parent = parent.canonicalize().unwrap();
    let path = parent.join(name);
    match fs::create_dir(&path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => panic!(
            "could not create fixture output {}: {error}",
            path.display()
        ),
    }
    assert!(path.is_dir(), "fixture output must be a directory");
    assert_eq!(
        path.canonicalize().unwrap(),
        resolved_parent.join(name),
        "fixture output escaped its intended path"
    );
    path
}

fn snapshot(root: &Path, directory: &Path, entries: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap().to_path_buf();
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            entries.insert(relative, None);
            snapshot(root, &path, entries);
        } else {
            assert!(kind.is_file(), "unexpected entry at {}", path.display());
            entries.insert(relative, Some(fs::read(path).unwrap()));
        }
    }
}

#[test]
fn valid_tree_is_read_only() {
    let fixture = Fixture::new();
    let mut before = BTreeMap::new();
    snapshot(&fixture.root, &fixture.root, &mut before);
    assert!(!fixture.check(0).contains("WARN:"));
    let mut after = BTreeMap::new();
    snapshot(&fixture.root, &fixture.root, &mut after);
    assert_eq!(after, before);
}

const RETRY_INPUT: &str = "Load connection settings from the owner configuration. Keep the \
    requested endpoint, credential source, timeout and retry budget unchanged. Record the \
    fixture revision and use the existing offline service emulator before comparing results.";
const RETRY_RUN: &str = "Run the request with the fixture's fake clock and inspect the retry \
    event stream. The first failure schedules a bounded delay. A successful retry clears \
    the pending timer. Cancellation must stop the queue without reporting a success.";
const RETRY_LIMIT: &str = "Record compiler and runtime versions with every observation. Report \
    the retry count, terminal result and exact exit code. The emulator verifies the connection \
    workflow without granting access to production credentials or asserting network availability.";

fn retry_document() -> String {
    format!("# Connection retry\n\n{RETRY_INPUT}\n\n{RETRY_RUN}\n\n{RETRY_LIMIT}\n")
}

fn hygiene_findings<'a>(report: &'a str, header: &str) -> Vec<&'a str> {
    report
        .match_indices(header)
        .filter(|(start, _)| *start == 0 || report.as_bytes()[start - 1] == b'\n')
        .map(|(start, _)| {
            let tail = &report[start..];
            let end = ["\nINFO:", "\nWARN:", "\nERROR:", "\nPASS:"]
                .into_iter()
                .filter_map(|prefix| tail.find(prefix))
                .min()
                .unwrap_or(tail.len());
            &tail[..end]
        })
        .collect()
}

#[test]
fn hygiene_exact_duplicate_warns_deterministically_without_changing_files() {
    let fixture = Fixture::new();
    let source = retry_document();
    fixture.pair("docs/connection-retry.md", &source);
    fixture.pair("docs/connection retry copy.md", &source);
    fixture.write(
        "README.md",
        "# Index\n\n[Retry](<en/docs/connection retry copy.md>)\n",
    );
    fixture.write(".superpowers/evidence/prior.md", &source);
    let mut before = BTreeMap::new();
    snapshot(&fixture.root, &fixture.root, &mut before);

    let report = fixture.check(0);
    let findings = hygiene_findings(&report, "WARN: DOC_EXACT_DUPLICATE");
    assert_eq!(findings.len(), 1, "{report}");
    for evidence in [
        "en/docs/connection-retry.md",
        "en/docs/connection retry copy.md",
        "README.md",
        "REVIEW_MERGE",
    ] {
        assert!(
            findings[0].contains(evidence),
            "missing {evidence}: {report}"
        );
    }
    assert!(!report.contains(".superpowers/evidence/prior.md"));
    assert_eq!(report, fixture.check(0));

    let mut after = BTreeMap::new();
    snapshot(&fixture.root, &fixture.root, &mut after);
    assert_eq!(after, before);
}

#[test]
fn hygiene_normalized_duplicate_preserves_code_and_meaningful_values() {
    let fixture = Fixture::new();
    let source = format!(
        "{}\n~~~yaml\nmode: safe\nlimits:\n  retries: 3\n~~~\n\nStatus: PASS\n",
        retry_document()
    );
    fixture.pair("docs/connection-retry.md", &source);
    fixture.pair("docs/connection-copy.md", format!("{source}\n\n"));
    let report = fixture.check(0);
    assert_eq!(
        hygiene_findings(&report, "WARN: DOC_NORMALIZED_DUPLICATE").len(),
        1,
        "{report}"
    );
    assert!(!report.contains("DOC_EXACT_DUPLICATE"), "{report}");

    for changed in [
        source.replace("  retries: 3", "retries: 3"),
        source.replace("retries: 3", "retries: 9"),
        source.replace("Status: PASS", "Status: BLOCKED"),
    ] {
        fixture.pair("docs/connection-copy.md", changed);
        assert!(
            !fixture.check(0).contains("DOC_NORMALIZED_DUPLICATE"),
            "a meaningful change must not be normalized away"
        );
    }
}

#[test]
fn hygiene_reordered_and_reworded_sections_report_measured_similarity() {
    let fixture = Fixture::new();
    fixture.pair("docs/connection-retry.md", retry_document());
    fixture.pair(
        "docs/retry-workflow.md",
        format!(
            "# Connection retry workflow\n\n{RETRY_LIMIT}\n\n{}\n\n{RETRY_INPUT}\n",
            RETRY_RUN.replace("fake clock", "simulated clock")
        ),
    );
    let report = fixture.check(0);
    let findings = hygiene_findings(&report, "WARN: DOC_NEAR_DUPLICATE");
    assert_eq!(findings.len(), 1, "{report}");
    for evidence in [
        "en/docs/connection-retry.md",
        "en/docs/retry-workflow.md",
        "similarity:",
        "reason:",
        "suggestion:",
    ] {
        assert!(
            findings[0].contains(evidence),
            "missing {evidence}: {report}"
        );
    }
}

#[test]
fn hygiene_new_files_distinguish_revisions_from_independent_purpose() {
    let fixture = Fixture::new();
    fixture.pair("docs/connection-retry.md", retry_document());
    fixture.pair(
        "docs/connection-retry-v2.md",
        format!(
            "# Connection retry revised\n\n{RETRY_INPUT}\n\n{RETRY_RUN}\n\n\
             Gather worker latency from the queue telemetry log. Collect enough \
             observations under load to confirm the scheduling interval and distribution, \
             then compare the recorded timestamps with the monotonic clock.\n"
        ),
    );
    fixture.pair(
        "docs/connection-retry-final.md",
        "# Map archive publication\n\nEnsure catalog pages identify map editions and scales. \
         Preserve historical sheets in their original folders. Verify that search filters \
         can select a region and export its public map metadata. Confirm that printed legends \
         and shoreline symbols remain readable at each cartographic scale.\n",
    );
    fixture.pair(
        "docs/independent exports.md",
        "# Export map entries\n\nSpecify the destination format and license.\n",
    );
    let report = fixture.check_with(
        0,
        &[
            "--new-file",
            "en/docs/connection-retry-v2.md",
            "--new-file=en/docs/connection-retry-final.md",
            "--new-file",
            "en/docs/independent exports.md",
        ],
    );
    let updates: Vec<_> = hygiene_findings(&report, "WARN: DOC_NEAR_DUPLICATE")
        .into_iter()
        .filter(|finding| finding.contains("suggestion: UPDATE_EXISTING"))
        .collect();
    assert_eq!(updates.len(), 1, "{report}");
    assert!(
        updates[0].contains("en/docs/connection-retry.md"),
        "{report}"
    );
    assert!(
        updates[0].contains("en/docs/connection-retry-v2.md"),
        "{report}"
    );
    let reviews = hygiene_findings(&report, "INFO: NEW_DOCUMENT_REVIEW");
    assert_eq!(reviews.len(), 2, "{report}");
    for path in [
        "en/docs/connection-retry-final.md",
        "en/docs/independent exports.md",
    ] {
        assert!(
            reviews.iter().any(|finding| finding.contains(path)),
            "missing review for {path}: {report}"
        );
    }
    assert!(!report.contains("DOC_NEW_DUPLICATE"), "{report}");
}

#[test]
fn hygiene_new_exact_copy_fails_for_the_named_path() {
    let fixture = Fixture::new();
    let source = retry_document();
    fixture.pair("docs/connection-retry.md", &source);
    fixture.pair("docs/connection-retry-final.md", &source);
    let report = fixture.check_with(1, &["--new-file", "en/docs/connection-retry-final.md"]);
    let findings = hygiene_findings(&report, "ERROR: DOC_NEW_DUPLICATE");
    assert_eq!(findings.len(), 1, "{report}");
    for evidence in [
        "en/docs/connection-retry.md",
        "en/docs/connection-retry-final.md",
        "UPDATE_EXISTING",
    ] {
        assert!(
            findings[0].contains(evidence),
            "missing {evidence}: {report}"
        );
    }
}

#[test]
fn hygiene_keeps_intentional_copies_in_separate_roles() {
    let fixture = Fixture::new();
    let source = retry_document();
    fixture.pair("docs/connection-retry.md", &source);
    let original_vi = fs::read_to_string(fixture.root.join("vi/docs/connection-retry.md")).unwrap();
    fixture.write(
        "vi/docs/connection-retry.md",
        original_vi.replace("# Bản dịch\n", &source),
    );
    fixture.pair("plans/network/plan.md", &source);
    fixture.pair("specs/network/spec.md", &source);
    fixture.pair("templates/retry.md", &source);
    fixture.pair("docs/samples/retry.md", &source);
    fixture.pair("docs/fixtures/retry.md", &source);
    fixture.pair("docs/components/README.md", "# Setup\n");
    fixture.pair("docs/tools/README.md", "# Setup\n");
    fixture.pair(
        "docs/paths.md",
        "# Setup\n\nFollow the local testing policy.\n",
    );
    fixture.pair(
        "docs/tooling.md",
        "# Tools\n\nFollow the local testing policy.\n",
    );
    fixture.pair(
        "docs/thermometer.md",
        format!("# Read thermometer\n\n{RETRY_RUN}\n\nRecord cold junction offsets.\n"),
    );
    fixture.pair(
        "docs/wheel.md",
        format!("# Check wheels\n\n{RETRY_RUN}\n\nInspect each rim visually.\n"),
    );
    fixture.write("tests/fixtures/retry.md", &source);
    let report = fixture.check_with(
        0,
        &[
            "--new-file",
            "en/AGENTS.md",
            "--new-file=CLAUDE.md",
            "--new-file=en/templates/retry.md",
        ],
    );
    assert!(!report.contains("DOC_EXACT_DUPLICATE"), "{report}");
    assert!(!report.contains("DOC_NORMALIZED_DUPLICATE"), "{report}");
    assert!(!report.contains("DOC_NEAR_DUPLICATE"), "{report}");
    assert!(!report.contains("DOC_NEW_DUPLICATE"), "{report}");
}

#[test]
fn hygiene_historical_reports_are_advisory_and_stay_intact() {
    let fixture = Fixture::new();
    let source = retry_document();
    fixture.pair(
        "docs/history/retry-2026-04-01.md",
        format!("{source}\nRevision r01, verification FAIL.\n"),
    );
    let latest = format!("{source}\nRevision r02, verification PASS.\n");
    fixture.pair("docs/history/retry-2026-04-02.md", &latest);
    fixture.pair("docs/history/retry-2026-04-03.md", &latest);
    let mut before = BTreeMap::new();
    snapshot(&fixture.root, &fixture.root, &mut before);
    let report = fixture.check_with(0, &["--new-file", "en/docs/history/retry-2026-04-03.md"]);
    assert!(!report.contains("DOC_NEW_DUPLICATE"), "{report}");
    assert!(!report.contains("UPDATE_EXISTING"), "{report}");
    assert!(report.contains("KEEP_SEPARATE"), "{report}");
    let mut after = BTreeMap::new();
    snapshot(&fixture.root, &fixture.root, &mut after);
    assert_eq!(after, before);
}

#[test]
fn hygiene_new_file_selector_rejects_paths_outside_the_document_inventory() {
    let fixture = Fixture::new();
    fixture.write("tests/guide.md", GUIDE);
    fixture.write(".superpowers/private.md", GUIDE);
    for invalid in [
        "../outside.md",
        "en/../README.md",
        "/README.md",
        "C:\\outside.md",
        "en/missing.md",
        "tests/guide.md",
        ".superpowers/private.md",
        "LICENSE",
    ] {
        fixture.check_with(2, &["--new-file", invalid]);
    }
    fixture.check_with(2, &["--new-file"]);
}

#[test]
fn missing_translation_fails() {
    let fixture = Fixture::new();
    fs::remove_file(fixture.root.join("vi/guide.md")).unwrap();
    fixture.check(1);
}

#[test]
fn orphan_translation_fails() {
    let fixture = Fixture::new();
    fixture.write("vi/orphan.md", "# Orphan\n");
    fixture.check(1);
}

#[test]
fn stale_source_hash_fails() {
    let fixture = Fixture::new();
    fixture.write("en/guide.md", format!("{GUIDE}\nChanged.\n"));
    fixture.check(1);
}

#[test]
fn wrong_source_metadata_fails() {
    let fixture = Fixture::new();
    let original = fs::read_to_string(fixture.root.join("vi/guide.md")).unwrap();
    fixture.write(
        "vi/guide.md",
        original.replace("source: en/guide.md", "source: en/AGENTS.md"),
    );
    fixture.check(1);
}

#[test]
fn missing_or_nonterminal_metadata_fails() {
    let fixture = Fixture::new();
    let original = fs::read_to_string(fixture.root.join("vi/guide.md")).unwrap();
    fixture.write("vi/guide.md", "# Missing stamps\n");
    fixture.check(1);
    fixture.write("vi/guide.md", original + "After metadata.\n");
    fixture.check(1);
}

#[test]
fn metadata_examples_do_not_override_terminal_pair() {
    let fixture = Fixture::new();
    let original = fs::read_to_string(fixture.root.join("vi/guide.md")).unwrap();
    fixture.write(
        "vi/guide.md",
        format!(
            "```md\n<!-- source: en/example.md -->\n<!-- source-sha256: example -->\n```\n\n{original}"
        ),
    );
    fixture.check(0);
}

#[test]
fn broken_link_fails() {
    let fixture = Fixture::new();
    fixture.write("README.md", "[Missing](en/missing.md)\n");
    fixture.check(1);
}

#[test]
fn broken_fragment_fails() {
    let fixture = Fixture::new();
    fixture.write("README.md", "[Missing](en/guide.md#missing)\n");
    fixture.check(1);
}

#[test]
fn reference_links_and_encoded_fragment() {
    let fixture = Fixture::new();
    fixture.pair("guide.md", "# Tiếng Việt\n\n## Same\n\n## Same\n");
    fixture.write(
        "README.md",
        "[Read][guide]\n\n[guide]: en/guide.md#ti%E1%BA%BFng-vi%E1%BB%87t\n[Again](en/guide.md#same-1)\n",
    );
    fixture.check(0);
    fixture.write("README.md", "[Read][guide]\n\n[guide]: en/missing.md\n");
    fixture.check(1);
    fixture.write(
        "README.md",
        "[Read][guide]\n\n[guide]: en/missing.md\n[GUIDE]: en/guide.md\n",
    );
    fixture.check(1);
}

#[test]
fn missing_reference_definition_fails() {
    let fixture = Fixture::new();
    fixture.write("README.md", "[Read][unknown]\n");
    fixture.check(1);
}

#[test]
fn heading_inline_code_keeps_its_text_in_fragment() {
    let fixture = Fixture::new();
    fixture.pair(
        "guide.md",
        "# Use `command` here\n\nSetext heading\n--------------\n",
    );
    fixture.write(
        "README.md",
        "[Code heading](en/guide.md#use-command-here)\n[Setext](en/guide.md#setext-heading)\n",
    );
    fixture.check(0);
}

#[test]
fn frontmatter_is_not_a_heading() {
    let fixture = Fixture::new();
    fixture.write(
        "README.md",
        "[Metadata](en/skills/debugging/SKILL.md#description-use-when-work-needs-debugging)\n",
    );
    fixture.check(1);
}

#[test]
fn skill_frontmatter_requires_two_supported_single_line_strings() {
    let fixture = Fixture::new();
    for source in [
        "# Missing metadata\n",
        "---\nname: debugging\ndescription: Debug work.\nextra: value\n---\n",
        "---\nname: wrong-name\ndescription: Debug work.\n---\n",
        "---\nname: debugging\ndescription: [nested, values]\n---\n",
        "---\nname: debugging\ndescription:  [nested, values]\n---\n",
        "---\nname: debugging\ndescription: - nested value\n---\n",
        "---\nname: debugging\ndescription: true\n---\n",
        "---\nname: debugging\ndescription: .NaN\n---\n",
        "---\nname: debugging\ndescription: -3.14e5\n---\n",
        "---\nname: debugging\ndescription: 0xFF\n---\n",
        "---\nname: debugging\ndescription: \n---\n",
    ] {
        fixture.pair("skills/debugging/SKILL.md", source);
        fixture.check(1);
    }
}

#[test]
fn fourth_skill_and_ordinary_neighbor_directory_are_accepted() {
    let fixture = Fixture::new();
    fixture.pair(
        "skills/extra/SKILL.md",
        "---\nname: extra\ndescription: Extra.\n---\n",
    );
    fixture.pair("skills/notes/overview.md", "# Library notes\n");
    fixture.check(0);
}

#[test]
fn misnamed_or_nested_skill_file_is_rejected() {
    let fixture = Fixture::new();
    fixture.pair("skills/misc/skill.md", "# Wrong case\n");
    assert!(fixture.check(1).contains("exactly SKILL.md"));
    fs::remove_file(fixture.root.join("en/skills/misc/skill.md")).unwrap();
    fs::remove_file(fixture.root.join("vi/skills/misc/skill.md")).unwrap();
    fixture.pair("skills/misc/nested/SKILL.md", "# Wrong depth\n");
    assert!(fixture.check(1).contains("direct skill directory"));
}

#[test]
fn skill_name_and_description_limits_are_enforced() {
    for name in ["Mixed-Case", "double--hyphen", "-leading", "a_b", "ending-"] {
        let fixture = Fixture::new();
        fixture.pair(
            &format!("skills/{name}/SKILL.md"),
            format!("---\nname: {name}\ndescription: Description.\n---\n"),
        );
        assert!(fixture.check(1).contains("invalid skill name"), "{name}");
    }

    let fixture = Fixture::new();
    let name = "a".repeat(65);
    fixture.pair(
        &format!("skills/{name}/SKILL.md"),
        format!("---\nname: {name}\ndescription: Description.\n---\n"),
    );
    assert!(fixture.check(1).contains("invalid skill name"));
    let fixture = Fixture::new();
    let description = "é".repeat(1025);
    fixture.pair(
        "skills/testing/SKILL.md",
        format!("---\nname: testing\ndescription: {description}\n---\n"),
    );
    assert!(fixture.check(1).contains("1024 characters"));
}

#[test]
fn quoted_string_for_numeric_skill_name_is_accepted() {
    let fixture = Fixture::new();
    fixture.pair(
        "skills/123/SKILL.md",
        "---\nname: '123'\ndescription: 'Choose this for rule 123: its checks are focused.'\n---\n",
    );
    fixture.check(0);
}

#[test]
fn duplicate_skill_metadata_name_is_rejected() {
    let fixture = Fixture::new();
    fixture.pair(
        "skills/z-copy/SKILL.md",
        "---\nname: debugging\ndescription: A second skill using the same name.\n---\n",
    );
    assert!(fixture.check(1).contains("duplicate skill name"));
}

#[test]
fn skill_name_must_match_its_folder() {
    let fixture = Fixture::new();
    fixture.pair(
        "skills/portable/SKILL.md",
        "---\nname: mismatch\ndescription: Description.\n---\n",
    );
    assert!(fixture.check(1).contains("must match the parent directory"));
}

#[test]
fn skill_resource_can_use_portable_local_links() {
    let fixture = Fixture::new();
    fixture.pair(
        "skills/debugging/SKILL.md",
        "---\nname: debugging\ndescription: Debug work.\n---\n\n[Read](<references/Tiếng Việt.md#details>)\n",
    );
    fixture.pair(
        "skills/debugging/references/Tiếng Việt.md",
        "# Details\n\n![Diagram](../assets/diagram.txt)\n\n[Web](https://example.invalid/docs)\n",
    );
    fixture.write("en/skills/debugging/assets/diagram.txt", "Diagram\n");
    fixture.check(0);
    fs::remove_file(fixture.root.join("en/skills/debugging/assets/diagram.txt")).unwrap();
    let report = fixture.check(1);
    assert!(report.contains("references/Tiếng Việt.md"));
    assert!(report.contains("target does not exist"));
}

#[test]
fn skill_resource_cannot_escape_its_packaged_directory() {
    let fixture = Fixture::new();
    fixture.write("en/skills/debugging/assets/diagram.txt", "Diagram\n");
    for target in [
        "../../guide.md",
        "%2e%2e/%2e%2e/guide.md",
        "../debugging/SKILL.md",
        "/en/skills/debugging/SKILL.md",
        "C:/checkout/guide.md",
        "file:///checkout/guide.md",
        "assets%5cdiagram.txt",
        "diagram.txt:stream",
        "assets/diagram.txt:stream",
        "assets/COM1.txt",
        "assets//diagram.txt",
        "assets/",
        "assets",
    ] {
        fixture.pair(
            "skills/debugging/SKILL.md",
            format!("---\nname: debugging\ndescription: Debug work.\n---\n\n[Read][r]\n\n[r]: {target}\n"),
        );
        let report = fixture.check(1);
        assert!(
            report.contains("portable skill resource"),
            "{target}: {report}"
        );
    }
}

#[test]
fn eager_import_warning_counts_distinct_skills() {
    let fixture = Fixture::new();
    let repeated = "# Agent rules\n\n@en/skills/debugging/SKILL.md\n@en/skills/debugging/SKILL.md\n@en/skills/debugging/SKILL.md\n";
    fixture.write("AGENTS.md", repeated);
    fixture.pair("AGENTS.md", repeated);
    let report = fixture.check(0);
    assert!(report.contains("duplicate startup"));
    assert!(!report.contains("eager startup"));
    let eager = "# Agent rules\n\n@en/skills/debugging/SKILL.md\n@en/skills/implementation/SKILL.md\n@en/skills/verification/SKILL.md\n";
    fixture.write("AGENTS.md", eager);
    fixture.pair("AGENTS.md", eager);
    let report = fixture.check(0);
    assert!(report.contains("eager startup imports name every Skill"));
}

#[test]
fn skill_resource_path_is_case_sensitive_on_every_platform() {
    let fixture = Fixture::new();
    fixture.write("en/skills/debugging/assets/Diagram.txt", "Diagram\n");
    fixture.pair(
        "skills/debugging/SKILL.md",
        "---\nname: debugging\ndescription: Debug work.\n---\n\n![Diagram](assets/diagram.txt)\n",
    );
    let report = fixture.check(1);
    assert!(report.contains("portable skill resource"));
}

#[test]
fn version_file_must_be_one_canonical_triplet() {
    let fixture = Fixture::new();
    fixture.write("VERSION", "2.04.0\n");
    let report = fixture.check(1);
    assert!(report.contains("ERROR: VERSION:"));
    assert!(report.contains("MAJOR.MINOR.PATCH"));
}

#[test]
fn version_is_checked_against_selected_checkout_manifest() {
    let fixture = Fixture::new();
    fixture.write(
        "Cargo.toml",
        "[package]\nversion = \"2.3.0\"\n[dependencies.other]\nversion = \"2.4.0\"\n",
    );
    let report = fixture.check(1);
    assert!(report.contains("ERROR: Cargo.toml:"));
    assert!(report.contains("VERSION"));
}

#[test]
fn both_changelogs_must_start_with_the_toolkit_version() {
    let fixture = Fixture::new();
    fixture.pair(
        "docs/changelog.md",
        "# Changelog\n\n```md\n## 2.4.0\n```\n\n## 2.3.0\n\n## 2.4.0\n",
    );
    let report = fixture.check(1);
    for prefix in ["en", "vi"] {
        assert!(report.contains(&format!("ERROR: {prefix}/docs/changelog.md:")));
    }
}

#[test]
fn context_budget_warnings_remain_advisory() {
    let fixture = Fixture::new();
    let baseline = format!("# Agent rules\n\n{}!\n", "word ".repeat(510));
    fixture.write("AGENTS.md", &baseline);
    fixture.pair("AGENTS.md", baseline);
    fixture.pair(
        "skills/README.md",
        format!("# Router\n\n{}!\n", "word ".repeat(310)),
    );
    fixture.pair(
        "skills/debugging/SKILL.md",
        format!(
            "---\nname: debugging\ndescription: {}\n---\n\n#!\n{}!\n",
            "x".repeat(241),
            "word ".repeat(901)
        ),
    );
    let report = fixture.check(0);
    for path in [
        "AGENTS.md",
        "en/skills/README.md",
        "en/skills/debugging/SKILL.md",
    ] {
        assert!(report.contains(&format!("WARN: {path}:")), "{report}");
    }
    assert!(report.contains("INFO: en/skills: metadata total"));
}

#[test]
fn repeated_imports_and_conflicting_rules_warn_without_failing() {
    let fixture = Fixture::new();
    let paragraph = "Before completing work, review the project test results with the person who approved the work, then record any missing evidence in the report for the people who will maintain this project.";
    let baseline = format!(
        "# Agent rules\n\n@en/AGENTS.md\n@en/skills/\n\n{paragraph}\n\n{paragraph}\n\nAlways run the local test.\n\nNever run the local test.\n"
    );
    fixture.write("AGENTS.md", &baseline);
    fixture.pair("AGENTS.md", baseline);
    fixture.write("AGENTS.override.md", "# User override\n");
    let report = fixture.check(0);
    for reason in [
        "duplicate startup",
        "eager startup",
        "repeated prose",
        "possible conflicting",
        "override",
    ] {
        assert!(
            report
                .lines()
                .any(|line| line.starts_with("WARN:") && line.contains(reason)),
            "missing {reason}: {report}"
        );
    }
}

#[test]
fn example_imports_and_rule_text_in_code_blocks_do_not_warn() {
    let fixture = Fixture::new();
    let baseline = "# Example\n\n```md\n@en/skills/\n@en/AGENTS.md\nAlways open all documents.\nNever open all documents.\n```\n";
    fixture.write("AGENTS.md", baseline);
    fixture.pair("AGENTS.md", baseline);
    assert!(!fixture.check(0).contains("WARN:"));
}

#[test]
fn root_agent_mirror_drift_fails() {
    let fixture = Fixture::new();
    fixture.write("AGENTS.md", "# Drift\n");
    fixture.check(1);
}

#[test]
fn adapter_is_exact() {
    let fixture = Fixture::new();
    fixture.write("CLAUDE.md", "@en/AGENTS.md\n");
    fixture.check(1);
}

#[test]
fn invalid_utf8_fails() {
    let fixture = Fixture::new();
    fixture.write("README.md", b"# Invalid \xff\n");
    fixture.check(1);
}

#[test]
fn crlf_and_trailing_whitespace_fail() {
    let fixture = Fixture::new();
    for content in [
        b"# CRLF\r\n".as_slice(),
        b"# Trailing \n",
        b"# No final newline",
    ] {
        fixture.write("README.md", content);
        fixture.check(1);
    }
}

#[test]
fn code_and_external_links_are_ignored() {
    let fixture = Fixture::new();
    fixture.write(
        "README.md",
        "# Examples\n\n`[ignored](missing.md)`\n\n\
         ```md\n[ignored](missing.md)\n```\n\n\
         ~~~md\n[ignored](missing.md)\n~~~\n\n\
         [Web](https://example.invalid/missing)\n[Email](mailto:someone@example.invalid)\n",
    );
    fixture.check(0);
}

#[test]
fn directory_and_non_markdown_targets_exist() {
    let fixture = Fixture::new();
    fixture.write("asset.txt", "Asset\n");
    fixture.write("README.md", "[Directory](en/skills/)\n[Asset](asset.txt)\n");
    fixture.check(0);
}

#[test]
fn traversal_outside_selected_root_fails() {
    let fixture = Fixture::new();
    fs::write(fixture.scratch.join("outside.md"), "# Exists\n").unwrap();
    for target in ["../outside.md", "%2e%2e/outside.md", "..%5coutside.md"] {
        fixture.write("README.md", format!("[Escape]({target})\n"));
        fixture.check(1);
    }
}

#[cfg(any(unix, windows))]
#[test]
#[cfg_attr(
    windows,
    ignore = "requires permission to create a Windows directory symlink"
)]
fn external_symlink_is_rejected() {
    let fixture = Fixture::new();
    let outside = fixture.scratch.join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("hidden.txt"), "Outside\n").unwrap();
    let link = fixture.root.join("en/escape");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, &link).expect("could not create directory symlink");
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&outside, &link).expect("could not create directory symlink");
    fixture.check(1);
}

#[test]
fn excluded_trees_are_not_scanned() {
    let fixture = Fixture::new();
    fixture.write(".superpowers/ignored.md", b"\xff");
    fixture.write(".git/ignored.md", b"\xff");
    fixture.check(0);
}

#[test]
fn mixed_case_excluded_links_are_rejected() {
    let fixture = Fixture::new();
    for directory in [".SUPERPOWERS", ".Git"] {
        fixture.write(&format!("{directory}/hidden.md"), "# Hidden\n");
        fixture.write("README.md", format!("[Excluded]({directory}/hidden.md)\n"));
        fixture.check(1);
    }
}

#[test]
fn nested_mixed_case_excluded_trees_are_not_scanned() {
    let fixture = Fixture::new();
    for language in ["en", "vi"] {
        for directory in [".SUPERPOWERS", ".Git"] {
            fixture.write(&format!("{language}/nested/{directory}/hidden.md"), b"\xff");
        }
    }
    fixture.check(0);
}

#[test]
fn invalid_root_exits_two() {
    let fixture = Fixture::new();
    Fixture::check_root(&fixture.root.join("does-not-exist"), 2, &[]);
}
