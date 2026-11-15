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

Checks direct English Skill metadata and portable local resources, and
matches VERSION to Cargo.toml and the first release heading in both changelogs.
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

fn valid_skill_name(name: &str) -> bool {
    (1..=64).contains(&name.len())
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--")
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

// Recognize this repository's [package] version declaration, not arbitrary TOML.
fn cargo_version(manifest: &str) -> Option<&str> {
    let mut package = manifest
        .lines()
        .map(str::trim)
        .skip_while(|line| *line != "[package]");
    package.next()?;
    let mut versions = package
        .take_while(|line| !line.starts_with('['))
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            (key.trim() == "version").then_some(value.trim())
        });
    let version = versions.next()?.strip_prefix('"')?.strip_suffix('"')?;
    versions.next().is_none().then_some(version)
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

fn skill_citation(destination: &str) -> bool {
    destination.split_once(':').is_some_and(|(scheme, rest)| {
        if scheme.eq_ignore_ascii_case("mailto") {
            !rest.is_empty()
        } else {
            (scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https"))
                && rest.starts_with("//")
        }
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

fn portable_component(name: &str) -> bool {
    let device = name
        .split('.')
        .next()
        .unwrap()
        .trim_end_matches(' ')
        .to_ascii_lowercase();
    let reserved = matches!(
        device.as_str(),
        "con" | "prn" | "aux" | "nul" | "conin$" | "conout$"
    ) || device
        .strip_prefix("com")
        .or_else(|| device.strip_prefix("lpt"))
        .is_some_and(|number| {
            matches!(
                number,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        });
    !reserved
        && !name.ends_with([' ', '.'])
        && !name.contains(|ch: char| ch.is_control() || "<>:\"\\|?*".contains(ch))
}

fn skill_target(source: &Path, root: &Path, name: &str) -> Result<PathBuf, &'static str> {
    if name.is_empty() {
        return Ok(source.to_path_buf());
    }
    if name.starts_with('/') || name.contains('\\') {
        return Err("expected a relative path using forward slashes");
    }
    let mut target = source.parent().unwrap().to_path_buf();
    for part in name.split('/') {
        match part {
            "" => return Err("empty path component"),
            "." => {}
            ".." if target == root => return Err("target leaves the packaged skill"),
            ".." => {
                target.pop();
            }
            component if portable_component(component) => target.push(component),
            _ => return Err("invalid or reserved path component"),
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

    fn version_file(&mut self, path: &Path) -> Option<String> {
        match self.metadata(path).and_then(|metadata| {
            if !metadata.is_file() {
                return Err("expected a regular file".into());
            }
            fs::read_to_string(self.root.join(path)).map_err(|error| error.to_string())
        }) {
            Ok(text) => Some(text),
            Err(error) => {
                self.fail(path, error);
                None
            }
        }
    }

    fn versions(&mut self) {
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
        let manifest = Path::new("Cargo.toml");
        if let Some(contents) = self.version_file(manifest) {
            if cargo_version(&contents) != Some(version) {
                self.fail(
                    manifest,
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

    fn skills(&mut self, english: &BTreeSet<PathBuf>) -> BTreeSet<PathBuf> {
        let directory = Path::new("en/skills");
        match self.metadata(directory) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => self.fail(directory, "expected a Skills directory"),
            Err(error) => self.fail(directory, error),
        }
        let mut roots = BTreeSet::new();
        let mut names = BTreeMap::new();
        for path in english {
            let Ok(relative) = path.strip_prefix(directory) else {
                continue;
            };
            let file = relative.file_name().unwrap();
            if !file.eq_ignore_ascii_case("SKILL.md") {
                continue;
            }
            if file != "SKILL.md" {
                self.fail(path, "skill entrypoint must be named exactly SKILL.md");
                continue;
            }
            if relative.components().count() != 2 {
                self.fail(
                    path,
                    "SKILL.md must be in a direct skill directory under en/skills",
                );
                continue;
            }
            let root = path.parent().unwrap();
            roots.insert(root.to_path_buf());
            let Some(text) = self.documents.get(path) else {
                continue; // The file read already produced a diagnostic.
            };
            let (name, description, _) = match markdown::skill_metadata(text) {
                Ok(metadata) => metadata,
                Err(error) => {
                    self.fail(path, error);
                    continue;
                }
            };
            let errors = [
                (!valid_skill_name(name)).then_some("invalid skill name: use 1–64 lowercase ASCII letters, digits or single hyphens between characters"),
                (root.file_name().unwrap() != name).then_some("skill name must match the parent directory exactly"),
                (description.trim().is_empty() || description.chars().count() > 1024)
                    .then_some("skill description must be nonempty and at most 1024 characters"),
            ];
            let previous = names.insert(name.to_owned(), path);
            for error in errors.into_iter().flatten() {
                self.fail(path, error);
            }
            if let Some(previous) = previous {
                self.fail(
                    path,
                    format!("duplicate skill name also used in {}", path_text(previous)),
                );
            }
        }
        roots
    }

    fn startup_warnings(&mut self, root_files: &[PathBuf], skills: &BTreeSet<PathBuf>) {
        let path = Path::new("AGENTS.md");
        let Some(baseline) = self.documents.get(path) else {
            return;
        };
        let mut seen = BTreeSet::new();
        let mut imported_skills = BTreeSet::new();
        for import in context::startup_imports(baseline) {
            let normalized = import.to_ascii_lowercase();
            if !seen.insert(normalized.clone())
                || matches!(
                    normalized.as_str(),
                    "agents.md" | "en/agents.md" | "claude.md"
                )
            {
                self.warn(
                    path,
                    format!("duplicate startup entrypoint import {import:?}"),
                );
            }
            if matches!(
                normalized.as_str(),
                "en" | "en/*"
                    | "en/**"
                    | "vi"
                    | "vi/*"
                    | "vi/**"
                    | "en/skills"
                    | "en/skills/*"
                    | "en/skills/**"
            ) {
                self.warn(
                    path,
                    format!("eager startup import of library content {import:?}"),
                );
            } else if skills
                .iter()
                .any(|skill| normalized == path_text(&skill.join("SKILL.md")).to_ascii_lowercase())
            {
                imported_skills.insert(normalized);
            } else if let Err(error) =
                link_target(path, &import).and_then(|target| self.metadata(&target))
            {
                self.warn(
                    path,
                    format!("unresolved startup import {import:?}: {error}"),
                );
            }
        }
        if skills.len() > 1 && imported_skills.len() == skills.len() {
            self.warn(path, "eager startup imports name every Skill entrypoint");
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

    fn context_checks(&mut self, root_files: &[PathBuf], skills: &BTreeSet<PathBuf>) {
        self.startup_warnings(root_files, skills);
        for (name, max_words, max_bytes) in [
            ("AGENTS.md", 500, 4_000),
            ("en/skills/README.md", 300, usize::MAX),
        ] {
            let path = Path::new(name);
            if let Some(text) = self.documents.get(path) {
                let size = context::Size::of(text);
                let warnings = context::suspicious_prose(text);
                if size.words > max_words || size.bytes > max_bytes {
                    self.warn(path, format!("context budget exceeded ({size})"));
                }
                self.info(path, size);
                for warning in warnings {
                    self.warn(path, warning);
                }
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
        let mut metadata_bytes = 0;
        let mut metadata_count = 0;
        for root in skills {
            let path = root.join("SKILL.md");
            let Some((name, description, body)) = self
                .documents
                .get(&path)
                .and_then(|text| markdown::skill_metadata(text).ok())
            else {
                continue;
            };
            let body_size = context::Size::of(body);
            let description_chars = description.chars().count();
            let bytes = name.len() + description.len();
            let warnings = context::suspicious_prose(body);
            metadata_bytes += bytes;
            metadata_count += 1;
            if body_size.words > 900 || body_size.lines > 250 {
                self.warn(&path, format!("Skill body budget exceeded ({body_size}); prefer at most 900 words and 250 lines"));
            }
            if description_chars > 240 {
                self.warn(
                    &path,
                    format!("description is {description_chars} characters; prefer at most 240"),
                );
            }
            self.info(&path, format!("body {body_size}; metadata values {bytes} bytes, description {description_chars} characters"));
            for warning in warnings {
                self.warn(&path, warning);
            }
        }
        self.info(Path::new("en/skills"), format!("metadata total: {metadata_bytes} bytes in parsed name and description values across {metadata_count} Skills"));
    }

    fn validate_link(
        &mut self,
        source: &Path,
        destination: &str,
        skill_root: Option<&Path>,
    ) -> Result<Option<PathBuf>, String> {
        let destination = destination.trim_matches(['<', '>']);
        if skill_root.map_or_else(|| external(destination), |_| skill_citation(destination)) {
            return Ok(None);
        }
        let (url, fragment) = destination.split_once('#').unwrap_or((destination, ""));
        let path = url.split_once('?').map_or(url, |(path, _)| path);
        let name = percent_decode(path);
        let target = if let Some(root) = skill_root {
            skill_target(source, root, &name)?
        } else {
            link_target(source, &name.replace('\\', "/"))?
        };
        let metadata = self.path_metadata(&target, skill_root.is_some())?;
        if skill_root.is_some() && !metadata.is_file() {
            return Err("expected a regular resource file".into());
        }
        if !fragment.is_empty() {
            if !is_markdown(&target) || !metadata.is_file() {
                return Err("fragment target must be Markdown".into());
            }
            let text = self.read(&target).ok_or("could not read Markdown target")?;
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
        self.versions();
        let skills = self.skills(&english);
        self.context_checks(&root_files, &skills);
        let inventory: Vec<_> = self.documents.keys().cloned().collect();
        let mut incoming = BTreeMap::<PathBuf, BTreeSet<PathBuf>>::new();
        for path in &inventory {
            let skill_root = path
                .ancestors()
                .skip(1)
                .find(|parent| skills.contains(*parent));
            let text = &self.documents[path];
            let body = if skill_root.is_some() && path.file_name().unwrap() == "SKILL.md" {
                markdown::frontmatter(text).map_or(text.as_str(), |(_, body)| body)
            } else {
                text.as_str()
            };
            let (links, undefined) = markdown::links(body);
            for destination in links {
                match self.validate_link(path, &destination, skill_root) {
                    Ok(Some(target))
                        if target != *path && inventory.binary_search(&target).is_ok() =>
                    {
                        incoming.entry(target).or_default().insert(path.clone());
                    }
                    Ok(_) => {}
                    Err(error) => {
                        let kind = if skill_root.is_some() {
                            "portable skill resource"
                        } else {
                            "link"
                        };
                        self.fail(path, format!("invalid {kind} {destination:?}: {error}"));
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
