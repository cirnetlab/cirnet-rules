use crate::{docs, workspace};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) const MAX_EVENTS: usize = 1024;
pub(super) const MAX_RECEIPTS: usize = 512;
pub(super) const MAX_SESSIONS: usize = 128;

#[derive(Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StoreIdentity {
    pub schema: u8,
    pub authority_sha256: String,
    pub reader_version: u8,
    pub writer_version: u8,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Coordination {
    pub sessions: BTreeMap<String, Session>,
    pub generation: u64,
    pub lease: Option<Lease>,
    pub submission: Option<Submission>,
    pub submissions: Vec<Submission>,
    pub dependencies: Vec<Dependency>,
    pub receipts: BTreeMap<String, RequestReceipt>,
    pub events: Vec<AuditEvent>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Session {
    pub token_sha256: String,
    pub workspace_sha256: String,
    pub binding_sha256: String,
    pub scope_sha256: String,
    pub issued_at: Clock,
    pub expires_at: u64,
    pub role: String,
    pub mode: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Clock {
    pub unix_seconds: u64,
    pub uptime_ms: u64,
    pub boot_id: String,
}

impl Clock {
    pub fn read() -> Result<Self, String> {
        let unix_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "Local lease clock is unavailable")?
            .as_secs();
        let uptime_ms = uptime_ms()?;
        #[cfg(target_os = "linux")]
        let boot_id = std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
            .map_err(|_| "Cannot read the local boot identity")?
            .trim()
            .to_owned();
        #[cfg(not(target_os = "linux"))]
        let boot_id = String::new();
        Ok(Self {
            unix_seconds,
            uptime_ms,
            boot_id,
        })
    }

    pub fn expired_since(&self, earlier: &Self, expires_at: u64) -> bool {
        if self.boot_id != earlier.boot_id
            || self.unix_seconds < earlier.unix_seconds
            || self.uptime_ms < earlier.uptime_ms
            || self.unix_seconds >= expires_at
        {
            return true;
        }
        let uptime_delta = self.uptime_ms - earlier.uptime_ms;
        let wall_delta = (self.unix_seconds - earlier.unix_seconds).saturating_mul(1_000);
        let duration = expires_at
            .saturating_sub(earlier.unix_seconds)
            .saturating_mul(1_000);
        uptime_delta >= duration
            || (self.boot_id.is_empty() && wall_delta.abs_diff(uptime_delta) > 2_000)
    }
}

#[cfg(windows)]
fn uptime_ms() -> Result<u64, String> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetTickCount64() -> u64;
    }
    // This Windows API reads a system-wide monotonic millisecond counter.
    Ok(unsafe { GetTickCount64() })
}

#[cfg(unix)]
fn uptime_ms() -> Result<u64, String> {
    #[cfg(target_os = "linux")]
    const CLOCK: libc::clockid_t = libc::CLOCK_BOOTTIME;
    #[cfg(not(target_os = "linux"))]
    const CLOCK: libc::clockid_t = libc::CLOCK_MONOTONIC;
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // clock_gettime initializes the supplied timespec on success.
    if unsafe { libc::clock_gettime(CLOCK, &mut time) } != 0 || time.tv_sec < 0 || time.tv_nsec < 0
    {
        return Err("Local monotonic lease clock is unavailable".into());
    }
    Ok((time.tv_sec as u64).saturating_mul(1_000) + time.tv_nsec as u64 / 1_000_000)
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(super) enum LeaseStatus {
    Active,
    Sealed,
    RecoveryRequired,
    Released,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Lease {
    pub task_id: String,
    pub session_id: String,
    pub generation: u64,
    pub role: String,
    pub mode: String,
    pub acquired_at: u64,
    pub renewed_at: Clock,
    pub expires_at: u64,
    pub status: LeaseStatus,
    pub scope_sha256: String,
    pub binding_sha256: String,
}

impl Lease {
    pub fn status_at(&self, now: &Clock) -> LeaseStatus {
        if self.status == LeaseStatus::Active
            && now.expired_since(&self.renewed_at, self.expires_at)
        {
            LeaseStatus::RecoveryRequired
        } else {
            self.status
        }
    }

    pub fn summary(&self, now: &Clock) -> Value {
        json!({
            "generation":self.generation, "status":self.status_at(now),
            "role":self.role,"mode":self.mode,"expires_at":self.expires_at
        })
    }
}

#[derive(Clone, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Dependency {
    pub task_id: String,
    pub revision: u64,
    pub approval_epoch: u64,
    pub source_sha256: String,
    pub evidence_sha256: String,
    pub outputs: Vec<Output>,
    pub selected_inputs: Vec<Output>,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Output {
    pub repository: String,
    pub path: String,
    pub sha256: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Submission {
    pub submission_id: String,
    pub task_revision: u64,
    pub approval_epoch: u64,
    pub generation: u64,
    pub session_id: String,
    pub source_sha256: String,
    pub authority_sha256: String,
    pub binding_sha256: String,
    pub dependencies_sha256: String,
    pub source: Vec<Source>,
    pub report: Report,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Source {
    pub repository: String,
    pub identity: String,
    pub base_commit: String,
    pub observed_changes: Vec<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Report {
    pub summary: String,
    #[serde(default)]
    pub changed_paths: Vec<ReportPath>,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub commands: Vec<ReportCommand>,
    #[serde(default)]
    pub artifacts: Vec<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReportPath {
    pub repo: String,
    pub path: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReportCommand {
    pub command: String,
    pub exit_code: Option<i32>,
    pub result: Option<String>,
    pub log_sha256: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RequestReceipt {
    pub operation: String,
    pub args_sha256: String,
    pub session_id: String,
    pub revision: u64,
    pub response: Value,
    pub response_sha256: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AuditEvent {
    pub sequence: u64,
    pub operation: String,
    pub task_id: String,
    pub revision: u64,
    pub actor: Option<String>,
    pub session_id: Option<String>,
    pub before_sha256: Option<String>,
    pub after_sha256: String,
    pub previous_event_sha256: Option<String>,
    pub outcome: String,
    pub evidence_kind: String,
    pub timestamp: u64,
    pub event_sha256: String,
}

pub(super) fn digest(value: &impl Serialize) -> Result<String, String> {
    let value =
        serde_json::to_value(value).map_err(|_| "Cannot identify local coordination data")?;
    serde_json::to_vec(&value)
        .map(|bytes| docs::sha256(&bytes))
        .map_err(|_| "Cannot identify local coordination data".into())
}

pub(super) fn public_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_@.".contains(&byte))
        && !workspace::contains_known_secrets(value)
}

impl AuditEvent {
    pub fn digest(&self) -> Result<String, String> {
        let mut value = serde_json::to_value(self).map_err(|_| "Invalid coordination event")?;
        value.as_object_mut().unwrap().remove("event_sha256");
        digest(&value)
    }

    pub fn projection(&self, previous: Option<&AuditEvent>, sequence: u64) -> Value {
        let valid = self.sequence == sequence
            && self.previous_event_sha256.as_ref() == previous.map(|event| &event.event_sha256)
            && self
                .digest()
                .is_ok_and(|digest| digest == self.event_sha256);
        if !valid {
            return json!({"sequence":sequence,"integrity":"FAIL"});
        }
        json!({
            "operation":self.operation.as_str().split_ascii_whitespace().next().filter(|value| known_operation(value)), "task":Some(self.task_id.as_str()).filter(|value| public_label(value)), "revision":self.revision,
            "sequence":self.sequence, "actor":self.actor.as_deref().filter(|value| public_label(value)),
            "session_id":self.session_id.as_deref().filter(|value| public_label(value)),
            "before_sha256":self.before_sha256.as_deref().filter(|value| is_sha256(value)), "after_sha256":Some(self.after_sha256.as_str()).filter(|value| is_sha256(value)),
            "time_unix_seconds":self.timestamp,"result":Some(self.outcome.as_str()).filter(|value| matches!(*value,"PASS"|"BLOCKED"|"FAIL")),
            "evidence_kind":Some(self.evidence_kind.as_str()).filter(|value| matches!(*value,"AGENT_ASSERTION"|"LOCAL_ACTOR_ASSERTION"|"LOCAL_CAPABILITY"|"CONTROLLED_RUNNER"|"LOCAL_DOCUMENTATION_SYNC")),"event_sha256":self.event_sha256,"integrity":"PASS",
            "trust":"LOCAL_ASSERTION_OR_LOCAL_CAPABILITY"
        })
    }
}

#[cfg(test)]
#[test]
fn elapsed_monotonic_time_fences_even_if_wall_time_is_set_back() {
    let issued = Clock {
        unix_seconds: 1000,
        uptime_ms: 70_000,
        boot_id: "fixture".into(),
    };
    let mut observed = Clock {
        unix_seconds: 1030,
        uptime_ms: 100_000,
        boot_id: "fixture".into(),
    };
    assert!(observed.expired_since(&issued, 1030));
    observed.unix_seconds = 1029;
    assert!(observed.expired_since(&issued, 1030));
    observed.uptime_ms -= 1_000;
    assert!(!observed.expired_since(&issued, 1030));
    observed.boot_id = "next boot".into();
    assert!(observed.expired_since(&issued, 1030));
}

fn known_operation(value: &str) -> bool {
    matches!(
        value,
        "create"
            | "recover-legacy"
            | "migrate"
            | "revise"
            | "ready"
            | "start"
            | "implement"
            | "block"
            | "cancel"
            | "supersede"
            | "reopen"
            | "transition"
            | "checkpoint"
            | "task-verify"
            | "task-review"
            | "task-accept"
            | "task-close"
            | "docs-apply"
            | "session-prepare"
            | "claim"
            | "renew"
            | "release"
            | "expire"
            | "recover"
            | "report"
            | "update"
    )
}

pub(super) fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
