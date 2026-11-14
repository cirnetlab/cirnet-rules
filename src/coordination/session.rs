use super::{
    clock, coordinate,
    model::{Clock, MAX_SESSIONS, Session, digest},
    scope,
};
use crate::{docs, tasks, workspace};
use serde_json::{Value, json};
use std::{fs, io::Write, path::Path};

const SESSION_DIR: &str = ".cirnetrules/sessions";
const SESSION_SECONDS: u64 = 86_400;

pub(super) fn random_id() -> Result<String, String> {
    let mut entropy = [0_u8; 32];
    getrandom::fill(&mut entropy)
        .map_err(|_| "OS randomness unavailable for local coordination")?;
    Ok(entropy.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn token_parts(token: &str) -> Result<(&str, &str), String> {
    let (task, secret) = token
        .split_once('.')
        .ok_or("Invalid local session capability")?;
    if tasks::valid_id(task).is_err() || !hex(secret) {
        return Err("Invalid local session capability".into());
    }
    Ok((task, secret))
}

pub(crate) fn contains_session_capability(value: &Value) -> bool {
    fn in_text(text: &str) -> bool {
        text.split(|character: char| {
            !character.is_ascii_alphanumeric() && !matches!(character, '-' | '_' | '.')
        })
        .any(|word| {
            let mut parts = word.split('.');
            let mut task = parts.next().unwrap_or_default();
            parts.any(|secret| {
                let capability = hex(secret) && tasks::valid_id(task).is_ok();
                task = secret;
                capability
            })
        })
    }

    match value {
        Value::String(text) => in_text(text),
        Value::Array(items) => items.iter().any(contains_session_capability),
        Value::Object(fields) => fields
            .iter()
            .any(|(name, value)| in_text(name) || contains_session_capability(value)),
        _ => false,
    }
}

fn workspace_scope(root: &Path, workspace: Option<&Path>) -> Result<String, String> {
    let authority = workspace::workspace_path(root, Some(root))?;
    let work = workspace::workspace_path(root, workspace)?;
    digest(&(authority, work))
}

pub(super) fn authenticate(
    root: &Path,
    work: Option<&Path>,
    task: &Value,
    token: &str,
    now: &Clock,
    current: bool,
) -> Result<String, String> {
    let (id, secret) = token_parts(token)?;
    if task["descriptor"]["id"] != id {
        return Err("Local session capability is unavailable for this task".into());
    }
    let secret_sha256 = docs::sha256(secret.as_bytes());
    let workspace = workspace_scope(root, work)?;
    let state = coordinate(task)?;
    let found = state
        .sessions
        .iter()
        .find(|(_, session)| constant_eq(&session.token_sha256, &secret_sha256));
    let (session_id, session) =
        found.ok_or("Local session capability is unavailable for this task")?;
    if session.workspace_sha256 != workspace
        || (current
            && (session.scope_sha256 != scope(task)?
                || now.expired_since(&session.issued_at, session.expires_at)))
    {
        return Err("Local session capability is stale or scoped to another workspace".into());
    }
    Ok(session_id.to_owned())
}

fn constant_eq(left: &str, right: &str) -> bool {
    left.len() == right.len()
        && left
            .bytes()
            .zip(right.bytes())
            .fold(0, |diff, (a, b)| diff | (a ^ b))
            == 0
}

pub(crate) fn check(root: &Path, work: Option<&Path>, token: &str) -> Result<(), String> {
    let (id, _) = token_parts(token)?;
    let task =
        tasks::load_current(root, id).map_err(|_| "Local session task authority is unavailable")?;
    authenticate(root, work, &task, token, &clock()?, true)?;
    Ok(())
}

pub(crate) fn read_token(root: &Path, relative: &str) -> Result<String, String> {
    let name = relative
        .strip_prefix(".cirnetrules/sessions/")
        .and_then(|name| name.strip_suffix(".token"))
        .filter(|name| hex(name))
        .ok_or("Capability file must be a prepared local session file")?;
    let relative = format!("{SESSION_DIR}/{name}.token");
    let path = workspace::scoped_path(root, &relative)?;
    let raw = super::credential::read(&path, 131)?;
    let token = raw.trim_end_matches(['\r', '\n']);
    token_parts(token)?;
    Ok(token.to_owned())
}

pub(super) fn prepare(
    root: &Path,
    work: Option<&Path>,
    task: &Value,
    actor: &str,
    now: &Clock,
    binding_sha256: String,
    state: &mut super::model::Coordination,
) -> Result<(String, Value), String> {
    tasks::actor_is(task, actor, true)?;
    if task["state"] != "READY" || task["descriptor"]["mode"] != "WORK" {
        return Err("Session preparation requires a preauthorized WORK task".into());
    }
    tasks::authorize(task, "read")?;
    tasks::authorize(task, "write")?;
    if !task["descriptor"]["write_scope"]
        .as_object()
        .is_some_and(|scope| {
            scope
                .values()
                .any(|paths| paths.as_array().is_some_and(|paths| !paths.is_empty()))
        })
    {
        return Err("Session preparation requires a nonempty approved worktree write scope".into());
    }
    let scope_sha256 = scope(task)?;
    let active = state
        .sessions
        .values()
        .filter(|session| {
            session.scope_sha256 == scope_sha256
                && !now.expired_since(&session.issued_at, session.expires_at)
        })
        .count();
    if active >= 16 || state.sessions.len() >= MAX_SESSIONS {
        return Err("Local session budget exhausted for this task".into());
    }
    let id = random_id()?;
    let secret = random_id()?;
    let expires_at = now.unix_seconds.saturating_add(SESSION_SECONDS).min(
        task["descriptor"]["authorization"]["expires_at"]
            .as_u64()
            .unwrap_or(u64::MAX),
    );
    let session = Session {
        token_sha256: docs::sha256(secret.as_bytes()),
        workspace_sha256: workspace_scope(root, work)?,
        scope_sha256,
        binding_sha256,
        role: task["descriptor"]["role"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        mode: task["descriptor"]["mode"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        issued_at: now.clone(),
        expires_at,
    };
    let task_id = task["descriptor"]["id"]
        .as_str()
        .ok_or("Invalid task identity")?;
    let folder = tasks::directory(root, SESSION_DIR, true)?;
    let file = folder.join(format!("{id}.token"));
    let mut out = super::credential::create(&file)?;
    if out
        .write_all(format!("{task_id}.{secret}\n").as_bytes())
        .and_then(|_| out.sync_all())
        .is_err()
    {
        drop(out);
        let _ = fs::remove_file(file);
        return Err("Cannot persist local capability file".into());
    }
    state.sessions.insert(id.clone(), session);
    Ok((
        id.clone(),
        json!({
            "session_id":id, "session_file":format!("{SESSION_DIR}/{id}.token"),
            "session_expires_at":expires_at
        }),
    ))
}
