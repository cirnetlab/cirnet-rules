<!-- doc-id: CGACP-DESKTOP-OPERATOR -->
<!-- revision: 5 -->
<a id="operator"></a>

# CGACP Dashboard

The product generation is CGACP v1. The application version comes from Cargo.
The local candidate release title is `cirnet-rules-0.1.0`. Release review is
separate from everyday repository, conversation and automation operations.

## Build and run

From the repository root, run `run-dashboard.bat` on Windows or
`sh ./run-dashboard.sh` on Linux. Each launcher builds the Rust runtime and
Qt desktop from the current source, then opens the dashboard. Pass application
options after the launcher command.

The Windows launcher requires Visual Studio 2022 with C++ build tools. On
Linux, CMake must be able to find a C++ compiler and Qt 6 Widgets and
LinguistTools development packages.

Use the pinned Rust toolchain and a Qt 6 Widgets development kit for the local
C++ compiler. Build `cgacp-runtime` with Cargo, then configure CMake with
`CGACP_RUNTIME_EXECUTABLE` set to the resulting binary. Build
`cgacp-dashboard` and run CTest. Install to a separate staging directory with
`CGACP_DEPLOY_QT` enabled for a Windows distribution candidate. Launch the
installed Dashboard from its bin directory. Use `--data-dir` with a dedicated
absolute test directory for disposable verification.

## Operator workflow

Drag the sidebar divider to resize it. Use its chevron or Ctrl+B to collapse
or expand it. Open Settings from the footer or with Ctrl+, and select a
category on the left. Appearance provides theme previews, density, text
size and Reduced motion. Back to workspace or Escape returns to the previous
page. Reset layout restores sidebar and repository table presentation. Toasts
stack up from the lower right and leave the composer clear. Use Notifications
to review or clear read messages from this session. Warnings and errors remain
eligible for a toast.

In Settings → Repositories, add a folder or file and review the resolved Git
root, branch, worktree and file hint before saving. Use Select for task or
choose a binding in New Chat or Add Automation. Actions can check health,
copy the full path, open the verified local folder, rename, relink or archive.
The table can shorten long paths for display. Copy uses the stored path.
Use Relink when a saved checkout has moved. It invalidates older binding
revisions and affected work needs review. Archive preserves historical
references and source files.

New Chat opens a conversation. Ctrl+Enter sends the message and creates a
scoped intake proposal. Switch conversations in the sidebar or use Back from
Settings to continue your draft. Scroll to the end of the conversation list
to load older conversations, or choose Load more from the list context menu
while another page is available. Unsent drafts live in the open application.
The handoff profile shows waiting approval. View committed messages after
reopening. Automations provide manual runs and a daily IANA timezone
schedule. Use the preview before enabling and confirm an inactive one-shot.
The Rust runtime owns the schedule. To keep scheduling while the Dashboard is
closed, run `cgacp-runtime --scheduler` in a separate foreground terminal
with the same data directory. Scheduler state remains durable during downtime.
Missed daily occurrences coalesce. Paused or stale definitions cannot start a
new proposal silently.

## Storage and operations

Rust owns the SQLite database in the local application data directory or the
explicit operator directory. Opening a schema 3 store with the new runtime
creates an integrity-checked pre-upgrade snapshot before migrating to schema 4.
An earlier runtime requires a reviewed snapshot in a separate directory.
Use `storage.backup` through the local runtime
for an online snapshot and `storage.verify` to check it. To rehearse restore,
close the runtime, restore a reviewed backup into a fresh private data
directory and verify it there before switching the operator path. Preserve the
original directory and its WAL files. Backups are local sensitive data.

The legacy import can preview and archive a stopped CLI file store using
`legacy.preview` and `legacy.apply`. Apply requires the reviewed manifest
digest and confirmation that legacy writers are stopped. Inspect archived
records and link an old alias to a separately reviewed live binding with
`legacy.inspect` and `legacy.link`. Import leaves the source intact and
does not reactivate grants, leases or old runs.

Run `scripts/package-dashboard.ps1` against an installed Windows staging tree
to record artifact SHA-256 and source provenance. The candidate remains
`RELEASE_BLOCKED` until native Agent, isolation, authenticated review,
clean Windows and redistribution gates are evidenced.

[Architecture and limitations](dashboard-architecture.md) · [Interaction states](desktop-interaction.md) · [Release candidate](dashboard-release.md) · [Task workflow](cgacp-local.md)
