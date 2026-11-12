use serde_json::{Value, json};

pub(super) use cirnetrules::operator::AGENTS;
pub(super) const ENTRY_MODES: &[&str] = &["strict", "managed-block"];
pub(super) const SURFACES: &[&str] = &["cli", "ide", "app"];
pub(super) const OPERATOR_OPTIONS: &[&str] = &[
    "governance-root",
    "workspace",
    "repo",
    "binding",
    "task",
    "agent",
    "bundle",
    "mcp-bundle",
    "target",
    "report",
    "surface",
    "profile",
    "actor",
];

struct Action {
    words: &'static [&'static str],
    allowed: &'static [&'static str],
    required: &'static [&'static str],
    review: bool,
}

const AGENT_ACTIONS: &[Action] = &[
    Action {
        words: &["list"],
        allowed: &[],
        required: &[],
        review: false,
    },
    Action {
        words: &["export"],
        allowed: &[
            "repo",
            "agents",
            "output",
            "profile",
            "task",
            "workspace",
            "binding",
            "dry-run",
            "expect",
        ],
        required: &["repo", "agents", "output"],
        review: false,
    },
    Action {
        words: &["check"],
        allowed: &["bundle", "workspace", "binding"],
        required: &["bundle"],
        review: false,
    },
    Action {
        words: &["sync"],
        allowed: &[
            "bundle",
            "target",
            "workspace",
            "binding",
            "dry-run",
            "yes",
            "expect",
            "task",
            "actor",
            "entry-mode",
        ],
        required: &["target", "bundle"],
        review: true,
    },
    Action {
        words: &["rollback"],
        allowed: &[
            "target",
            "workspace",
            "binding",
            "receipt",
            "dry-run",
            "yes",
            "expect",
            "task",
            "actor",
        ],
        required: &["target", "receipt"],
        review: true,
    },
    Action {
        words: &["mcp-config", "export"],
        allowed: &["output", "binary", "workspace", "expect"],
        required: &["workspace", "binary", "output"],
        review: false,
    },
    Action {
        words: &["mcp-config", "preview-export"],
        allowed: &["output", "binary", "workspace"],
        required: &["workspace", "binary", "output"],
        review: false,
    },
    Action {
        words: &["mcp-config", "check"],
        allowed: &["bundle"],
        required: &["bundle"],
        review: false,
    },
    Action {
        words: &["mcp-config", "inspect"],
        allowed: &[
            "agent",
            "scope",
            "repo",
            "bundle",
            "target",
            "workspace",
            "binding",
            "task",
            "surface",
        ],
        required: &["agent", "target"],
        review: false,
    },
    Action {
        words: &["mcp-config", "probe"],
        allowed: &["workspace", "repo", "task"],
        required: &["workspace"],
        review: false,
    },
    Action {
        words: &["mcp-config", "install"],
        allowed: &[
            "agent",
            "scope",
            "repo",
            "bundle",
            "target",
            "workspace",
            "binding",
            "dry-run",
            "yes",
            "expect",
            "task",
            "actor",
            "surface",
            "trust-reviewed",
        ],
        required: &["agent", "target", "bundle", "repo"],
        review: true,
    },
    Action {
        words: &["mcp-config", "rollback"],
        allowed: &[
            "agent",
            "scope",
            "repo",
            "target",
            "workspace",
            "binding",
            "dry-run",
            "yes",
            "expect",
            "task",
            "actor",
            "receipt",
            "surface",
        ],
        required: &["agent", "target", "receipt"],
        review: true,
    },
];

fn agent_action(words: &[&str]) -> Option<&'static Action> {
    AGENT_ACTIONS.iter().find(|action| action.words == words)
}

pub(super) fn agent_command(words: &[&str], dry_run: bool) -> Option<String> {
    let action = agent_action(words)?;
    Some(if dry_run && action.words == ["export"] {
        "agent-preview-export".into()
    } else {
        format!("agent-{}", action.words.join("-"))
    })
}

pub(super) fn action_words(command: &str, words: &[&str]) -> Option<Vec<&'static str>> {
    let choices = match (command, words) {
        ("check", []) => cirnetrules::operator::TARGETS.to_vec(),
        ("setup", []) => vec!["agent"],
        ("agent", _) => {
            let mut next: Vec<_> = AGENT_ACTIONS
                .iter()
                .filter(|action| action.words.starts_with(words))
                .filter_map(|action| action.words.get(words.len()).copied())
                .collect();
            next.sort_unstable();
            next.dedup();
            next
        }
        ("check" | "setup", _) => Vec::new(),
        _ => return None,
    };
    Some(choices)
}

pub(super) fn uses_agent_option(option: &str) -> bool {
    AGENT_ACTIONS
        .iter()
        .any(|action| action.allowed.contains(&option))
}

pub(super) fn supports_option(command: &str, words: &[String], flag: &str) -> bool {
    let flag = flag.trim_start_matches("--");
    if matches!(
        flag,
        "help" | "version" | "json" | "lang" | "no-banner" | "non-interactive" | "governance-root"
    ) {
        return true;
    }
    let words: Vec<_> = words.iter().map(String::as_str).collect();
    match command {
        "agent" => AGENT_ACTIONS.iter().any(|action| {
            action.words.starts_with(&words)
                && action
                    .allowed
                    .contains(&if flag == "project" { "workspace" } else { flag })
        }),
        "check" => match flag {
            "project" | "workspace" | "repo" | "binding" | "task" | "verbose" | "strict"
            | "report" => true,
            "agent" | "bundle" | "target" | "surface" => {
                matches!(
                    words.as_slice(),
                    [] | ["all" | "agent" | "agent-config" | "mcp"]
                )
            }
            "mcp-bundle" => matches!(words.as_slice(), [] | ["all" | "agent" | "mcp"]),
            _ => false,
        },
        "setup" => matches!(
            flag,
            "project"
                | "workspace"
                | "repo"
                | "binding"
                | "task"
                | "agent"
                | "bundle"
                | "mcp-bundle"
                | "target"
                | "surface"
                | "profile"
                | "actor"
        ),
        _ => true,
    }
}

pub(super) fn operator_option_allowed(command: &str, option: &str) -> bool {
    // An option's group is independent of order and scope. Action-specific
    // restrictions and prerequisites are checked against its parsed selector.
    OPERATOR_OPTIONS.contains(&option)
        && match command {
            "check" => !matches!(option, "profile" | "actor"),
            "setup" => option != "report",
            _ => false,
        }
}

pub(super) fn accepts_inherited_project(command: &str, words: &[String]) -> bool {
    match command {
        "audit" => words.is_empty() || matches!(words[0].as_str(), "list" | "tooling"),
        "ci" => false,
        "governance" => !matches!(
            words.first().map(String::as_str),
            Some("inspect" | "migrate")
        ),
        _ => supports_option(command, words, "--project"),
    }
}

pub(super) struct Diagnostic {
    pub(super) en: &'static str,
    pub(super) vi: &'static str,
    code: &'static str,
    missing: Vec<&'static str>,
    field: Option<&'static str>,
}

impl Diagnostic {
    pub(super) fn is_unresolved(&self) -> bool {
        self.code == "CNR_ARGUMENT_UNRESOLVED"
    }

    pub(super) fn envelope(&self, vi: bool) -> Value {
        json!({
            "status": "FAIL",
            "code": self.code,
            "reason": if vi { self.vi } else { self.en },
            "missing_fields": self.missing,
            "invalid_field": self.field,
            "remediation": {
                "action": if self.is_unresolved() { "replace_unresolved_value" } else { "supply_required_fields" },
                "fields": self.missing,
            }
        })
    }
}

fn unresolved(field: Option<&'static str>) -> Diagnostic {
    Diagnostic {
        en: "Replace shell variables or placeholders with actual values. Use a quoted path or choose the value in Agent setup.",
        vi: "Thay biến shell hoặc giá trị mẫu bằng dữ liệu thật. Đặt đường dẫn trong dấu nháy hoặc chọn trong Thiết lập Agent.",
        code: "CNR_ARGUMENT_UNRESOLVED",
        missing: Vec::new(),
        field,
    }
}

fn missing(fields: Vec<&'static str>) -> Diagnostic {
    let (code, en, vi) = match fields[0] {
        "target" => (
            "CNR_ARGUMENT_MISSING_TARGET",
            "Select the bound worktree with --target and the exported instructions with --bundle or open Agent setup.",
            "Chọn worktree đã liên kết bằng --target và gói chỉ dẫn đã xuất bằng --bundle hoặc mở Thiết lập Agent.",
        ),
        "bundle" => (
            "CNR_ARGUMENT_MISSING_BUNDLE",
            "Select the exported bundle with --bundle or open Agent setup.",
            "Chọn gói đã xuất bằng --bundle hoặc mở Thiết lập Agent.",
        ),
        "agent" => (
            "CNR_ARGUMENT_MISSING_AGENT",
            "Select one Agent with --agent codex, --agent claude, or --agent antigravity.",
            "Chọn một Agent bằng --agent codex, --agent claude hoặc --agent antigravity.",
        ),
        "dry-run|yes" => (
            "CNR_ARGUMENT_MISSING_REVIEW_MODE",
            "Preview with --dry-run. Apply a reviewed plan with --yes, --expect, --task and --actor.",
            "Xem trước bằng --dry-run. Áp dụng kế hoạch đã duyệt với --yes, --expect, --task và --actor.",
        ),
        _ => (
            "CNR_ARGUMENT_MISSING",
            "Supply the missing options shown in the command help or continue in Agent setup.",
            "Bổ sung các tham số trong trợ giúp lệnh hoặc tiếp tục trong Thiết lập Agent.",
        ),
    };
    Diagnostic {
        en,
        vi,
        code,
        missing: fields,
        field: None,
    }
}

fn shell_value(value: &str) -> bool {
    let identifier = |text: &str| {
        !text.is_empty()
            && text
                .bytes()
                .all(|ch| ch.is_ascii_alphanumeric() || b"_.:-".contains(&ch))
    };
    value == "`"
        || value
            .strip_prefix('$')
            .is_some_and(|rest| rest.starts_with('(') || rest.starts_with('{') || identifier(rest))
        || value
            .strip_prefix('<')
            .and_then(|text| text.strip_suffix('>'))
            .is_some_and(identifier)
}

pub(super) fn preflight(args: &[String]) -> Option<Diagnostic> {
    check_arguments(args, false)
}

pub(super) fn preflight_in_prompt(args: &[String]) -> Option<Diagnostic> {
    // The child invocation resolves an inherited maintenance workspace.
    check_arguments(args, true)
}

fn check_arguments(args: &[String], defer_workspace: bool) -> Option<Diagnostic> {
    if args
        .first()
        .is_some_and(|arg| arg == "&" || shell_value(arg))
    {
        return Some(unresolved(None));
    }
    let mut words = Vec::new();
    let mut options = Vec::new();
    let mut pending = None;
    for arg in args {
        if matches!(arg.as_str(), "--help" | "-h" | "--version" | "-V") {
            return None;
        }
        if let Some(name) = arg.strip_prefix("--") {
            pending = None;
            if name.is_empty() {
                continue;
            }
            let (name, value) = name
                .split_once('=')
                .map_or((name, None), |(name, value)| (name, Some(value)));
            let value_option = super::GOVERNANCE_OPTIONS
                .iter()
                .chain(&[
                    "project",
                    "governance",
                    "scope",
                    "type",
                    "lang",
                    "timeout-ms",
                ])
                .find(|&&key| key == name)
                .copied();
            if value != Some("") {
                options.push(name);
            }
            if let Some(value_option) = value_option {
                if value.is_some_and(shell_value) {
                    return Some(unresolved(Some(value_option)));
                }
                if value.is_none() {
                    pending = Some(value_option);
                }
            }
        } else if let Some(field) = pending.take() {
            if shell_value(arg) {
                return Some(unresolved(Some(field)));
            }
        } else {
            if words
                .first()
                .is_some_and(|word| matches!(*word, "agent" | "check" | "setup"))
                && shell_value(arg)
            {
                return Some(unresolved(None));
            }
            words.push(arg.as_str());
            if words.len() == 1
                && !matches!(
                    words[0],
                    "agent"
                        | "check"
                        | "setup"
                        | "workspace"
                        | "task"
                        | "governance"
                        | "audit"
                        | "ci"
                )
            {
                return None;
            }
        }
    }
    let ["agent", rest @ ..] = words.as_slice() else {
        // In the TUI setup without an Agent opens the chooser. The headless
        // setup handler owns the explicit selection diagnostic.
        return None;
    };
    let action = agent_action(rest)?;
    let has = |key: &str| options.contains(&key);
    let present =
        |key: &str| has(key) || (key == "workspace" && (has("project") || defer_workspace));
    let mut absent: Vec<_> = action
        .required
        .iter()
        .copied()
        .filter(|key| !present(key))
        .collect();
    if action.review {
        if present("yes") && !present("dry-run") {
            absent.extend(
                ["expect", "task", "actor"]
                    .into_iter()
                    .filter(|key| !present(key)),
            );
        }
        if !present("yes") && !present("dry-run") {
            absent.push("dry-run|yes");
        }
    }
    (!absent.is_empty()).then(|| missing(absent))
}
