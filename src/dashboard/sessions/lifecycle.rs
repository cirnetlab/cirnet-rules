//! Lifecycle policy. The storage port owns atomicity and the SQLite adapter owns SQL.
use super::{AppError, cursor, digest, id, limit, now, parse, request_key};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const PREVIEW_TTL_SECONDS: i64 = 120;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Target {
    pub conversation: i64,
    pub message: Option<i64>,
}

#[derive(Serialize)]
pub(super) struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub state: String,
    pub profile: String,
    pub source: String,
    pub revision: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub provider_thread: Option<String>,
}

#[derive(Serialize)]
pub(super) struct ConversationPage {
    pub items: Vec<ConversationSummary>,
    pub next_before: Option<String>,
    pub total: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ConversationFilter {
    #[serde(default)]
    pub query: String,
    #[serde(default = "active")]
    pub state: String,
    pub binding_id: Option<String>,
    pub profile: Option<String>,
    pub source: Option<String>,
    pub updated_before: Option<i64>,
    pub updated_after: Option<i64>,
    #[serde(default)]
    pub limit: Value,
    #[serde(default)]
    pub before: Value,
    #[serde(default)]
    pub include_total: bool,
}

fn active() -> String {
    "ACTIVE".into()
}

impl ConversationFilter {
    fn validate(&self) -> Result<(), AppError> {
        if self.query.len() > 160
            || !matches!(self.state.as_str(), "ACTIVE" | "ARCHIVED")
            || self.binding_id.as_ref().is_some_and(|id| id.len() > 96)
            || self
                .profile
                .as_deref()
                .is_some_and(|profile| profile != "handoff")
            || self
                .source
                .as_deref()
                .is_some_and(|source| !matches!(source, "DASHBOARD" | "AUTOMATION"))
            || matches!((self.updated_after, self.updated_before), (Some(after), Some(before)) if after > before)
        {
            return Err(AppError::new(
                "INVALID_REQUEST",
                "Invalid conversation filter",
            ));
        }
        Ok(())
    }

    pub fn page_size(&self) -> Result<i64, AppError> {
        limit(&self.limit)
    }

    pub fn cursor(&self) -> Result<i64, AppError> {
        cursor(&self.before, i64::MAX)
    }
}

#[derive(Serialize)]
pub(super) struct Blocker {
    pub kind: &'static str,
    pub id: String,
    pub state: String,
    pub attempt: Option<i64>,
    pub lease_expires_at: Option<i64>,
}

pub(super) struct Inspection {
    pub revision: i64,
    pub title: String,
    pub message_count: i64,
    pub task_count: i64,
    pub run_count: i64,
    pub automation: Option<AutomationLink>,
    pub blocker_count: i64,
    pub blockers: Vec<Blocker>,
}

pub(super) struct AutomationLink {
    pub id: i64,
    pub name: String,
    pub revision: i64,
    pub enabled: bool,
}

impl Inspection {
    fn scope_digest(&self) -> String {
        digest(&json!([
            self.revision,
            self.message_count,
            self.task_count,
            self.run_count,
            self.automation
                .as_ref()
                .map(|link| (link.id, link.revision, link.enabled)),
            self.blocker_count,
            self.blockers
        ]))
    }
}

pub(super) struct Preview {
    pub target: Target,
    pub revision: i64,
    pub scope_digest: String,
    pub expires_at: i64,
}

pub(super) struct Receipt {
    pub target: Target,
    pub revision: i64,
    pub token_digest: String,
    pub deleted_at: i64,
}

impl Receipt {
    fn response(&self) -> Value {
        json!({
            "conversation_id": self.target.conversation.to_string(),
            "message_id": self.target.message.map(|id| id.to_string()),
            "revision": self.revision,
            "status": "DELETED",
            "deleted_at": self.deleted_at,
        })
    }
}

pub(super) trait LifecycleRepository {
    fn page(&self, filter: ConversationFilter) -> Result<ConversationPage, AppError>;
    fn transaction<T>(
        &mut self,
        execute: impl FnOnce(&dyn LifecycleUnitOfWork) -> Result<T, AppError>,
    ) -> Result<T, AppError>;
}

pub(super) trait LifecycleUnitOfWork {
    fn transition(
        &self,
        id: i64,
        revision: i64,
        state: &str,
        time: i64,
    ) -> Result<ConversationSummary, AppError>;
    fn inspect(&self, target: Target) -> Result<Inspection, AppError>;
    fn issue_preview(&self, preview: &Preview, now: i64) -> Result<String, AppError>;
    fn read_preview(&self, token: &str) -> Result<Option<Preview>, AppError>;
    fn read_receipt(&self, key_digest: &str) -> Result<Option<Receipt>, AppError>;
    fn record_receipt(&self, key_digest: &str, receipt: &Receipt) -> Result<(), AppError>;
    fn purge(
        &self,
        target: Target,
        automation: Option<&AutomationLink>,
        time: i64,
    ) -> Result<(), AppError>;
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PreviewRequest {
    id: String,
    message_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteRequest {
    id: String,
    message_id: Option<String>,
    revision: i64,
    token: String,
    key: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StateRequest {
    id: String,
    revision: i64,
}

fn target(method: &str, conversation: &str, message: Option<&str>) -> Result<Target, AppError> {
    if method.starts_with("message.") != message.is_some() {
        return Err(AppError::new(
            "INVALID_REQUEST",
            "The delete target is invalid",
        ));
    }
    Ok(Target {
        conversation: id(conversation)?,
        message: message.map(id).transpose()?,
    })
}

pub(super) fn dispatch(
    repo: &mut impl LifecycleRepository,
    method: &str,
    params: Value,
) -> Result<Value, AppError> {
    match method {
        "conversation.list" => {
            let filter: ConversationFilter = parse(params)?;
            filter.validate()?;
            Ok(json!(repo.page(filter)?))
        }
        "conversation.archive" | "conversation.unarchive" => {
            let request: StateRequest = parse(params)?;
            let id = id(&request.id)?;
            let state = if method == "conversation.archive" {
                "ARCHIVED"
            } else {
                "ACTIVE"
            };
            repo.transaction(|work| {
                Ok(json!(work.transition(
                    id,
                    request.revision,
                    state,
                    now()
                )?))
            })
        }
        "conversation.delete.preview" | "message.delete.preview" => {
            let request: PreviewRequest = parse(params)?;
            let target = target(method, &request.id, request.message_id.as_deref())?;
            repo.transaction(|work| preview(work, target))
        }
        "conversation.delete" | "message.delete" => {
            let request: DeleteRequest = parse(params)?;
            let target = target(method, &request.id, request.message_id.as_deref())?;
            request_key(&request.key)?;
            if request.token.len() != 64
                || !request.token.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(stale_preview());
            }
            repo.transaction(|work| delete(work, target, request))
        }
        _ => Err(AppError::new(
            "METHOD_NOT_FOUND",
            "Unsupported lifecycle operation",
        )),
    }
}

fn preview(work: &dyn LifecycleUnitOfWork, target: Target) -> Result<Value, AppError> {
    let scope = work.inspect(target)?;
    let time = now();
    let expires_at = time.saturating_add(PREVIEW_TTL_SECONDS);
    let token = work.issue_preview(
        &Preview {
            target,
            revision: scope.revision,
            scope_digest: scope.scope_digest(),
            expires_at,
        },
        time,
    )?;
    let automations: Vec<_> = scope
        .automation
        .iter()
        .map(|link| {
            json!({
                "id": link.id.to_string(),
                "name": link.name,
                "revision": link.revision,
                "enabled": link.enabled,
            })
        })
        .collect();
    Ok(json!({
        "conversation_id": target.conversation.to_string(),
        "message_id": target.message.map(|id| id.to_string()),
        "revision": scope.revision,
        "title": scope.title,
        "message_count": scope.message_count,
        "task_count": scope.task_count,
        "run_count": scope.run_count,
        "automations": automations,
        "automation_count": automations.len(),
        "attachment_count": 0,
        "child_count": 0,
        "blockers": scope.blockers,
        "blocker_count": scope.blocker_count,
        "token": token,
        "expires_at": expires_at,
        "retained": ["Existing backups and any external provider copies",
            "Repository files, other conversations and terminal run audit metadata"],
    }))
}

fn delete(
    work: &dyn LifecycleUnitOfWork,
    target: Target,
    request: DeleteRequest,
) -> Result<Value, AppError> {
    let key = digest(&json!(request.key));
    let token_digest = digest(&json!(request.token));
    if let Some(receipt) = work.read_receipt(&key)? {
        if receipt.target == target
            && receipt.revision == request.revision
            && receipt.token_digest == token_digest
        {
            return Ok(receipt.response());
        }
        return Err(AppError::new(
            "CONFLICT",
            "Delete key was used for another operation",
        ));
    }
    let time = now();
    let preview = work
        .read_preview(&request.token)?
        .ok_or_else(stale_preview)?;
    if preview.target != target
        || preview.revision != request.revision
        || preview.expires_at <= time
    {
        return Err(stale_preview());
    }
    let current = work.inspect(target)?;
    if current.blocker_count > 0 {
        return Err(AppError::new(
            "ACTIVE_DEPENDENCY",
            "Resolve the active proposals or runs before deleting this content",
        )
        .with_details(json!({"blockers":current.blockers,"blocker_count":current.blocker_count})));
    }
    if current.scope_digest() != preview.scope_digest {
        return Err(stale_preview());
    }
    work.purge(target, current.automation.as_ref(), time)?;
    let receipt = Receipt {
        target,
        revision: request.revision,
        token_digest,
        deleted_at: time,
    };
    work.record_receipt(&key, &receipt)?;
    Ok(receipt.response())
}

fn stale_preview() -> AppError {
    AppError::new(
        "STALE_PREVIEW",
        "The delete preview expired or its scope changed. Preview again.",
    )
}
