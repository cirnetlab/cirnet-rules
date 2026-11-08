//! Repository and worktree operations for the local Dashboard.
mod git;
#[cfg(test)]
mod tests;

use super::storage::{self, AppError, Database};
#[cfg(test)]
use git::git_executable;
use git::{Checkout, contains_commit, inspect};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

const BINDING_ROW: &str = "
 SELECT b.id, b.repo_id, b.selected_path, b.root, b.git_dir, b.common_dir,
        b.identity, b.anchor, b.branch, b.file_hint, b.linked_worktree, b.status,
        b.revision, b.verified_at, b.selected_at, r.display_name, r.revision,
        r.archived, r.common_identity
   FROM bindings b JOIN repositories r ON r.id = b.repo_id";
const PREVIEW_TTL: i64 = 600;

#[derive(Debug)]
pub struct Binding {
    pub id: String,
    pub repo_id: String,
    pub root: PathBuf,
    pub revision: i64,
    pub commit: String,
    pub identity: String,
}

struct BindingRow {
    id: String,
    repo_id: String,
    selected_path: String,
    root: PathBuf,
    git_dir: PathBuf,
    common_dir: PathBuf,
    identity: String,
    anchor: String,
    branch: Option<String>,
    file_hint: Option<String>,
    linked_worktree: bool,
    status: String,
    revision: i64,
    verified_at: i64,
    selected_at: Option<i64>,
    display_name: String,
    repo_revision: i64,
    archived: bool,
    common_identity: String,
}

impl BindingRow {
    fn from_sql(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            repo_id: row.get(1)?,
            selected_path: row.get(2)?,
            root: PathBuf::from(row.get::<_, String>(3)?),
            git_dir: PathBuf::from(row.get::<_, String>(4)?),
            common_dir: PathBuf::from(row.get::<_, String>(5)?),
            identity: row.get(6)?,
            anchor: row.get(7)?,
            branch: row.get(8)?,
            file_hint: row.get(9)?,
            linked_worktree: row.get(10)?,
            status: row.get(11)?,
            revision: row.get(12)?,
            verified_at: row.get(13)?,
            selected_at: row.get(14)?,
            display_name: row.get(15)?,
            repo_revision: row.get(16)?,
            archived: row.get(17)?,
            common_identity: row.get(18)?,
        })
    }

    fn view(&self, live: Option<&Checkout>, status: &str) -> Value {
        json!({
            "binding_id":self.id,"repo_id":self.repo_id,"display_name":self.display_name,
            "selected_path":self.selected_path,"root":self.root,"git_dir":self.git_dir,
            "common_dir":self.common_dir,"branch":live.map(|row| &row.branch).unwrap_or(&self.branch),
            "commit":live.map(|row| &row.commit).unwrap_or(&self.anchor),
            "file_hint":self.file_hint,"linked_worktree":self.linked_worktree,
            "revision":self.revision,"repo_revision":self.repo_revision,
            "status":if self.archived {"ARCHIVED"} else {status},"archived":self.archived,
            "verified_at":self.verified_at,"selected_at":self.selected_at,
            "permissions_granted":false,
        })
    }

    fn verify(&self) -> Result<Checkout, AppError> {
        if self.archived {
            return Err(AppError::new(
                "ARCHIVED",
                "The repository registration is archived.",
            ));
        }
        let current = inspect(&self.root)?;
        if current.root != self.root
            || current.identity != self.identity
            || current.common_identity != self.common_identity
            || !contains_commit(&current, &self.anchor)?
        {
            return Err(AppError::new(
                "REBIND_REQUIRED",
                "The saved path identifies a different Git worktree.",
            ));
        }
        Ok(current)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Selection {
    path: PathBuf,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Add {
    path: PathBuf,
    token: String,
    display_name: Option<String>,
    repo_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct List {
    include_archived: Option<bool>,
    limit: Option<u32>,
    after: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    binding_id: String,
    revision: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Relink {
    binding_id: String,
    revision: i64,
    path: PathBuf,
    token: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RepositoryChange {
    repo_id: String,
    revision: i64,
    display_name: Option<String>,
}

pub fn dispatch(db: &mut Database, method: &str, params: Value) -> Result<Value, AppError> {
    match method {
        "repo.preview" => preview(db, storage::decode(params)?),
        "repo.add" => add(db, storage::decode(params)?),
        "repo.list" => list(db, storage::decode(params)?),
        "repo.health" => {
            let input: Reference = storage::decode(params)?;
            let row = binding_row(&db.conn, &db.machine_id, &input.binding_id)?;
            check_revision(row.revision, input.revision)?;
            health(&row)
        }
        "repo.select" => select(db, storage::decode(params)?),
        "repo.relink" => relink(db, storage::decode(params)?),
        "repo.rename" | "repo.archive" => change(db, method, storage::decode(params)?),
        _ => Err(AppError::new(
            "UNKNOWN_METHOD",
            "Unknown repository operation.",
        )),
    }
}

pub fn resolve_binding(
    db: &Database,
    id: &str,
    expected_revision: Option<i64>,
) -> Result<Binding, AppError> {
    let row = binding_row(&db.conn, &db.machine_id, id)?;
    if row.archived {
        return Err(AppError::new(
            "ARCHIVED",
            "The repository registration is archived.",
        ));
    }
    check_revision(row.revision, expected_revision)?;
    let checkout = row.verify().map_err(|error| {
        if error.code == "NOT_A_REPOSITORY" {
            AppError::new(
                "REBIND_REQUIRED",
                "The saved path is no longer a Git worktree.",
            )
        } else {
            error
        }
    })?;
    Ok(Binding {
        id: row.id,
        repo_id: row.repo_id,
        root: checkout.root,
        revision: row.revision,
        commit: checkout.commit,
        identity: checkout.identity,
    })
}

pub(super) fn verifies_archived_anchor(root: &Path, anchor: &str) -> Result<bool, AppError> {
    let checkout = inspect(root)?;
    Ok(checkout.root == root && contains_commit(&checkout, anchor)?)
}

fn health(row: &BindingRow) -> Result<Value, AppError> {
    match row.verify() {
        Ok(live) => Ok(row.view(Some(&live), "ACTIVE")),
        Err(error) => {
            let status = match error.code {
                "MISSING" => "MISSING",
                "INACCESSIBLE" | "GIT_TIMEOUT" | "GIT_UNAVAILABLE" => "INACCESSIBLE",
                "ARCHIVED" => "ARCHIVED",
                _ => "REBIND_REQUIRED",
            };
            let mut result = row.view(None, status);
            result["warning"] = json!(error.message);
            Ok(result)
        }
    }
}

fn preview(db: &Database, input: Selection) -> Result<Value, AppError> {
    let checkout = inspect(&input.path)?;
    let signed = token(&db.preview_secret, &checkout, storage::now()?)?;
    Ok(json!({
        "status":"VERIFIED","selected_path":checkout.selected_path,"root":checkout.root,
        "git_dir":checkout.git_dir,"common_dir":checkout.common_dir,"file_hint":checkout.file_hint,
        "branch":checkout.branch,"commit":checkout.commit,"linked_worktree":checkout.linked_worktree,
        "suggested_name":checkout.suggested_name,"token":signed,"permissions_granted":false,
    }))
}

fn add(db: &mut Database, input: Add) -> Result<Value, AppError> {
    let checkout = confirmed(db, &input.path, &input.token)?;
    let display = checked_name(
        input
            .display_name
            .as_deref()
            .unwrap_or(&checkout.suggested_name),
    )?;
    let time = storage::now()?;
    let machine_id = &db.machine_id;
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some((id, repo_id, root)) = tx
        .query_row(
            "SELECT id, repo_id, root FROM bindings WHERE machine_id=?1 AND identity=?2",
            params![machine_id, checkout.identity],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()?
    {
        if Path::new(&root) != checkout.root {
            return Err(AppError::new(
                "REBIND_REQUIRED",
                "The registered worktree moved. Use Relink.",
            ));
        }
        if input
            .repo_id
            .as_ref()
            .is_some_and(|selected| selected != &repo_id)
        {
            return Err(AppError::new(
                "CONFLICT",
                "This worktree already belongs to a different registration.",
            ));
        }
        let row = binding_row(&tx, machine_id, &id)?;
        if row.archived {
            return Err(AppError::new(
                "ARCHIVED",
                "The repository registration is archived.",
            ));
        }
        tx.commit()?;
        return Ok(row.view(Some(&checkout), "ACTIVE"));
    }
    let existing: Option<(String, bool)> = tx
        .query_row(
            "SELECT id, archived FROM repositories WHERE common_identity=?1",
            [&checkout.common_identity],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let repo_id = match existing {
        Some((_, true)) => {
            return Err(AppError::new(
                "ARCHIVED",
                "The repository registration is archived.",
            ));
        }
        Some((id, false)) if input.repo_id.as_ref().is_none_or(|desired| desired == &id) => id,
        _ if input.repo_id.is_some() => {
            return Err(AppError::new(
                "REBIND_REQUIRED",
                "The selected worktree does not belong to this registration.",
            ));
        }
        None => {
            let id = new_id(&tx, "repo")?;
            tx.execute(
                "INSERT INTO repositories(id,common_identity,display_name,created_at,updated_at) VALUES(?1,?2,?3,?4,?4)",
                params![id,checkout.common_identity,display,time],
            )?;
            id
        }
        _ => {
            return Err(AppError::new(
                "REBIND_REQUIRED",
                "The selected repository identity changed.",
            ));
        }
    };
    let id = new_id(&tx, "binding")?;
    tx.execute(
        "INSERT INTO bindings(id,repo_id,machine_id,selected_path,root,git_dir,common_dir,identity,anchor,branch,file_hint,linked_worktree,status,verified_at)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,'ACTIVE',?13)",
        params![id,repo_id,machine_id,path_text(&checkout.selected_path)?,path_text(&checkout.root)?,
            path_text(&checkout.git_dir)?,path_text(&checkout.common_dir)?,checkout.identity,checkout.commit,
            checkout.branch,checkout.file_hint,checkout.linked_worktree,time],
    )?;
    let row = binding_row(&tx, machine_id, &id)?;
    tx.commit()?;
    Ok(row.view(Some(&checkout), "ACTIVE"))
}

fn list(db: &Database, input: List) -> Result<Value, AppError> {
    let limit = input.limit.unwrap_or(50);
    if !(1..=100).contains(&limit) {
        return Err(AppError::new(
            "INVALID_INPUT",
            "The page limit must be between 1 and 100.",
        ));
    }
    if let Some(after) = &input.after {
        validate_id(after, "binding")?;
    }
    let sql = format!(
        "{BINDING_ROW} WHERE b.machine_id=?1 AND (r.archived=0 OR ?2) AND b.id>?3 ORDER BY b.id LIMIT ?4"
    );
    let mut statement = db.conn.prepare(&sql)?;
    let rows = statement
        .query_map(
            params![
                db.machine_id,
                input.include_archived.unwrap_or(false),
                input.after.unwrap_or_default(),
                limit + 1
            ],
            BindingRow::from_sql,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let next = (rows.len() > limit as usize).then(|| rows[(limit - 1) as usize].id.clone());
    let items = rows
        .iter()
        .take(limit as usize)
        .map(|row| {
            let status = match fs::symlink_metadata(&row.root) {
                Ok(meta) if !meta.is_dir() || storage::redirect(&meta) => "REBIND_REQUIRED",
                Ok(_) => &row.status,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => "MISSING",
                Err(_) => "INACCESSIBLE",
            };
            row.view(None, status)
        })
        .collect::<Vec<_>>();
    Ok(
        json!({"items":items,"next":next,"health":"Use repo.health to verify current Git identity."}),
    )
}

fn select(db: &mut Database, input: Reference) -> Result<Value, AppError> {
    let row = binding_row(&db.conn, &db.machine_id, &input.binding_id)?;
    check_revision(row.revision, input.revision)?;
    let checkout = row.verify()?;
    let time = storage::now()?;
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    if tx.execute(
        "UPDATE bindings SET selected_at=?1,verified_at=?1,status='ACTIVE',branch=?2
         WHERE id=?3 AND machine_id=?4 AND revision=?5
           AND EXISTS(SELECT 1 FROM repositories WHERE id=bindings.repo_id AND archived=0)",
        params![time, checkout.branch, row.id, db.machine_id, row.revision],
    )? != 1
    {
        return Err(revision_conflict());
    }
    let selected = binding_row(&tx, &db.machine_id, &row.id)?;
    tx.commit()?;
    Ok(selected.view(Some(&checkout), "ACTIVE"))
}

fn relink(db: &mut Database, input: Relink) -> Result<Value, AppError> {
    let original = binding_row(&db.conn, &db.machine_id, &input.binding_id)?;
    check_revision(original.revision, Some(input.revision))?;
    if original.archived {
        return Err(AppError::new(
            "ARCHIVED",
            "The repository registration is archived.",
        ));
    }
    let checkout = confirmed(db, &input.path, &input.token)?;
    if checkout.common_identity != original.common_identity
        || !contains_commit(&checkout, &original.anchor)?
    {
        return Err(AppError::new(
            "REBIND_REQUIRED",
            "Relink requires the same Git repository. Add an unrelated checkout separately.",
        ));
    }
    let time = storage::now()?;
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    let affected = tx.execute(
        "UPDATE bindings SET selected_path=?1,root=?2,git_dir=?3,common_dir=?4,identity=?5,anchor=?6,
             branch=?7,file_hint=?8,linked_worktree=?9,verified_at=?10,status='ACTIVE',revision=revision+1
         WHERE id=?11 AND machine_id=?12 AND revision=?13
           AND EXISTS(SELECT 1 FROM repositories WHERE id=bindings.repo_id AND archived=0)",
        params![path_text(&checkout.selected_path)?,path_text(&checkout.root)?,path_text(&checkout.git_dir)?,
            path_text(&checkout.common_dir)?,checkout.identity,checkout.commit,checkout.branch,
            checkout.file_hint,checkout.linked_worktree,time,input.binding_id,db.machine_id,input.revision],
    )?;
    if affected != 1 {
        return Err(revision_conflict());
    }
    let row = binding_row(&tx, &db.machine_id, &input.binding_id)?;
    tx.commit()?;
    let mut result = row.view(Some(&checkout), "ACTIVE");
    result["prior_grants_stale"] = json!(true);
    Ok(result)
}

fn change(db: &mut Database, method: &str, input: RepositoryChange) -> Result<Value, AppError> {
    validate_id(&input.repo_id, "repo")?;
    let archive = method == "repo.archive";
    let display = match (archive, input.display_name.as_deref()) {
        (false, Some(name)) => Some(checked_name(name)?),
        (true, None) => None,
        _ => {
            return Err(AppError::new(
                "INVALID_INPUT",
                "Supply a display name for rename only.",
            ));
        }
    };
    let time = storage::now()?;
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (name, current, archived): (String, i64, bool) = tx.query_row(
        "SELECT display_name, revision, archived FROM repositories r WHERE id=?1
           AND EXISTS(SELECT 1 FROM bindings WHERE repo_id=r.id AND machine_id=?2)",
        params![input.repo_id, db.machine_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    check_revision(current, Some(input.revision))?;
    if archived {
        return Err(AppError::new(
            "ARCHIVED",
            "The repository registration is archived.",
        ));
    }
    let name = display.unwrap_or(&name);
    if tx.execute(
        "UPDATE repositories SET display_name=?1,archived=?2,revision=revision+1,updated_at=?3 WHERE id=?4 AND revision=?5",
        params![name,archive,time,input.repo_id,current],
    )? != 1 { return Err(revision_conflict()); }
    if archive {
        tx.execute(
            "UPDATE bindings SET revision=revision+1 WHERE repo_id=?1 AND machine_id=?2",
            params![input.repo_id, db.machine_id],
        )?;
    }
    tx.commit()?;
    Ok(json!({
        "repo_id":input.repo_id,"display_name":name,"revision":current+1,
        "status":if archive {"ARCHIVED"} else {"ACTIVE"},"history_preserved":true,
    }))
}

fn binding_row(conn: &Connection, machine_id: &str, id: &str) -> Result<BindingRow, AppError> {
    validate_id(id, "binding")?;
    Ok(conn.query_row(
        &format!("{BINDING_ROW} WHERE b.machine_id=?1 AND b.id=?2"),
        params![machine_id, id],
        BindingRow::from_sql,
    )?)
}

fn confirmed(db: &Database, path: &Path, supplied: &str) -> Result<Checkout, AppError> {
    let (timestamp, proof) = supplied.split_once('.').ok_or_else(stale_preview)?;
    let timestamp = timestamp.parse::<i64>().ok().ok_or_else(stale_preview)?;
    let now = storage::now()?;
    if !(now.saturating_sub(PREVIEW_TTL)..=now).contains(&timestamp) || proof.len() != 64 {
        return Err(stale_preview());
    }
    let checkout = inspect(path)?;
    let expected = token(&db.preview_secret, &checkout, timestamp)?;
    if expected.len() != supplied.len()
        || expected
            .bytes()
            .zip(supplied.bytes())
            .fold(0, |diff, (left, right)| diff | (left ^ right))
            != 0
    {
        return Err(stale_preview());
    }
    Ok(checkout)
}

fn token(secret: &[u8], checkout: &Checkout, timestamp: i64) -> Result<String, AppError> {
    let bytes = serde_json::to_vec(&(timestamp, checkout))
        .map_err(|_| AppError::new("INVALID_PATH", "Cannot encode the selected local path."))?;
    let mut hash = Sha256::new();
    hash.update(b"cgacp-local-checkout-preview\0");
    hash.update(bytes);
    hash.update(secret);
    Ok(format!("{timestamp}.{:x}", hash.finalize()))
}

fn stale_preview() -> AppError {
    AppError::new(
        "STALE_PREVIEW",
        "The preview expired or the checkout changed. Preview it again.",
    )
}

fn checked_name(name: &str) -> Result<&str, AppError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 120 || name.chars().any(char::is_control) {
        return Err(AppError::new(
            "INVALID_INPUT",
            "Use a display name with 1–120 printable characters.",
        ));
    }
    Ok(name)
}

fn path_text(path: &Path) -> Result<&str, AppError> {
    path.to_str()
        .ok_or_else(|| AppError::new("INVALID_PATH", "The path is not representable as UTF-8."))
}

fn check_revision(actual: i64, expected: Option<i64>) -> Result<(), AppError> {
    if expected.is_some_and(|value| value != actual) {
        return Err(revision_conflict());
    }
    Ok(())
}

fn revision_conflict() -> AppError {
    AppError::new(
        "CONFLICT",
        "The repository or worktree revision changed. Refresh it and retry.",
    )
}

fn new_id(conn: &Connection, prefix: &str) -> Result<String, AppError> {
    let nonce: String =
        conn.query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))?;
    Ok(format!("{prefix}-{nonce}"))
}

fn validate_id(id: &str, kind: &str) -> Result<(), AppError> {
    if !id
        .strip_prefix(kind)
        .and_then(|rest| rest.strip_prefix('-'))
        .is_some_and(|nonce| {
            nonce.len() == 32
                && nonce
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        })
    {
        return Err(AppError::new(
            "INVALID_INPUT",
            "Invalid repository or binding ID.",
        ));
    }
    Ok(())
}

fn digest(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}
