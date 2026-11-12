<!-- doc-id: CNR-OPERATOR -->
<!-- revision: 8 -->
<a id="cnr-cli"></a>

# CNR operator guide

CNR is the CIRNET RULES CLI for governance, environments and development tools.
Build it with the repository's pinned Rust and Cargo:

~~~sh
cargo build --locked --bin cnr
./target/debug/cnr --help
./target/debug/cnr --project . env plan --json
~~~

On Windows use `target/debug/cnr.exe`. Run `cnr` or `cnr tui` in an
interactive terminal. The wide layout shows the selected command beside
the list. `/` or Ctrl+K searches the menu. `?` opens contextual help.

Type to open Command or press `:` to recover an unsent draft. Suggestions
open below the input. Wide menus place Command in the preview column.
Smaller terminals stack it below the menu and reduce the suggestions.
Tab accepts and cycles through suggestions. Shift+Tab cycles backward.
Enter submits the text in Command. In the main menu, Esc closes Command
and an arrow key closes it and moves the selection. Version suggestions
use project pins and verified managed installations.

The menu keeps its shortcut bar at the bottom. Results show the outcome,
exit code and elapsed milliseconds above Command. In results, PgUp/PgDn
and the mouse wheel scroll the output.
Ctrl+Home and Ctrl+End go to the start and end. Ctrl+F searches the output.
F3 and Shift+F3 move between matches. Up and Down recall recent read-only
commands and Ctrl+R searches that history by prefix. Ctrl+C cancels a
running command or exits while idle. Esc returns to the menu from idle results.

`--lang vi` selects Vietnamese terminal labels and command summaries.
`--no-banner` selects the compact title. `NO_COLOR` shows plain text and
the wordmark without its colored shadow. A noninteractive launch prints
help. The version, help and catalog can run independently of a project.
The executable source lives under `src/cli/`
and its integration tests live in `src/tests/cnr.rs`.

<a id="environment"></a>

## Project configuration

- `cnr init` creates `cnr.toml` while retaining any existing configuration.
  Resolve a directory with `--project PATH` or the nearest project marker.
- `cnr pin rust@1.99.1` sets a project pin. `cnr use rust@1.99.1` selects
  it in the active profile. Inspect and edit with `config show`,
  `config get KEY`, `config set KEY VALUE` or `config unset KEY`. A removed pin
  stays protected by an older lock until the lock is refreshed. Set keys such as
  `profile`, `tools.rust` or `profiles.dev.rust`.
- `cnr env plan` lists native requirements, resolved versions, and
  conflicts or unresolved pins. `cnr env lock` records exact versions and
  native manifest digests in `cnr.lock`. `--locked` requires a matching
  lock and checks for drift. Native dependency lockfiles remain authoritative.
- `cnr env sync --dry-run` plans the exact selected versions. Add
  `--yes` to confirm the project-scoped plan after review. Completed
  steps can be reused if a later step fails. Use `env profiles`,
  `env export --json` and `env import PATH` to exchange project configuration.
  Store the exported JSON separately from the tool installations.

<a id="tool-providers"></a>

## Providers and ownership

`search QUERY`, `list` and `info TOOL` read the same catalog. Filter with
`--type runtime|toolchain|sdk|package-manager|development-tool`,
`--installed` or `--available`. Available refers to the local catalog.
`doctor` reports local prerequisites and project environment status.

Rust, Python and Node use an existing native `rustup`, `uv` or `fnm`
respectively. An installation uses an exact release and project-local
storage. Review `cnr install TOOL@VERSION --dry-run`, then explicitly
confirm it with `--yes`. `cnr update` previews; an explicit
`update TOOL@VERSION` selects a single target. `update --all` selects
the exact versions in the current environment. `remove TOOL[@VERSION]`
checks ownership, project references and active use. User and system scope
changes require a separately authorized installer.

Catalog entries report local discovery and capability status. Use their approved installers when a native
mutation is unavailable. `--offline` limits managed installations to
already satisfied operations. Use `cache` and `audit` for local state and
receipts. A failed external installer may leave a recovery record; rerun
the same reviewed operation or use the recovery information it reports.

<a id="execution"></a>

## Native execution and governance

Use `build`, `run` and `test` with a supported engine, for example
`cnr build rust --locked --offline --dry-run`. Use `package ENGINE ACTION`
for supported native dependency operations, and `exec -- PROGRAM ARG...`
for an explicit executable. For changes and execution, review the plan
and confirm the operation with `--yes`. Arguments after the delimiter
are passed as values, with platform and policy validation.

Use `venv create NAME [VERSION]`, `venv list`, `venv info NAME`,
`venv run NAME -- ARG...` and `venv remove NAME`. Runs use the environment's
interpreter in a child process. A virtual environment stays bound to its
project and interpreter.

Native dependency options and deadlines apply only where the selected
engine can enforce them. Build hooks, package lifecycle scripts and
explicit commands can act with the local process's permissions. The
process harness bounds captured output and owns cancellation and cleanup.

Use `cnr workspace discover` and `cnr task show --task ID` for governance.
`--governance-root PATH` selects the authority checkout. The compatibility
delimiter still accepts maintenance options such as `cnr task show -- --task ID`.
For maintenance commands `--project PATH` selects the product workspace.
See the [operator workflow](cirnetrules-local.md) for grants and verification.
Release requires separate authorization.

`--json` returns one JSON document on stdout. Native process output is
bounded and sent separately to stderr. Exit 0 means success, 1 means
failure, 2 means blocked or unrun, and 130 means cancelled.
`child_exit_code` reports the child separately. `cnr completion SHELL` emits completion for
Bash, Zsh, Fish or PowerShell.

<a id="mcp-coordination"></a>

## MCP and shared coordination

Use [MCP STDIO](mcp.md) for Agent configuration and the tool profiles.
The [coordination workflow](shared-task-coordination.md) covers named
bindings, leases, reports, local audit and the portable candidate gate.
