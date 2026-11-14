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
        fixture.write(
            "README.md",
            "# Index\n\n[Guide](en/guide.md#first-heading)\n",
        );
        fixture.pair("AGENTS.md", "# Agent rules\n");
        fixture.pair("guide.md", GUIDE);
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
    fn check(&self, expected: i32) {
        Self::check_root(&self.root, expected);
    }

    #[track_caller]
    fn check_root(root: &Path, expected: i32) {
        let result = Command::new(env!("CARGO_BIN_EXE_check-docs"))
            .arg("--root")
            .arg(root)
            .output()
            .expect("could not run check-docs");
        assert_eq!(
            result.status.code(),
            Some(expected),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
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
    fixture.check(0);
    let mut after = BTreeMap::new();
    snapshot(&fixture.root, &fixture.root, &mut after);
    assert_eq!(after, before);
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
fn skill_frontmatter_requires_only_plain_name_and_description() {
    let fixture = Fixture::new();
    for source in [
        "# Missing metadata\n",
        "---\nname: debugging\ndescription: Debug work.\nextra: value\n---\n",
        "---\nname: wrong-name\ndescription: Debug work.\n---\n",
        "---\nname: debugging\ndescription: [nested, values]\n---\n",
        "---\nname: debugging\ndescription: - nested value\n---\n",
        "---\nname: debugging\ndescription: true\n---\n",
        "---\nname: debugging\ndescription: \n---\n",
    ] {
        fixture.pair("skills/debugging/SKILL.md", source);
        fixture.check(1);
    }
}

#[test]
fn skill_directory_set_is_exact() {
    let fixture = Fixture::new();
    fixture.pair(
        "skills/extra/SKILL.md",
        "---\nname: extra\ndescription: Extra.\n---\n",
    );
    fixture.check(1);
}

#[test]
fn skill_file_is_required() {
    let fixture = Fixture::new();
    fs::remove_file(fixture.root.join("en/skills/debugging/SKILL.md")).unwrap();
    fs::remove_file(fixture.root.join("vi/skills/debugging/SKILL.md")).unwrap();
    fixture.check(1);
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
    Fixture::check_root(&fixture.root.join("does-not-exist"), 2);
}
