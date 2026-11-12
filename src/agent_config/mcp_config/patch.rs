use serde::{Deserialize, Serialize};
use serde_json::Value;

const CONFLICT: &str =
    "The existing cirnetrules server differs. Review it in the native client before retrying.";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Insertion {
    pub(super) offset: usize,
    pub(super) bytes: Vec<u8>,
}

impl Insertion {
    pub(super) fn apply(&self, before: &[u8]) -> Result<Vec<u8>, String> {
        if self.offset > before.len() || self.bytes.len() > 16 * 1024 {
            return Err("Invalid MCP registration patch".into());
        }
        let mut after = before.to_vec();
        after.splice(self.offset..self.offset, self.bytes.iter().copied());
        Ok(after)
    }

    pub(super) fn reverse(&self, after: &[u8]) -> Result<Vec<u8>, String> {
        let end = self.offset.saturating_add(self.bytes.len());
        if after.get(self.offset..end) != Some(self.bytes.as_slice()) {
            return Err(
                "MCP registration fragment changed. Review the current configuration manually."
                    .into(),
            );
        }
        let mut before = after.to_vec();
        before.drain(self.offset..end);
        Ok(before)
    }
}

fn parse_toml(bytes: &[u8]) -> Result<toml::Table, String> {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|text| toml::from_str(text).ok())
        .ok_or_else(|| "The Codex project TOML is invalid or ambiguous".into())
}

fn toml_servers(table: &mut toml::Table) -> Result<&mut toml::Table, String> {
    table
        .entry("mcp_servers".to_owned())
        .or_insert_with(|| toml::Value::Table(toml::Table::new()))
        .as_table_mut()
        .ok_or_else(|| "The Codex mcp_servers key is not a table".into())
}

fn toml_patch(original: &[u8], template: &[u8]) -> Result<Option<Insertion>, String> {
    let mut before = parse_toml(original)?;
    let mut desired = parse_toml(template)?;
    let entry = toml_servers(&mut desired)?
        .remove("cirnetrules")
        .ok_or("Missing Codex template server")?;
    let servers = toml_servers(&mut before)?;
    if let Some(existing) = servers.get("cirnetrules") {
        return if existing == &entry {
            Ok(None)
        } else {
            Err(CONFLICT.into())
        };
    }
    servers.insert("cirnetrules".into(), entry);
    let newline = if original.windows(2).any(|pair| pair == b"\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let body = std::str::from_utf8(template).map_err(|_| "Invalid Codex template")?;
    let bytes = format!("{newline}{newline}{}", body.replace('\n', newline)).into_bytes();
    let insertion = Insertion {
        offset: original.len(),
        bytes,
    };
    if parse_toml(&insertion.apply(original)?)? != before {
        return Err("Codex project TOML cannot be amended without affecting other settings".into());
    }
    Ok(Some(insertion))
}

fn parse_json(bytes: &[u8]) -> Result<serde_json::Map<String, Value>, String> {
    crate::mcp::unique_json(bytes)
        .and_then(|()| serde_json::from_slice(bytes))
        .map_err(|_| "The project MCP JSON is invalid or has duplicate keys".into())
}

fn json_servers(
    object: &mut serde_json::Map<String, Value>,
) -> Result<&mut serde_json::Map<String, Value>, String> {
    object
        .entry("mcpServers")
        .or_insert_with(|| Value::Object(Default::default()))
        .as_object_mut()
        .ok_or_else(|| "The project mcpServers key is not an object".into())
}

// The source was already parsed as duplicate-free JSON. Find the original
// byte position of the top-level map without serializing unrelated entries.
fn json_servers_offset(bytes: &[u8]) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'{' | b'[' => depth += 1,
            b'}' | b']' => depth -= 1,
            b'"' => {
                let start = index;
                index += 1;
                while bytes[index] != b'"' {
                    if bytes[index] == b'\\' {
                        index += 1;
                    }
                    index += 1;
                }
                if depth == 1 {
                    let mut trailing = bytes[index + 1..]
                        .iter()
                        .enumerate()
                        .filter(|(_, byte)| !byte.is_ascii_whitespace());
                    if matches!(trailing.next(), Some((_, b':')))
                        && serde_json::from_slice::<String>(&bytes[start..=index])
                            .ok()
                            .as_deref()
                            == Some("mcpServers")
                    {
                        return trailing
                            .next()
                            .filter(|(_, byte)| **byte == b'{')
                            .map(|(offset, _)| index + offset + 2);
                    }
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn json_patch(original: &[u8], template: &[u8]) -> Result<Option<Insertion>, String> {
    let mut before = parse_json(original)?;
    let mut desired = parse_json(template)?;
    let entry = json_servers(&mut desired)?
        .remove("cirnetrules")
        .ok_or("Missing JSON template server")?;
    let has_servers = before.contains_key("mcpServers");
    let other_root_keys = !before.is_empty();
    let servers = json_servers(&mut before)?;
    if let Some(existing) = servers.get("cirnetrules") {
        return if existing == &entry {
            Ok(None)
        } else {
            Err(CONFLICT.into())
        };
    }
    let encoded =
        serde_json::to_string(&entry).map_err(|_| "Cannot serialize the CIRNETRULES MCP entry")?;
    let (offset, insertion) = if has_servers {
        let offset = json_servers_offset(original)
            .ok_or("Could not locate the existing mcpServers object")?;
        let comma = if servers.is_empty() { "" } else { "," };
        (offset, format!("\"cirnetrules\":{encoded}{comma}"))
    } else {
        let offset = original
            .iter()
            .position(|byte| *byte == b'{')
            .ok_or("Missing JSON root object")?
            + 1;
        let comma = if other_root_keys { "," } else { "" };
        (
            offset,
            format!("\"mcpServers\":{{\"cirnetrules\":{encoded}}}{comma}"),
        )
    };
    servers.insert("cirnetrules".into(), entry);
    let insertion = Insertion {
        offset,
        bytes: insertion.into_bytes(),
    };
    if parse_json(&insertion.apply(original)?)? != before {
        return Err("Project MCP JSON cannot be amended without affecting other settings".into());
    }
    Ok(Some(insertion))
}

pub(super) fn plan(
    agent: &str,
    original: Option<&[u8]>,
    template: &[u8],
) -> Result<Option<Insertion>, String> {
    match (agent, original) {
        ("codex", Some(bytes)) => toml_patch(bytes, template),
        (_, Some(bytes)) => json_patch(bytes, template),
        (_, None) => Ok(Some(Insertion {
            offset: 0,
            bytes: template.to_vec(),
        })),
    }
}

pub(super) fn preview(agent: &str, template: &[u8]) -> Result<Value, String> {
    let entry = if agent == "codex" {
        let mut source = parse_toml(template)?;
        let entry = toml_servers(&mut source)?
            .remove("cirnetrules")
            .ok_or("Missing Codex template server")?;
        serde_json::to_value(entry).map_err(|_| "Cannot render the Codex entry preview")?
    } else {
        let mut source = parse_json(template)?;
        json_servers(&mut source)?
            .remove("cirnetrules")
            .ok_or("Missing JSON template server")?
    };
    Ok(entry)
}
