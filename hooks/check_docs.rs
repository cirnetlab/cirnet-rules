mod architecture;
mod context;
mod document_hygiene;
mod markdown;
mod sha256;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, Metadata};
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "Usage: check-docs [--root PATH] [--new-file PATH ...]";
const HELP: &str = "Read-only, offline bilingual documentation check.

Checks root Markdown and the en/vi trees. Supports simple inline links,
images, reference definitions/uses and ATX/Setext heading fragments.
External links, code spans, fenced code and HTML comments are ignored.
Nested brackets/parentheses, multiline destinations, HTML links/anchors
and renderer-specific anchor rules are outside this subset.

Matches VERSION to Cargo.toml and the first release heading in both changelogs.
In the toolkit, checks architecture source integrity and Markdown links,
owner routes in both languages, superseded links and divergent copies.
Context and cross-document similarity warnings are advisory. Use repeatable
--new-file PATH to review a readable, root-relative Markdown file in the scan.
The checker uses the paths you list. Unprotected byte or normalized duplicates
fail when either or both paths are listed. Near matches remain advisory.

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

fn valid_version(version: &str) -> bool {
    let mut parts = version.split('.');
    (0..3).all(|_| {
        parts.next().is_some_and(|part| {
            (part == "0" || !part.starts_with('0'))
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && part.parse::<u64>().is_ok()
        })
    }) && parts.next().is_none()
}

// Recognize the toolkit's string declarations in [package], not arbitrary TOML.
fn cargo_package_field<'a>(manifest: &'a str, field: &str) -> Option<&'a str> {
    let mut package = manifest
        .lines()
        .map(str::trim)
        .skip_while(|line| *line != "[package]");
    package.next()?;
    let mut values = package
        .take_while(|line| !line.starts_with('['))
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            (key.trim() == field).then_some(value.trim())
        });
    let value = values.next()?.strip_prefix('"')?.strip_suffix('"')?;
    values.next().is_none().then_some(value)
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
    warnings: Vec<String>,
    information: Vec<String>,
}

enum RunError {
    Read(String),
    InvalidNewFile(PathBuf),
}

impl Checker {
    fn fail(&mut self, path: &Path, message: impl std::fmt::Display) {
        self.errors.push(format!("{}: {message}", path_text(path)));
    }

    fn warn(&mut self, path: &Path, message: impl std::fmt::Display) {
        self.warnings
            .push(format!("{}: {message}", path_text(path)));
    }

    fn info(&mut self, path: &Path, message: impl std::fmt::Display) {
        self.information
            .push(format!("{}: {message}", path_text(path)));
    }

    // All paths inside the checker are relative to the canonical checkout.
    // Inspect each component before reading a file or descending into a directory.
    fn metadata(&self, path: &Path) -> Result<Metadata, String> {
        self.path_metadata(path, false)
    }

    fn path_metadata(&self, path: &Path, exact_case: bool) -> Result<Metadata, String> {
        let mut absolute = self.root.clone();
        let mut metadata = None;
        for part in path.components() {
            let Component::Normal(name) = part else {
                return Err("target leaves selected root".into());
            };
            if excluded(&name.to_string_lossy()) {
                return Err("target enters an excluded directory".into());
            }
            if exact_case {
                let entries = fs::read_dir(&absolute).map_err(|error| error.to_string())?;
                let names = entries
                    .map(|entry| entry.map(|entry| entry.file_name()))
                    .collect::<io::Result<Vec<_>>>()
                    .map_err(|error| error.to_string())?;
                if !names.iter().any(|entry| entry == name) {
                    return Err("target does not exist with the specified spelling and case".into());
                }
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
            let text = match self.regular_file_text(path, false) {
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

    fn regular_file_text(&self, path: &Path, exact_case: bool) -> Result<String, String> {
        if !self.path_metadata(path, exact_case)?.is_file() {
            return Err("expected a regular file".into());
        }
        fs::read_to_string(self.root.join(path)).map_err(|error| error.to_string())
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
        let adapter = Path::new("CLAUDE.md");
        for path in [baseline, adapter] {
            if !self.documents.contains_key(path) {
                self.fail(path, "required readable Markdown file is missing");
            }
        }
        if self.documents.get(adapter).map(String::as_str) != Some("@AGENTS.md\n") {
            self.fail(adapter, "must contain exactly @AGENTS.md and LF");
        }
    }

    fn version_file(&mut self, path: &Path) -> Option<String> {
        match self.regular_file_text(path, false) {
            Ok(text) => Some(text),
            Err(error) => {
                self.fail(path, error);
                None
            }
        }
    }

    fn versions(&mut self, manifest: Option<&str>) {
        let version_path = Path::new("VERSION");
        let Some(file) = self.version_file(version_path) else {
            return;
        };
        let Some(version) = file
            .strip_suffix('\n')
            .filter(|version| valid_version(version))
        else {
            self.fail(
                version_path,
                "expected one MAJOR.MINOR.PATCH line ending in LF, with no leading zeros",
            );
            return;
        };
        if let Some(contents) = manifest {
            if cargo_package_field(contents, "version") != Some(version) {
                self.fail(
                    Path::new("Cargo.toml"),
                    format!("[package] version must match VERSION ({version})"),
                );
            }
        }
        let candidate = format!("{version} (candidate)");
        let mut english_heading = None;
        for path in ["en/docs/changelog.md", "vi/docs/changelog.md"] {
            let path = Path::new(path);
            let first = self
                .documents
                .get(path)
                .and_then(|text| markdown::first_level_two_heading(text));
            if !matches!(first.as_deref(), Some(heading) if heading == version || heading == candidate)
            {
                self.fail(path, format!("first level-two ATX heading must be {version:?} or {candidate:?} from VERSION"));
            } else if let Some(ref english) = english_heading {
                if first.as_ref() != Some(english) {
                    self.fail(path, "candidate status must match the English changelog");
                }
            } else {
                english_heading = first;
            }
        }
    }

    fn startup_warnings(&mut self, root_files: &[PathBuf]) {
        let path = Path::new("AGENTS.md");
        let Some(baseline) = self.documents.get(path) else {
            return;
        };
        let mut seen = BTreeSet::new();
        for import in context::startup_imports(baseline) {
            let normalized = import.to_ascii_lowercase();
            if !seen.insert(normalized.clone())
                || matches!(normalized.as_str(), "agents.md" | "claude.md")
            {
                self.warn(
                    path,
                    format!("duplicate startup entrypoint import {import:?}"),
                );
            }
            if matches!(
                normalized.as_str(),
                "en" | "en/*" | "en/**" | "vi" | "vi/*" | "vi/**"
            ) {
                self.warn(
                    path,
                    format!("eager startup import of library content {import:?}"),
                );
            } else if let Err(error) =
                link_target(path, &import).and_then(|target| self.metadata(&target))
            {
                self.warn(
                    path,
                    format!("unresolved startup import {import:?}: {error}"),
                );
            }
        }
        for file in root_files {
            if file
                .file_name()
                .is_some_and(|name| name.eq_ignore_ascii_case("AGENTS.override.md"))
            {
                self.warn(file, "override entrypoint may replace the shared AGENTS.md on a client that supports it");
            }
        }
    }

    fn context_checks(&mut self, root_files: &[PathBuf]) {
        self.startup_warnings(root_files);
        let path = Path::new("AGENTS.md");
        if let Some(text) = self.documents.get(path) {
            let size = context::Size::of(text);
            let warnings = context::suspicious_prose(text);
            if size.words > 500 || size.bytes > 4_000 {
                self.warn(path, format!("context budget exceeded ({size})"));
            }
            self.info(path, size);
            for warning in warnings {
                self.warn(path, warning);
            }
        }
        let adapter = Path::new("CLAUDE.md");
        if let Some(text) = self.documents.get(adapter) {
            self.info(
                adapter,
                format!(
                    "adapter {}; common baseline reported separately",
                    context::Size::of(text)
                ),
            );
        }
    }

    fn validate_link(
        &mut self,
        source: &Path,
        destination: &str,
    ) -> Result<Option<PathBuf>, String> {
        let destination = destination.trim_matches(['<', '>']);
        if external(destination) {
            architecture::check_github_link(destination)?;
            return Ok(None);
        }
        let (url, fragment) = destination.split_once('#').unwrap_or((destination, ""));
        let path = url.split_once('?').map_or(url, |(path, _)| path);
        let name = percent_decode(path);
        let target = link_target(source, &name.replace('\\', "/"))?;
        architecture::check_link(&target)?;
        let snapshot = architecture::is_snapshot(&target);
        let metadata = self.path_metadata(&target, snapshot)?;
        if !fragment.is_empty() {
            if !is_markdown(&target) || !metadata.is_file() {
                return Err("fragment target must be Markdown".into());
            }
            let snapshot_text;
            let text = if snapshot {
                snapshot_text = self.regular_file_text(&target, true)?;
                &snapshot_text
            } else {
                self.read(&target).ok_or("could not read Markdown target")?
            };
            if !markdown::anchors(text).contains(&percent_decode(fragment)) {
                return Err("heading fragment does not exist".into());
            }
        }
        Ok(Some(target))
    }

    fn run(&mut self, new_files: &BTreeSet<PathBuf>) -> Result<(), RunError> {
        let root_files = self.entries(Path::new("")).map_err(RunError::Read)?;
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
        if let Some(path) = new_files
            .iter()
            .find(|path| !self.documents.contains_key(*path))
        {
            return Err(RunError::InvalidNewFile(path.clone()));
        }
        self.translations(&english, &vietnamese);
        self.entrypoints();
        let cargo_manifest = self.version_file(Path::new("Cargo.toml"));
        self.versions(cargo_manifest.as_deref());
        let is_toolkit = cargo_manifest
            .as_deref()
            .and_then(|text| cargo_package_field(text, "name"))
            == Some("cirnet-rules");
        self.context_checks(&root_files);
        let inventory: Vec<_> = self.documents.keys().cloned().collect();
        let mut incoming = BTreeMap::<PathBuf, BTreeSet<PathBuf>>::new();
        for path in &inventory {
            let (links, undefined) = markdown::links(&self.documents[path]);
            for destination in links {
                match self.validate_link(path, &destination) {
                    Ok(Some(target))
                        if target != *path && inventory.binary_search(&target).is_ok() =>
                    {
                        incoming.entry(target).or_default().insert(path.clone());
                    }
                    Ok(_) => {}
                    Err(error) => {
                        self.fail(path, format!("invalid link {destination:?}: {error}"));
                    }
                }
            }
            for label in undefined {
                self.fail(path, format!("undefined link reference {label:?}"));
            }
        }
        // Fragment validation can read Markdown outside the ordinary scan.
        // Keep those link targets out of similarity and new-file review.
        self.documents
            .retain(|path, _| inventory.binary_search(path).is_ok());
        architecture::check(self, is_toolkit);
        for diagnostic in document_hygiene::review(&self.documents, new_files, &incoming) {
            use document_hygiene::Diagnostic;
            match diagnostic {
                Diagnostic::Info(message) => self.information.push(message),
                Diagnostic::Warning(message) => self.warnings.push(message),
                Diagnostic::Error(message) => self.errors.push(message),
            }
        }
        Ok(())
    }
}

fn usage_error(message: impl std::fmt::Display) -> ExitCode {
    eprintln!("{USAGE}\ncheck-docs: {message}");
    ExitCode::from(2)
}

fn new_file_path(arg: &str) -> Option<PathBuf> {
    if arg.is_empty() || arg.contains(':') || arg.contains(char::is_control) {
        return None;
    }
    let arg = arg.replace('\\', "/");
    let mut path = PathBuf::new();
    for component in Path::new(&arg).components() {
        match component {
            Component::Normal(name) => path.push(name),
            Component::CurDir => {}
            _ => return None,
        }
    }
    is_markdown(&path).then_some(path)
}

fn main() -> ExitCode {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut new_files = BTreeSet::new();
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
        } else if arg == "--new-file" {
            match args
                .next()
                .as_deref()
                .and_then(|value| value.to_str())
                .and_then(new_file_path)
            {
                Some(path) => {
                    new_files.insert(path);
                }
                None => return usage_error("--new-file requires a root-relative Markdown path"),
            }
        } else if let Some(value) = arg.to_str().and_then(|arg| arg.strip_prefix("--new-file=")) {
            match new_file_path(value) {
                Some(path) => {
                    new_files.insert(path);
                }
                None => return usage_error("--new-file requires a root-relative Markdown path"),
            }
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
        warnings: Vec::new(),
        information: Vec::new(),
    };
    match checker.run(&new_files) {
        Ok(()) => {}
        Err(RunError::Read(error)) => checker.errors.push(error),
        Err(RunError::InvalidNewFile(path)) => {
            return usage_error(format!(
                "--new-file path {:?} must be a readable Markdown file in the scanned inventory",
                path_text(&path)
            ));
        }
    }
    for info in checker.information {
        println!("INFO: {info}");
    }
    for warning in checker.warnings {
        println!("WARN: {warning}");
    }
    if checker.errors.is_empty() {
        println!("PASS: bilingual documentation checks");
        ExitCode::SUCCESS
    } else {
        for error in checker.errors {
            println!("ERROR: {error}");
        }
        ExitCode::FAILURE
    }
}
