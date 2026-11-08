<!-- doc-id: CGACP-DESKTOP-OPERATOR -->
<!-- revision: 9 -->
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

Drag the sidebar divider to resize it. Use the panel button or Ctrl+B to
collapse or expand it. Activate Local in the footer to see the current
connection status. Open Settings from the footer or with Ctrl+, and select a
category on the left. Appearance provides theme previews, density, text
size and Reduced motion. Back to workspace or Escape returns to the previous
page. Reset layout restores sidebar and repository table presentation. Toasts
appear in the upper right in Conversation and the lower right elsewhere.
Visible toasts follow the current page, leaving the composer clear. The glyph and
color identify the notification severity. An automation run queued by the
runtime displays an informational toast. Use Notifications to review or clear
read messages from this session. Warnings and errors remain eligible for a toast.

In Settings → Repositories, add a folder or file and review the resolved Git
root, branch, worktree and file hint before saving. Use Select for task or
choose a binding in New Chat or Add Automation. Actions can check health,
copy the full path, open the verified local folder, rename, relink or archive.
Repositories and Automation Run history show full-width row highlighting.
In Repositories, click a row to select it, including the space after the
last column. The table can shorten long paths for display. Copy uses the stored path.
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
explicit operator directory. Before upgrading an existing store to schema 6,
the runtime creates an integrity-checked pre-upgrade snapshot. An earlier
runtime requires a reviewed compatible snapshot in a separate directory.
In Settings → Storage, Scan measures only the local data root. Physical files,
logical message bytes and reusable SQLite pages describe different quantities.
Create and verify an online backup from Storage. Compact requires a current
scan and confirmation of the observed page counts. Operations report progress
and support cancellation. The app has no managed worktree cleanup capability.
To rehearse restore,
close the runtime, restore a reviewed backup into a fresh private data
directory and verify it there before switching the operator path. Preserve the
original directory and its WAL files. Backups are local sensitive data.

## Conversation lifecycle and Settings

Clear draft affects unsent text and offers Undo clear in the same session.
Use the transcript context menu to delete a submitted message, or Actions
to delete its conversation. Review the stable IDs, local scope and blockers
before confirming. Cancel or resolve active tasks and runs separately before
retrying a blocked deletion. A deletion removes live SQLite payload and its
local derivatives. The action is irreversible in the Dashboard. SQLite free
pages, WAL, existing backups and external provider copies can retain bytes.
When an Automation run created the Conversation, the preview identifies its
Automation. Confirming deletion also removes that definition and stops its
schedule. Any active run of that definition blocks deletion. Repository files,
other Conversations, and terminal run audit metadata remain available.
Use Actions → Delete automation to remove a standalone Automation or a card
whose Conversation was already deleted. Archive preserves history. In Archived chats, search the archived
catalog and select an individual chat to unarchive or permanently delete.
Unarchive keeps its original identity and does not schedule any work.

Configuration reports effective runtime capabilities and their authority.
Native execution stays HANDOFF_ONLY where applicable. Appearance persists
supported accent, UI font, content and code text scales through the local
preference API. Reset appearance acts independently from Reset layout.

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
