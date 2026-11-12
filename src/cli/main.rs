mod command_catalog;
mod menu;
mod operator;

use cirnetrules::{environment, execution, governance, mcp, tooling};
use clap::{Arg, ArgAction, ArgMatches, Command, error::ErrorKind, value_parser};
use serde_json::{Value, json};
use std::{
    env,
    io::{self, IsTerminal},
    path::PathBuf,
    process::ExitCode,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const COMMANDS: &[(&str, &str, &str)] = &[
    (
        "tui",
        "Open the interactive terminal",
        "Mở giao diện terminal",
    ),
    (
        "init",
        "Initialize a project without installing tools",
        "Khởi tạo dự án",
    ),
    ("search", "Search the tool catalog", "Tìm công cụ"),
    (
        "install",
        "Plan or install an explicit version",
        "Lập kế hoạch hoặc cài phiên bản",
    ),
    (
        "update",
        "Review available updates or apply an explicit version",
        "Kiểm tra hoặc cập nhật phiên bản",
    ),
    (
        "remove",
        "Remove an owned tool version",
        "Gỡ phiên bản do dự án sở hữu",
    ),
    ("list", "List tools and installations", "Liệt kê công cụ"),
    (
        "info",
        "Inspect a tool and its capabilities",
        "Xem công cụ và khả năng hỗ trợ",
    ),
    (
        "use",
        "Select an exact version in the current profile",
        "Chọn phiên bản trong profile hiện tại",
    ),
    (
        "pin",
        "Pin an exact version in the project",
        "Ghim phiên bản cho dự án",
    ),
    (
        "doctor",
        "Diagnose the local environment",
        "Chẩn đoán môi trường",
    ),
    (
        "config",
        "Read or edit project configuration",
        "Xem hoặc sửa cấu hình dự án",
    ),
    (
        "env",
        "Plan, lock, synchronize, export or import the environment",
        "Lập kế hoạch, khóa, đồng bộ, xuất hoặc nhập môi trường",
    ),
    (
        "venv",
        "Manage project virtual environments",
        "Quản lý môi trường ảo",
    ),
    (
        "package",
        "Work with the project's native package manager",
        "Quản lý gói của dự án",
    ),
    ("build", "Build a project", "Biên dịch dự án"),
    ("run", "Run a project", "Chạy dự án"),
    ("test", "Test a project", "Kiểm thử dự án"),
    (
        "exec",
        "Execute an explicit command after --",
        "Chạy lệnh tường minh sau --",
    ),
    (
        "workspace",
        "Discover, bind or inspect a governance workspace",
        "Dò, liên kết hoặc kiểm tra workspace",
    ),
    (
        "task",
        "Work with existing governance tasks",
        "Thao tác với task governance",
    ),
    (
        "governance",
        "Validate, resolve or migrate governance authority",
        "Kiểm tra, phân giải hoặc di chuyển nguồn governance",
    ),
    (
        "agent",
        "Export and sync Agent instructions",
        "Xuất và đồng bộ chỉ dẫn Agent",
    ),
    (
        "check",
        "Inspect system readiness",
        "Kiểm tra trạng thái hệ thống",
    ),
    ("setup", "Guide Agent setup", "Hướng dẫn thiết lập Agent"),
    (
        "cache",
        "Inspect or maintain project caches",
        "Kiểm tra hoặc bảo trì cache dự án",
    ),
    (
        "audit",
        "Inspect local task, migration, Agent and tooling receipts",
        "Xem biên nhận task, migration, Agent và công cụ",
    ),
    (
        "ci",
        "Verify a portable task candidate",
        "Xác minh candidate task có thể di chuyển",
    ),
    (
        "completion",
        "Generate bash, zsh, fish or PowerShell completion",
        "Tạo hoàn thành lệnh cho bash, zsh, fish hoặc PowerShell",
    ),
];

const GOVERNANCE_OPTIONS: &[&str] = &[
    "governance-root",
    "workspace",
    "repo",
    "task",
    "profile",
    "risk",
    "actor",
    "path",
    "paths",
    "descriptor",
    "revision",
    "to",
    "checkpoint",
    "legacy-review",
    "step",
    "session",
    "agent",
    "action",
    "proposal",
    "reason",
    "id",
    "source",
    "destination",
    "archive",
    "additional-archive",
    "preview-out",
    "expected-digest",
    "bundle",
    "target",
    "agents",
    "output",
    "expect",
    "receipt",
    "replace",
    "decision",
    "basis",
    "binding",
    "request-id",
    "generation",
    "ttl",
    "report",
    "session-file",
    "binary",
    "authority-commit",
    "candidate",
    "confirm",
    "stopped",
    "entry-mode",
    "surface",
    "mcp-bundle",
];

fn parser(vi: bool) -> Command {
    let flag = |name, help| {
        Arg::new(name)
            .long(name)
            .global(true)
            .action(ArgAction::SetTrue)
            .help(help)
    };
    let mut command = Command::new("cnr")
        .version(VERSION)
        .about(if vi {
            "CIRNET RULES — Governance, môi trường và công cụ phát triển"
        } else {
            "CIRNET RULES — Governance, environments and development tools"
        })
        .arg(
            Arg::new("project")
                .long("project")
                .global(true)
                .value_parser(value_parser!(PathBuf))
                .value_name("PATH")
                .help("Select a tooling project or maintenance workspace"),
        )
        .arg(
            Arg::new("scope")
                .long("scope")
                .global(true)
                .value_parser(["project", "user", "system"])
                .default_value("project")
                .help("Select the management scope"),
        )
        .arg(
            Arg::new("type")
                .long("type")
                .global(true)
                .value_name("TYPE")
                .help("Filter by catalog category"),
        )
        .arg(
            Arg::new("lang")
                .long("lang")
                .global(true)
                .value_parser(["en", "vi"])
                .default_value("en")
                .help("Select the terminal language"),
        )
        .arg(
            Arg::new("timeout-ms")
                .long("timeout-ms")
                .global(true)
                .value_parser(value_parser!(u64).range(1..=600_000))
                .default_value("120000")
                .help("Bound a child process"),
        )
        .arg(flag("installed", "Filter to installed tools"))
        .arg(flag("available", "Filter to catalog entries"))
        .arg(flag(
            "all",
            "Select every exact tool in the current environment for update",
        ))
        .arg(flag("json", "Write one JSON document to stdout"))
        .arg(flag("dry-run", "Preview effects without applying changes"))
        .arg(flag("offline", "Require the command's offline path"))
        .arg(flag("locked", "Require a matching environment lock"))
        .arg(flag("non-interactive", "Avoid interactive prompts"))
        .arg(flag("no-banner", "Omit the terminal banner"))
        .arg(flag("yes", "Confirm a reviewed project-scoped operation"));
    for (name, en, vietnamese) in COMMANDS {
        let mut sub = Command::new(*name)
            .about(if vi { *vietnamese } else { *en })
            .after_help(usage_notes(name, vi))
            .arg(Arg::new("words").value_name("ARGS").num_args(0..));
        if matches!(
            *name,
            "workspace" | "task" | "governance" | "agent" | "audit" | "ci"
        ) {
            for &option in GOVERNANCE_OPTIONS {
                sub = sub.arg(option_arg(option, name).hide(
                    option != "governance-root"
                        && (*name != "agent" || !command_catalog::uses_agent_option(option)),
                ));
            }
        }
        if matches!(*name, "check" | "setup") {
            for &option in command_catalog::OPERATOR_OPTIONS {
                if command_catalog::operator_option_allowed(name, option) {
                    sub = sub.arg(option_arg(option, name));
                }
            }
            if *name == "check" {
                sub = sub
                    .arg(
                        Arg::new("verbose")
                            .long("verbose")
                            .action(ArgAction::SetTrue)
                            .help("Show check evidence and coverage details"),
                    )
                    .arg(
                        Arg::new("strict")
                            .long("strict")
                            .action(ArgAction::SetTrue)
                            .help("Treat warnings as blocking"),
                    );
            }
        }
        if *name == "agent" {
            sub = sub.arg(Arg::new("trust-reviewed")
                .long("trust-reviewed")
                .action(ArgAction::SetTrue)
                .help("Confirm independent review of the native project trust for MCP registration"));
        }
        command = command.subcommand(sub);
    }
    let mut serve = Command::new("serve").about("Serve CIRNETRULES MCP over stdin and stdout");
    for (key, help) in [
        ("governance-root", "Absolute CIRNETRULES authority checkout"),
        ("workspace", "Absolute workspace containing bound checkouts"),
        ("access", "read-only or coordinated"),
        (
            "session-file",
            "Operator-prepared capability file for coordinated access",
        ),
    ] {
        serve = serve.arg(
            Arg::new(key)
                .long(key)
                .value_name("VALUE")
                .help(help)
                .required(matches!(key, "governance-root" | "workspace")),
        );
    }
    command.subcommand(
        Command::new("mcp")
            .about("Model Context Protocol (STDIO)")
            .subcommand_required(true)
            .subcommand(serve),
    )
}

fn option_arg(option: &'static str, command: &str) -> Arg {
    let help = match option {
        "agent" => "Select the Agent adapter",
        "actor" => "Use the authorized task actor",
        "agents" => "Select a comma-separated list of Agent adapters",
        "binary" => "Select the CNR executable for MCP launch export",
        "binding" => "Select the named worktree binding",
        "bundle" => "Select the existing export bundle",
        "entry-mode" => "Choose strict ownership or reviewed managed-block reconciliation",
        "expect" => "Confirm the digest of the displayed preview",
        "governance-root" => "Select the governance authority checkout",
        "mcp-bundle" => "Select the MCP launch bundle",
        "output" => "Select the export directory",
        "profile" => "Select a governance context profile",
        "receipt" => "Select an existing receipt",
        "repo" => "Select a repository ID from the governance registry",
        "report" => "Save the check report to a new file",
        "surface" => "Select the documented client surface",
        "target" => "Select the exact bound worktree",
        "task" => "Select a task ID",
        "workspace" => "Select the maintenance workspace",
        _ => "",
    };
    let mut arg = Arg::new(option).long(option).value_name("VALUE").help(help);
    if option == "agent" && command == "check" {
        return arg.value_parser(clap::builder::PossibleValuesParser::new(
            command_catalog::AGENTS.iter().copied().chain(["all"]),
        ));
    }
    let choices = match option {
        "agent" if matches!(command, "agent" | "setup") => Some(command_catalog::AGENTS),
        "entry-mode" => Some(command_catalog::ENTRY_MODES),
        "surface" => Some(command_catalog::SURFACES),
        _ => None,
    };
    if let Some(choices) = choices {
        arg = arg.value_parser(clap::builder::PossibleValuesParser::new(
            choices.iter().copied(),
        ));
    }
    arg
}

fn usage_notes(name: &str, vi: bool) -> &'static str {
    match name {
        "install" | "update" | "remove" if vi => {
            "Chọn TOOL[@VERSION]. Xem trước với --dry-run. Xác nhận thay đổi bằng --yes."
        }
        "install" | "update" | "remove" => {
            "Select TOOL[@VERSION]. Preview with --dry-run. Confirm mutations with --yes."
        }
        "env" => "plan | lock | sync | profiles | export | import PATH",
        "config" => "show | get KEY | set KEY VALUE | unset KEY",
        "venv" => "list | info NAME | create NAME [VERSION] | run NAME -- ARG... | remove NAME",
        "build" | "run" | "test" => "[rust|node|python|cmake] [-- ARG...]",
        "exec" => "-- PROGRAM [ARG...]",
        "package" => "[rust|node|python] list|add|remove|sync|outdated [ARG...]",
        "workspace" => {
            "discover | bind --repo NAME --path PATH --actor NAME | health\nbinding list | add | show | validate [--repo NAME] [--binding ID]\n--project selects the maintenance workspace. --governance-root selects the authority checkout."
        }
        "task" => {
            "create | show | resume | revise | context | bootstrap | diff-check | verify | review | accept | close\nclaim | renew | release | report | status | recover | session-prepare\ndependency check | evidence export | evidence inspect\nMaintenance options may also follow --."
        }
        "governance" => {
            "validate | inspect | context --repo NAME [--task ID] [--profile ID]\nmigrate inspect --source FILE | export --source FILE --destination DIR | validate --archive DIR\nmigrate import --archive DIR --dry-run [--preview-out DIR] | --yes --expected-digest DIGEST\nmigrate rollback --expected-digest DIGEST --yes"
        }
        "agent" => {
            "list | export --repo NAME --agents codex,claude,antigravity --output DIR | check --bundle DIR\nmcp-config export --binary PATH --workspace PATH --output DIR | check --bundle DIR\nmcp-config install --agent AGENT --repo NAME --bundle DIR --target WORKTREE --dry-run\nmcp-config rollback --agent AGENT --target WORKTREE --receipt ID --dry-run\nmcp-config inspect | probe | preview-export\nsync --bundle DIR --target WORKTREE --dry-run [--entry-mode strict|managed-block]\nrollback --target WORKTREE --receipt ID --dry-run\nApply a reviewed sync, rollback or MCP registration with --yes --expect DIGEST --task ID --actor NAME."
        }
        "check" => {
            "Defaults to all. Select governance, registry, policies, rules, skills, profiles, references, hooks, repositories, bindings, routing, task, diff, verification, audit, environment, agent-config, agent, or mcp.\nChecks inspect the selected scope. Native client evidence is reported separately."
        }
        "setup" => {
            "setup agent [--repo NAME] [--agent codex|claude|antigravity]\nThe terminal offers a chooser. Headless setup requires one explicit Agent and inspects the supplied context."
        }
        "audit" => {
            "tooling [--project PATH] | task --task ID | migration --receipt DIGEST | agent --receipt ID\nThe existing audit list form keeps its tooling output."
        }
        "ci" => {
            "verify --bundle FILE --target CHECKOUT --expected-digest SHA256 --authority-commit COMMIT --governance-root PATH"
        }
        "completion" => "bash | zsh | fish | powershell",
        _ => "",
    }
}

fn main() -> ExitCode {
    let args: Result<Vec<_>, _> = env::args_os()
        .skip(1)
        .map(|arg| arg.into_string())
        .collect();
    let Ok(args) = args else {
        return emit("cnr", error("Arguments must be valid Unicode"), false);
    };
    run(&args)
}

fn run(args: &[String]) -> ExitCode {
    let boundary = args
        .iter()
        .position(|word| word == "--")
        .unwrap_or(args.len());
    let (flags, trailing) = args.split_at(boundary);
    let json_mode = flags.iter().any(|arg| arg == "--json");
    let mcp_stdio = flags.windows(2).any(|pair| pair == ["mcp", "serve"]);
    let vi = flags.windows(2).any(|pair| pair == ["--lang", "vi"])
        || flags.iter().any(|arg| arg == "--lang=vi");
    let preflight = if mcp_stdio {
        None
    } else {
        command_catalog::preflight(args)
    };
    let command = parser(vi);
    let matched = match command
        .clone()
        .try_get_matches_from(std::iter::once("cnr").chain(flags.iter().map(String::as_str)))
    {
        Ok(matched) => matched,
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            if json_mode {
                let value = if error.kind() == ErrorKind::DisplayVersion {
                    json!({"status":"PASS","name":"cnr","version":VERSION,"os":env::consts::OS,"arch":env::consts::ARCH})
                } else {
                    json!({"status":"PASS","help":error.to_string()})
                };
                println!("{value}");
            } else {
                print!("{error}");
            }
            return ExitCode::SUCCESS;
        }
        Err(problem) if mcp_stdio => {
            eprint!("{problem}");
            return ExitCode::FAILURE;
        }
        Err(problem) => {
            return emit(
                "cnr",
                preflight
                    .as_ref()
                    .filter(|diagnostic| diagnostic.is_unresolved())
                    .map_or_else(
                        || error(&problem.to_string()),
                        |diagnostic| diagnostic.envelope(vi),
                    ),
                json_mode,
            );
        }
    };
    let (name, sub) = matched.subcommand().unwrap_or(("tui", &matched));
    if name == "mcp" {
        return mcp_command(&matched, sub, trailing);
    }
    if let Some(diagnostic) = preflight {
        let mut result = diagnostic.envelope(vi);
        if name == "agent"
            && let Some(words) = sub.get_many::<String>("words")
            && let Some(command) = command_catalog::agent_command(
                &words.map(String::as_str).collect::<Vec<_>>(),
                matched.get_flag("dry-run"),
            )
            && let Value::Object(mut envelope) = result.clone()
        {
            envelope.insert("command".into(), command.into());
            envelope.insert("result".into(), result);
            result = Value::Object(envelope);
        }
        return emit(name, result, json_mode);
    }
    if matches!(name, "check" | "setup") {
        if let Err(reason) = validate_flags(name, &[], &matched, false) {
            return emit(name, error(reason), json_mode);
        }
        return operator::run(name, sub, &matched, trailing);
    }
    let words: Vec<_> = sub
        .try_get_many::<String>("words")
        .ok()
        .flatten()
        .into_iter()
        .flatten()
        .cloned()
        .collect();
    if name == "tui" {
        if !words.is_empty() || !trailing.is_empty() {
            return emit(name, error("tui takes no command arguments"), json_mode);
        }
        let context = match validate_flags(name, &words, &matched, false) {
            Ok(()) => &matched,
            Err(reason) => return emit(name, error(reason), json_mode),
        };
        let show_help = json_mode
            || context.get_flag("non-interactive")
            || !io::stdin().is_terminal()
            || !io::stdout().is_terminal()
            || env::var("TERM").is_ok_and(|term| term == "dumb");
        if show_help {
            let help = command.clone().render_long_help().to_string();
            if json_mode {
                println!("{}", json!({"status":"PASS","help":help}));
            } else {
                print!("{help}");
            }
            return ExitCode::SUCCESS;
        }
        return menu::run(context, vi);
    }
    let governance_group = matches!(name, "workspace" | "task" | "governance" | "agent" | "ci")
        || name == "audit"
            && words
                .first()
                .is_some_and(|action| !matches!(action.as_str(), "list" | "tooling"));
    if let Err(reason) = validate_flags(name, &words, &matched, governance_group) {
        return emit(name, error(reason), json_mode);
    }
    if governance_group {
        return governance_command(name, &words, trailing, &matched, sub);
    }
    if name == "audit" && GOVERNANCE_OPTIONS.iter().any(|key| sub.contains_id(key)) {
        return emit(
            name,
            error("Tooling audit uses --project; select an audit source for other options"),
            json_mode,
        );
    }
    let result = dispatch(name, &words, trailing, &matched).unwrap_or_else(|reason| error(&reason));
    emit(name, result, json_mode)
}

fn validate_flags(
    name: &str,
    words: &[String],
    flags: &ArgMatches,
    governance_group: bool,
) -> Result<(), &'static str> {
    let provider = matches!(
        name,
        "search"
            | "list"
            | "info"
            | "doctor"
            | "install"
            | "update"
            | "remove"
            | "venv"
            | "cache"
            | "audit"
    );
    let process = matches!(name, "package" | "build" | "run" | "test" | "exec");
    let environment = matches!(name, "init" | "config" | "env" | "use" | "pin");
    let sync = name == "env" && words.first().is_some_and(|action| action == "sync");
    if flags.get_flag("all") && (name != "update" || !words.is_empty()) {
        return Err("--all requires update with no individual tool");
    }
    if flags.get_flag("all") && flags.get_one::<String>("type").is_some() {
        return Err("--all selects the environment without a catalog type filter");
    }
    if flags.get_one::<String>("type").is_some()
        && !matches!(
            name,
            "search" | "list" | "info" | "install" | "update" | "remove"
        )
    {
        return Err("--type is only available for catalog commands");
    }
    if (flags.get_flag("installed") || flags.get_flag("available"))
        && !matches!(name, "search" | "list" | "info")
    {
        return Err("--installed and --available filter search, list or info");
    }
    if flags.get_flag("offline") && (governance_group || !(provider || process || sync)) {
        return Err("--offline is only available for providers, execution and env sync");
    }
    if flags.get_flag("locked") && !(process || environment || matches!(name, "install" | "update"))
    {
        return Err("--locked is only available for environment, installs and execution");
    }
    if flags.get_flag("dry-run") && !(provider || process || environment || governance_group) {
        return Err("--dry-run is unavailable for this command; pass maintenance options after --");
    }
    let provider_change = matches!(name, "install" | "update" | "remove" | "venv" | "cache");
    if flags.get_flag("yes") && !(provider_change || process || sync || governance_group) {
        return Err("--yes only confirms project-scoped provider or execution operations");
    }
    if flags.value_source("timeout-ms") == Some(clap::parser::ValueSource::CommandLine) && !process
    {
        return Err("--timeout-ms only applies to build, run, test, exec and package");
    }
    let mcp_scope = name == "agent"
        && matches!(
            words
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .as_slice(),
            ["mcp-config", "install" | "rollback" | "inspect"]
        );
    if !mcp_scope
        && flags.value_source("scope") == Some(clap::parser::ValueSource::CommandLine)
        && (governance_group || !(provider || process || environment))
    {
        return Err("--scope is unavailable for this command");
    }
    Ok(())
}

fn project(flags: &ArgMatches, required: bool) -> Result<Option<environment::Project>, String> {
    if let Some(path) = flags.get_one::<PathBuf>("project") {
        return environment::Project::open(path).map(Some);
    }
    let cwd = env::current_dir().map_err(|_| "Cannot determine the working directory")?;
    let discovered = cwd.ancestors().find(|path| {
        [
            "cnr.toml",
            "cnr.lock",
            ".git",
            "Cargo.toml",
            "package.json",
            "pyproject.toml",
            "CMakeLists.txt",
        ]
        .iter()
        .any(|name| path.join(name).symlink_metadata().is_ok())
    });
    match discovered.or_else(|| required.then_some(cwd.as_path())) {
        Some(path) => environment::Project::open(path).map(Some),
        None => Ok(None),
    }
}

fn dispatch(
    name: &str,
    words: &[String],
    trailing: &[String],
    flags: &ArgMatches,
) -> Result<Value, String> {
    if name == "completion" {
        return match words {
            [shell] if trailing.is_empty() => completion(shell),
            _ => Err("Usage: cnr completion bash|zsh|fish|powershell".into()),
        };
    }
    if !trailing.is_empty()
        && !matches!(name, "build" | "run" | "test" | "exec" | "package" | "venv")
    {
        return Err("Unexpected arguments after --".into());
    }
    let options = tooling::Options {
        scope: flags.get_one::<String>("scope").unwrap().clone(),
        dry_run: flags.get_flag("dry-run"),
        offline: flags.get_flag("offline"),
        yes: flags.get_flag("yes"),
        kind: flags.get_one::<String>("type").cloned(),
        installed: flags.get_flag("installed"),
        available: flags.get_flag("available"),
        locked: flags.get_flag("locked"),
        expected_selection: None,
    };
    if options.scope != "project" {
        return Ok(blocked(
            "CNR operations use project scope. User and system scopes require a separate authorized installer.",
        ));
    }
    let mut full = vec![name.to_owned()];
    full.extend_from_slice(words);
    full.extend_from_slice(trailing);
    let locked = flags.get_flag("locked");
    if name == "update" && flags.get_flag("all") {
        return sync(&project(flags, true)?.unwrap(), locked, options);
    }
    match name {
        "init" | "use" | "pin" | "config" | "env" => {
            let project = project(flags, true)?.unwrap();
            if full.get(1).is_some_and(|word| word == "sync") && name == "env" {
                if full.len() != 2 {
                    return Err("Usage: cnr env sync".into());
                }
                return sync(&project, locked, options);
            }
            if locked && let Err(reason) = project.selected_locked() {
                return Ok(blocked(&reason));
            }
            environment::dispatch(&project, &full, locked, options.dry_run)
        }
        "build" | "run" | "test" | "exec" | "package" => {
            let project = project(flags, true)?.unwrap();
            let selected = match selection(&project, locked) {
                Ok(selected) => selected,
                Err(reason) => return Ok(blocked(&reason)),
            };
            execution::dispatch(
                project.root(),
                &full,
                &selected,
                &execution::Options {
                    dry_run: options.dry_run,
                    offline: options.offline,
                    locked,
                    yes: options.yes,
                    timeout_ms: *flags.get_one::<u64>("timeout-ms").unwrap(),
                },
            )
        }
        "search" | "install" | "update" | "remove" | "list" | "info" | "doctor" | "venv"
        | "cache" | "audit" => {
            let required = matches!(name, "install" | "remove" | "venv" | "cache" | "audit")
                || (name == "update" && !words.is_empty());
            let project = project(flags, required || locked)?;
            if locked && let Some(project) = &project {
                let selected = match project.selected_locked() {
                    Ok(selected) => selected,
                    Err(reason) => return Ok(blocked(&reason)),
                };
                if let Some(target) = words.first() {
                    let (tool, version) = target
                        .split_once('@')
                        .ok_or("--locked requires an explicit tool@version")?;
                    if selected.get(tool).map(String::as_str) != Some(version) {
                        return Ok(blocked(
                            "Requested tool version does not match the project lock.",
                        ));
                    }
                }
            }
            let mut report = tooling::dispatch(
                project.as_ref().map(environment::Project::root),
                &full,
                &options,
            )?;
            if name == "doctor"
                && let Some(project) = project
            {
                report["project_environment"] = match project.selected() {
                    Ok(pins) => json!({"status":"PASS","selected":pins}),
                    Err(reason) => blocked(&reason),
                };
            }
            Ok(report)
        }
        _ => Err("Unknown CNR command".into()),
    }
}

fn selection(
    project: &environment::Project,
    locked: bool,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    if locked {
        project.selected_locked()
    } else {
        project.selected()
    }
}

fn sync(
    project: &environment::Project,
    locked: bool,
    options: tooling::Options,
) -> Result<Value, String> {
    let resolve = || {
        if locked {
            project.selected_locked()
        } else {
            project.selected_for_sync()
        }
    };
    let selected = match resolve() {
        Ok(selected) => selected,
        Err(reason) => return Ok(blocked(&reason)),
    };
    let targets = selected
        .iter()
        .map(|(tool, version)| vec!["install".into(), format!("{tool}@{version}")])
        .collect::<Vec<_>>();
    let mut preview = Vec::new();
    let mut check = tooling::Options {
        dry_run: true,
        expected_selection: Some(selected.clone()),
        ..options
    };
    for target in &targets {
        let step = tooling::dispatch(Some(project.root()), target, &check)
            .unwrap_or_else(|reason| blocked(&reason));
        preview.push(step);
    }
    if preview.iter().any(|step| step["status"] != "PASS") {
        return Ok(
            json!({"status":"BLOCKED","reason":"At least one tool cannot be synchronized.","plan":preview}),
        );
    }
    if options.dry_run || targets.is_empty() {
        return Ok(json!({"status":"PASS","plan":preview,"applied":false}));
    }
    if !options.yes {
        return Ok(
            json!({"status":"BLOCKED","reason":"Review the plan and confirm the project operation with --yes.","plan":preview}),
        );
    }
    check.dry_run = false;
    let mut applied = Vec::new();
    for target in &targets {
        if resolve().as_ref() != Ok(&selected) {
            return Ok(
                json!({"status":"BLOCKED","reason":"Project environment changed after planning.","steps":applied}),
            );
        }
        let step = tooling::dispatch(Some(project.root()), target, &check)
            .unwrap_or_else(|reason| error(&reason));
        let status = step["status"].as_str().unwrap_or("FAIL").to_owned();
        applied.push(step);
        if status != "PASS" {
            return Ok(
                json!({"status":status,"reason":"Sync stopped at the first unsuccessful step.","steps":applied}),
            );
        }
    }
    Ok(json!({"status":"PASS","applied":true,"steps":applied}))
}

fn mcp_command(flags: &ArgMatches, group: &ArgMatches, trailing: &[String]) -> ExitCode {
    let Some(("serve", options)) = group.subcommand() else {
        eprintln!("Use cnr mcp serve");
        return ExitCode::FAILURE;
    };
    if !trailing.is_empty()
        || flags.get_one::<PathBuf>("project").is_some()
        || [
            "json",
            "dry-run",
            "yes",
            "locked",
            "offline",
            "installed",
            "available",
            "all",
        ]
        .iter()
        .any(|key| flags.get_flag(key))
        || ["scope", "type", "timeout-ms"]
            .iter()
            .any(|key| flags.value_source(key) == Some(clap::parser::ValueSource::CommandLine))
    {
        eprintln!("MCP STDIO requires explicit roots and only the serve options");
        return ExitCode::FAILURE;
    }
    let value = |key| options.get_one::<String>(key).map(String::as_str);
    let root = std::path::Path::new(value("governance-root").expect("required root"));
    let work = std::path::Path::new(value("workspace").expect("required workspace"));
    let access = value("access").unwrap_or("read-only");
    let session_file = value("session-file").map(std::path::Path::new);
    if access == "coordinated" && session_file.is_none() {
        eprintln!("MCP coordinated access requires an operator-prepared session file");
        return ExitCode::from(2);
    }
    match mcp::serve(root, work, access, session_file) {
        Ok(()) => ExitCode::SUCCESS,
        Err(reason) => {
            eprintln!("MCP: {reason}");
            ExitCode::FAILURE
        }
    }
}

fn governance_command(
    group: &str,
    words: &[String],
    trailing: &[String],
    flags: &ArgMatches,
    sub: &ArgMatches,
) -> ExitCode {
    let json_mode = flags.get_flag("json");
    let dry_run = flags.get_flag("dry-run");
    let yes = flags.get_flag("yes");
    let command = match (
        group,
        words
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice(),
    ) {
        (
            "governance",
            [
                "migrate",
                action @ ("inspect" | "export" | "validate" | "rollback" | "adoption"),
            ],
        ) => {
            format!("migration-{action}")
        }
        ("governance", ["migrate", "import"]) if dry_run != yes => if dry_run {
            "migration-preview"
        } else {
            "migration-apply"
        }
        .into(),
        ("workspace", ["binding", action @ ("list" | "add" | "show" | "validate")]) => {
            format!("workspace-binding-{action}")
        }
        (
            "task",
            [
                action @ ("claim" | "renew" | "release" | "report" | "status" | "recover"
                | "session-prepare"),
            ],
        ) => {
            format!("coordination-{action}")
        }
        ("task", ["coordination", "migrate"]) => "coordination-migrate".into(),
        ("task", ["dependency", "check"]) => "coordination-dependency-check".into(),
        ("task", ["evidence", action @ ("export" | "inspect")]) => format!("ci-evidence-{action}"),
        ("agent", action) => match command_catalog::agent_command(action, dry_run) {
            Some(command) => command,
            None => return emit(group, error("Choose a supported Agent action"), json_mode),
        },
        ("ci", ["verify"]) if sub.get_one::<String>("governance-root").is_some() => {
            "ci-verify".into()
        }
        ("audit", [action]) => format!("audit-{action}"),
        ("governance", ["inspect"]) => "authority-inspect".into(),
        ("governance", ["context"]) if trailing.is_empty() => "authority-context".into(),
        (group, [action]) => {
            let action = *action;
            match (group, action) {
                ("workspace", "discover" | "bind" | "health") => format!("workspace-{action}"),
                ("workspace", "route" | "context-prune") => action.to_owned(),
                (
                    "task",
                    "create" | "show" | "resume" | "revise" | "checkpoint" | "transition"
                    | "verify" | "review" | "accept" | "close" | "context" | "bootstrap",
                ) => format!("task-{action}"),
                (
                    "task",
                    "route" | "grant-evaluate" | "run-step" | "cancel-step" | "diff-check"
                    | "review-context" | "docs-preview" | "docs-apply" | "docs-check",
                ) => action.to_owned(),
                ("governance", action) if governance::COMMANDS.contains(&action) => {
                    action.to_owned()
                }
                _ => return emit(group, error("Unknown maintenance action"), json_mode),
            }
        }
        _ => {
            return emit(
                group,
                error("Choose a supported action; migration import requires --dry-run or --yes"),
                json_mode,
            );
        }
    };
    let mut native: Vec<_> = trailing.iter().skip(1).cloned().collect();
    for option in GOVERNANCE_OPTIONS {
        if let Some(value) = sub.get_one::<String>(option) {
            let option = if *option == "governance-root" {
                "governance"
            } else {
                option
            };
            native.extend([format!("--{option}"), value.to_owned()]);
        }
    }
    if dry_run
        && !matches!(
            command.as_str(),
            "migration-preview" | "agent-preview-export"
        )
    {
        native.push("--dry-run".into());
    }
    if yes {
        native.push("--yes".into());
    }
    if flags.value_source("scope") == Some(clap::parser::ValueSource::CommandLine)
        && let Some(scope) = flags.get_one::<String>("scope")
    {
        native.extend(["--scope".into(), scope.clone()]);
    }
    if group == "agent" && sub.get_flag("trust-reviewed") {
        native.extend(["--trust-reviewed".into(), "true".into()]);
    }
    if let Some(path) = flags.get_one::<PathBuf>("project") {
        if native.iter().any(|arg| arg == "--workspace") {
            return emit(
                group,
                error("Choose either --project or the maintenance --workspace option"),
                json_mode,
            );
        }
        let Some(path) = path.to_str() else {
            return emit(
                group,
                error("Project path must be valid Unicode"),
                json_mode,
            );
        };
        native.extend(["--workspace".to_owned(), path.to_owned()]);
    }
    let (code, result) = governance::run(&command, native);
    println!(
        "{}",
        if json_mode {
            result.to_string()
        } else {
            serde_json::to_string_pretty(&result).unwrap()
        }
    );
    ExitCode::from(code)
}

fn completion(shell: &str) -> Result<Value, String> {
    let commands = COMMANDS
        .iter()
        .map(|(name, _, _)| *name)
        .collect::<Vec<_>>()
        .join(" ");
    let script = match shell {
        "bash" => format!("complete -W '{commands}' cnr\n"),
        "zsh" => format!("#compdef cnr\n_arguments '1:command:({commands})'\n"),
        "fish" => format!("complete -c cnr -n '__fish_use_subcommand' -f -a '{commands}'\n"),
        "powershell" => format!(
            "Register-ArgumentCompleter -Native -CommandName cnr -ScriptBlock {{\n    param($wordToComplete, $commandAst, $cursorPosition)\n    '{commands}'.Split(' ') | Where-Object {{ $_.StartsWith($wordToComplete, [StringComparison]::OrdinalIgnoreCase) }}\n}}\n"
        ),
        _ => return Err("Unsupported shell; choose bash, zsh, fish or powershell".into()),
    };
    Ok(json!({"status":"PASS","shell":shell,"script":script}))
}

fn error(reason: &str) -> Value {
    json!({"status":"FAIL","reason":reason})
}
fn blocked(reason: &str) -> Value {
    json!({"status":"BLOCKED","reason":reason})
}

fn emit(command: &str, result: Value, machine: bool) -> ExitCode {
    let status = result["status"].as_str().unwrap_or("FAIL");
    if machine {
        println!("{result}");
    } else if let Some(script) = result["script"]
        .as_str()
        .filter(|_| command == "completion")
    {
        print!("{script}");
    } else {
        println!(
            "CNR · {command} · {status}\n{}",
            serde_json::to_string_pretty(&result).unwrap()
        );
    }
    ExitCode::from(match status {
        "PASS" => 0,
        "BLOCKED" | "NOT_RUN" => 2,
        "CANCELLED" => 130,
        _ => 1,
    })
}
