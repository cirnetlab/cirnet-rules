//! Offline preview of explicitly selected upstream formats. No mapping grants authority.
use super::{AppError, Value, decode, json};
use serde::Deserialize;
use serde_json::Map;

const OPENHANDS_REVISION: &str = "1a66039ecd20d93e792eec257c7f5abf4d709a10";
const OPENCODE_REVISION: &str = "fee1ca5aa9e1d62a7ca42ef7eeaa8c02fe6ed080";
const MAX_SOURCE: usize = 128 * 1024;
const MAX_FIELDS: usize = 64;

#[derive(Clone, Copy, Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
enum Format {
    OpenhandsSkill,
    OpencodeV1Agent,
    OpencodeV2Agent,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Preview {
    format: Format,
    source: String,
}

#[derive(serde::Serialize)]
struct Diagnostic {
    code: &'static str,
    message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    column: Option<usize>,
}

impl Diagnostic {
    fn new(code: &'static str, message: &'static str) -> Self {
        Self {
            code,
            message,
            line: None,
            column: None,
        }
    }

    fn at(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }
}

pub(super) fn preview(params: Value) -> Result<Value, AppError> {
    let Preview { format, source } = decode(params)?;
    if source.len() > MAX_SOURCE {
        return Err(AppError::new(
            "IMPORT_LIMIT",
            "Integration preview accepts at most 128 KiB of UTF-8.",
        ));
    }
    let mut diagnostics = Vec::new();
    let mut fields = Vec::new();
    let mut metadata = Map::new();
    match document(format, &source) {
        Ok(data) if data.len() <= MAX_FIELDS => {
            for (key, value) in data {
                let (status, name) = map_field(format, &key, &value, &mut diagnostics);
                fields.push(json!({"field":key,"status":status}));
                if let Some(name) = name {
                    metadata.insert(name.to_owned(), value);
                }
            }
            if matches!(format, Format::OpenhandsSkill) {
                for name in ["name", "description"] {
                    if !metadata.contains_key(name) {
                        diagnostics.push(Diagnostic::new(
                            "SKILL_METADATA",
                            "Skill frontmatter needs a name and description.",
                        ));
                        break;
                    }
                }
            }
        }
        Ok(_) => diagnostics.push(Diagnostic::new(
            "IMPORT_LIMIT",
            "Integration preview accepts at most 64 fields.",
        )),
        Err(diagnostic) => diagnostics.push(diagnostic),
    }
    if !diagnostics.is_empty() {
        metadata.clear();
    }
    let revision = match format {
        Format::OpenhandsSkill => OPENHANDS_REVISION,
        Format::OpencodeV1Agent | Format::OpencodeV2Agent => OPENCODE_REVISION,
    };
    Ok(json!({
        "format":format,"upstream_revision":revision,"fields":fields,
        "metadata":metadata,"diagnostics":diagnostics
    }))
}

fn document(format: Format, source: &str) -> Result<Map<String, Value>, Diagnostic> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let first = source.lines().next();
    let data = if first == Some("---") {
        let start = source.find('\n').map_or(source.len(), |offset| offset + 1);
        let remainder = &source[start..];
        let Some((end, _)) = remainder
            .split_inclusive('\n')
            .scan(0, |offset, line| {
                let position = *offset;
                *offset += line.len();
                Some((position, line))
            })
            .find(|(_, line)| line.trim_end_matches(['\r', '\n']) == "---")
        else {
            return Err(Diagnostic::new(
                "FRONTMATTER",
                "The closing frontmatter delimiter is missing.",
            )
            .at(1, 1));
        };
        yaml_to_json(&remainder[..end], 1)?
    } else {
        if matches!(format, Format::OpenhandsSkill) {
            return Err(Diagnostic::new(
                "FRONTMATTER",
                "A skill requires YAML frontmatter at the start of SKILL.md.",
            )
            .at(1, 1));
        }
        let data = serde_json::from_str::<Value>(source).map_err(|error| {
            Diagnostic::new(
                "INVALID_JSON",
                "Enter a strict JSON object or Markdown with YAML frontmatter.",
            )
            .at(error.line(), error.column().max(1))
        })?;
        // The JSON parser keeps the final duplicate key. Reject ambiguous objects before mapping.
        let _: serde_yaml::Value =
            serde_yaml::from_str(source).map_err(|error| yaml_diagnostic(error, 0))?;
        data
    };
    match data {
        Value::Object(values) => Ok(values),
        _ => Err(Diagnostic::new(
            "INVALID_OBJECT",
            "The configuration metadata must be an object.",
        )
        .at(1, 1)),
    }
}

fn yaml_to_json(source: &str, line_offset: usize) -> Result<Value, Diagnostic> {
    // Disable YAML aliases, anchors, tags and directives in this portable subset.
    // Conservatively reject these token forms even inside quoted YAML; JSON is an option for agent profiles.
    for (index, line) in source.lines().enumerate() {
        let mut previous = ' ';
        let mut chars = line.char_indices().peekable();
        while let Some((column, current)) = chars.next() {
            let at_token = previous.is_whitespace() || "[{,:".contains(previous);
            let starts_value = chars.peek().is_some_and(|(_, next)| !next.is_whitespace());
            if at_token && starts_value && matches!(current, '*' | '&' | '!' | '%') {
                return Err(Diagnostic::new(
                    "YAML_FEATURE",
                    "Aliases, anchors, tags and directives are outside this import subset.",
                )
                .at(line_offset + index + 1, column + 1));
            }
            previous = current;
        }
    }
    let value: serde_yaml::Value =
        serde_yaml::from_str(source).map_err(|error| yaml_diagnostic(error, line_offset))?;
    serde_json::to_value(value).map_err(|_| {
        Diagnostic::new(
            "INVALID_METADATA",
            "YAML metadata must be representable as JSON.",
        )
    })
}

fn yaml_diagnostic(error: serde_yaml::Error, line_offset: usize) -> Diagnostic {
    let diagnostic = Diagnostic::new(
        "INVALID_YAML",
        "The metadata is invalid or contains duplicate keys.",
    );
    match error.location() {
        Some(location) => diagnostic.at(line_offset + location.line(), location.column()),
        None => diagnostic,
    }
}

fn map_field(
    format: Format,
    key: &str,
    value: &Value,
    diagnostics: &mut Vec<Diagnostic>,
) -> (&'static str, Option<&'static str>) {
    if key.len() > 128 || key.chars().any(char::is_control) {
        diagnostics.push(Diagnostic::new(
            "INVALID_FIELD",
            "Field names must be printable and at most 128 bytes.",
        ));
        return ("REJECTED", None);
    }
    let dialect_mismatch = match format {
        Format::OpenhandsSkill => false,
        Format::OpencodeV1Agent => matches!(
            key,
            "agents" | "permissions" | "request" | "system" | "disabled"
        ),
        Format::OpencodeV2Agent => matches!(
            key,
            "agent" | "permission" | "prompt" | "disable" | "maxSteps"
        ),
    };
    if dialect_mismatch {
        diagnostics.push(Diagnostic::new(
            "DIALECT_FIELD",
            "The configuration mixes fields from different OpenCode dialects.",
        ));
        return ("REJECTED", None);
    }
    if matches!(
        key.to_ascii_lowercase().as_str(),
        "permission"
            | "permissions"
            | "policy"
            | "policies"
            | "request"
            | "headers"
            | "credentials"
            | "secrets"
            | "api_key"
            | "apikey"
            | "token"
            | "password"
            | "tools"
            | "mcp"
            | "plugins"
            | "hooks"
            | "scripts"
            | "command"
            | "commands"
    ) {
        return ("REJECTED", None);
    }

    let text = value
        .as_str()
        .is_some_and(|value| !value.trim().is_empty() && value.len() <= 4096);
    let metadata = match (format, key) {
        (Format::OpenhandsSkill, "name") => Some(("name", text, "SUPPORTED")),
        (_, "description") => Some(("description", text, "SUPPORTED")),
        (Format::OpenhandsSkill, "license") => Some(("license", text, "SUPPORTED")),
        (Format::OpenhandsSkill, "compatibility") => Some(("compatibility", text, "SUPPORTED")),
        (Format::OpenhandsSkill, _) => None,
        (_, "model") if !value.is_object() => Some(("model", text, "SUPPORTED")),
        (_, "mode") => Some((
            "mode",
            matches!(value.as_str(), Some("subagent" | "primary" | "all")),
            "SUPPORTED",
        )),
        (_, "disable") => Some(("disabled", value.is_boolean(), "TRANSLATED")),
        (_, "disabled") => Some(("disabled", value.is_boolean(), "SUPPORTED")),
        _ => None,
    };
    match metadata {
        Some((name, true, status)) => (status, Some(name)),
        Some((_, false, _)) => {
            diagnostics.push(Diagnostic::new(
                "FIELD_TYPE",
                "A mapped field has an invalid type or value.",
            ));
            ("REJECTED", None)
        }
        None => ("RETAINED_AS_SOURCE", None),
    }
}
