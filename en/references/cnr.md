<!-- doc-id: CNR-OPERATOR -->
<!-- revision: 10 -->
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
commands and Ctrl+R searches that history by prefix. Esc first dismisses an
open popup or text selection. It can then cancel a running job or return to
the menu from idle results.

<a id="tui-editing"></a>

### Text editing and clipboard

These shortcuts apply in Command, search and setup fields.

| Shortcut | Effect |
| --- | --- |
| Shift+Left/Right, Shift+Home/End | Extend the selection by displayed characters or to an edge |
| Ctrl+A | Select the field text |
| Ctrl+C | Copy the selection. With no selection, cancel a running job, go back in setup or exit CNR as appropriate. |
| Ctrl+X, Delete, Backspace | Cut or remove the selection |
| Ctrl+V | Paste at the caret, replacing any selection |
| Ctrl+Z, Ctrl+Y or Ctrl+Shift+Z | Undo or redo a text edit |
| Home, End or Ctrl+E | Move to a field edge |
| Shift+F10 or right-click | Open the context menu when the terminal forwards the event |

Click to place the caret and drag to select text when the terminal forwards
mouse events. In the result pane, drag to select readable text or use the
context menu to copy a line, a finding, a value or the readable report.
Ctrl+A selects visible text when the result pane has focus. F6 moves between
the generic output pane and editor. The result pane is read-only. Raw JSON
is a separate inspection mode outside CNR clipboard actions.
Paste edits text without submitting a command or accepting a setup preview.
Undo and redo apply to text edits.

| Terminal host | Clipboard and selection |
| --- | --- |
| Windows Terminal | CNR uses the Windows clipboard on explicit requests. When the host handles a gesture, use Shift+drag for host selection while CNR mouse capture is active, then use the terminal's copy or paste action. Host right-click behavior depends on its settings. |
| Windows Console Host | Native clipboard access requires an available Windows desktop. Mouse events and shortcuts depend on console input mode and host settings. Host selection and paste can be used when available. |
| Linux, macOS, WSL, or detected SSH | Use the terminal's own copy and paste. CNR reports `CLIPBOARD_UNSUPPORTED` for native clipboard actions in these sessions. |

Native clipboard access can report `CLIPBOARD_UNAVAILABLE` for a busy
clipboard or missing text and `CLIPBOARD_PERMISSION_DENIED` for denied access.

[Windows Terminal selection](https://learn.microsoft.com/en-us/windows/terminal/selection) ·
[Interaction settings](https://learn.microsoft.com/en-us/windows/terminal/customize-settings/interaction) ·
[Terminal actions](https://learn.microsoft.com/en-us/windows/terminal/customize-settings/actions)

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

<a id="system-checks"></a>

## System checks

Open `cnr` and choose System checks, or enter `cnr check all` in the command
panel. An individual target such as `cnr check bindings` limits inspection to
that component. Headless `cnr check` defaults to `all`. Use `--repo`,
`--binding`, `--task` and `--agent` to narrow the evidence. Check output can
distinguish a discovered checkout from a confirmed binding and a verified MCP
protocol exchange from a native Agent session. Review the reason and next
action for every required item that lacks evidence.

The summary keeps counts for all statuses. The list shows issues first and
groups pending and passed findings below them. Use Up/Down to select,
Enter to expand a group or open one finding, and Esc to return. Use `/`
or Ctrl+F to search, `f` to filter by status, `r` to filter by repository and
`a` to show all. `b` selects blocked findings. `j` opens raw JSON. On wide
terminals, Tab moves between the list and details. Details include evidence,
source and the available Next Actions. Review write actions in the relevant
setup flow with the required authorization.

Use `cnr check all --json --non-interactive` for a versioned report. Exit 1
indicates an error, exit 2 a mandatory gap or a warning in strict mode, and
exit 130 a cancellation. Exit 0 means the selected required checks passed and
can coexist with optional gaps. Checks inspect existing state. Setup actions
handle binding, exports, synchronization and registration separately.

<a id="guided-agent-setup"></a>

## Guided Agent setup

From `cnr`, choose Agent setup or enter `cnr setup agent`. Select Codex,
Claude Code or Antigravity, then review the Governance root, workspace,
repository and binding. A discovered checkout needs a reviewed binding.
If several bindings exist, choose one explicitly. Headless `cnr setup agent`
requires `--agent` and reports the selected context.

Preview and confirm an instruction bundle in a fresh directory outside Git
and native Agent configuration. The manifest lists the selected profile,
skills and references. For headless use, `cnr agent export --dry-run` accepts
the export selectors and an output path. The export can then take the reviewed
`--expect` digest. A bundle exported with `--binding` pins the selected binding
and its Git HEAD. Pass the same binding to `agent check` or `agent sync` when
you want to select it explicitly.

Review the synchronization plan against the actual worktree. If a user-owned
`AGENTS.md` or `CLAUDE.md` blocks strict sync, select the managed-block preview.
It shows the proposed managed content and a preimage digest without printing
existing user text. Review the new instructions alongside local instructions
before approval. The first adoption appends a separate block using the first
observed line ending (LF when no line ending exists). It preserves an existing
UTF-8 BOM and all prior bytes. A previously verified full-file receipt can be
converted explicitly. Updates change only the verified managed block, and
malformed or edited markers require local review. Skills and references retain
strict file ownership. Codex and Antigravity share a single `AGENTS.md` block
and deduplicate the shared `.agents/skills` paths when content matches.

Select a current task and its declared operator before confirming a write.
The managed preview binds the task revision, exact target, bundle and preimage.
For headless use, include `--entry-mode managed-block --dry-run --task` with
the task ID and `--actor` with the operator, then apply the reviewed digest
using `--yes --expect`. Use `agent rollback` with a receipt and review the
rollback preview. A changed entry after sync blocks rollback and preserves its
current bytes for local reconciliation. Review `cnr audit agent --receipt`
with the receipt ID before recovering an interrupted operation.

Export the MCP launch bundle, then review the project registration or follow
the manual import instructions shown for the selected Agent and surface.
Run the protocol check in CNR. Native skill discovery, configuration loading
and tool use require separate observations in the selected Agent session.

<a id="mcp-coordination"></a>

## MCP and shared coordination

Use [MCP STDIO](mcp.md) for Agent configuration and the tool profiles.
The [coordination workflow](shared-task-coordination.md) covers named
bindings, leases, reports, local audit and the portable candidate gate.
