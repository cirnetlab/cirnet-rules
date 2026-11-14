mod markdown;
mod sha256;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, Metadata};
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

const SKILLS: [&str; 3] = ["debugging", "implementation", "verification"];
const USAGE: &str = "Usage: check-docs [--root PATH]";
const HELP: &str = "+Read-only, offline bilingual documentation check.

Checks root Markdown and the en/vi trees. Supports simple inline links,
images, reference definitions/uses and ATX/Setext heading fragments.
External links, code spans, fenced code and HTML comments are ignored.
Nested brackets/parentheses, multiline destinations, HTML links/anchors
and renderer-specific anchor rules are outside this subset.

Use --root PATH to select a checkout. The default is the build checkout.
Exit 0: PASS. Exit 1: failed checks. Exit 2: invalid invocation.";

fn excluded(name: &str) -> bool {
    name.eq_ignore_ascii_case(".git") || name.eq_ignore_ascii_case(".superpowers")
}

fn is_redirect(metadata: &Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const REPARSE_POINT: u32 = 0x400;
        metadata.file_attributes() & REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

fn is_markdown(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn percent_decode(text: &str) -> String {
    let mut bytes = Vec::with_capacity(text.len());
    let mut input = text.bytes();
    while let Some(byte) = input.next() {
        if byte == b'%' {
            let mut lookahead = input.clone();
            let high = lookahead
                .next()
                .and_then(|byte| (byte as char).to_digit(16));
            let low = lookahead
                .next()
                .and_then(|byte| (byte as char).to_digit(16));
            if let (Some(high), Some(low)) = (high, low) {
                bytes.push((high * 16 + low) as u8);
                input = lookahead;
                continue;
            }
        }
        bytes.push(byte);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn external(destination: &str) -> bool {
    destination.starts_with("//")
        || destination.split_once(':').is_some_and(|(scheme, _)| {
            scheme
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphabetic)
                && scheme
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'.' | b'-'))
        })
}

fn link_target(source: &Path, name: &str) -> Result<PathBuf, String> {
    if name.is_empty() {
        return Ok(source.to_path_buf());
    }
    let (mut target, name) = if name.starts_with('/') {
        (PathBuf::new(), name.trim_start_matches('/'))
    } else {
        (source.parent().unwrap_or(Path::new("")).to_path_buf(), name)
    };
    for part in Path::new(name).components() {
        match part {
            Component::Normal(name) => target.push(name),
            Component::CurDir => {}
            Component::ParentDir if target.pop() => {}
            _ => return Err("target leaves selected root".into()),
        }
    }
    Ok(target)
}

fn stamp(text: &str) -> Option<(&str, &str)> {
    let mut lines = text.strip_suffix('\n').unwrap_or(text).rsplit('\n');
    let hash = lines
        .next()?
        .strip_prefix("<!-- source-sha256: ")?
        .strip_suffix(" -->")?;
    let source = lines
        .next()?
        .strip_prefix("<!-- source: ")?
        .strip_suffix(" -->")?;
    if hash.len() == 64
        && hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && source.starts_with("en/")
        && source.ends_with(".md")
    {
        Some((source, hash))
    } else {
        None
    }
}

struct Checker {
    root: PathBuf,
    documents: BTreeMap<PathBuf, String>,
    errors: Vec<String>,
}

impl Checker {
    fn fail(&mut self, path: &Path, message: impl std::fmt::Display) {
        self.errors.push(format!("{}: {message}", path_text(path)));
    }

    // All paths inside the checker are relative to the canonical checkout.
    // Inspect each component before reading a file or descending into a directory.
    fn metadata(&self, path: &Path) -> Result<Metadata, String> {
        let mut absolute = self.root.clone();
        let mut metadata = None;
        for part in path.components() {
            let Component::Normal(name) = part else {
                return Err("target leaves selected root".into());
            };
            if excluded(&name.to_string_lossy()) {
                return Err("target enters an excluded directory".into());
            }
            absolute.push(name);
            let entry = fs::symlink_metadata(&absolute).map_err(|error| {
                if error.kind() == io::ErrorKind::NotFound {
                    "target does not exist".into()
                } else {
                    error.to_string()
                }
            })?;
            if is_redirect(&entry) {
                return Err("symlink/reparse point is not allowed".into());
            }
            metadata = Some(entry);
        }
        metadata.map_or_else(|| fs::metadata(absolute).map_err(|e| e.to_string()), Ok)
    }

    fn entries(&self, directory: &Path) -> Result<Vec<PathBuf>, String> {
        let mut entries = fs::read_dir(self.root.join(directory))
            .and_then(|entries| {
                entries
                    .map(|entry| entry.map(|entry| directory.join(entry.file_name())))
                    .collect::<io::Result<Vec<_>>>()
            })
            .map_err(|error| error.to_string())?;
        entries.sort();
        Ok(entries)
    }

    fn collect(&mut self, directory: &Path, files: &mut BTreeSet<PathBuf>) {
        let entries = match self
            .metadata(directory)
            .and_then(|_| self.entries(directory))
        {
            Ok(entries) => entries,
            Err(error) => {
                self.fail(directory, error);
                return;
            }
        };
        for path in entries {
            if path
                .file_name()
                .is_some_and(|name| excluded(&name.to_string_lossy()))
            {
                continue;
            }
            match self.metadata(&path) {
                Ok(metadata) if metadata.is_dir() => self.collect(&path, files),
                Ok(_) if is_markdown(&path) => {
                    files.insert(path);
                }
                Ok(_) => {}
                Err(error) => self.fail(&path, error),
            }
        }
    }

    fn read(&mut self, path: &Path) -> Option<&str> {
        if !self.documents.contains_key(path) {
            let content = self.metadata(path).and_then(|metadata| {
                if !metadata.is_file() {
                    return Err("expected a regular Markdown file".into());
                }
                fs::read_to_string(self.root.join(path)).map_err(|error| error.to_string())
            });
            let text = match content {
                Ok(text) => text,
                Err(error) => {
                    self.fail(path, error);
                    return None;
                }
            };
            if text.contains('\r') || (!text.is_empty() && !text.ends_with('\n')) {
                self.fail(path, "use LF line endings and a final newline");
            }
            if text.lines().any(|line| line.ends_with([' ', '\t'])) {
                self.fail(path, "trailing whitespace");
            }
            self.documents.insert(path.to_path_buf(), text);
        }
        self.documents.get(path).map(String::as_str)
    }

    fn translations(&mut self, english: &BTreeSet<PathBuf>, vietnamese: &BTreeSet<PathBuf>) {
        for source in english {
            let translation = Path::new("vi").join(source.strip_prefix("en").unwrap());
            if !vietnamese.contains(&translation) {
                self.fail(source, "missing Vietnamese translation");
                continue;
            }
            let metadata = self
                .documents
                .get(&translation)
                .and_then(|text| stamp(text));
            match metadata {
                None => self.fail(
                    &translation,
                    "end with source and source-sha256 metadata comments",
                ),
                Some((path, _)) if path != path_text(source) => {
                    self.fail(
                        &translation,
                        "source metadata does not match paired English path",
                    );
                }
                Some((_, hash)) => {
                    if self
                        .documents
                        .get(source)
                        .is_some_and(|text| sha256::hex(text.as_bytes()) != hash)
                    {
                        self.fail(&translation, "source hash is stale");
                    }
                }
            }
        }
        for translation in vietnamese {
            let source = Path::new("en").join(translation.strip_prefix("vi").unwrap());
            if !english.contains(&source) {
                self.fail(translation, "orphan Vietnamese translation");
            }
        }
    }

    fn entrypoints(&mut self) {
        let baseline = Path::new("AGENTS.md");
        let english = Path::new("en/AGENTS.md");
        let adapter = Path::new("CLAUDE.md");
        for path in [baseline, english, adapter] {
            if !self.documents.contains_key(path) {
                self.fail(path, "required readable Markdown file is missing");
            }
        }
        if self.documents.get(baseline) != self.documents.get(english) {
            self.fail(baseline, "must exactly mirror en/AGENTS.md");
        }
        if self.documents.get(adapter).map(String::as_str) != Some("@AGENTS.md\n") {
            self.fail(adapter, "must contain exactly @AGENTS.md and LF");
        }
    }

    fn skills(&mut self) {
        let directory = Path::new("en/skills");
        let directories = self.metadata(directory).and_then(|_| {
            self.entries(directory)?
                .into_iter()
                .filter_map(|path| match self.metadata(&path) {
                    Ok(metadata) if metadata.is_dir() => {
                        Some(Ok(path.file_name().unwrap().to_string_lossy().into_owned()))
                    }
                    Ok(_) => None,
                    Err(error) => Some(Err(error)),
                })
                .collect::<Result<BTreeSet<_>, _>>()
        });
        match directories {
            Ok(names) if names.iter().map(String::as_str).eq(SKILLS) => {}
            Ok(_) => self.fail(
                directory,
                "expected exactly implementation, debugging, verification directories",
            ),
            Err(error) => self.fail(directory, error),
        }
        for name in SKILLS {
            let path = directory.join(name).join("SKILL.md");
            if !self
                .documents
                .get(&path)
                .is_some_and(|text| markdown::valid_skill(text, name))
            {
                self.fail(
                    &path,
                    "require plain name/description frontmatter matching the skill directory",
                );
            }
        }
    }

    fn validate_link(&mut self, source: &Path, destination: &str) -> Result<(), String> {
        let destination = destination.trim_matches(['<', '>']);
        if external(destination) {
            return Ok(());
        }
        let (url, fragment) = destination.split_once('#').unwrap_or((destination, ""));
        let path = url.split_once('?').map_or(url, |(path, _)| path);
        let name = percent_decode(path).replace('\\', "/");
        let target = link_target(source, &name)?;
        let metadata = self.metadata(&target)?;
        if !fragment.is_empty() {
            if !is_markdown(&target) || !metadata.is_file() {
                return Err("fragment target must be Markdown".into());
            }
            let text = self.read(&target).ok_or("could not read Markdown target")?;
            if !markdown::anchors(text).contains(&percent_decode(fragment)) {
                return Err("heading fragment does not exist".into());
            }
        }
        Ok(())
    }

    fn run(&mut self) -> Result<(), String> {
        let root_files = self.entries(Path::new(""))?;
        let mut english = BTreeSet::new();
        let mut vietnamese = BTreeSet::new();
        self.collect(Path::new("en"), &mut english);
        self.collect(Path::new("vi"), &mut vietnamese);
        for path in root_files
            .iter()
            .filter(|path| is_markdown(path))
            .chain(&english)
            .chain(&vietnamese)
        {
            self.read(path);
        }
        self.translations(&english, &vietnamese);
        self.entrypoints();
        self.skills();
        let documents: Vec<_> = self.documents.keys().cloned().collect();
        for path in documents {
            let (links, undefined) = markdown::links(&self.documents[&path]);
            for destination in links {
                if let Err(error) = self.validate_link(&path, &destination) {
                    self.fail(&path, format!("invalid link {destination:?}: {error}"));
                }
            }
            for label in undefined {
                self.fail(&path, format!("undefined link reference {label:?}"));
            }
        }
        Ok(())
    }
}

fn usage_error(message: impl std::fmt::Display) -> ExitCode {
    eprintln!("{USAGE}\ncheck-docs: {message}");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut args = env::args_os().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            println!("{USAGE}\n\n{HELP}");
            return ExitCode::SUCCESS;
        }
        if arg == "--root" {
            match args.next() {
                Some(value) if !value.is_empty() && !value.to_string_lossy().starts_with('-') => {
                    root = PathBuf::from(value);
                }
                _ => return usage_error("--root requires a path"),
            }
        } else if let Some(value) = arg.to_str().and_then(|arg| arg.strip_prefix("--root=")) {
            if value.is_empty() {
                return usage_error("--root requires a path");
            }
            root = PathBuf::from(value);
        } else {
            return usage_error(format!("unknown argument {arg:?}"));
        }
    }
    let root = match fs::canonicalize(&root) {
        Ok(root) if root.is_dir() => root,
        Ok(_) => return usage_error("root must be a directory"),
        Err(error) => return usage_error(error),
    };
    let mut checker = Checker {
        root,
        documents: BTreeMap::new(),
        errors: Vec::new(),
    };
    if let Err(error) = checker.run() {
        checker.errors.push(error);
    }
    if checker.errors.is_empty() {
        println!("PASS: bilingual documentation checks");
        ExitCode::SUCCESS
    } else {
        for error in checker.errors {
            println!("FAIL: {error}");
        }
        ExitCode::FAILURE
    }
}
