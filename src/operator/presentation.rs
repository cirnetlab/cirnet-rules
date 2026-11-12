//! Display-only rendering and redaction for structured checks.

use super::{CheckReport, Effect, Finding, Status, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Attention,
    Pending,
    Passed,
}

pub const GROUPS: [Group; 3] = [Group::Attention, Group::Pending, Group::Passed];
pub const STATUS_ORDER: [Status; 8] = [
    Status::Error,
    Status::Blocked,
    Status::Warning,
    Status::Cancelled,
    Status::NotRun,
    Status::Unsupported,
    Status::Skipped,
    Status::Pass,
];

fn localized(en: &'static str, vi: &'static str, translate: bool) -> &'static str {
    if translate { vi } else { en }
}

impl Group {
    pub fn of(status: Status) -> Self {
        match status {
            Status::Error | Status::Blocked | Status::Warning | Status::Cancelled => {
                Self::Attention
            }
            Status::NotRun | Status::Unsupported | Status::Skipped => Self::Pending,
            Status::Pass => Self::Passed,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Attention => 0,
            Self::Pending => 1,
            Self::Passed => 2,
        }
    }

    pub fn label(self, vi: bool) -> &'static str {
        match self {
            Self::Attention => localized("Issues needing attention", "Vấn đề cần xử lý", vi),
            Self::Pending => localized("Not yet verified", "Chưa kiểm chứng", vi),
            Self::Passed => localized("Passed checks", "Kiểm tra đạt", vi),
        }
    }
}

pub fn marker(status: Status) -> &'static str {
    match status {
        Status::Error => "x",
        Status::Blocked | Status::Warning => "!",
        Status::Cancelled => "~",
        Status::NotRun | Status::Unsupported | Status::Skipped => "-",
        Status::Pass => "+",
    }
}

fn secret_field(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    [
        "token",
        "secret",
        "password",
        "credential",
        "authorization",
        "api_key",
        "apikey",
        "private_key",
        "privatekey",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

pub fn text(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    let url_credential = lower.split_whitespace().any(|word| {
        word.split_once("://").is_some_and(|(_, rest)| {
            rest.split('/')
                .next()
                .is_some_and(|host| host.contains('@'))
        })
    });
    if crate::workspace::contains_known_secrets(value)
        || url_credential
        || [
            "bearer ",
            "session-token=",
            "session_token=",
            "session token:",
            "capability-token=",
            "capability_token=",
            "capability token:",
            "api_key=",
            "token=",
            "token:",
            "token\":",
            "password:",
            "password\":",
            "secret=",
            "secret\":",
            "--session ",
            "--capability ",
            "--token ",
        ]
        .iter()
        .any(|marker| lower.contains(marker))
        || value.contains(['\u{1b}', '\u{90}', '\u{9b}', '\u{9d}'])
    {
        return "[redacted]".into();
    }
    value
        .chars()
        .map(|ch| {
            if ch.is_control() || matches!(ch, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}') {
                ' '
            } else {
                ch
            }
        })
        .collect()
}

fn redact_observed(value: &Value, left: &mut usize, depth: usize) -> Value {
    if *left == 0 || depth == 0 {
        return Value::String("[display limit]".into());
    }
    *left -= 1;
    match value {
        Value::Object(fields) => {
            let mut output = serde_json::Map::new();
            for (key, value) in fields {
                if *left == 0 {
                    output.insert("[display limit]".into(), Value::Null);
                    break;
                }
                let value = if secret_field(key) {
                    *left -= 1;
                    Value::String("[redacted]".into())
                } else {
                    redact_observed(value, left, depth - 1)
                };
                output.insert(text(key).chars().take(128).collect(), value);
            }
            Value::Object(output)
        }
        Value::Array(items) => {
            let mut output = Vec::new();
            for item in items {
                if *left == 0 {
                    output.push(Value::String("[display limit]".into()));
                    break;
                }
                output.push(redact_observed(item, left, depth - 1));
            }
            Value::Array(output)
        }
        Value::String(value) => Value::String(text(value).chars().take(512).collect()),
        other => other.clone(),
    }
}

pub fn clip(value: &str, width: usize) -> &str {
    let mut used = 0;
    let end = value
        .grapheme_indices(true)
        .find_map(|(offset, grapheme)| {
            used += grapheme.width();
            (used > width).then_some(offset)
        })
        .unwrap_or(value.len());
    &value[..end]
}

pub fn wrap(value: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }
    let mut lines = Vec::new();
    for line in value.split('\n') {
        let mut remaining = line.trim_end();
        if remaining.is_empty() {
            lines.push(String::new());
        }
        while !remaining.is_empty() {
            let fit = clip(remaining, width);
            if fit.len() == remaining.len() {
                lines.push(fit.into());
                break;
            }
            if fit.is_empty() {
                // A terminal narrower than this grapheme still advances.
                let grapheme = remaining.graphemes(true).next().expect("nonempty line");
                lines.push("?".into());
                remaining = &remaining[grapheme.len()..];
                continue;
            }
            let end = fit
                .rfind(' ')
                .filter(|offset| !fit[..*offset].trim().is_empty())
                .unwrap_or(fit.len());
            lines.push(remaining[..end].trim_end().into());
            remaining = remaining[end..].trim_start();
        }
    }
    lines
}

pub fn title(row: &Finding) -> String {
    std::iter::once(row.component.as_str())
        .chain(row.repository.as_deref())
        .chain(row.agent_id.as_deref())
        .map(text)
        .collect::<Vec<_>>()
        .join(" / ")
}

pub fn overview(report: &CheckReport, vi: bool) -> String {
    format!(
        "{} {} · {} · {} ms · Exit {} · {} {}",
        localized("Overall", "Tổng thể", vi),
        report.overall_status.label(),
        text(report.requested_scope["target"].as_str().unwrap_or("setup")),
        report.duration_ms,
        report.exit_code,
        report.findings.len(),
        localized("findings", "mục", vi)
    )
}

pub fn counts(report: &CheckReport) -> String {
    STATUS_ORDER
        .into_iter()
        .filter_map(|status| {
            report
                .counts_by_status
                .get(&status)
                .map(|count| format!("{} {count}", status.label()))
        })
        .collect::<Vec<_>>()
        .join("  ")
}

pub fn scope(report: &CheckReport, vi: bool) -> String {
    let mut parts = Vec::new();
    for (key, label) in [
        ("repo", "Repo"),
        ("binding", "Binding"),
        ("task", "Task"),
        ("agent", "Agent"),
    ] {
        if let Some(value) = report.requested_scope[key].as_str() {
            parts.push(format!("{label} {}", text(value)));
        }
    }
    let repos = report
        .findings
        .iter()
        .filter_map(|row| row.repository.as_deref())
        .collect::<BTreeSet<_>>();
    if !repos.is_empty() {
        parts.push(format!(
            "{} {}",
            repos.len(),
            localized("repositories", "repository", vi)
        ));
    }
    let mut bindings = BTreeMap::new();
    for row in &report.findings {
        if row.component == "bindings"
            && let Some(repo) = row.repository.as_deref()
        {
            *bindings.entry(repo).or_insert(false) |= row.status == Status::Pass;
        }
    }
    if !bindings.is_empty() {
        parts.push(format!(
            "{} {}/{}",
            localized("Bindings confirmed", "Binding đã xác nhận", vi),
            bindings.values().filter(|pass| **pass).count(),
            bindings.len()
        ));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(
            "{} {}",
            localized("Scope:", "Phạm vi:", vi),
            parts.join(" · ")
        )
    }
}

pub fn detail_lines(row: &Finding, width: usize, vi: bool) -> Vec<String> {
    let mut lines = wrap(
        &format!(
            "{} {}  {}",
            marker(row.status),
            row.status.label(),
            title(row)
        ),
        width,
    );
    let mut field = |label: &str, value: &str| {
        lines.extend(wrap(&format!("{label}: {}", text(value)), width));
    };
    field(localized("Reason", "Lý do", vi), &row.message);
    field(localized("Code", "Mã", vi), &row.code);
    field("ID", &row.id);
    field(
        localized("Required", "Bắt buộc", vi),
        if row.required {
            localized("yes", "có", vi)
        } else {
            localized("no", "không", vi)
        },
    );
    for (name, value) in [
        ("Repo", row.repository.as_deref()),
        ("Task", row.task_id.as_deref()),
        ("Binding", row.binding_id.as_deref()),
        ("Agent", row.agent_id.as_deref()),
        (
            localized("Evidence", "Bằng chứng", vi),
            row.evidence_level.as_deref(),
        ),
        (
            localized("Expected", "Mong đợi", vi),
            row.expected.as_deref(),
        ),
    ] {
        if let Some(value) = value {
            field(name, value);
        }
    }
    for source in &row.sources {
        field(localized("Source", "Nguồn", vi), source);
    }
    for blocker in &row.blocked_by {
        field(localized("Blocked by", "Bị chặn bởi", vi), blocker);
    }
    for action in &row.actions {
        field(localized("Next", "Tiếp theo", vi), &action.label);
        let effect = match action.effect {
            Effect::ReadOnly => localized("read-only", "chỉ đọc", vi),
            Effect::LocalWrite => localized("local write", "ghi cục bộ", vi),
            Effect::RepositoryWrite => localized("repository write", "ghi repository", vi),
            Effect::UserConfigWrite => {
                localized("user configuration write", "ghi cấu hình người dùng", vi)
            }
        };
        let confirmation = if action.requires_confirmation {
            localized(" · confirmation required", " · cần xác nhận", vi)
        } else {
            ""
        };
        field(
            localized("Effect", "Tác động", vi),
            &format!("{effect}{confirmation}"),
        );
    }
    if let Some(observed) = &row.observed {
        lines.extend(wrap(localized("Observed:", "Quan sát:", vi), width));
        let observed = redact_observed(observed, &mut 256, 6);
        if let Ok(value) = serde_json::to_string_pretty(&observed) {
            lines.extend(wrap(&value, width));
        }
    }
    lines
}

pub(super) fn report(report: &CheckReport, vi: bool) -> String {
    let mut output = String::new();
    let mut append = |value: &str, indent: &str| {
        for line in wrap(value, 88 - indent.len()) {
            let _ = writeln!(output, "{indent}{line}");
        }
    };
    append(&overview(report, vi), "");
    append(&counts(report), "");
    let scope = scope(report, vi);
    if !scope.is_empty() {
        append(&scope, "");
    }
    for group in GROUPS {
        let count = report
            .findings
            .iter()
            .filter(|row| Group::of(row.status) == group)
            .count();
        if count == 0 {
            continue;
        }
        append("", "");
        append(&format!("{} ({count})", group.label(vi)), "");
        for status in STATUS_ORDER
            .into_iter()
            .filter(|status| Group::of(*status) == group)
        {
            for row in report.findings.iter().filter(|row| row.status == status) {
                append(
                    &format!("{} {}  {}", marker(status), status.label(), title(row)),
                    "  ",
                );
                append(&text(&row.message), "    ");
                for action in &row.actions {
                    append(
                        &format!(
                            "{} {}",
                            localized("Next:", "Tiếp theo:", vi),
                            text(&action.label)
                        ),
                        "    ",
                    );
                }
            }
        }
    }
    output
}
