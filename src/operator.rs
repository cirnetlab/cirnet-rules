mod checks;
mod context;
pub mod presentation;
mod setup;

pub use checks::check;
pub use setup::{
    SetupAction, SetupInput, SetupPreview, inspect_setup, perform_setup, preview_setup,
};

pub fn check_console(target: &str, scope: &Scope, strict: bool) -> CheckReport {
    let cancellation = crate::execution::Cancellation::begin().ok();
    check(target, scope, strict, || {
        cancellation
            .as_ref()
            .is_some_and(|cancel| cancel.requested())
    })
}

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

pub const TARGETS: &[&str] = &[
    "all",
    "governance",
    "registry",
    "policies",
    "rules",
    "skills",
    "profiles",
    "references",
    "hooks",
    "repositories",
    "bindings",
    "routing",
    "task",
    "diff",
    "verification",
    "audit",
    "environment",
    "agent-config",
    "agent",
    "mcp",
];
pub const AGENTS: &[&str] = &["codex", "claude", "antigravity"];

#[derive(Clone, Debug, Default)]
pub struct Scope {
    pub governance_root: Option<PathBuf>,
    pub workspace: Option<PathBuf>,
    pub repo: Option<String>,
    pub binding: Option<String>,
    pub task: Option<String>,
    pub agent: Option<String>,
    pub surface: Option<String>,
    pub bundle: Option<PathBuf>,
    pub mcp_bundle: Option<PathBuf>,
    pub target: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    Pass,
    Warning,
    Error,
    Blocked,
    NotRun,
    Unsupported,
    Skipped,
    Cancelled,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warning => "WARNING",
            Self::Error => "ERROR",
            Self::Blocked => "BLOCKED",
            Self::NotRun => "NOT_RUN",
            Self::Unsupported => "UNSUPPORTED",
            Self::Skipped => "SKIPPED",
            Self::Cancelled => "CANCELLED",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    ReadOnly,
    LocalWrite,
    RepositoryWrite,
    UserConfigWrite,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NextAction {
    pub id: String,
    pub label: String,
    pub operation: String,
    pub typed_arguments: BTreeMap<String, String>,
    pub effect: Effect,
    pub requires_confirmation: bool,
    pub required_grant: Option<String>,
    pub plan_digest: Option<String>,
    pub stale_when: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Finding {
    pub id: String,
    pub component: String,
    pub repository: Option<String>,
    pub binding_id: Option<String>,
    pub task_id: Option<String>,
    pub agent_id: Option<String>,
    pub status: Status,
    pub required: bool,
    pub evidence_level: Option<String>,
    pub code: String,
    pub message: String,
    pub expected: Option<String>,
    pub observed: Option<Value>,
    pub sources: Vec<String>,
    pub blocked_by: Vec<String>,
    pub duration_ms: u64,
    pub actions: Vec<NextAction>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CheckReport {
    pub schema_version: u8,
    pub run_id: String,
    pub requested_scope: Value,
    pub resolved_context: Value,
    pub started_at: u64,
    pub duration_ms: u64,
    pub completion: String,
    pub overall_status: Status,
    pub findings: Vec<Finding>,
    pub counts_by_status: BTreeMap<Status, usize>,
    pub coverage: Value,
    pub exit_code: u8,
}

impl CheckReport {
    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        let path = crate::agent_config::paths::export_path(path)?;
        let mut report =
            serde_json::to_vec_pretty(self).map_err(|_| "Cannot encode check report")?;
        report.push(b'\n');
        crate::agent_config::paths::write_new(&path, &report)
    }

    pub fn report_save_failed(&mut self, reason: &str) {
        if self.overall_status != Status::Cancelled {
            self.overall_status = Status::Error;
            self.exit_code = 1;
        }
        self.findings.push(Finding {
            id: "report-save".into(),
            component: "report".into(),
            repository: None,
            binding_id: None,
            task_id: None,
            agent_id: None,
            status: Status::Error,
            required: true,
            evidence_level: None,
            code: "REPORT_SAVE_FAILED".into(),
            message: safe(reason),
            expected: None,
            observed: None,
            sources: Vec::new(),
            blocked_by: Vec::new(),
            duration_ms: 0,
            actions: Vec::new(),
        });
        *self.counts_by_status.entry(Status::Error).or_insert(0) += 1;
        for key in ["total", "required", "observed"] {
            self.coverage[key] = serde_json::json!(self.coverage[key].as_u64().unwrap_or(0) + 1);
        }
    }
    pub fn render(&self) -> String {
        self.render_for(false)
    }

    pub fn render_for(&self, vi: bool) -> String {
        presentation::report(self, vi)
    }
}

fn finish(
    started: Instant,
    wall: SystemTime,
    requested_scope: Value,
    resolved_context: Value,
    mut findings: Vec<Finding>,
    strict: bool,
) -> CheckReport {
    let mut counts = BTreeMap::new();
    for row in &mut findings {
        *counts.entry(row.status).or_insert(0) += 1;
        for action in &mut row.actions {
            for (key, value) in [
                ("repo", &row.repository),
                ("binding", &row.binding_id),
                ("task", &row.task_id),
                ("agent", &row.agent_id),
            ] {
                if let Some(value) = value.as_ref().filter(|value| !value.starts_with('[')) {
                    action.typed_arguments.insert(key.into(), value.clone());
                }
            }
        }
    }
    let gaps = |row: &Finding| {
        matches!(
            row.status,
            Status::Blocked | Status::NotRun | Status::Unsupported | Status::Skipped
        )
    };
    let (overall_status, exit_code) = if counts.contains_key(&Status::Cancelled) {
        (Status::Cancelled, 130)
    } else if counts.contains_key(&Status::Error) {
        (Status::Error, 1)
    } else if findings.iter().any(|row| row.required && gaps(row))
        || (strict && counts.contains_key(&Status::Warning))
    {
        (Status::Blocked, 2)
    } else if counts.contains_key(&Status::Warning)
        || findings.iter().any(|row| row.status == Status::Blocked)
    {
        (Status::Warning, 0)
    } else {
        (Status::Pass, 0)
    };
    let timestamp = wall.duration_since(UNIX_EPOCH).unwrap_or_default();
    CheckReport {
        schema_version: 1,
        run_id: format!("{}-{}", std::process::id(), timestamp.as_nanos()),
        requested_scope,
        resolved_context,
        started_at: timestamp.as_secs(),
        duration_ms: elapsed(started),
        completion: if overall_status == Status::Cancelled {
            "CANCELLED"
        } else {
            "COMPLETE"
        }
        .into(),
        overall_status,
        coverage: serde_json::json!({
            "total": findings.len(),
            "required": findings.iter().filter(|row| row.required).count(),
            "required_passed": findings.iter().filter(|row| row.required && row.status == Status::Pass).count(),
            "observed": findings.iter().filter(|row| !gaps(row) && row.status != Status::Cancelled).count(),
            "native_verified": 0
        }),
        findings,
        counts_by_status: counts,
        exit_code,
    }
}

fn elapsed(started: Instant) -> u64 {
    started.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
}

fn safe(value: &str) -> String {
    if crate::workspace::contains_known_secrets(value)
        || value.to_ascii_lowercase().contains("bearer ")
    {
        return "[redacted]".into();
    }
    value
        .chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .take(384)
        .collect()
}

fn label(value: Option<&str>) -> Option<String> {
    value.map(|value| {
        if value.len() <= 96
            && value
                .bytes()
                .all(|ch| ch.is_ascii_alphanumeric() || b"._-".contains(&ch))
        {
            safe(value)
        } else {
            "[invalid identifier]".into()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn plain_report_keeps_status_groups_and_next_actions_without_table_columns() {
        let blocked = Finding {
            id: "binding-1".into(),
            component: "bindings".into(),
            repository: Some("cirnet-fpga".into()),
            binding_id: None,
            task_id: None,
            agent_id: None,
            status: Status::Blocked,
            required: false,
            evidence_level: None,
            code: "NO_BINDING".into(),
            message: "No confirmed worktree".into(),
            expected: None,
            observed: None,
            sources: Vec::new(),
            blocked_by: Vec::new(),
            duration_ms: 1,
            actions: vec![NextAction {
                id: "select-binding".into(),
                label: "Select a verified worktree".into(),
                operation: "select-binding".into(),
                typed_arguments: BTreeMap::new(),
                effect: Effect::ReadOnly,
                requires_confirmation: false,
                required_grant: None,
                plan_digest: None,
                stale_when: Vec::new(),
            }],
        };
        let mut passed = blocked.clone();
        passed.status = Status::Pass;
        let report = finish(
            Instant::now(),
            SystemTime::now(),
            serde_json::json!({"target":"all"}),
            Value::Null,
            vec![passed, blocked],
            false,
        );
        let rendered = report.render();
        assert!(rendered.contains("Overall WARNING"));
        assert!(rendered.contains("Issues needing attention (1)"));
        assert!(rendered.contains("    Next: Select a verified worktree"));
        assert!(rendered.find("Issues needing attention") < rendered.find("Passed checks"));
        assert!(!rendered.contains("Component | Status"));
        assert!(!rendered.contains('\u{1b}'));
        assert_eq!(report.overall_status, Status::Warning);
        assert_eq!(report.exit_code, 0);
    }

    #[test]
    fn cancellation_keeps_completed_findings_and_never_reports_pass() {
        let scope = Scope {
            governance_root: Some(PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
            ..Scope::default()
        };
        let polls = Cell::new(0);
        let report = check("all", &scope, false, || {
            polls.set(polls.get() + 1);
            polls.get() > 1
        });
        assert_eq!(report.overall_status, Status::Cancelled);
        assert_eq!(report.exit_code, 130);
        assert_eq!(report.completion, "CANCELLED");
        assert!(
            report
                .findings
                .iter()
                .any(|row| row.component == "registry" && row.status == Status::Pass)
        );
        assert_eq!(report.counts_by_status.get(&Status::Cancelled), Some(&1));
    }

    #[test]
    fn report_sanitizes_credentials_and_terminal_controls() {
        assert_eq!(safe("error Bearer private-fixture"), "[redacted]");
        assert!(!safe("error\u{1b}[2J").contains('\u{1b}'));
        assert_eq!(
            label(Some("path with spaces")),
            Some("[invalid identifier]".into())
        );
    }
}
