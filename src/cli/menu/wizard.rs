use super::{Effect, Input, Menu, OutputView, Tone, runner, text, viewport::Buffer};
use cirnetrules::operator::{self, AGENTS, SetupAction, SetupInput, SetupPreview};
use serde_json::Value;
use std::{collections::BTreeMap, fmt::Write, path::PathBuf};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    Agent,
    Repo,
    Binding,
    Target,
    Task,
    Actor,
    Bundle,
    McpBundle,
    Surface,
    Governance,
    Workspace,
    Profile,
    Receipt,
    Replace,
    Proposal,
    Spec,
    Plan,
}

const FIELDS: &[(Field, &str, [&str; 2])] = &[
    (
        Field::Agent,
        "a",
        [
            "Agent (codex, claude, antigravity; comma for multiple)",
            "Agent (codex, claude, antigravity; nhiều Agent cách bằng dấu phẩy)",
        ],
    ),
    (Field::Repo, "r", ["Repository ID", "ID repository"]),
    (Field::Binding, "b", ["Binding ID", "ID binding"]),
    (
        Field::Target,
        "t",
        [
            "Candidate path (absolute or relative to workspace)",
            "Đường dẫn ứng viên (tuyệt đối hoặc tương đối từ workspace)",
        ],
    ),
    (
        Field::Task,
        "k",
        ["Approved task ID", "ID task đã được phê duyệt"],
    ),
    (Field::Actor, "o", ["Operator label", "Nhãn người vận hành"]),
    (
        Field::Bundle,
        "u",
        ["Instruction bundle directory", "Thư mục bundle chỉ dẫn"],
    ),
    (
        Field::McpBundle,
        "m",
        ["MCP bundle directory", "Thư mục bundle MCP"],
    ),
    (
        Field::Surface,
        "s",
        [
            "Native surface (cli, ide, app)",
            "Ứng dụng native (cli, ide, app)",
        ],
    ),
    (
        Field::Governance,
        "g",
        ["Governance checkout", "Checkout governance"],
    ),
    (
        Field::Workspace,
        "w",
        ["Workspace directory", "Thư mục workspace"],
    ),
    (
        Field::Profile,
        "f",
        ["Governance profile", "Profile governance"],
    ),
    (Field::Receipt, "e", ["Receipt ID", "ID receipt"]),
    (
        Field::Replace,
        "x",
        ["Previous binding identity", "Danh tính binding trước đó"],
    ),
    (Field::Proposal, "p", ["New proposal ID", "ID proposal mới"]),
    (
        Field::Spec,
        "d",
        [
            "Central English spec path",
            "Đường dẫn spec tiếng Anh trung tâm",
        ],
    ),
    (
        Field::Plan,
        "l",
        [
            "Central English plan path",
            "Đường dẫn plan tiếng Anh trung tâm",
        ],
    ),
];

const ACTIONS: &[(SetupAction, [&str; 2])] = &[
    (
        SetupAction::Bind,
        ["Confirm candidate binding", "Xác nhận binding ứng viên"],
    ),
    (
        SetupAction::ExportInstructions,
        ["Export instruction bundle", "Xuất bundle chỉ dẫn"],
    ),
    (
        SetupAction::SyncStrict,
        [
            "Sync with strict ownership",
            "Sync với ownership nghiêm ngặt",
        ],
    ),
    (
        SetupAction::SyncManaged,
        [
            "Reconcile AGENTS.md / CLAUDE.md as a managed block",
            "Hợp nhất khối quản lý trong AGENTS.md / CLAUDE.md",
        ],
    ),
    (
        SetupAction::ExportMcp,
        ["Export MCP launch templates", "Xuất các mẫu khởi chạy MCP"],
    ),
    (
        SetupAction::RegisterMcp,
        [
            "Register project MCP entry or view manual import",
            "Đăng ký MCP phạm vi dự án hoặc xem cách nhập thủ công",
        ],
    ),
    (
        SetupAction::RollbackSync,
        ["Roll back instruction sync", "Hoàn tác sync chỉ dẫn"],
    ),
    (
        SetupAction::RollbackMcp,
        ["Roll back MCP registration", "Hoàn tác đăng ký MCP"],
    ),
    (
        SetupAction::ProposeTask,
        [
            "Propose a task with no grants",
            "Tạo đề xuất task chưa có grant",
        ],
    ),
];

pub(super) struct Wizard {
    input: SetupInput,
    remaining_agents: Vec<String>,
    edit: Option<Field>,
    pending: Option<SetupPreview>,
    context: Result<Value, String>,
    notice: String,
    receipts: BTreeMap<String, Vec<(SetupAction, String)>>,
    pub(super) suspended: bool,
}

impl Wizard {
    fn new(input: SetupInput) -> Self {
        let edit = if input.scope.agent.is_none() {
            Some(Field::Agent)
        } else if input.scope.repo.is_none() {
            Some(Field::Repo)
        } else {
            None
        };
        let context = operator::inspect_setup(&input.scope);
        Self {
            input,
            remaining_agents: Vec::new(),
            edit,
            pending: None,
            context,
            notice: String::new(),
            receipts: BTreeMap::new(),
            suspended: false,
        }
    }

    fn refresh(&mut self) {
        self.context = operator::inspect_setup(&self.input.scope);
    }

    pub(super) fn editing_text(&self) -> bool {
        self.edit.is_some() && self.pending.is_none() && !self.suspended
    }

    fn set(&mut self, field: Field, value: &str) -> Result<(), &'static str> {
        let value = value.trim();
        let value = value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .unwrap_or(value);
        if value.starts_with(['$', '<', char::from(96)]) || value.chars().any(char::is_control) {
            return Err(
                "Use an actual path or identifier. Shell variables and placeholders are not expanded.",
            );
        }
        let selected = (!value.is_empty()).then(|| value.to_owned());
        let path = || selected.as_ref().map(PathBuf::from);
        let scope = &mut self.input.scope;
        match field {
            Field::Agent => {
                let agents: Vec<_> = value.split(',').map(str::trim).collect();
                if !agents.iter().all(|agent| AGENTS.contains(agent)) {
                    return Err(
                        "Choose codex, claude or antigravity, separated by commas for sequential setup.",
                    );
                }
                if scope.agent.as_deref() != Some(agents[0]) {
                    scope.surface = None;
                    scope.bundle = None;
                    self.input.receipt = None;
                }
                scope.agent = Some(agents[0].into());
                self.remaining_agents.clear();
                for agent in agents.into_iter().skip(1) {
                    if scope.agent.as_deref() != Some(agent)
                        && !self.remaining_agents.iter().any(|id| id == agent)
                    {
                        self.remaining_agents.push(agent.into());
                    }
                }
            }
            Field::Repo => {
                if let Ok(context) = &self.context
                    && selected.as_ref().is_some_and(|id| {
                        !context["repositories"]
                            .as_array()
                            .is_some_and(|rows| rows.iter().any(|row| row == id))
                    })
                {
                    return Err("Select a repository from the governance registry list.");
                }
                if scope.repo != selected {
                    scope.binding = None;
                    scope.target = None;
                    scope.task = None;
                    scope.bundle = None;
                    self.input.replace_binding = None;
                    self.input.receipt = None;
                }
                scope.repo = selected;
            }
            Field::Binding => {
                if scope.binding != selected {
                    self.input.receipt = None;
                }
                scope.binding = selected;
                scope.target = None;
            }
            Field::Target => scope.target = path(),
            Field::Task => scope.task = selected,
            Field::Actor => self.input.actor = selected,
            Field::Bundle => scope.bundle = path(),
            Field::McpBundle => scope.mcp_bundle = path(),
            Field::Surface => {
                if !matches!(selected.as_deref(), None | Some("cli" | "ide" | "app")) {
                    return Err("Choose cli, ide or app.");
                }
                scope.surface = selected;
            }
            Field::Governance | Field::Workspace => {
                if field == Field::Governance {
                    scope.governance_root = path();
                }
                if field == Field::Workspace {
                    scope.workspace = path();
                }
                scope.binding = None;
                scope.target = None;
                scope.task = None;
                scope.bundle = None;
                scope.mcp_bundle = None;
                self.input.replace_binding = None;
                self.input.receipt = None;
            }
            Field::Profile => self.input.profile = selected,
            Field::Receipt => self.input.receipt = selected,
            Field::Replace => self.input.replace_binding = selected,
            Field::Proposal => self.input.proposal = selected,
            Field::Spec => self.input.spec = selected,
            Field::Plan => self.input.plan = selected,
        }
        Ok(())
    }

    fn check_args(&self, selector: &str) -> Vec<String> {
        let scope = &self.input.scope;
        let mut args = vec!["check".into(), selector.into()];
        for (flag, value) in [
            ("repo", &scope.repo),
            ("binding", &scope.binding),
            ("task", &scope.task),
            ("agent", &scope.agent),
            ("surface", &scope.surface),
        ] {
            if let Some(value) = value {
                args.extend([format!("--{flag}"), value.clone()]);
            }
        }
        for (flag, value) in [
            ("governance-root", &scope.governance_root),
            ("workspace", &scope.workspace),
            ("bundle", &scope.bundle),
            ("mcp-bundle", &scope.mcp_bundle),
            ("target", &scope.target),
        ] {
            if let Some(value) = value {
                args.extend([format!("--{flag}"), value.to_string_lossy().into_owned()]);
            }
        }
        args
    }

    fn display(&self, vi: bool) -> String {
        let mut out = String::new();
        let scope = &self.input.scope;
        let _ = writeln!(
            out,
            "Repo {}   Agent {}   Task {}",
            scope.repo.as_deref().unwrap_or("—"),
            scope.agent.as_deref().unwrap_or("—"),
            scope.task.as_deref().unwrap_or("—")
        );
        match &self.context {
            Ok(context) => {
                let _ = writeln!(
                    out,
                    "Governance  {}",
                    context["governance_root"].as_str().unwrap_or("—")
                );
                let _ = writeln!(
                    out,
                    "Workspace   {}",
                    context["workspace"].as_str().unwrap_or("—")
                );
                let _ = writeln!(
                    out,
                    "Binding {}   Target {}",
                    context["binding"]["id"].as_str().unwrap_or("—"),
                    context["binding"]["target"].as_str().unwrap_or("—")
                );
                for key in ["binding_reason", "task_reason"] {
                    if let Some(reason) = context[key].as_str() {
                        let _ = writeln!(out, "{reason}");
                    }
                }
            }
            Err(reason) => {
                let _ = writeln!(out, "BLOCKED · {reason}");
            }
        }
        if !self.notice.is_empty() {
            let _ = writeln!(out, "\n{}", self.notice);
        }
        if !self.receipts.is_empty() {
            let _ = writeln!(
                out,
                "\n{}",
                text(
                    ["Receipts from this setup", "Receipt của phiên thiết lập"],
                    vi
                )
            );
            for (agent, receipts) in &self.receipts {
                for (action, receipt) in receipts {
                    let name = action_name(*action, vi);
                    let _ = writeln!(out, "  {agent} · {name} · {receipt}");
                }
            }
        }
        if let Some(preview) = &self.pending {
            let hint = if preview.action == SetupAction::RegisterMcp {
                [
                    "Review native project trust and the displayed config change. Type YES then Enter to apply. Esc cancels.",
                    "Kiểm tra trust dự án trong Agent và thay đổi hiển thị. Gõ YES rồi Enter để áp dụng. Esc để hủy.",
                ]
            } else {
                [
                    "Review the plan. Type YES then Enter to apply. Esc cancels.",
                    "Xem trước kế hoạch. Gõ YES rồi Enter để áp dụng. Esc để hủy.",
                ]
            };
            let _ = writeln!(
                out,
                "\n{}\n\n{}",
                text(hint, vi),
                render_report(preview.action, &preview.report, vi)
            );
            return out;
        }
        if let Some(edit) = self.edit {
            if let Some((_, _, hint)) = FIELDS.iter().find(|(field, _, _)| *field == edit) {
                let _ = writeln!(out, "\n{} · Enter", text(*hint, vi));
            }
            let key = match edit {
                Field::Repo => Some("repositories"),
                Field::Binding => Some("bindings"),
                Field::Target => Some("candidates"),
                _ => None,
            };
            if let (Ok(context), Some(key)) = (&self.context, key) {
                for row in context[key].as_array().into_iter().flatten().take(32) {
                    let _ = writeln!(
                        out,
                        "  {}",
                        row.as_str()
                            .or_else(|| row["path"].as_str())
                            .unwrap_or_default()
                    );
                }
            }
            return out;
        }
        let _ = writeln!(
            out,
            "\n{}",
            text(
                [
                    "Enter a selection. PageUp/PageDown scroll.",
                    "Nhập lựa chọn. PageUp/PageDown để cuộn."
                ],
                vi
            )
        );
        for (index, (_, name)) in ACTIONS.iter().enumerate() {
            let _ = writeln!(out, "  {}  {}", index + 1, text(*name, vi));
        }
        let _ = writeln!(
            out,
            "  c  {}\n  v  {}\n  h  {}",
            text(["Inspect prerequisites", "Kiểm tra điều kiện"], vi),
            text(["Probe MCP protocol", "Kiểm tra giao thức MCP"], vi),
            text(["Native handoff", "Bàn giao sang Agent native"], vi)
        );
        if let Some(agent) = self.remaining_agents.first() {
            let _ = writeln!(out, "  n  Agent → {agent}");
        }
        let _ = writeln!(
            out,
            "\n{}",
            text(
                ["Edit context or inputs", "Chỉnh ngữ cảnh hoặc đầu vào"],
                vi
            )
        );
        for (_, key, hint) in FIELDS {
            let _ = writeln!(out, "  {key}  {}", text(*hint, vi));
        }
        out
    }
}

fn action_name(action: SetupAction, vi: bool) -> &'static str {
    ACTIONS
        .iter()
        .find(|(candidate, _)| *candidate == action)
        .map_or("", |(_, name)| text(*name, vi))
}

fn render_report(action: SetupAction, report: &Value, vi: bool) -> String {
    let name = action_name(action, vi);
    let mut out = String::new();
    if let Some(status) = report["status"].as_str() {
        let _ = write!(out, "{status} · ");
    }
    let _ = write!(out, "{name}");
    for key in ["state", "registration"] {
        if let Some(value) = report[key].as_str() {
            let _ = write!(out, " · {value}");
        }
    }
    let _ = writeln!(out);
    if let Some(reason) = report["reason"].as_str() {
        let _ = writeln!(out, "  {reason}");
    }
    if let Some(next) = report["next_action"]
        .as_str()
        .or_else(|| report["next"].as_str())
    {
        let _ = writeln!(
            out,
            "  {} · {next}",
            text(["Next action", "Bước tiếp theo"], vi)
        );
    }
    for (key, label) in [
        ("destination", ["Destination", "Đích"]),
        ("path", ["Path", "Đường dẫn"]),
        ("file", ["File", "Tệp"]),
        ("receipt", ["Receipt", "Receipt"]),
        ("authorization", ["Authorization", "Quyền"]),
        ("native_client", ["Native client", "Agent native"]),
        ("native_discovery", ["Native discovery", "Phát hiện native"]),
    ] {
        if let Some(value) = report[key].as_str() {
            let _ = writeln!(out, "  {} · {value}", text(label, vi));
        }
    }
    let mut files = report["files"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|file| {
            let path = file.as_str().or_else(|| file["path"].as_str())?;
            Some((path, file["action"].as_str(), file["reason"].as_str()))
        })
        .chain(
            report["files"]
                .as_object()
                .into_iter()
                .flat_map(|files| files.keys().map(|path| (path.as_str(), None, None))),
        )
        .peekable();
    if files.peek().is_some() {
        let _ = writeln!(out, "\n{}", text(["Files", "Tệp"], vi));
        for (path, operation, reason) in files.by_ref().take(20) {
            let _ = write!(out, "  ");
            if let Some(operation) = operation {
                let _ = write!(out, "{operation} · ");
            }
            let _ = writeln!(out, "{path}");
            if let Some(reason) = reason {
                let _ = writeln!(out, "    {reason}");
            }
        }
        if files.peek().is_some() {
            let _ = writeln!(
                out,
                "  {}",
                text(
                    [
                        "See the full file list in the details below.",
                        "Xem danh sách tệp đầy đủ trong chi tiết bên dưới."
                    ],
                    vi
                )
            );
        }
    }
    if let Ok(details) = serde_json::to_string_pretty(report) {
        let _ = writeln!(
            out,
            "\n{}\n{details}",
            text(["Details (JSON)", "Chi tiết (JSON)"], vi)
        );
    }
    out
}

impl Menu {
    pub(super) fn open_wizard(&mut self, input: SetupInput) {
        self.wizard = Some(Wizard::new(input));
        self.show_wizard();
    }

    pub(super) fn show_wizard(&mut self) {
        let Some(wizard) = &mut self.wizard else {
            return;
        };
        wizard.suspended = false;
        let mut output = OutputView::new(
            text(["Agent setup", "Thiết lập Agent"], self.vi).into(),
            self.vi,
        );
        output.running = false;
        output.status = text(
            if wizard.pending.is_some() {
                ["Review preview · Esc cancels", "Xem trước · Esc để hủy"]
            } else {
                [
                    "Enter to select · Esc to go back",
                    "Enter để chọn · Esc để quay lại",
                ]
            },
            self.vi,
        )
        .into();
        output.tone = Tone::Accent;
        output.buffer = Buffer::from_streams(
            runner::sanitize(wizard.display(self.vi).as_bytes()),
            "",
            None,
        );
        self.output = Some(output);
        self.open_prompt(Input::default());
    }

    pub(super) fn wizard_active(&self) -> bool {
        self.wizard.as_ref().is_some_and(|wizard| !wizard.suspended)
    }

    pub(super) fn wizard_back(&mut self) -> Effect {
        let Some(wizard) = &mut self.wizard else {
            return Effect::Idle;
        };
        if wizard.suspended {
            self.show_wizard();
        } else if wizard.pending.take().is_some() || wizard.edit.take().is_some() {
            wizard.notice.clear();
            self.show_wizard();
        } else {
            self.wizard = None;
            self.output = None;
            self.close_main_prompt();
        }
        Effect::Redraw
    }

    pub(super) fn submit_wizard(&mut self) -> Effect {
        let input = self.prompt.as_ref().expect("wizard input");
        if input.error == Some(super::COMMAND_TOO_LONG) {
            return Effect::Idle;
        }
        let value = input.input.value.trim().to_owned();
        let wizard = self.wizard.as_mut().expect("wizard active");
        wizard.notice.clear();
        if let Some(preview) = wizard.pending.take() {
            if value == "YES" {
                let action = preview.action;
                let trust = action == SetupAction::RegisterMcp;
                wizard.notice = match operator::perform_setup(preview, trust) {
                    Ok(result) => {
                        if let Some(receipt) = result["receipt"].as_str() {
                            wizard.input.receipt = Some(receipt.into());
                            if let Some(agent) = &wizard.input.scope.agent {
                                let receipts = wizard.receipts.entry(agent.clone()).or_default();
                                if receipts.len() < 32
                                    && !receipts
                                        .iter()
                                        .any(|row| row.0 == action && row.1 == receipt)
                                {
                                    receipts.push((action, receipt.into()));
                                }
                            }
                        }
                        render_report(action, &result, self.vi)
                    }
                    Err(reason) => format!("BLOCKED · {reason}"),
                };
                wizard.refresh();
            } else {
                wizard.pending = Some(preview);
                wizard.notice = text(
                    [
                        "Type YES to apply the displayed plan.",
                        "Gõ YES để áp dụng kế hoạch đã hiển thị.",
                    ],
                    self.vi,
                )
                .into();
            }
        } else if let Some(field) = wizard.edit.take() {
            match wizard.set(field, &value) {
                Ok(()) => {
                    wizard.refresh();
                    if field == Field::Agent && wizard.input.scope.repo.is_none() {
                        wizard.edit = Some(Field::Repo);
                    }
                }
                Err(reason) => {
                    wizard.edit = Some(field);
                    wizard.notice = reason.into();
                }
            }
        } else if matches!(value.as_str(), "c" | "v") {
            wizard.suspended = true;
            return Effect::Run(wizard.check_args(if value == "v" { "mcp" } else { "all" }));
        } else if value == "n" && !wizard.remaining_agents.is_empty() {
            wizard.input.scope.agent = Some(wizard.remaining_agents.remove(0));
            wizard.input.scope.surface = None;
            wizard.input.scope.bundle = None;
            wizard.input.receipt = None;
            wizard.refresh();
        } else if value == "h" {
            let agent = wizard
                .input
                .scope
                .agent
                .as_deref()
                .unwrap_or("selected Agent");
            wizard.notice = format!(
                "{agent} · NATIVE_CLIENT_VERIFIED = NOT_RUN\n{}",
                text(
                    [
                        "Open the confirmed worktree in this native client. Verify its instructions, selected skills and MCP tools in that client. Project registration, manual import and the CNR probe are separate actions.",
                        "Mở worktree đã xác nhận trong Agent này. Xác minh chỉ dẫn, skill được chọn và công cụ MCP từ chính Agent. Đăng ký, nhập thủ công và probe CNR là các thao tác riêng."
                    ],
                    self.vi
                )
            );
        } else if let Some((field, _, _)) = FIELDS.iter().find(|(_, key, _)| *key == value) {
            wizard.edit = Some(*field);
        } else if let Some((action, _)) = value
            .parse::<usize>()
            .ok()
            .and_then(|index| index.checked_sub(1))
            .and_then(|index| ACTIONS.get(index))
        {
            match operator::preview_setup(*action, &wizard.input) {
                Ok(preview)
                    if preview.report["status"] == "PASS"
                        && preview.report["plan_sha256"].is_string()
                        && preview.report["state"] != "UNCHANGED"
                        && preview.report["registration"] != "UNCHANGED" =>
                {
                    wizard.pending = Some(preview)
                }
                Ok(preview) => {
                    wizard.notice = render_report(preview.action, &preview.report, self.vi);
                }
                Err(reason) => wizard.notice = format!("BLOCKED · {reason}"),
            }
        } else if !value.is_empty() {
            wizard.notice = text(
                [
                    "Choose one of the displayed selections.",
                    "Chọn một mục đang hiển thị.",
                ],
                self.vi,
            )
            .into();
        }
        self.show_wizard();
        Effect::Redraw
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cirnetrules::operator::Scope;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn preview_status_reason_file_and_next_action_precede_the_raw_plan() {
        let report = serde_json::json!({
            "status": "BLOCKED",
            "state": "DRY_RUN",
            "reason": "Review approval",
            "next_action": "Choose a task",
            "files": [{"path": "AGENTS.md", "action": "UPDATE"}],
            "native_client": "NOT_RUN"
        });
        let rendered = render_report(SetupAction::SyncManaged, &report, true);
        let (summary, evidence) = rendered.split_once("Chi tiết (JSON)\n").unwrap();
        assert!(summary.starts_with("BLOCKED"));
        assert!(summary.contains("DRY_RUN"));
        assert!(summary.contains("Review approval"));
        assert!(summary.contains("Bước tiếp theo · Choose a task"));
        assert!(summary.contains("UPDATE · AGENTS.md"));
        assert!(summary.contains("Agent native · NOT_RUN"));
        assert_eq!(serde_json::from_str::<Value>(evidence).unwrap(), report);
    }

    #[test]
    fn wizard_selects_agents_sequentially_and_clears_repo_dependent_fields() {
        let mut wizard = Wizard::new(SetupInput {
            scope: Scope {
                governance_root: Some(PathBuf::from("missing-fixture-governance")),
                repo: Some("original".into()),
                binding: Some("named".into()),
                task: Some("fixture".into()),
                ..Scope::default()
            },
            ..SetupInput::default()
        });
        assert!(
            wizard
                .set(Field::Agent, "codex,claude,codex,antigravity")
                .is_ok()
        );
        assert_eq!(wizard.input.scope.agent.as_deref(), Some("codex"));
        assert_eq!(wizard.remaining_agents, ["claude", "antigravity"]);
        assert!(wizard.set(Field::Target, "$Target").is_err());
        assert!(wizard.set(Field::Repo, "replacement").is_ok());
        assert!(wizard.input.scope.binding.is_none());
        assert!(wizard.input.scope.task.is_none());
    }

    #[test]
    fn pasted_confirmation_is_never_submitted_and_escape_discards_wizard() {
        let mut menu = Menu::new(false, None);
        menu.open_wizard(SetupInput {
            scope: Scope {
                governance_root: Some(PathBuf::from("missing-fixture-governance")),
                ..Scope::default()
            },
            ..SetupInput::default()
        });
        assert!(menu.wizard.as_ref().is_some_and(Wizard::editing_text));
        assert!(matches!(menu.paste("YES\r\n"), Effect::Redraw));
        assert!(menu.wizard.as_ref().unwrap().input.scope.agent.is_none());
        let escape = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(matches!(menu.handle(escape), Effect::Redraw));
        assert!(matches!(menu.handle(escape), Effect::Redraw));
        assert!(menu.wizard.is_none());
        assert!(menu.history.is_empty());
    }
}
