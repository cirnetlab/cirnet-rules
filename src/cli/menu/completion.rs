/// Returns whole edited lines, leaving text outside the current word unchanged.
/// Quoted input is left alone rather than risking changes to a shell argument.
pub(super) fn suggestions(input: &str, cursor: usize) -> Vec<String> {
    if cursor > input.len()
        || !input.is_char_boundary(cursor)
        || input[..cursor].contains(['\'', '"'])
    {
        return Vec::new();
    }
    let start = input[..cursor]
        .char_indices()
        .rev()
        .find(|(_, ch)| ch.is_whitespace())
        .map_or(0, |(index, ch)| index + ch.len_utf8());
    let end = input[cursor..]
        .find(char::is_whitespace)
        .map_or(input.len(), |index| cursor + index);
    let prefix = &input[start..cursor];
    let mut previous: Vec<_> = input[..start].split_whitespace().collect();
    if previous.first() == Some(&"cnr") {
        previous.remove(0);
    }
    if previous.contains(&"--") {
        return Vec::new();
    }

    let parser = super::super::parser(false);
    let mut command_index = None;
    let mut index = 0;
    while index < previous.len() {
        let word = previous[index];
        if word.starts_with('-') {
            index += 1 + usize::from(option_takes_value(word));
        } else {
            command_index = Some(index);
            break;
        }
    }
    let command = command_index.and_then(|index| {
        parser
            .get_subcommands()
            .find(|sub| sub.get_name() == previous[index])
    });
    let names: Vec<String> = if matches!(previous.last(), Some(&"--scope")) {
        ["project", "user", "system"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    } else if matches!(previous.last(), Some(&"--lang")) {
        ["en", "vi"].into_iter().map(str::to_owned).collect()
    } else if previous.last().is_some_and(|word| option_takes_value(word)) {
        Vec::new()
    } else if prefix.starts_with('-') {
        parser
            .get_arguments()
            .chain(command.into_iter().flat_map(|sub| sub.get_arguments()))
            .filter_map(|arg| arg.get_long().map(|long| format!("--{long}")))
            .chain(["--help".to_owned(), "--version".to_owned()])
            .collect()
    } else if let Some(command) = command {
        let words = &previous[command_index.expect("matched command") + 1..];
        command_words(command.get_name(), words)
            .iter()
            .map(|word| (*word).to_owned())
            .collect()
    } else if command_index.is_none() {
        parser
            .get_subcommands()
            .filter(|sub| !matches!(sub.get_name(), "tui" | "mcp"))
            .map(|sub| sub.get_name().to_owned())
            .chain(["clear".to_owned(), "help".to_owned()])
            .collect()
    } else {
        Vec::new()
    };
    let mut result: Vec<_> = names
        .into_iter()
        .filter(|name| name.starts_with(prefix))
        .map(|name| format!("{}{name}{}", &input[..start], &input[end..]))
        .collect();
    result.sort_unstable();
    result.dedup();
    result
}

fn option_takes_value(option: &str) -> bool {
    matches!(
        option,
        "--project" | "--scope" | "--type" | "--lang" | "--timeout-ms"
    ) || super::super::GOVERNANCE_OPTIONS
        .iter()
        .any(|name| option.strip_prefix("--") == Some(name))
}

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

    #[test]
    fn completes_catalog_commands_and_preserves_arguments_after_cursor() {
        assert!(suggestions("cnr env pl --json", 10).contains(&"cnr env plan --json".into()));
        assert!(suggestions("lis", 3).contains(&"list".into()));
    }

    #[test]
    fn completes_options_and_known_values_without_mutating_unknown_input() {
        assert!(suggestions("env --dry", 9).contains(&"env --dry-run".into()));
        assert!(suggestions("--scope sy", 10).contains(&"--scope system".into()));
        assert!(suggestions("env \"unfinished", 15).is_empty());
        assert!(suggestions("env plan ", 9).is_empty());
        assert!(suggestions("x", 2).is_empty());
        assert!(suggestions("exec -- git", 11).is_empty());
        assert!(suggestions("--project en", 12).is_empty());
    }

    #[test]
    fn handles_unicode_boundaries_and_global_options_before_command() {
        let line = "cnr\u{3000}lis --json";
        assert!(
            suggestions(line, "cnr\u{3000}lis".len()).contains(&"cnr\u{3000}list --json".into())
        );
        assert!(suggestions("cnr\u{3000}lis", 4).is_empty());
        assert!(suggestions("--scope project lis", 19).contains(&"--scope project list".into()));
    }

    #[test]
    fn rust_package_actions_exclude_unsupported_outdated() {
        assert!(!suggestions("package rust out", 16).contains(&"package rust outdated".into()));
        assert!(suggestions("package node out", 16).contains(&"package node outdated".into()));
    }

    #[test]
    fn excludes_commands_unavailable_inside_the_terminal() {
        assert!(!suggestions("tu", 2).contains(&"tui".into()));
        assert!(!suggestions("mc", 2).contains(&"mcp".into()));
        assert!(!suggestions("mcp se", 6).contains(&"mcp serve".into()));
    }
}
