use super::{
    catalog,
    storage::{AppError, Database},
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

mod lifecycle;
mod sqlite;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Create {
    key: String,
    title: String,
    #[serde(default = "handoff")]
    profile: String,
    profile_id: Option<String>,
    bindings: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Send {
    id: String,
    key: String,
    content: String,
    revision: Option<i64>,
    #[serde(default)]
    read_scope: BTreeMap<String, Vec<String>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Edit {
    id: String,
    revision: i64,
    title: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cancel {
    id: String,
    task_id: String,
}

pub(super) fn parse<T: DeserializeOwned>(value: Value) -> Result<T, AppError> {
    serde_json::from_value(value)
        .map_err(|_| AppError::new("INVALID_REQUEST", "Invalid request fields or types"))
}

pub(super) fn id(value: &str) -> Result<i64, AppError> {
    value
        .parse::<i64>()
        .ok()
        .filter(|id| *id > 0 && id.to_string() == value)
        .ok_or_else(|| AppError::new("INVALID_REQUEST", "Invalid record ID"))
}

pub(super) fn request_key(value: &str) -> Result<(), AppError> {
    if value.is_empty()
        || value.len() > 80
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
    {
        return Err(AppError::new(
            "INVALID_REQUEST",
            "Idempotency key must contain 1–80 letters, digits, underscores or hyphens",
        ));
    }
    Ok(())
}

pub(super) fn handoff() -> String {
    "handoff".into()
}

pub(super) fn profile(value: &str) -> Result<(), AppError> {
    if value != "handoff" {
        return Err(AppError::new(
            "CAPABILITY_BLOCKED",
            "The selected Agent profile is not available through this runtime",
        ));
    }
    Ok(())
}

pub(super) fn text(value: &str, max: usize) -> Result<&str, AppError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > max
        || value
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        return Err(AppError::new(
            "INVALID_REQUEST",
            "Text is empty, too long, or contains unsupported characters",
        ));
    }
    if sensitive_text(value) {
        return Err(AppError::new(
            "SENSITIVE_INPUT",
            "Remove credentials before submitting text",
        ));
    }
    Ok(value)
}

pub(super) fn sensitive_text(value: &str) -> bool {
    let upper = value.to_ascii_uppercase();
    [
        "-----BEGIN PRIVATE KEY-----",
        "-----BEGIN OPENSSH PRIVATE KEY-----",
        "-----BEGIN RSA PRIVATE KEY-----",
        "-----BEGIN ENCRYPTED PRIVATE KEY-----",
        "-----BEGIN EC PRIVATE KEY-----",
        "AUTHORIZATION: BEARER",
        "OPENAI_API_KEY=",
        "ANTHROPIC_API_KEY=",
        "GITHUB_TOKEN=",
        "AWS_SECRET_ACCESS_KEY=",
        "PASSWORD=",
        "GITHUB_PAT_",
        "GHP_",
        "GHO_",
        "XOXB-",
    ]
    .iter()
    .any(|marker| upper.contains(marker))
}

pub(super) fn digest(value: &Value) -> String {
    format!("{:x}", Sha256::digest(value.to_string().as_bytes()))
}

pub(super) fn limit(value: &Value) -> Result<i64, AppError> {
    if value.is_null() {
        return Ok(50);
    }
    value
        .as_i64()
        .filter(|n| (1..=100).contains(n))
        .ok_or_else(|| AppError::new("INVALID_REQUEST", "Page size must be from 1 to 100"))
}

pub(super) fn cursor(value: &Value, default: i64) -> Result<i64, AppError> {
    if value.is_null() {
        return Ok(default);
    }
    let raw = value
        .as_str()
        .ok_or_else(|| AppError::new("INVALID_REQUEST", "Invalid pagination cursor"))?;
    if raw == "0" {
        return Ok(0);
    }
    id(raw)
}

pub(super) fn now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_secs().min(i64::MAX as u64) as i64,
        Err(earlier) => -(earlier.duration().as_secs().min(i64::MAX as u64) as i64),
    }
}

fn conflict(message: &'static str) -> AppError {
    AppError::new("CONFLICT", message)
}
fn missing(message: &'static str) -> AppError {
    AppError::new("NOT_FOUND", message)
}

pub(super) fn matches_binding(
    conn: &Connection,
    binding: &catalog::Binding,
) -> Result<(), AppError> {
    let valid: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM bindings b JOIN repositories r ON r.id=b.repo_id
            WHERE b.id=?1 AND b.revision=?2 AND b.status='ACTIVE' AND r.id=?3 AND r.archived=0)",
        params![binding.id, binding.revision, binding.repo_id],
        |row| row.get(0),
    )?;
    if valid {
        Ok(())
    } else {
        Err(conflict("Repository binding changed before commit"))
    }
}

fn conversation_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    Ok(json!({
        "id": row.get::<_,i64>(0)?.to_string(), "title": row.get::<_,String>(1)?,
        "state": row.get::<_,String>(2)?, "profile": row.get::<_,String>(3)?,
        "source": row.get::<_,String>(4)?, "revision": row.get::<_,i64>(5)?,
        "created_at": row.get::<_,i64>(6)?, "updated_at": row.get::<_,i64>(7)?,
        "provider_thread":row.get::<_,Option<String>>(8)?
    }))
}
const CONVERSATION_COLS: &str =
    "id,title,state,profile,source,revision,created_at,updated_at,provider_thread";

fn conversation(conn: &Connection, id: i64) -> Result<Value, AppError> {
    let query = format!("SELECT {CONVERSATION_COLS} FROM conversations WHERE id=?1");
    let mut result = conn
        .query_row(&query, [id], conversation_row)
        .optional()?
        .ok_or_else(|| missing("Conversation does not exist"))?;
    if let Some(snapshot) = super::governance::conversation_snapshot(conn, id)? {
        result["governance"] = snapshot;
    }
    Ok(result)
}

fn bound(conn: &Connection, conversation_id: i64) -> Result<Vec<(String, i64)>, AppError> {
    let mut statement = conn.prepare("SELECT binding_id,binding_revision FROM conversation_bindings WHERE conversation_id=?1 ORDER BY ordinal")?;
    Ok(statement
        .query_map([conversation_id], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?)
}

fn existing_create(conn: &Connection, key: &str, hash: &str) -> Result<Option<Value>, AppError> {
    sqlite::retired_key(conn, None, key)?;
    let row = conn
        .query_row(
            "SELECT id,create_digest FROM conversations WHERE create_key=?1",
            [key],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;
    match row {
        None => Ok(None),
        Some((id, original)) if original == hash => Ok(Some(conversation(conn, id)?)),
        Some(_) => Err(conflict(
            "Idempotency key has already been used for another conversation",
        )),
    }
}

fn insert_conversation(
    conn: &Connection,
    key: &str,
    hash: &str,
    title: &str,
    source: &str,
    bindings: &[catalog::Binding],
    time: i64,
) -> Result<i64, AppError> {
    let id = sqlite::next_id(conn, sqlite::Sequence::Conversation)?;
    conn.execute(
        "INSERT INTO conversations(id,create_key,create_digest,title,profile,state,source,revision,created_at,updated_at)
        VALUES(?1,?2,?3,?4,'handoff','ACTIVE',?5,1,?6,?6)",params![id,key,hash,title,source,time]
    )?;
    for (index, binding) in bindings.iter().enumerate() {
        matches_binding(conn, binding)?;
        conn.execute(
            "INSERT INTO conversation_bindings VALUES(?1,?2,?3,?4)",
            params![id, binding.id, binding.revision, index as i64],
        )?;
    }
    Ok(id)
}

pub(super) fn event(
    conn: &Connection,
    refs: (Option<i64>, Option<i64>, Option<i64>, Option<i64>),
    correlation: &str,
    kind: &str,
    payload: &Value,
    time: i64,
) -> Result<(), AppError> {
    let sequence = sqlite::next_id(conn, sqlite::Sequence::Event)?;
    conn.execute(
        "INSERT INTO session_events(sequence,conversation_id,task_id,run_id,attempt,correlation,event_type,payload_json,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![sequence,refs.0,refs.1,refs.2,refs.3,correlation,kind,payload.to_string(),time]
    )?;
    Ok(())
}

pub(super) fn safe_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && !value.contains(['\\', ':', '*', '?', '"', '<', '>', '|'])
        && !value.chars().any(char::is_control)
        && value.split('/').all(|part| {
            let lower = part.to_ascii_lowercase();
            let stem = lower.split('.').next().unwrap_or("");
            !matches!(part, "" | "." | "..")
                && !part.ends_with([' ', '.'])
                && !matches!(
                    lower.as_str(),
                    ".git"
                        | ".cirnetrules"
                        | ".openrules"
                        | ".cgacp"
                        | ".ssh"
                        | ".env"
                        | ".npmrc"
                        | ".netrc"
                        | "credentials"
                        | "credentials.json"
                        | "id_rsa"
                        | "id_ed25519"
                        | "secrets"
                )
                && !matches!(stem, "con" | "prn" | "aux" | "nul" | "conin$" | "conout$")
                && !(stem.len() == 4
                    && (stem.starts_with("com") || stem.starts_with("lpt"))
                    && stem.as_bytes()[3].is_ascii_digit())
                && !lower.starts_with(".env.")
                && ![".key", ".pem", ".p12", ".pfx"]
                    .iter()
                    .any(|suffix| lower.ends_with(suffix))
        })
}

fn review_boundary(path: &str) -> bool {
    path.split(['/', '.', '-', '_']).any(|part| {
        matches!(
            part.to_ascii_lowercase().as_str(),
            "auth"
                | "authentication"
                | "api"
                | "run"
                | "stop"
                | "safety"
                | "permissions"
                | "deployment"
                | "schema"
        )
    })
}

fn validate_hint(root: &std::path::Path, relative: &str) -> Result<(), AppError> {
    let mut cursor = root.to_path_buf();
    for component in relative.split('/') {
        cursor.push(component);
        match std::fs::symlink_metadata(&cursor) {
            Ok(metadata) if super::storage::redirect(&metadata) => {
                return Err(AppError::new(
                    "SCOPE_DENIED",
                    "Read hint crosses a redirected path",
                ));
            }
            Ok(_) => {}
            // Proposals can name files that have not been created yet.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(_) => return Err(AppError::new("INACCESSIBLE", "Read hint cannot be checked")),
        }
    }
    Ok(())
}

pub(super) fn route(
    bindings: &[catalog::Binding],
    scope: &BTreeMap<String, Vec<String>>,
) -> Result<(Value, &'static str), AppError> {
    let count = scope.values().map(Vec::len).sum::<usize>();
    if count > 32
        || scope.iter().any(|(id, paths)| {
            !bindings.iter().any(|binding| &binding.id == id)
                || paths.iter().any(|path| !safe_path(path))
                || paths.iter().collect::<BTreeSet<_>>().len() != paths.len()
        })
    {
        return Err(AppError::new(
            "SCOPE_DENIED",
            "Read hints must be unique safe relative paths for selected bindings",
        ));
    }
    let risk = if scope.values().flatten().any(|path| review_boundary(path)) {
        "REVIEW_REQUIRED"
    } else {
        "SCOPED"
    };
    let selected: Vec<_> = bindings.iter().map(|binding| json!({
        "binding_id":binding.id,"repo_id":binding.repo_id,"revision":binding.revision,"commit":binding.commit,
        "worktree_identity":binding.identity,
        "read_hints":scope.get(&binding.id).cloned().unwrap_or_default(),
    })).collect();
    for binding in bindings {
        if let Some(paths) = scope.get(&binding.id) {
            for path in paths {
                validate_hint(&binding.root, path)?;
            }
        }
    }
    Ok((
        json!({"bindings":selected,"write_scope":[],"network":false,"release_checked":false,
        "files_read":0,"metadata_only":true,"requires_review":risk=="REVIEW_REQUIRED"}),
        risk,
    ))
}

fn message_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    Ok(
        json!({"id":row.get::<_,i64>(0)?.to_string(),"ordinal":row.get::<_,i64>(1)?,
        "role":row.get::<_,String>(2)?,"content":row.get::<_,String>(3)?,"created_at":row.get::<_,i64>(4)?}),
    )
}

fn stored_json(input: String) -> Result<Value, AppError> {
    serde_json::from_str(&input)
        .map_err(|_| AppError::new("INTEGRITY", "Stored runtime data is invalid"))
}

fn task(conn: &Connection, id: i64) -> Result<Value, AppError> {
    let (state, mode, risk, route): (String, String, String, String) = conn.query_row(
        "SELECT state,mode,risk,route_json FROM task_proposals WHERE id=?1",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )?;
    Ok(
        json!({"id":id.to_string(),"state":state,"mode":mode,"risk":risk,"route":stored_json(route)?}),
    )
}

fn result_for(conn: &Connection, message_id: i64, task_id: i64) -> Result<Value, AppError> {
    let message = conn.query_row(
        "SELECT id,ordinal,role,content,created_at FROM messages WHERE id=?1",
        [message_id],
        message_row,
    )?;
    let revision: i64 = conn.query_row(
        "SELECT c.revision FROM conversations c JOIN messages m ON m.conversation_id=c.id WHERE m.id=?1",
        [message_id], |row| row.get(0),
    )?;
    Ok(json!({"message":message,"task":task(conn,task_id)?,"conversation_revision":revision}))
}

fn existing_send(
    conn: &Connection,
    conversation_id: i64,
    key: &str,
    hash: &str,
) -> Result<Option<Value>, AppError> {
    sqlite::retired_key(conn, Some(conversation_id), key)?;
    let row = conn.query_row(
        "SELECT m.id,t.id,m.request_digest FROM messages m JOIN task_proposals t ON t.message_id=m.id WHERE m.conversation_id=?1 AND m.request_key=?2",
        params![conversation_id,key], |row| Ok((row.get::<_,i64>(0)?,row.get::<_,i64>(1)?,row.get::<_,String>(2)?)),
    ).optional()?;
    match row {
        None => Ok(None),
        Some((message, task, original)) if original == hash => {
            Ok(Some(result_for(conn, message, task)?))
        }
        Some(_) => Err(conflict(
            "Idempotency key has already been used for another turn",
        )),
    }
}

pub(super) struct Intake<'a> {
    pub content: &'a str,
    pub route: &'a Value,
    pub mode: &'a str,
    pub risk: &'a str,
    pub time: i64,
}

fn insert_turn(
    conn: &Connection,
    conversation_id: i64,
    key: &str,
    hash: &str,
    request: &Intake<'_>,
    run: Option<(i64, i64)>,
) -> Result<(i64, i64), AppError> {
    if conn.execute(
        "UPDATE conversations SET updated_at=?2,revision=revision+1 WHERE id=?1 AND state='ACTIVE'",
        params![conversation_id, request.time],
    )? != 1
    {
        return Err(conflict("Conversation is archived"));
    }
    let ordinal = sqlite::next_ordinal(conn, conversation_id)?;
    let message_id = sqlite::next_id(conn, sqlite::Sequence::Message)?;
    conn.execute(
        "INSERT INTO messages(id,conversation_id,request_key,request_digest,ordinal,role,content,created_at)
        VALUES (?1,?2,?3,?4,?5,'USER',?6,?7)",
        params![message_id,conversation_id,key,hash,ordinal,request.content,request.time],
    )?;
    let task_id = sqlite::next_id(conn, sqlite::Sequence::Proposal)?;
    conn.execute(
        "INSERT INTO task_proposals(id,conversation_id,message_id,state,mode,risk,route_json,created_at) VALUES(?1,?2,?3,'WAITING_APPROVAL',?4,?5,?6,?7)",
        params![task_id,conversation_id,message_id,request.mode,request.risk,request.route.to_string(),request.time],
    )?;
    event(
        conn,
        (
            Some(conversation_id),
            Some(task_id),
            run.map(|value| value.0),
            run.map(|value| value.1),
        ),
        key,
        "TURN_PROPOSED",
        &json!({"message_id":message_id.to_string(),"state":"WAITING_APPROVAL"}),
        request.time,
    )?;
    Ok((message_id, task_id))
}

pub(super) fn from_automation(
    conn: &Connection,
    run: (i64, i64),
    name: &str,
    binding: &catalog::Binding,
    request: &Intake<'_>,
) -> Result<(i64, i64), AppError> {
    let key = format!("auto-{}", run.0);
    // A dashboard request is not allowed to use the auto- prefix.
    let hash = digest(&json!([run.0, run.1, request.route]));
    let id = insert_conversation(
        conn,
        &key,
        &hash,
        name,
        "AUTOMATION",
        std::slice::from_ref(binding),
        request.time,
    )?;
    super::governance::copy_run_snapshot(conn, run.0, id)?;
    let (_, task_id) = insert_turn(conn, id, &key, &hash, request, Some(run))?;
    Ok((id, task_id))
}

pub fn dispatch(db: &mut Database, method: &str, params: Value) -> Result<Value, AppError> {
    match method {
        "conversation.create" => create(db, parse(params)?),
        "conversation.send" => send(db, parse(params)?),
        "conversation.rename" => rename(db, parse(params)?),
        "conversation.archive"
        | "conversation.unarchive"
        | "conversation.list"
        | "conversation.delete.preview"
        | "message.delete.preview"
        | "conversation.delete"
        | "message.delete" => {
            lifecycle::dispatch(&mut sqlite::Repository(&mut db.conn), method, params)
        }
        "conversation.cancel" => cancel(db, parse(params)?),
        "conversation.get" | "conversation.resume" => get(db, method, &params),
        "conversation.events" => events(db, &params),
        _ => Err(AppError::new(
            "METHOD_NOT_FOUND",
            "Unsupported conversation operation",
        )),
    }
}

fn create(db: &mut Database, request: Create) -> Result<Value, AppError> {
    request_key(&request.key)?;
    if request.key.starts_with("auto-") {
        return Err(AppError::new(
            "INVALID_REQUEST",
            "Reserved idempotency key prefix",
        ));
    }
    let title = text(&request.title, 160)?;
    profile(&request.profile)?;
    if request.bindings.is_empty()
        || request.bindings.len() > 8
        || request.bindings.iter().collect::<BTreeSet<_>>().len() != request.bindings.len()
    {
        return Err(AppError::new(
            "INVALID_REQUEST",
            "Select one to eight distinct repository bindings",
        ));
    }
    let hash = if let Some(profile_id) = &request.profile_id {
        digest(&json!([
            title,
            request.profile,
            request.bindings,
            profile_id
        ]))
    } else {
        digest(&json!([title, request.profile, request.bindings]))
    };
    if let Some(result) = existing_create(&db.conn, &request.key, &hash)? {
        return Ok(result);
    }
    let bindings = request
        .bindings
        .iter()
        .map(|id| catalog::resolve_binding(db, id, None))
        .collect::<Result<Vec<_>, _>>()?;
    let time = now();
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(result) = existing_create(&tx, &request.key, &hash)? {
        return Ok(result);
    }
    let id = insert_conversation(
        &tx,
        &request.key,
        &hash,
        title,
        "DASHBOARD",
        &bindings,
        time,
    )?;
    super::governance::pin_conversation(&tx, id, request.profile_id.as_deref(), &request.bindings)?;
    event(
        &tx,
        (Some(id), None, None, None),
        &request.key,
        "CONVERSATION_CREATED",
        &json!({}),
        time,
    )?;
    let result = conversation(&tx, id)?;
    tx.commit()?;
    Ok(result)
}

fn send(db: &mut Database, request: Send) -> Result<Value, AppError> {
    let id = id(&request.id)?;
    request_key(&request.key)?;
    let content = text(&request.content, 16_384)?;
    let hash = digest(&json!([content, request.read_scope]));
    if let Some(existing) = existing_send(&db.conn, id, &request.key, &hash)? {
        return Ok(existing);
    }
    if conversation(&db.conn, id)?["state"] != "ACTIVE" {
        return Err(conflict("Conversation is archived"));
    }
    let bindings = bound(&db.conn, id)?
        .into_iter()
        .map(|(binding, revision)| catalog::resolve_binding(db, &binding, Some(revision)))
        .collect::<Result<Vec<_>, _>>()?;
    let (route, risk) = route(&bindings, &request.read_scope)?;
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(existing) = existing_send(&tx, id, &request.key, &hash)? {
        return Ok(existing);
    }
    if let Some(revision) = request.revision
        && conversation(&tx, id)?["revision"] != revision
    {
        return Err(conflict("Conversation revision is stale"));
    }
    for binding in &bindings {
        matches_binding(&tx, binding)?;
    }
    let intake = Intake {
        content,
        route: &route,
        mode: "WORK",
        risk,
        time: now(),
    };
    let (message_id, task_id) = insert_turn(&tx, id, &request.key, &hash, &intake, None)?;
    let result = result_for(&tx, message_id, task_id)?;
    tx.commit()?;
    Ok(result)
}

fn rename(db: &mut Database, request: Edit) -> Result<Value, AppError> {
    let id = id(&request.id)?;
    let time = now();
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    let title = text(request.title.as_deref().unwrap_or(""), 160)?;
    let changed=tx.execute(
        "UPDATE conversations SET title=?3,revision=revision+1,updated_at=?4 WHERE id=?1 AND revision=?2",
        params![id,request.revision,title,time])?;
    if changed != 1 {
        return Err(conflict("Conversation revision is stale"));
    }
    event(
        &tx,
        (Some(id), None, None, None),
        &request.id,
        "CONVERSATION_RENAMED",
        &json!({}),
        time,
    )?;
    let result = conversation(&tx, id)?;
    tx.commit()?;
    Ok(result)
}

fn cancel(db: &mut Database, request: Cancel) -> Result<Value, AppError> {
    let cid = id(&request.id)?;
    let tid = id(&request.task_id)?;
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    if tx.execute("UPDATE task_proposals SET state='CANCELLED' WHERE id=?1 AND conversation_id=?2 AND state='WAITING_APPROVAL'",params![tid,cid])? != 1 {
        return Err(conflict("Only a waiting proposal in this conversation may be cancelled"));
    }
    let time = now();
    let run: Option<i64> = tx.query_row(
        "UPDATE automation_runs SET status='CANCELLED',outcome='OPERATOR_CANCELLED',updated_at=?2
         WHERE task_id=?1 AND status='WAITING_APPROVAL' RETURNING id",
        params![tid,time],
        |row|row.get(0),
    ).optional()?;
    tx.execute(
        "UPDATE conversations SET revision=revision+1,updated_at=?2 WHERE id=?1",
        params![cid, time],
    )?;
    event(
        &tx,
        (Some(cid), Some(tid), run, None),
        &request.task_id,
        "PROPOSAL_CANCELLED",
        &json!({}),
        time,
    )?;
    let result = task(&tx, tid)?;
    tx.commit()?;
    Ok(result)
}

fn get(db: &Database, method: &str, params: &Value) -> Result<Value, AppError> {
    let id = id(params["id"].as_str().unwrap_or(""))?;
    let after = !params["after"].is_null();
    if after && !params["before"].is_null() {
        return Err(AppError::new(
            "INVALID_REQUEST",
            "Choose one message page direction",
        ));
    }
    let cursor = if after {
        cursor(&params["after"], 0)?
    } else {
        cursor(&params["before"], i64::MAX)?
    };
    let limit = limit(&params["limit"])?.min(4);
    let predicate = if after {
        ">?2 ORDER BY m.ordinal ASC"
    } else {
        "<?2 ORDER BY m.ordinal DESC"
    };
    let tx = db.conn.unchecked_transaction()?;
    let conversation = conversation(&tx, id)?;
    let mut rows = {
        let mut statement = tx.prepare(&format!(
            "SELECT m.id,m.ordinal,m.role,m.content,m.created_at,t.id,t.state,t.mode,t.risk,t.route_json
            FROM messages m LEFT JOIN task_proposals t ON t.message_id=m.id
            WHERE m.conversation_id=?1 AND m.ordinal{predicate} LIMIT ?3"
        ))?;
        statement
            .query_map(params![id, cursor, limit + 1], |row| {
                let task = row
                    .get::<_, Option<i64>>(5)?
                    .map(|id| -> rusqlite::Result<_> {
                        Ok((
                            id.to_string(),
                            row.get::<_, String>(6)?,
                            row.get::<_, String>(7)?,
                            row.get::<_, String>(8)?,
                            row.get::<_, String>(9)?,
                        ))
                    })
                    .transpose()?;
                Ok((message_row(row)?, task))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
    };
    let has_more = rows.len() as i64 > limit;
    if has_more {
        rows.pop();
    }
    if !after {
        rows.reverse();
    }
    let mut messages = Vec::with_capacity(rows.len());
    let mut tasks = Vec::with_capacity(rows.len());
    for (message, task) in rows {
        if let Some((task_id, state, mode, risk, route)) = task {
            tasks.push(
                json!({"id":task_id,"message_id":message["id"],"state":state,
                "mode":mode,"risk":risk,"route":stored_json(route)?}),
            );
        }
        messages.push(message);
    }
    tasks.reverse();
    let bindings = bound(&tx, id)?;
    let (blocker_count, blockers) = sqlite::blockers(
        &tx,
        lifecycle::Target {
            conversation: id,
            message: None,
        },
    )?;
    tx.commit()?;
    if method == "conversation.resume" {
        for (binding, revision) in &bindings {
            catalog::resolve_binding(db, binding, Some(*revision))?;
        }
    }
    let selections: Vec<_> = bindings
        .into_iter()
        .map(|(binding, revision)| json!({"binding_id":binding,"revision":revision}))
        .collect();
    let next_after = messages
        .last()
        .and_then(|row| row["ordinal"].as_i64())
        .or(after.then_some(cursor))
        .unwrap_or(0)
        .to_string();
    Ok(
        json!({"conversation":conversation,"next_before":messages.first().and_then(|row|row["ordinal"].as_i64()).map(|n|n.to_string()),
        "next_after":next_after,"has_more":has_more,"messages":messages,"tasks":tasks,"bindings":selections,
        "blockers":blockers,"blocker_count":blocker_count}),
    )
}

fn events(db: &Database, params: &Value) -> Result<Value, AppError> {
    let cid = id(params["id"].as_str().unwrap_or(""))?;
    conversation(&db.conn, cid)?;
    replay(&db.conn, Some(cid), None, params)
}

pub(super) fn replay(
    conn: &Connection,
    conversation: Option<i64>,
    run: Option<i64>,
    params: &Value,
) -> Result<Value, AppError> {
    let after = cursor(&params["after"], 0)?;
    let column = if conversation.is_some() {
        "conversation_id"
    } else {
        "run_id"
    };
    let selected = conversation
        .or(run)
        .ok_or_else(|| AppError::new("INVALID_REQUEST", "Event target is required"))?;
    let mut statement=conn.prepare(&format!("SELECT sequence,task_id,run_id,attempt,correlation,event_type,payload_json,created_at,conversation_id
        FROM session_events WHERE {column}=?1 AND sequence>?2 ORDER BY sequence LIMIT ?3"))?;
    let rows = statement
        .query_map(params![selected, after, limit(&params["limit"])?], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<i64>>(1)?,
                row.get::<_, Option<i64>>(2)?,
                row.get::<_, Option<i64>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, Option<i64>>(8)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let next = rows.last().map_or(after, |row| row.0);
    let items=rows.into_iter().map(|(sequence,task,run,attempt,correlation,kind,payload,time,conversation)|Ok(json!({
        "sequence":sequence.to_string(),"conversation_id":conversation.map(|n|n.to_string()),"task_id":task.map(|n|n.to_string()),
        "run_id":run.map(|n|n.to_string()),"attempt":attempt,"correlation":correlation,"type":kind,
        "payload":stored_json(payload)?,"created_at":time
    }))).collect::<Result<Vec<_>,AppError>>()?;
    Ok(json!({"items":items,"next_after":next.to_string()}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protected_or_escaping_paths_cannot_be_proposed() {
        for path in [
            "../repo/file",
            "C:/outside",
            "src/.git/config",
            ".env",
            "dir/.ssh/id_rsa",
            "src/con.rs",
            "src//file",
        ] {
            assert!(!safe_path(path), "{path}");
        }
        assert!(safe_path("src/thư mục/toolbar.vue"));
        assert!(review_boundary("src/auth/session.rs"));
        assert!(text("Authorization: Bearer fixture", 1024).is_err());
    }
}
