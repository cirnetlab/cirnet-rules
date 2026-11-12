use cirnetrules::tooling::{CompletionTool, completion_catalog};
use clap::{Arg, ArgMatches};
use std::{borrow::Cow, ops::Range, path::Path, sync::OnceLock};

const LIMIT: usize = 7;
const MAX_INPUT: usize = 8192;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Command,
    Subcommand,
    Option,
    Value,
    Tool,
    Version,
    History,
}

impl Kind {
    pub(super) fn label(self, vi: bool) -> &'static str {
        let labels = match self {
            Self::Command => ["command", "lệnh"],
            Self::Subcommand => ["action", "thao tác"],
            Self::Option => ["option", "tùy chọn"],
            Self::Value => ["value", "giá trị"],
            Self::Tool => ["tool", "công cụ"],
            Self::Version => ["version", "phiên bản"],
            Self::History => ["history", "lịch sử"],
        };
        labels[usize::from(vi)]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Source {
    Cli,
    Menu,
    Catalog,
    Pinned,
    Installed,
    History,
}

impl Source {
    pub(super) fn label(self, vi: bool) -> &'static str {
        let labels = match self {
            Self::Cli => ["CNR", "CNR"],
            Self::Menu => ["menu", "menu"],
            Self::Catalog => ["catalog", "danh mục"],
            Self::Pinned => ["pinned", "đã ghim"],
            Self::Installed => ["installed", "đã cài"],
            Self::History => ["history", "lịch sử"],
        };
        labels[usize::from(vi)]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Suggestion {
    pub(super) text: String,
    pub(super) insert_range: Range<usize>,
    pub(super) label: String,
    pub(super) kind: Kind,
    pub(super) description: String,
    pub(super) source: Source,
}

struct Token<'a> {
    range: Range<usize>,
    value: Cow<'a, str>,
}

fn tokens(input: &str) -> Result<Vec<Token<'_>>, &'static str> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();
    while let Some((start, ch)) = chars.peek().copied() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        let mut quote = None;
        let mut decoded = None::<String>;
        let mut literal_start = start;
        while let Some((index, ch)) = chars.peek().copied() {
            if quote.is_none() && ch.is_whitespace() {
                break;
            }
            chars.next();
            if ch == '\\' && quote != Some('\'') {
                // Preserve Windows separators. Only a run immediately before
                // a double quote participates in Windows quote escaping.
                let mut count = 1;
                while chars.next_if(|(_, next)| *next == '\\').is_some() {
                    count += 1;
                }
                if let Some((quote_index, _)) = chars.next_if(|(_, next)| *next == '"') {
                    let value = decoded.get_or_insert_with(String::new);
                    value.push_str(&input[literal_start..index]);
                    value.extend(std::iter::repeat_n('\\', count / 2));
                    if count % 2 == 1 {
                        value.push('"');
                    } else {
                        quote = if quote.is_some() { None } else { Some('"') };
                    }
                    literal_start = quote_index + 1;
                }
            } else if matches!(ch, '\'' | '"') && quote.is_none_or(|open| open == ch) {
                decoded
                    .get_or_insert_with(String::new)
                    .push_str(&input[literal_start..index]);
                quote = if quote.is_some() { None } else { Some(ch) };
                literal_start = index + 1;
            }
        }
        if quote.is_some() {
            return Err("Unclosed quote");
        }
        let end = chars.peek().map_or(input.len(), |(index, _)| *index);
        let value = decoded.map_or_else(
            || Cow::Borrowed(&input[start..end]),
            |mut value| {
                value.push_str(&input[literal_start..end]);
                Cow::Owned(value)
            },
        );
        tokens.push(Token {
            range: start..end,
            value,
        });
    }
    Ok(tokens)
}

pub(super) fn arguments(input: &str) -> Result<Vec<String>, &'static str> {
    Ok(tokens(input)?
        .into_iter()
        .map(|token| token.value.into_owned())
        .collect())
}

struct Flag {
    name: String,
    description: String,
    takes_value: bool,
    values: Vec<String>,
    visible: bool,
}

impl Flag {
    fn from_arg(arg: &Arg) -> Option<Self> {
        let long = arg.get_long()?;
        let values = if long == "type" {
            [
                "runtime",
                "toolchain",
                "sdk",
                "development-tool",
                "package-manager",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect()
        } else {
            arg.get_value_parser()
                .possible_values()
                .into_iter()
                .flatten()
                .filter(|value| !value.is_hide_set())
                .map(|value| value.get_name().to_owned())
                .collect()
        };
        Some(Self {
            name: format!("--{long}"),
            description: arg.get_help().map_or_else(String::new, ToString::to_string),
            takes_value: arg.get_action().takes_values(),
            values,
            visible: !arg.is_hide_set(),
        })
    }

    fn builtin(name: &str, description: &str) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            takes_value: false,
            values: Vec::new(),
            visible: true,
        }
    }
}

struct CommandSpec {
    name: String,
    description: String,
    flags: Vec<Flag>,
}

struct Grammar {
    commands: Vec<CommandSpec>,
    global_flags: Vec<Flag>,
    policy: Vec<(String, ArgMatches)>,
}

impl Grammar {
    fn load(vi: bool) -> &'static Self {
        static EN: OnceLock<Grammar> = OnceLock::new();
        static VI: OnceLock<Grammar> = OnceLock::new();
        let cache = if vi { &VI } else { &EN };
        cache.get_or_init(|| {
            let parser = super::super::parser(vi);
            let mut global_flags: Vec<_> =
                parser.get_arguments().filter_map(Flag::from_arg).collect();
            let policy = [
                "--all",
                "--type=runtime",
                "--installed",
                "--available",
                "--offline",
                "--locked",
                "--dry-run",
                "--yes",
                "--timeout-ms=1",
                "--scope=project",
            ]
            .into_iter()
            .filter_map(|arg| {
                let name = arg.split_once('=').map_or(arg, |(name, _)| name);
                parser
                    .clone()
                    .try_get_matches_from(["cnr", arg])
                    .ok()
                    .map(|matches| (name.to_owned(), matches))
            })
            .collect();
            global_flags.push(Flag::builtin(
                "--help",
                if vi { "Xem trợ giúp" } else { "Print help" },
            ));
            global_flags.push(Flag::builtin(
                "--version",
                if vi {
                    "Xem phiên bản CNR"
                } else {
                    "Print CNR version"
                },
            ));
            let mut commands: Vec<_> = parser
                .get_subcommands()
                .filter(|command| !matches!(command.get_name(), "tui" | "mcp"))
                .map(|command| CommandSpec {
                    name: command.get_name().into(),
                    description: command
                        .get_about()
                        .map_or_else(String::new, ToString::to_string),
                    flags: command.get_arguments().filter_map(Flag::from_arg).collect(),
                })
                .collect();
            for (name, labels) in [
                ("clear", ["Clear the current output", "Xóa đầu ra hiện tại"]),
                ("help", ["Show command help", "Xem trợ giúp về lệnh"]),
            ] {
                commands.push(CommandSpec {
                    name: name.into(),
                    description: labels[usize::from(vi)].into(),
                    flags: Vec::new(),
                });
            }
            Self {
                commands,
                global_flags,
                policy,
            }
        })
    }

    fn command(&self, name: &str) -> Option<&CommandSpec> {
        self.commands.iter().find(|command| command.name == name)
    }

    fn flags<'a>(&'a self, command: Option<&'a CommandSpec>) -> impl Iterator<Item = &'a Flag> {
        self.global_flags
            .iter()
            .filter(move |flag| command.is_none() || flag.name != "--version")
            .chain(command.into_iter().flat_map(|command| &command.flags))
    }

    fn flag<'a>(&'a self, command: Option<&'a CommandSpec>, name: &str) -> Option<&'a Flag> {
        self.flags(command).find(|flag| flag.name == name)
    }

    fn supports<'a>(
        &self,
        command: &str,
        words: &[String],
        flags: impl Iterator<Item = &'a str> + Clone,
    ) -> bool {
        if command == "clear" {
            return flags.count() == 0 && words.is_empty();
        }
        if flags.clone().any(|flag| flag == "--all") && flags.clone().any(|flag| flag == "--type") {
            return false;
        }
        if command == "audit"
            && words
                .first()
                .is_some_and(|word| matches!(word.as_str(), "list" | "tooling"))
            && flags.clone().any(|flag| flag == "--governance-root")
        {
            return false;
        }
        let maintenance = matches!(
            command,
            "workspace" | "task" | "governance" | "agent" | "ci"
        ) || command == "audit"
            && words
                .first()
                .is_some_and(|word| !matches!(word.as_str(), "list" | "tooling"));
        let permitted = |words: &[String]| {
            flags.clone().all(|flag| {
                self.policy
                    .iter()
                    .find(|(name, _)| name == flag)
                    .is_none_or(|(_, matches)| {
                        super::super::validate_flags(command, words, matches, maintenance).is_ok()
                    })
            })
        };
        // These options can precede the action. Validate the concrete action
        // as soon as it appears, including when it is the candidate.
        permitted(words) || (command == "env" && words.is_empty() && permitted(&["sync".into()]))
    }
}

#[derive(Default)]
struct Context<'g, 'i> {
    command: Option<&'g CommandSpec>,
    words: Vec<String>,
    used: Vec<&'g str>,
    pending: Option<&'g Flag>,
    project_override: bool,
    category: Option<&'i str>,
}

impl<'g, 'i> Context<'g, 'i> {
    fn from_prefix(grammar: &'g Grammar, tokens: &'i [Token<'_>]) -> Option<Self> {
        let mut context = Self::default();
        for (index, token) in tokens.iter().enumerate() {
            let value = token.value.as_ref();
            if index == 0 && value == "cnr" {
                continue;
            }
            if value == "--" || matches!(value, "-h" | "-V" | "--help" | "--version") {
                return None;
            }
            if let Some(flag) = context.pending.take() {
                context.read_value(flag, value)?;
            } else if value.starts_with('-') {
                let (name, inline) = value
                    .split_once('=')
                    .map_or((value, None), |(name, value)| (name, Some(value)));
                let flag = grammar.flag(context.command, name)?;
                if context.used.contains(&flag.name.as_str()) {
                    return None;
                }
                context.used.push(&flag.name);
                if let Some(inline) = inline {
                    if !flag.takes_value {
                        return None;
                    }
                    context.read_value(flag, inline)?;
                } else if flag.takes_value {
                    context.pending = Some(flag);
                }
            } else if context.command.is_none() {
                context.command = Some(grammar.command(value)?);
            } else {
                context.words.push(value.to_owned());
            }
        }
        Some(context)
    }

    fn read_value(&mut self, flag: &Flag, value: &'i str) -> Option<()> {
        if value.is_empty()
            || value.starts_with('-')
            || (!flag.values.is_empty() && !flag.values.iter().any(|choice| choice == value))
        {
            return None;
        }
        match flag.name.as_str() {
            "--project" => self.project_override = true,
            "--type" => self.category = Some(value),
            _ => {}
        }
        Some(())
    }

    fn allows(&self, grammar: &Grammar, next: Option<&str>) -> bool {
        self.command.is_none_or(|command| {
            grammar.supports(
                &command.name,
                &self.words,
                self.used.iter().copied().chain(next),
            )
        })
    }
}

struct Candidate<'a> {
    text: Cow<'a, str>,
    kind: Kind,
    description: &'a str,
    source: Source,
}

impl<'a> Candidate<'a> {
    fn cli(text: &'a str, kind: Kind, description: &'a str) -> Self {
        Self {
            text: Cow::Borrowed(text),
            kind,
            description,
            source: Source::Cli,
        }
    }
}

pub(super) struct Completer {
    grammar: &'static Grammar,
    tools: Vec<CompletionTool>,
    vi: bool,
}

impl Completer {
    pub(super) fn new(project: Option<&Path>, vi: bool) -> Self {
        Self {
            grammar: Grammar::load(vi),
            tools: completion_catalog(project),
            vi,
        }
    }

    pub(super) fn suggestions(&self, input: &str, cursor: usize) -> Vec<Suggestion> {
        if input.len() > MAX_INPUT || !input.is_char_boundary(cursor) {
            return Vec::new();
        }
        let Ok(tokens) = tokens(input) else {
            return Vec::new();
        };
        let index = tokens.partition_point(|token| token.range.end < cursor);
        let token = tokens
            .get(index)
            .filter(|token| token.range.start <= cursor);
        if token.is_some_and(|token| matches!(token.value, Cow::Owned(_))) {
            return Vec::new();
        }
        let mut range = token.map_or(cursor..cursor, |token| token.range.clone());
        let prefix = &input[range.start..cursor];
        let Some(context) = Context::from_prefix(self.grammar, &tokens[..index]) else {
            return Vec::new();
        };
        if !context.allows(self.grammar, None) {
            return Vec::new();
        }
        let mut candidates = Vec::new();
        if let Some(flag) = context.pending {
            candidates.extend(
                flag.values
                    .iter()
                    .map(|value| Candidate::cli(value, Kind::Value, &flag.description)),
            );
        } else if let Some((name, _)) = prefix
            .split_once('=')
            .filter(|(name, _)| name.starts_with("--"))
        {
            if let Some(flag) = self.grammar.flag(context.command, name)
                && !context.used.contains(&flag.name.as_str())
                && context.allows(self.grammar, Some(name))
            {
                range.start += name.len() + 1;
                candidates.extend(
                    flag.values
                        .iter()
                        .map(|value| Candidate::cli(value, Kind::Value, &flag.description)),
                );
            }
        } else if prefix.starts_with('-') {
            if &input[range.clone()] == "--"
                && context
                    .command
                    .is_some_and(|command| command.name == "exec")
            {
                return Vec::new();
            }
            candidates.extend(
                self.grammar
                    .flags(context.command)
                    .filter(|flag| {
                        flag.visible
                            && !context.used.contains(&flag.name.as_str())
                            && context.allows(self.grammar, Some(&flag.name))
                    })
                    .map(|flag| Candidate::cli(&flag.name, Kind::Option, &flag.description)),
            );
        } else if let Some(command) = context.command {
            self.actions(&context, command, prefix, &mut candidates);
        } else {
            candidates.extend(
                self.grammar
                    .commands
                    .iter()
                    .filter(|command| {
                        self.grammar
                            .supports(&command.name, &[], context.used.iter().copied())
                    })
                    .map(|command| {
                        Candidate::cli(&command.name, Kind::Command, &command.description)
                    }),
            );
        }
        rank(candidates, input, cursor, range, &context)
    }

    fn actions<'a>(
        &'a self,
        context: &Context<'_, '_>,
        command: &'a CommandSpec,
        prefix: &str,
        result: &mut Vec<Candidate<'a>>,
    ) {
        if command.name == "help" && context.words.is_empty() {
            result.extend(
                self.grammar
                    .commands
                    .iter()
                    .filter(|command| !matches!(command.name.as_str(), "clear" | "help"))
                    .map(|command| {
                        Candidate::cli(&command.name, Kind::Command, &command.description)
                    }),
            );
        }
        let words: Vec<_> = context.words.iter().map(String::as_str).collect();
        let usage = super::super::usage_notes(&command.name, self.vi)
            .lines()
            .next()
            .unwrap_or_default();
        let mut action_words = context.words.clone();
        for &action in command_words(&command.name, &words) {
            action_words.push(action.into());
            if self
                .grammar
                .supports(&command.name, &action_words, context.used.iter().copied())
            {
                let detail = action_description(&command.name, action, self.vi).unwrap_or(usage);
                result.push(Candidate::cli(action, Kind::Subcommand, detail));
            }
            action_words.pop();
        }

        if !words.is_empty() || context.used.contains(&"--all") {
            return;
        }
        let name = command.name.as_str();
        let managed = matches!(name, "install" | "update" | "remove");
        if managed || matches!(name, "search" | "info" | "use" | "pin") {
            self.tool_candidates(context, name, managed, prefix, result);
        }
    }

    fn tool_candidates<'a>(
        &'a self,
        context: &Context<'_, '_>,
        name: &str,
        managed: bool,
        prefix: &str,
        result: &mut Vec<Candidate<'a>>,
    ) {
        let tools = self.tools.iter().filter(|tool| {
            (!managed || tool.managed)
                && context
                    .category
                    .is_none_or(|category| tool.kind == category)
        });
        if let Some((tool_id, version_prefix)) = prefix.split_once('@') {
            if context.project_override || name == "search" {
                return;
            }
            let Some(tool) = tools.into_iter().find(|tool| tool.id == tool_id) else {
                return;
            };
            let leading_v = if version_prefix.starts_with('v') {
                "v"
            } else {
                ""
            };
            let version_prefix = version_prefix.strip_prefix('v').unwrap_or(version_prefix);
            for version in &tool.versions {
                if !version.version.starts_with(version_prefix)
                    || (name == "remove" && !version.installed)
                {
                    continue;
                }
                let source = if version.pinned {
                    Source::Pinned
                } else {
                    Source::Installed
                };
                let description = if version.pinned && version.installed {
                    if self.vi {
                        "Đã ghim và đã cài trong dự án"
                    } else {
                        "Pinned and installed in this project"
                    }
                } else {
                    source.label(self.vi)
                };
                result.push(Candidate {
                    text: Cow::Owned(format!("{}@{leading_v}{}", tool.id, version.version)),
                    kind: Kind::Version,
                    description,
                    source,
                });
            }
        } else {
            result.extend(tools.map(|tool| Candidate {
                text: Cow::Borrowed(tool.id),
                kind: Kind::Tool,
                description: tool.label,
                source: Source::Catalog,
            }));
        }
    }
}

fn rank(
    candidates: Vec<Candidate<'_>>,
    input: &str,
    cursor: usize,
    range: Range<usize>,
    context: &Context<'_, '_>,
) -> Vec<Suggestion> {
    let prefix = &input[range.start..cursor];
    let original = &input[range.clone()];
    let has_prefix = candidates
        .iter()
        .any(|candidate| candidate.text.starts_with(prefix));
    let mut candidates: Vec<_> = candidates
        .into_iter()
        .filter_map(|candidate| {
            if candidate.text == original {
                return None;
            }
            let score = if candidate.text.starts_with(prefix) {
                0
            } else if !has_prefix
                && matches!(
                    candidate.kind,
                    Kind::Command | Kind::Subcommand | Kind::Tool
                )
            {
                fuzzy_score(&candidate.text, prefix)?
            } else {
                return None;
            };
            Some((score, candidate))
        })
        .collect();
    candidates.sort_by(|(left_score, left), (right_score, right)| {
        left_score
            .cmp(right_score)
            .then_with(|| (right.source == Source::Pinned).cmp(&(left.source == Source::Pinned)))
            .then_with(|| {
                if left.kind == Kind::Version && right.kind == Kind::Version {
                    std::cmp::Ordering::Equal
                } else {
                    left.text.cmp(&right.text)
                }
            })
    });
    candidates.dedup_by(|(_, left), (_, right)| left.text == right.text);
    candidates
        .into_iter()
        .take(LIMIT)
        .map(|(_, candidate)| {
            let text = candidate.text.into_owned();
            let label = if candidate.kind == Kind::Subcommand
                && let Some(command) = context.command
            {
                let mut label = command.name.clone();
                for word in &context.words {
                    label.push(' ');
                    label.push_str(word);
                }
                label.push(' ');
                label.push_str(&text);
                label
            } else {
                text.clone()
            };
            Suggestion {
                label,
                text,
                insert_range: range.clone(),
                kind: candidate.kind,
                description: candidate.description.into(),
                source: candidate.source,
            }
        })
        .collect()
}

fn action_description(command: &str, action: &str, vi: bool) -> Option<&'static str> {
    let labels = match (command, action) {
        ("env", "plan") => [
            "Review requirements, version pins and conflicts",
            "Xem yêu cầu, phiên bản đã ghim và xung đột",
        ],
        ("env", "lock") => ["Write the environment lock file", "Ghi tệp khóa môi trường"],
        ("env", "profiles") => ["List environment profiles", "Liệt kê hồ sơ môi trường"],
        ("env", "sync") => [
            "Sync the selected project tools",
            "Đồng bộ công cụ đã chọn của dự án",
        ],
        ("env", "export") => ["Export the environment", "Xuất môi trường"],
        ("env", "import") => [
            "Import an environment from a file",
            "Nhập môi trường từ tệp",
        ],
        _ => return None,
    };
    Some(labels[usize::from(vi)])
}

fn fuzzy_score(candidate: &str, prefix: &str) -> Option<usize> {
    if !(2..=24).contains(&prefix.len())
        || !prefix
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return None;
    }
    let mut suffix = candidate.as_bytes();
    let mut gap = 0;
    for byte in prefix.bytes() {
        let index = suffix
            .iter()
            .position(|candidate| candidate.eq_ignore_ascii_case(&byte))?;
        gap += index;
        suffix = &suffix[index + 1..];
    }
    (gap <= 8).then_some(1 + gap)
}

// Some CLI groups accept positional action words rather than Clap subcommands.

fn command_words(command: &str, preceding: &[&str]) -> &'static [&'static str] {
    match (command, preceding) {
        ("env", []) => &["plan", "lock", "sync", "profiles", "export", "import"],
        ("config", []) => &["show", "get", "set", "unset"],
        ("venv", []) => &["list", "info", "create", "run", "remove"],
        ("workspace", []) => &["discover", "bind", "health", "binding"],
        ("workspace", ["binding"]) => &["list", "add", "show", "validate"],
        ("task", []) => &[
            "create",
            "show",
            "resume",
            "revise",
            "context",
            "bootstrap",
            "diff-check",
            "verify",
            "review",
            "accept",
            "close",
            "claim",
            "renew",
            "release",
            "report",
            "status",
            "recover",
            "session-prepare",
            "dependency",
            "evidence",
        ],
        ("task", ["dependency"]) => &["check"],
        ("task", ["evidence"]) => &["export", "inspect"],
        ("governance", []) => &["validate", "inspect", "context", "migrate"],
        ("governance", ["migrate"]) => &["inspect", "export", "validate", "import", "rollback"],
        ("agent", []) => &["list", "export", "check", "mcp-config", "sync", "rollback"],
        ("agent", ["mcp-config"]) => &["export", "check"],
        ("audit", []) => &["tooling", "task", "migration", "agent", "list"],
        ("ci", []) => &["verify"],
        ("completion", []) => &["bash", "zsh", "fish", "powershell"],
        ("build" | "run" | "test", []) => &["rust", "node", "python", "cmake"],
        ("package", []) => &["rust", "node", "python"],
        ("package", ["rust"]) => &["list", "add", "remove", "sync"],
        ("package", ["node" | "python"]) => &["list", "add", "remove", "sync", "outdated"],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cirnetrules::tooling::CompletionVersion;

    fn choices(completer: &Completer, input: &str) -> Vec<String> {
        completer
            .suggestions(input, input.len())
            .into_iter()
            .map(|candidate| candidate.text)
            .collect()
    }

    #[test]
    fn completes_the_supported_grammar_and_catalog_in_context() {
        let completer = Completer::new(None, false);
        assert!(
            !choices(&completer, "audit tooling --governance-r")
                .iter()
                .any(|flag| flag == "--governance-root")
        );
        for (input, expected) in [
            ("l", &["list"][..]),
            ("i", &["info", "init", "install"]),
            ("cnr se", &["search"]),
            ("env pl", &["plan"]),
            ("workspace b", &["bind", "binding"]),
            ("task ev", &["evidence"]),
            ("task evidence ex", &["export"]),
            ("--dr", &["--dry-run"]),
            ("--ver", &["--version"]),
            ("--scope ", &["project", "system", "user"]),
            ("--lang ", &["en", "vi"]),
            ("--lang=v", &["vi"]),
            ("--scope project lis", &["list"]),
            ("install ru", &["rust"]),
            ("package node out", &["outdated"]),
        ] {
            assert_eq!(choices(&completer, input), expected, "{input:?}");
        }
        for input in [
            "config show --ins",
            "env --ver",
            "workspace discover --scope ",
            "env --offline pl",
            "env plan ",
            "--project en",
            "update --all ru",
            "package rust out",
            "exec --",
            "exec -- git",
            "mcp se",
            "tui ",
        ] {
            assert!(choices(&completer, input).is_empty(), "{input:?}");
        }
    }

    #[test]
    fn quoted_windows_paths_and_unicode_keep_the_suffix_outside_the_replacement() {
        let completer = Completer::new(None, false);
        let input = "--project \"C:\\My Project Việt\"\u{3000}cnr";
        assert!(choices(&completer, input).is_empty());

        let before = "--project \"C:\\My Project Việt\"\u{3000}env pl";
        let input = format!("{before}n --json");
        let suggestions = completer.suggestions(&input, before.len());
        assert_eq!(suggestions.len(), 1);
        let candidate = &suggestions[0];
        assert_eq!(&input[candidate.insert_range.clone()], "pln");
        assert_eq!(candidate.text, "plan");
        let mut accepted = input;
        accepted.replace_range(candidate.insert_range.clone(), &candidate.text);
        assert_eq!(
            accepted,
            "--project \"C:\\My Project Việt\"\u{3000}env plan --json"
        );
        assert!(completer.suggestions("cnr\u{3000}lis", 4).is_empty());
        assert!(completer.suggestions("exec -- git", 6).is_empty());
        assert!(choices(&completer, "env \"unfinished").is_empty());
        assert!(choices(&completer, "\"lis\"").is_empty());
    }

    #[test]
    fn arguments_preserve_windows_backslashes_and_decode_escaped_quotes() {
        let line = r#"exec -- "C:\My Project Việt\run.exe" "\\server\share" "C:\path\\" "quote \"inside\"" 'C:\literal\' "" bare\"quote"#;
        assert_eq!(
            arguments(line).unwrap(),
            [
                "exec",
                "--",
                r"C:\My Project Việt\run.exe",
                r"\\server\share",
                "C:\\path\\",
                "quote \"inside\"",
                "C:\\literal\\",
                "",
                "bare\"quote",
            ]
        );
        assert!(arguments(r#"exec -- "not closed\""#).is_err());
    }

    #[test]
    fn versions_report_only_snapshot_pins_or_verified_installations() {
        let mut completer = Completer::new(None, false);
        assert!(choices(&completer, "install rust@").is_empty());
        let rust = completer
            .tools
            .iter_mut()
            .find(|tool| tool.id == "rust")
            .unwrap();
        rust.managed = true;
        rust.versions = vec![
            CompletionVersion {
                version: "1.99.1".into(),
                pinned: true,
                installed: false,
            },
            CompletionVersion {
                version: "1.100.0".into(),
                pinned: false,
                installed: true,
            },
        ];
        let input = "install rust@";
        let suggestions = completer.suggestions(input, input.len());
        assert_eq!(
            suggestions
                .iter()
                .map(|item| (item.text.as_str(), item.source))
                .collect::<Vec<_>>(),
            [
                ("rust@1.99.1", Source::Pinned),
                ("rust@1.100.0", Source::Installed)
            ]
        );
        assert_eq!(choices(&completer, "remove rust@"), ["rust@1.100.0"]);
        assert_eq!(choices(&completer, "install rust@v1.99"), ["rust@v1.99.1"]);
        assert!(choices(&completer, r#"--project "C:\Other Project" install rust@"#).is_empty());
    }

    #[test]
    fn limits_and_ranks_fuzzy_matches_below_prefixes() {
        let completer = Completer::new(None, false);
        assert_eq!(choices(&completer, "ls"), ["list"]);
        assert_eq!(choices(&completer, "list"), Vec::<String>::new());
        let all = choices(&completer, "");
        assert_eq!(all.len(), LIMIT);
        assert!(all.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(!all.iter().any(|item| item == "tui" || item == "mcp"));
    }
}
