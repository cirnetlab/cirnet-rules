use super::super::storage::AppError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const MAX_DOCUMENT_BYTES: usize = 225_000;
pub const MAX_ATTACHMENTS: usize = 100;
pub const MAX_RELATIONS: usize = 32;
pub const HARD_DENIES: &[&str] = &[
    "approval.self_approve",
    "global_config.write",
    "governance.publish",
    "hardware.control",
    "release.publish",
    "sandbox.disable",
];

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Rule,
    Skill,
    Profile,
    Reference,
    Policy,
    Command,
    Hook,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Rule => "rule",
            Self::Skill => "skill",
            Self::Profile => "profile",
            Self::Reference => "reference",
            Self::Policy => "policy",
            Self::Command => "command",
            Self::Hook => "hook",
        }
    }

    pub fn default_format(self) -> Format {
        match self {
            Self::Profile | Self::Policy | Self::Command | Self::Hook => Format::Json,
            Self::Rule | Self::Skill | Self::Reference => Format::Markdown,
        }
    }

    pub fn require_authenticated_review(self) -> Result<(), AppError> {
        if matches!(self, Self::Policy | Self::Command | Self::Hook) {
            return Err(AppError::new(
                "CAPABILITY_UNAVAILABLE",
                "Publishing policies, commands, and hooks requires authenticated independent review.",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    Markdown,
    Json,
    Yaml,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Attachment {
    pub path: String,
    pub content: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub en: String,
    pub vi: String,
    pub format: Format,
    pub description: String,
    pub files: Vec<Attachment>,
    pub dependencies: Vec<String>,
}

impl Document {
    pub fn empty(format: Format) -> Self {
        Self {
            en: String::new(),
            vi: String::new(),
            format,
            description: String::new(),
            files: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn sensitive(&self) -> bool {
        use super::super::sessions::sensitive_text;
        let raw_markers = sensitive_text(&self.en)
            || sensitive_text(&self.vi)
            || sensitive_text(&self.description)
            || self.files.iter().any(|file| sensitive_text(&file.content));
        raw_markers
            || match self.format {
                Format::Json => serde_json::from_str::<Value>(&self.en).ok(),
                Format::Yaml => parse_yaml::<Value>(&self.en, 0).ok(),
                Format::Markdown => None,
            }
            .as_ref()
            .is_some_and(|config| has_credential_field(config, 0))
    }

    pub fn check_bounds(&self) -> Result<(), AppError> {
        if self.files.len() > MAX_ATTACHMENTS || self.dependencies.len() > MAX_RELATIONS {
            return Err(AppError::new(
                "INVALID_INPUT",
                "Document has too many attachments or dependencies.",
            ));
        }
        checked_text(&self.description, 600, true)?;
        if self.sensitive() {
            return Err(AppError::new(
                "SENSITIVE_INPUT",
                "Remove credentials before saving governance content.",
            ));
        }
        let mut paths = BTreeSet::new();
        for file in &self.files {
            if !valid_attachment_path(&file.path) || !paths.insert(file.path.to_ascii_lowercase()) {
                return Err(AppError::new(
                    "INVALID_PATH",
                    "Attachment paths must be safe, unique relative names.",
                ));
            }
        }
        let mut dependencies = BTreeSet::new();
        for id in &self.dependencies {
            if !valid_asset_id(id) || !dependencies.insert(id) {
                return Err(AppError::new(
                    "INVALID_INPUT",
                    "Dependency IDs must be valid and unique.",
                ));
            }
        }
        let total = self.en.len()
            + self.vi.len()
            + self
                .files
                .iter()
                .map(|file| file.content.len())
                .sum::<usize>();
        if total > 220_000 || encode(self)?.len() > MAX_DOCUMENT_BYTES {
            return Err(AppError::new(
                "INVALID_INPUT",
                "Document exceeds the local transport budget.",
            ));
        }
        Ok(())
    }
}

pub fn valid_asset_id(id: &str) -> bool {
    id.strip_prefix("asset-").is_some_and(|suffix| {
        suffix.len() == 32
            && suffix
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    })
}

pub fn checked_text(value: &str, max: usize, empty: bool) -> Result<(), AppError> {
    if value.len() > max
        || (!empty && value.trim().is_empty())
        || value.chars().any(char::is_control)
    {
        return Err(AppError::new(
            "INVALID_INPUT",
            "Metadata text is invalid or too long.",
        ));
    }
    Ok(())
}

pub fn check_identity(slug: &str, title: &str) -> Result<(), AppError> {
    if slug.is_empty()
        || slug.len() > 64
        || slug.starts_with('-')
        || slug.ends_with('-')
        || !slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err(AppError::new(
            "INVALID_INPUT",
            "Use a lowercase slug with letters, numbers, and hyphens.",
        ));
    }
    checked_text(title, 160, false)
}

pub fn valid_attachment_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 240
        && value.split('/').count() <= 12
        && value.split('/').all(|part| {
            let stem = part
                .split('.')
                .next()
                .unwrap_or_default()
                .to_ascii_uppercase();
            !part.is_empty()
                && !part.starts_with('.')
                && !part.ends_with(['.', ' '])
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._- ".contains(&b))
                && !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                && !(stem.len() == 4
                    && (stem.starts_with("COM") || stem.starts_with("LPT"))
                    && stem.as_bytes()[3].is_ascii_digit())
        })
}

pub fn encode<T: Serialize + ?Sized>(value: &T) -> Result<String, AppError> {
    serde_json::to_string(value)
        .map_err(|_| AppError::new("INVALID_INPUT", "Cannot encode governance data."))
}

pub fn digest(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

#[derive(Serialize)]
pub struct Diagnostic {
    code: &'static str,
    field: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    column: Option<usize>,
}

impl Diagnostic {
    pub fn new(code: &'static str, field: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            field,
            message: message.into(),
            line: None,
            column: None,
        }
    }

    fn location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }
}

pub fn translation(document: &Document, active: Option<&Document>) -> &'static str {
    if document.vi.trim().is_empty() {
        "MISSING"
    } else if active.is_some_and(|old| old.en != document.en && old.vi == document.vi) {
        "STALE"
    } else if document.vi.trim() == document.en.trim() {
        "REVIEW_REQUIRED"
    } else {
        "CURRENT"
    }
}

fn parse_yaml<T: serde::de::DeserializeOwned>(
    source: &str,
    offset: usize,
) -> Result<T, Diagnostic> {
    if source.len() > 65_536 {
        return Err(Diagnostic::new(
            "YAML_LIMIT",
            "en",
            "YAML is limited to 64 KiB.",
        ));
    }
    // Reject alias and anchor token candidates before decoding. Conservatively reject
    // quoted text with these token sequences too, keeping the supported YAML subset explicit.
    for (row, line) in source.lines().enumerate() {
        let bytes = line.as_bytes();
        for (index, &byte) in bytes.iter().enumerate() {
            let token_start = index == 0
                || bytes[index - 1].is_ascii_whitespace()
                || b"[{,:?".contains(&bytes[index - 1]);
            if token_start
                && b"&*".contains(&byte)
                && bytes
                    .get(index + 1)
                    .is_some_and(|next| !next.is_ascii_whitespace())
            {
                return Err(Diagnostic::new(
                    "YAML_ALIAS",
                    "en",
                    "YAML anchors and aliases are unavailable.",
                )
                .location(row + 1 + offset, index + 1));
            }
        }
    }
    serde_yaml::from_str(source).map_err(|error| {
        let location = error.location();
        let diagnostic = Diagnostic::new("INVALID_YAML", "en", "Invalid or unsupported YAML.");
        match location {
            Some(location) => diagnostic.location(location.line() + offset, location.column()),
            None => diagnostic,
        }
    })
}

fn validate_skill(source: &str) -> Result<(), Diagnostic> {
    let Some(metadata) = source
        .strip_prefix("---\n")
        .or_else(|| source.strip_prefix("---\r\n"))
    else {
        return Err(Diagnostic::new(
            "SKILL_FRONTMATTER",
            "en",
            "SKILL.md needs YAML frontmatter with name and description.",
        )
        .location(1, 1));
    };
    let mut frontmatter = String::new();
    for line in metadata.lines() {
        if line == "---" {
            let value: serde_yaml::Mapping = parse_yaml(&frontmatter, 1)?;
            for field in ["name", "description"] {
                if !value
                    .get(field)
                    .and_then(serde_yaml::Value::as_str)
                    .is_some_and(|text| !text.trim().is_empty())
                {
                    return Err(Diagnostic::new(
                        "SKILL_FRONTMATTER",
                        "en",
                        format!("Missing a text value for {field}."),
                    )
                    .location(2, 1));
                }
            }
            return Ok(());
        }
        frontmatter.push_str(line);
        frontmatter.push('\n');
    }
    Err(Diagnostic::new(
        "SKILL_FRONTMATTER",
        "en",
        "Missing closing frontmatter delimiter.",
    )
    .location(1, 1))
}

fn credential_field(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().replace('-', "_").as_str(),
        "api_key"
            | "secret"
            | "token"
            | "password"
            | "credential"
            | "access_token"
            | "client_secret"
    )
}

fn has_credential_field(value: &Value, depth: usize) -> bool {
    if depth > 24 {
        return true;
    }
    match value {
        Value::Object(fields) => fields
            .iter()
            .any(|(key, value)| credential_field(key) || has_credential_field(value, depth + 1)),
        Value::Array(values) => values
            .iter()
            .any(|value| has_credential_field(value, depth + 1)),
        _ => false,
    }
}

fn inspect_config(value: &Value, depth: usize, nodes: &mut usize) -> Result<(), Diagnostic> {
    *nodes += 1;
    if depth > 24 || *nodes > 2048 {
        return Err(Diagnostic::new(
            "STRUCTURE_LIMIT",
            "en",
            "Structured content exceeds the depth or node budget.",
        ));
    }
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                let key = key.to_ascii_lowercase().replace('-', "_");
                if matches!(
                    key.as_str(),
                    "api_key"
                        | "secret"
                        | "token"
                        | "password"
                        | "credential"
                        | "access_token"
                        | "client_secret"
                ) {
                    return Err(Diagnostic::new(
                        "PLAINTEXT_SECRET",
                        "en",
                        "Use secret references. Credential fields are unavailable.",
                    ));
                }
                if matches!(key.as_str(), "allow" | "allowed_actions" | "tool_allowlist")
                    && value.as_array().is_some_and(|values| {
                        values.iter().any(|value| {
                            value
                                .as_str()
                                .is_some_and(|value| HARD_DENIES.contains(&value) || value == "*")
                        })
                    })
                    || HARD_DENIES.contains(&key.as_str()) && value.as_str() == Some("allow")
                {
                    return Err(Diagnostic::new(
                        "HARD_DENY",
                        "en",
                        "Configuration cannot grant a runtime hard-denied action.",
                    ));
                }
                inspect_config(value, depth + 1, nodes)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                inspect_config(value, depth + 1, nodes)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn structured(source: &str, format: Format) -> Result<(), Diagnostic> {
    let value: Value = match format {
        Format::Json => serde_json::from_str(source).map_err(|error| {
            Diagnostic::new("INVALID_JSON", "en", "Invalid JSON.")
                .location(error.line(), error.column())
        })?,
        Format::Yaml => parse_yaml(source, 0)?,
        Format::Markdown => return Ok(()),
    };
    if !value.is_object() {
        return Err(Diagnostic::new(
            "INVALID_STRUCTURE",
            "en",
            "Structured configuration must be an object.",
        ));
    }
    inspect_config(&value, 0, &mut 0)
}

pub fn validate(kind: Kind, document: &Document, active: Option<&Document>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if document.en.trim().is_empty() {
        diagnostics.push(Diagnostic::new(
            "MISSING_EN",
            "en",
            "Canonical English content is required.",
        ));
    } else {
        let typed = matches!(
            kind,
            Kind::Profile | Kind::Policy | Kind::Command | Kind::Hook
        );
        let result = match (kind, document.format) {
            (Kind::Skill, Format::Markdown) => validate_skill(&document.en),
            (Kind::Skill, _) => Err(Diagnostic::new(
                "INVALID_FORMAT",
                "format",
                "A skill uses SKILL.md Markdown.",
            )),
            (_, Format::Markdown) if typed => Err(Diagnostic::new(
                "INVALID_FORMAT",
                "format",
                "Use structured JSON or YAML for this asset type.",
            )),
            (_, format) => structured(&document.en, format),
        };
        if let Err(issue) = result {
            diagnostics.push(issue);
        }
    }
    let translation = translation(document, active);
    if translation != "CURRENT" {
        diagnostics.push(Diagnostic::new(
            translation,
            "vi",
            "Update the Vietnamese translation before review.",
        ));
    }
    for file in &document.files {
        let extension = file
            .path
            .rsplit_once('.')
            .map(|(_, extension)| extension.to_ascii_lowercase());
        if !matches!(
            extension.as_deref(),
            Some("md" | "txt" | "json" | "yaml" | "yml" | "csv")
        ) {
            diagnostics.push(Diagnostic::new(
                "ATTACHMENT_UNSUPPORTED",
                "files",
                format!("Review cannot activate this attachment type: {}", file.path),
            ));
        }
    }
    diagnostics
}
