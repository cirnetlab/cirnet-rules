use cirnetrules::operator::{self, Scope, SetupInput};
use clap::ArgMatches;
use serde_json::json;
use std::{
    io::{self, IsTerminal},
    path::PathBuf,
    process::ExitCode,
};

fn value(args: &ArgMatches, name: &str) -> Option<String> {
    args.try_get_one::<String>(name).ok().flatten().cloned()
}

fn scope(group: &ArgMatches, flags: &ArgMatches) -> Scope {
    let path = |name| value(group, name).map(PathBuf::from);
    Scope {
        governance_root: path("governance-root"),
        workspace: path("workspace").or_else(|| flags.get_one::<PathBuf>("project").cloned()),
        repo: value(group, "repo"),
        binding: value(group, "binding"),
        task: value(group, "task"),
        agent: value(group, "agent"),
        surface: value(group, "surface"),
        bundle: path("bundle"),
        mcp_bundle: path("mcp-bundle"),
        target: path("target"),
    }
}

fn setup_input(group: &ArgMatches, flags: &ArgMatches) -> Result<SetupInput, &'static str> {
    let mut scope = scope(group, flags);
    let words = group
        .get_many::<String>("words")
        .into_iter()
        .flatten()
        .map(String::as_str)
        .collect::<Vec<_>>();
    match words.as_slice() {
        ["agent"] => {}
        ["codex"] if scope.agent.as_deref().is_none_or(|agent| agent == "codex") => {
            scope.agent = Some("codex".into());
        }
        _ => return Err("Choose cnr setup agent and optionally --agent codex|claude|antigravity."),
    }
    Ok(SetupInput {
        scope,
        actor: value(group, "actor"),
        profile: value(group, "profile"),
        ..SetupInput::default()
    })
}

pub(super) fn setup_command(
    args: &[String],
    vi: bool,
    project: Option<PathBuf>,
) -> Result<Option<SetupInput>, String> {
    if !args.iter().any(|word| word == "setup")
        || args.iter().any(|arg| {
            matches!(
                arg.as_str(),
                "--help" | "-h" | "--json" | "--non-interactive"
            )
        })
    {
        return Ok(None);
    }
    let flags = match super::parser(vi)
        .try_get_matches_from(std::iter::once("cnr").chain(args.iter().map(String::as_str)))
    {
        Ok(flags) => flags,
        Err(error) if args.first().is_some_and(|word| word == "setup") => {
            return Err(error.to_string());
        }
        Err(_) => return Ok(None),
    };
    let Some(group) = flags.subcommand_matches("setup") else {
        return Ok(None);
    };
    super::validate_flags("setup", &[], &flags, false)?;
    let mut input = setup_input(group, &flags)?;
    if input.scope.workspace.is_none() {
        input.scope.workspace = project;
    }
    Ok(Some(input))
}

pub(super) fn run(
    name: &str,
    group: &ArgMatches,
    flags: &ArgMatches,
    trailing: &[String],
) -> ExitCode {
    let machine = flags.get_flag("json");
    let vi = flags
        .get_one::<String>("lang")
        .is_some_and(|value| value == "vi");
    if !trailing.is_empty() {
        return diagnostic(
            name,
            "CNR_UNEXPECTED_ARGUMENT",
            "Pass check and setup options before --.",
            &[],
            machine,
        );
    }
    if name == "setup" {
        let input = match setup_input(group, flags) {
            Ok(input) => input,
            Err(reason) => {
                return diagnostic(name, "CNR_INVALID_SETUP_ACTION", reason, &[], machine);
            }
        };
        if !machine
            && !flags.get_flag("non-interactive")
            && io::stdin().is_terminal()
            && io::stdout().is_terminal()
        {
            return super::menu::run_setup(flags, input, vi);
        }
        if input.scope.agent.is_none() {
            return diagnostic(
                name,
                "CNR_ARGUMENT_MISSING_AGENT",
                "Choose --agent codex, claude or antigravity.",
                &["agent"],
                machine,
            );
        }
        let mut report = operator::check_console("agent", &input.scope, false);
        report.requested_scope["target"] = json!("setup");
        if input.scope.repo.is_none() && report.exit_code == 0 {
            return diagnostic(
                name,
                "CNR_ARGUMENT_MISSING_REPOSITORY",
                "Choose a repository with --repo or use the interactive setup chooser.",
                &["repo"],
                machine,
            );
        }
        return emit_report(&report, machine, false, vi);
    }
    let request = match check_request(group, flags) {
        Ok(request) => request,
        Err(reason) => return diagnostic(name, "CNR_INVALID_CHECK_TARGET", reason, &[], machine),
    };
    let report = request.finalize(operator::check_console(
        &request.target,
        &request.scope,
        request.strict,
    ));
    emit_report(&report, request.machine, request.verbose, vi)
}

pub(super) struct CheckRequest {
    pub(super) target: String,
    pub(super) scope: Scope,
    pub(super) strict: bool,
    pub(super) machine: bool,
    pub(super) verbose: bool,
    report_path: Option<PathBuf>,
}

impl CheckRequest {
    pub(super) fn finalize(&self, mut report: operator::CheckReport) -> operator::CheckReport {
        if let Some(path) = &self.report_path
            && let Err(reason) = report.save(path)
        {
            report.report_save_failed(&reason);
        }
        report
    }
}

pub(super) fn check_request(
    group: &ArgMatches,
    flags: &ArgMatches,
) -> Result<CheckRequest, &'static str> {
    let words = group
        .get_many::<String>("words")
        .into_iter()
        .flatten()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let target = match words.as_slice() {
        [] => "all",
        [target] => target,
        _ => return Err("Choose a single check target."),
    };
    Ok(CheckRequest {
        target: target.into(),
        scope: scope(group, flags),
        strict: group.get_flag("strict"),
        machine: flags.get_flag("json"),
        verbose: group.get_flag("verbose"),
        report_path: value(group, "report").map(PathBuf::from),
    })
}

fn emit_report(report: &operator::CheckReport, machine: bool, verbose: bool, vi: bool) -> ExitCode {
    if machine || verbose {
        let result = if verbose && !machine {
            serde_json::to_string_pretty(report)
        } else {
            serde_json::to_string(report)
        };
        match result {
            Ok(text) => println!("{text}"),
            Err(_) => return ExitCode::FAILURE,
        }
    } else {
        print!("{}", report.render_for(vi));
    }
    ExitCode::from(report.exit_code)
}

fn diagnostic(name: &str, code: &str, reason: &str, fields: &[&str], machine: bool) -> ExitCode {
    super::emit(
        name,
        json!({"status":"ERROR","code":code,"reason":reason,"missing":fields,
        "remediation":{"message":reason}}),
        machine,
    )
}
