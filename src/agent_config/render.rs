use super::{Bundle, Manifest, Output, SourceRecord, paths};
use crate::{
    authority::{AssetKind, Resolved, Source},
    docs, workspace,
};
use pulldown_cmark::{Event, LinkType, Parser, Tag};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const REFERENCE_ROOT: &str = "docs/cirnetrules";

pub(super) fn capability(agent: &str) -> Value {
    let (entry, skills, surface, sources) = match agent {
        "codex" => (
            "AGENTS.md",
            ".agents/skills",
            "Codex repository context",
            [
                "https://learn.chatgpt.com/docs/agent-configuration/agents-md",
                "https://learn.chatgpt.com/docs/build-skills",
            ],
        ),
        "claude" => (
            "CLAUDE.md",
            ".claude/skills",
            "Claude Code project context",
            [
                "https://code.claude.com/docs/en/memory",
                "https://code.claude.com/docs/en/skills",
            ],
        ),
        "antigravity" => (
            "AGENTS.md",
            ".agents/skills",
            "Google Antigravity workspace context (App, CLI and IDE)",
            [
                "https://antigravity.google/docs/rules/",
                "https://antigravity.google/docs/skills",
            ],
        ),
        _ => unreachable!("Only validated targets have capabilities"),
    };
    json!({
        "surface": surface, "entry": entry, "skills": skills,
        "documentation_checked": "2026-11-11", "sources": sources,
        "version_basis": "Current documented repository paths",
        "installed_version": "NOT_PROBED_BY_EXPORTER",
        "native_discovery": "NOT_RUN",
        "profiles": "CONTEXT_REFERENCE",
        "hooks_and_commands": "INERT_REFERENCE",
        "native_permissions_and_executable_configuration": "UNSUPPORTED"
    })
}

fn ref_path(path: &str) -> String {
    format!("{REFERENCE_ROOT}/{path}")
}

fn slug(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--")
        && !matches!(name, "synced" | "anthropic-skills")
        && name
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

pub(super) fn guarded(body: &[u8]) -> Result<&str, String> {
    if body.len() as u64 > paths::MAX_FILE {
        return Err("Governance content exceeds the agent export file limit".into());
    }
    let text = std::str::from_utf8(body).map_err(|_| "Agent export content must be UTF-8")?;
    let lower = text.to_ascii_lowercase();
    if text.starts_with('\u{feff}')
        || text
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
        || workspace::contains_known_secrets(text)
        || ["\\users\\", "/users/", "/home/", "file://", "\\\\?\\"]
            .iter()
            .any(|needle| lower.contains(needle))
    {
        return Err(
            "Governance content is outside the agent export text or secret boundary".into(),
        );
    }
    Ok(text)
}

struct Skill<'a> {
    name: &'a str,
    description: String,
    body: &'a str,
}

fn instruction_skill<'a>(path: &'a str, text: &'a str) -> Option<Skill<'a>> {
    let folder = path.strip_suffix("/SKILL.md")?.rsplit('/').next()?;
    if !slug(folder) {
        return None;
    }
    let mut frontmatter = text.trim_start_matches(['\r', '\n']);
    while let Some(comment) = frontmatter.strip_prefix("<!--") {
        let (comment, rest) = comment.split_once("-->")?;
        let comment = comment.trim();
        if !comment.starts_with("doc-id:") && !comment.starts_with("revision:") {
            return None;
        }
        frontmatter = rest.trim_start_matches(['\r', '\n']);
    }
    let frontmatter = frontmatter
        .strip_prefix("---\n")
        .or_else(|| frontmatter.strip_prefix("---\r\n"))?;
    let (header, body) = frontmatter.split_once("\n---")?;
    let body = body
        .strip_prefix("\n")
        .or_else(|| body.strip_prefix("\r\n"))?;
    let fields: BTreeMap<String, String> = serde_yaml::from_str(header).ok()?;
    let description = fields.get("description")?;
    if fields.len() != 2
        || fields.get("name").map(String::as_str) != Some(folder)
        || description.trim().is_empty()
        || description.len() > 1024
        || ["!`", "$("].iter().any(|pattern| body.contains(pattern))
    {
        return None;
    }
    Some(Skill {
        name: folder,
        description: description.to_owned(),
        body,
    })
}

fn relative_path(from: &str, to: &str) -> String {
    let from: Vec<_> = from
        .rsplit_once('/')
        .map_or("", |pair| pair.0)
        .split('/')
        .filter(|p| !p.is_empty())
        .collect();
    let to: Vec<_> = to.split('/').collect();
    let shared = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    format!(
        "{}{}",
        "../".repeat(from.len() - shared),
        to[shared..].join("/")
    )
}

fn link_target(from: &str, destination: &str) -> Option<String> {
    if destination.is_empty()
        || destination.starts_with('/')
        || destination.contains(['\\', ':', '?', '%'])
    {
        return None;
    }
    let mut parts: Vec<_> = from
        .rsplit_once('/')
        .map_or("", |pair| pair.0)
        .split('/')
        .filter(|p| !p.is_empty())
        .collect();
    for part in destination.split('/') {
        match part {
            "." => {}
            ".." => {
                parts.pop()?;
            }
            "" => return None,
            _ => parts.push(part),
        }
    }
    Some(parts.join("/"))
}

fn portable_markdown(
    text: &str,
    source: &str,
    output: &str,
    sources: &BTreeSet<&str>,
) -> Result<String, String> {
    let mut result = String::new();
    let mut offset = 0;
    for (event, range) in Parser::new(text).into_offset_iter() {
        let (link_type, dest, image) = match event {
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                ..
            }) => (link_type, dest_url, false),
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                ..
            }) => (link_type, dest_url, true),
            _ => continue,
        };
        if range.start < offset || dest.is_empty() || dest.starts_with('#') {
            continue;
        }
        let external = dest
            .strip_prefix("https://")
            .or_else(|| dest.strip_prefix("http://"));
        if let Some(address) = external {
            if address
                .split('/')
                .next()
                .is_none_or(|host| host.is_empty() || host.contains('@'))
            {
                return Err("Governance contains a link with credentials or no destination".into());
            }
            continue;
        }
        let (path, fragment) = dest.split_once('#').map_or((&*dest, ""), |(a, b)| (a, b));
        let Some(target) = link_target(source, path) else {
            return Err("Governance contains a link outside the portable path boundary".into());
        };
        paths::validate(&target)?;
        let target = sources.contains(target.as_str()).then(|| ref_path(&target));
        let original = &text[range.clone()];
        let label = if link_type == LinkType::Inline {
            original
                .rsplit_once("](")
                .map(|(before, _)| before.trim_start_matches('!').trim_start_matches('['))
        } else {
            original
                .trim_start_matches('!')
                .strip_prefix('[')
                .and_then(|inside| inside.split_once(']').map(|(label, _)| label))
        }
        .ok_or("Cannot safely rewrite a governance Markdown link")?;
        result.push_str(&text[offset..range.start]);
        if let Some(target) = target {
            result.push_str(&format!(
                "{}[{}]({}{})",
                if image { "!" } else { "" },
                label,
                relative_path(output, &target),
                if fragment.is_empty() {
                    String::new()
                } else {
                    format!("#{fragment}")
                },
            ));
        } else {
            result.push_str(label);
        }
        offset = range.end;
    }
    result.push_str(&text[offset..]);
    Ok(result)
}

fn output(
    files: &mut BTreeMap<String, Output>,
    path: String,
    owners: Vec<String>,
    bytes: Vec<u8>,
) -> Result<(), String> {
    guarded(&bytes)?;
    if files.insert(path, Output { owners, bytes }).is_some() {
        return Err("Two selected governance assets produce the same agent file".into());
    }
    Ok(())
}

fn source_record(source: &Source) -> SourceRecord {
    SourceRecord {
        id: source.id.clone(),
        kind: source.kind,
        path: source.path.clone(),
        revision: source.revision,
        dependencies: source.dependencies.clone(),
        sha256: source.sha256.clone(),
        trust: source.trust.into(),
    }
}

pub(super) fn bundle(
    resolved: &Resolved,
    agents: &[String],
    ownership: Option<&str>,
) -> Result<Bundle, String> {
    let mut capabilities: BTreeMap<_, _> = agents
        .iter()
        .map(|agent| (agent.clone(), capability(agent)))
        .collect();
    let mut entries: BTreeMap<_, Vec<_>> = BTreeMap::new();
    let mut skills: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for (agent, data) in &capabilities {
        entries
            .entry(data["entry"].as_str().unwrap().to_owned())
            .or_default()
            .push(agent.clone());
        skills
            .entry(data["skills"].as_str().unwrap().to_owned())
            .or_default()
            .push(agent.clone());
    }
    let mut files = BTreeMap::new();
    let mut inactive_skills = Vec::new();
    let sources = resolved.included.iter().chain(&resolved.manual);
    let source_paths: BTreeSet<_> = sources.clone().map(|row| row.path.as_str()).collect();
    let mut task_instructions = String::new();
    if let Some(task) = &resolved.task {
        task_instructions.push_str(&format!(
            "Use `cnr task context --task {task}` for the selected task and its exact scope.\n"
        ));
    } else {
        task_instructions.push_str(
            "Use `cnr task context` with the approved task ID to load its exact scope.\n",
        );
    }
    if let Some(profile) = &resolved.profile {
        task_instructions.push_str(&format!(
            "The resolved governance profile is `{profile}`.\n"
        ));
    }
    let mut entry = format!(
        "# Repository instructions\n\nRepository `{}` is governed by CIRNETLAB. {}\n\n\
         English policy and task authority come from the cirnet-rules Git repository.\n\
         {task_instructions}Check exact file grants before editing and record verification with `cnr task verify`.\n\n",
        resolved.repository,
        ownership.unwrap_or("Follow the registered repository ownership."),
    );
    let mut has_context = false;
    for (index, source) in sources.clone().enumerate() {
        paths::validate(&source.path)?;
        if !source.path.starts_with("en/")
            || ![".md", ".txt", ".json", ".yaml", ".yml"]
                .iter()
                .any(|ext| source.path.ends_with(ext))
            || docs::sha256(&source.bytes) != source.sha256
        {
            return Err(
                "Selected governance asset is outside the portable English content boundary".into(),
            );
        }
        let body = guarded(&source.bytes)?;
        let path = ref_path(&source.path);
        let reference = if source.path.ends_with(".md") {
            portable_markdown(body, &source.path, &path, &source_paths)?
        } else {
            body.to_owned()
        };
        output(
            &mut files,
            path.clone(),
            agents.to_vec(),
            reference.into_bytes(),
        )?;
        if index >= resolved.included.len() {
            continue;
        }
        match source.kind {
            AssetKind::Skill => {
                if let Some(skill) = instruction_skill(&source.path, body) {
                    let description = serde_json::to_string(&skill.description)
                        .map_err(|_| "Cannot encode skill description")?;
                    for (dir, owners) in &skills {
                        let path = format!("{dir}/{}/SKILL.md", skill.name);
                        let body =
                            portable_markdown(skill.body, &source.path, &path, &source_paths)?;
                        let skill = format!(
                            "---\nname: {}\ndescription: {description}\n---\n\n{}",
                            skill.name,
                            body.trim_start_matches(['\r', '\n'])
                        );
                        output(&mut files, path, owners.clone(), skill.into_bytes())?;
                    }
                } else {
                    inactive_skills.push(source.id.clone());
                }
            }
            AssetKind::Rule | AssetKind::Policy | AssetKind::Profile | AssetKind::Reference => {
                if !has_context {
                    entry.push_str("Read the selected governance context before acting.\n\n");
                    has_context = true;
                }
                entry.push_str(&format!("- [{}]({path})\n", source.id));
            }
            AssetKind::Hook | AssetKind::Command => {}
        }
    }
    if entry.len() > 16 * 1024 {
        return Err("Agent entry exceeds the repository instruction budget".into());
    }
    let shared = entries
        .get("AGENTS.md")
        .is_some_and(|owners| owners.len() > 1);
    let claude_import = entries.contains_key("AGENTS.md") && entries.contains_key("CLAUDE.md");
    for data in capabilities.values_mut() {
        data["shared_entry_and_skills"] = json!(shared && data["entry"] == "AGENTS.md");
        data["reference_only_skills"] = json!(&inactive_skills);
    }
    if claude_import && let Some(data) = capabilities.get_mut("claude") {
        data["entry_import"] = json!("AGENTS.md");
    }
    for (path, owners) in entries {
        let text = if claude_import && path == "CLAUDE.md" {
            "@AGENTS.md\n"
        } else {
            &entry
        };
        output(&mut files, path, owners, text.as_bytes().to_vec())?;
    }
    paths::validate_names(files.keys().map(String::as_str))?;
    let manifest = Manifest::new(
        resolved,
        agents,
        capabilities,
        sources.map(source_record).collect(),
        &files,
    );
    Ok(Bundle { manifest, files })
}
