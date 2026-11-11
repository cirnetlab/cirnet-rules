use super::files;
use crate::{docs::sha256, workspace::contains_known_secrets};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const DOCUMENT_BYTES: usize = 225_000;
const CONTENT_BYTES: usize = 220_000;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(super) struct Finding {
    pub(super) code: String,
    pub(super) asset_ref: String,
    pub(super) revision: Option<i64>,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Format {
    Markdown,
    Json,
    Yaml,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Attachment {
    pub(super) path: String,
    pub(super) content: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Document {
    pub(super) en: String,
    pub(super) vi: String,
    pub(super) format: Format,
    description: String,
    pub(super) files: Vec<Attachment>,
    pub(super) dependencies: Vec<String>,
}

pub(super) fn decode(raw: &str) -> Result<Document, &'static str> {
    if raw.len() > DOCUMENT_BYTES {
        return Err("DOCUMENT_BUDGET");
    }
    let document: Document = serde_json::from_str(raw).map_err(|_| "INVALID_DOCUMENT")?;
    if document.files.len() > 100
        || document.dependencies.len() > 32
        || document.en.len()
            + document.vi.len()
            + document
                .files
                .iter()
                .map(|file| file.content.len())
                .sum::<usize>()
            > CONTENT_BYTES
    {
        return Err("DOCUMENT_BUDGET");
    }
    let mut paths = BTreeSet::new();
    for file in &document.files {
        let parts: Vec<_> = file.path.split('/').collect();
        if file.path.len() > 240
            || parts.len() > 12
            || parts.iter().any(|part| {
                !files::portable_part(part)
                    || part.starts_with('.')
                    || !part
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"._- ".contains(&byte))
            })
            || !paths.insert(file.path.to_ascii_lowercase())
        {
            return Err("ATTACHMENT_PATH");
        }
    }
    for path in &paths {
        if path
            .match_indices('/')
            .any(|(index, _)| paths.contains(&path[..index]))
        {
            return Err("ATTACHMENT_PATH");
        }
    }
    let dependencies: BTreeSet<_> = document.dependencies.iter().collect();
    if dependencies.len() != document.dependencies.len()
        || !dependencies.iter().all(|id| asset_id(id))
    {
        return Err("DEPENDENCY_IDS");
    }
    Ok(document)
}

pub(super) fn asset_id(value: &str) -> bool {
    value.strip_prefix("asset-").is_some_and(|id| {
        id.len() == 32
            && id
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    })
}

pub(super) fn category(kind: &str) -> Option<&'static str> {
    match kind {
        "rule" => Some("rules"),
        "skill" => Some("skills"),
        "profile" => Some("profiles"),
        "reference" => Some("references"),
        "policy" => Some("policies"),
        "command" => Some("commands"),
        "hook" => Some("hooks"),
        _ => None,
    }
}

pub(super) fn valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 64
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && files::portable_part(slug)
}

fn translation_issue(document: &Document, prior: Option<&Document>) -> Option<&'static str> {
    if document.vi.trim().is_empty() {
        return Some("MISSING_VI");
    }
    let expected = sha256(document.en.replace("\r\n", "\n").as_bytes());
    let mut markers = document.vi.lines().filter_map(|line| {
        line.strip_prefix("<!-- source-sha256: ")?
            .strip_suffix(" -->")
    });
    if let Some(marker) = markers.next()
        && (marker != expected || markers.next().is_some())
    {
        return Some("TRANSLATION_HASH_MISMATCH");
    }
    if prior.is_some_and(|old| old.en != document.en && old.vi == document.vi) {
        return Some("STALE_VI");
    }
    if document.vi.trim() == document.en.trim() {
        return Some("TRANSLATION_REVIEW_REQUIRED");
    }
    None
}

pub(super) fn contains_sensitive(document: &Document) -> bool {
    [&document.en, &document.vi, &document.description]
        .into_iter()
        .any(|text| contains_known_secrets(text))
        || document
            .files
            .iter()
            .any(|file| contains_known_secrets(&file.content))
}

struct Revision {
    number: i64,
    digest: String,
    raw: String,
    canonical_bytes: usize,
    prior: Option<i64>,
    rollback: bool,
}

fn add(output: &mut Vec<Finding>, id: &str, revision: Option<i64>, code: &str) {
    output.push(Finding {
        code: code.into(),
        asset_ref: sha256(id.as_bytes()),
        revision,
    });
}

fn verify_document(
    output: &mut Vec<Finding>,
    id: &str,
    revision: Option<i64>,
    raw: &str,
    prior: Option<&Document>,
) -> Option<Document> {
    match decode(raw) {
        Ok(document) => {
            if document.en.trim().is_empty() {
                add(output, id, revision, "MISSING_EN");
            }
            if let Some(issue) = translation_issue(&document, prior) {
                add(output, id, revision, issue);
            }
            if contains_sensitive(&document) {
                add(output, id, revision, "SENSITIVE_DOCUMENT");
            }
            Some(document)
        }
        Err(issue) => {
            add(output, id, revision, issue);
            None
        }
    }
}

fn sql_error(_: rusqlite::Error) -> String {
    "Cannot read legacy governance records".into()
}

pub(super) fn findings(db: &Connection) -> Result<Vec<Finding>, String> {
    let mut statement = db.prepare(
        "SELECT id,kind,slug,revision,active_revision,draft_json FROM governance_assets ORDER BY id",
    ).map_err(sql_error)?;
    let mut assets = statement.query([]).map_err(sql_error)?;
    let mut revision_query = db.prepare(
        "SELECT revision,digest,document_json,canonical_bytes,prior_revision,event FROM governance_revisions
         WHERE asset_id=?1 ORDER BY revision",
    ).map_err(sql_error)?;
    let mut relation_query = db.prepare(
        "SELECT target_id FROM governance_dependencies WHERE asset_id=?1 AND layer=?2 ORDER BY target_id",
    ).map_err(sql_error)?;
    let mut output = Vec::new();
    while let Some(asset) = assets.next().map_err(sql_error)? {
        let id: String = asset.get(0).map_err(sql_error)?;
        let kind: String = asset.get(1).map_err(sql_error)?;
        let slug: String = asset.get(2).map_err(sql_error)?;
        let current: i64 = asset.get(3).map_err(sql_error)?;
        let active: Option<i64> = asset.get(4).map_err(sql_error)?;
        let draft: Option<String> = asset.get(5).map_err(sql_error)?;
        if !asset_id(&id) || category(&kind).is_none() || !valid_slug(&slug) {
            add(&mut output, &id, None, "INVALID_ASSET_IDENTITY");
        }
        let history = revision_query
            .query_map([&id], |row| {
                Ok(Revision {
                    number: row.get(0)?,
                    digest: row.get(1)?,
                    raw: row.get(2)?,
                    canonical_bytes: row.get(3)?,
                    prior: row.get(4)?,
                    rollback: row.get::<_, String>(5)? == "ROLLBACK",
                })
            })
            .map_err(sql_error)?;
        let mut known = BTreeMap::new();
        for entry in history {
            let Revision {
                number,
                digest,
                raw,
                canonical_bytes,
                prior,
                rollback,
            } = entry.map_err(sql_error)?;
            if number > current || number < 1 || digest != sha256(raw.as_bytes()) {
                add(&mut output, &id, Some(number), "REVISION_DIGEST_OR_NUMBER");
            }
            if (rollback && prior.is_none())
                || prior
                    .is_some_and(|revision| revision >= number || !known.contains_key(&revision))
            {
                add(&mut output, &id, Some(number), "MISSING_PRIOR_REVISION");
            }
            let previous = prior
                .and_then(|prior| known.get(&prior))
                .and_then(Option::as_ref);
            let document = verify_document(&mut output, &id, Some(number), &raw, previous);
            if document
                .as_ref()
                .is_some_and(|document| document.en.len() != canonical_bytes)
            {
                add(&mut output, &id, Some(number), "REVISION_BYTE_COUNT");
            }
            known.insert(number, document);
        }
        let current_document = active
            .and_then(|number| known.get(&number))
            .and_then(Option::as_ref);
        if active.is_some_and(|number| !known.contains_key(&number)) {
            add(&mut output, &id, active, "MISSING_ACTIVE_REVISION");
        }
        let draft_document = draft
            .as_deref()
            .and_then(|raw| verify_document(&mut output, &id, None, raw, current_document));
        for (layer, document, revision) in [
            ("draft", draft_document.as_ref(), None),
            ("active", current_document, active),
        ] {
            let recorded: BTreeSet<String> = relation_query
                .query_map((&id, layer), |row| row.get(0))
                .map_err(sql_error)?
                .collect::<Result<_, _>>()
                .map_err(sql_error)?;
            let declared = document
                .into_iter()
                .flat_map(|document| document.dependencies.iter().cloned())
                .collect();
            if recorded != declared {
                add(&mut output, &id, revision, "DEPENDENCY_LAYER_MISMATCH");
            }
        }
    }
    Ok(output)
}
