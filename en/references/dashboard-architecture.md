<!-- doc-id: CIRNETRULES-DESKTOP-ARCHITECTURE -->
<!-- revision: 9 -->
<a id="authority"></a>

# Dashboard architecture and authority

The Qt Widgets client owns presentation and file dialogs. The Rust runtime
owns validation, repository and binding IDs, intake, scheduler and SQLite
writes. Each client launches a verified sibling binary and communicates over
its inherited private stdio pipes. The JSON line protocol uses request IDs,
a bounded frame, explicit errors and persisted event cursors. No listening
TCP endpoint is created. See [IPC schema](../schemas/dashboard-ipc.schema.json).

This choice keeps the initial operator channel within one OS account.
The runtime can be started independently for scheduling. A client restart
reloads committed rows and events, without replaying a command. Operator
identity is the account that owns the process and data directory. This
boundary cannot authenticate an independent reviewer or defend against a
compromised program running with the same account. Native provider and Agent
tool integration have separate gates.

Repository metadata comes from Git after path inspection. Database IDs are
distinct from display names, Git remotes and local paths. A saved path and
binding revision are checked again before the associated task or automation
dispatch. Git common identity distinguishes sibling worktrees from unrelated
repositories. The release profile separately defines producer inputs, gate
owners and compatibility domains in governance data.

SQLite uses transactional migrations, foreign keys, WAL and a bounded busy
timeout. Schema 5 adds lifecycle receipts, monotonic identities and
scoped deletion previews. Upgrades snapshot existing stores in private
`backups/` and stop if that snapshot cannot be verified.
Schema 6 records Automation deletion in a tombstone so a retired definition
cannot be scheduled or recreated by replay. A full Conversation deletion
resolves the originating run by its stored ID, checks the Automation revision
and active runs, and retires the definition in the same transaction.
The Qt client uses asynchronous `preferences.get` and `preferences.set`
requests and debounces layout writes. An expected revision prevents a
second window from silently overwriting a newer update. Preferences contain
layout and appearance data, with no task permissions or repository paths.
The runtime validates optional RGB colors, pixel dimensions, font family,
and language in the existing preference row. The client derives pixels
from the legacy scale only when a pixel value is absent. It applies one
application font for widgets and menus. The stylesheet applies the selected
family to existing widgets and text documents. Startup reads the saved language
on the verified runtime connection. Changes replace the active translation
and update text on the existing widgets, preserving application and runtime state.
The typed lifecycle repository and SQLite unit of work commit scoped deletion,
idempotency receipts, and retained run metadata together. Legacy intake and
automation paths still contain SQLite-specific code. PostgreSQL
migration requires a repository contract covering scoped queries,
optimistic updates, unique occurrences, claim fencing, outbox and rollback.
The adapter and its contract suite remain pending. Use an offline export and
verified import into a separate database, compare IDs and digests, then
switch the runtime in a maintenance window. Preserve the old store until
the new database and recovery path are proven.

Qt verifies the sibling executable on a cancellable worker. The GUI thread
owns the process transport and drains up to four frames or four milliseconds
between callbacks. It drains complete replies before settling unanswered
requests on exit. One expiry timer tracks up to 32 wire requests. Conversation
detail and event polling each keep one latest intent and up to two wire credits.
Superseded callbacks are released immediately. The runtime advertises
`read_cancellation` for the optional `request.cancel` control. The view checks
its visit and revision before applying a result. `event_watermark` is the last
event visible in the same SQLite snapshot as the latest transcript page.

The dispatcher admits requests to a FIFO writer, an interactive SQLite reader,
or a worker for slower reads including Git and filesystem work. The worker
connections open after bootstrap and share the runtime's preview authority.
The scheduler and existing storage worker retain separate connections.
Bounded delivery hands one response at a time to stdout and closes a stalled
pipe after its deadline. Running reads can be interrupted on their own
connection. Writes retain their transaction guards and receipts, including
when delivery fails. SQLite locks can delay writes within the busy budget.

The handshake starts the read-only `storage.chats` operation. Storage status,
backup, verification and compaction use the existing maintenance worker.
Transcript queries return at most four messages subject to the page byte
budget. The client inserts and trims by message boundaries and keeps a bounded
display window. Full saved text remains available from the message menu.
See [performance verification](../specs/cirnet-rules/desktop-performance/verification.md).

The existing CLI still manages its historical task and evidence files.
Dashboard intake does not elevate those records or confer write grants.
The initial Dashboard does not execute unattended source writes or use a
native provider. Runtime state is authoritative for Dashboard data.
Plan and spec authority remains in Git.

The sidebar owns the shared conversation history. Repository pickers use the
same paginated catalog as Settings. The display path is independent from the
saved canonical path and its binding identity. Automation cards render
persisted definitions and run states. Existing template provenance remains
available to the runtime. Notifications retain a bounded session history.

The screenshot informs the desktop layout. The implementation uses native
Widgets and project labels. [OpenHands reference](https://github.com/OpenHands/OpenHands) ·
[Qt process API](https://doc.qt.io/qt-6/qprocess.html) ·
[SQLite backup API](https://www.sqlite.org/backup.html) ·
[PostgreSQL isolation](https://www.postgresql.org/docs/current/transaction-iso.html) ·
[Codex App Server](https://learn.chatgpt.com/docs/app-server)

## Governance authoring and snapshots

Qt uses the same private, versioned stdio transport as repository, conversation
and storage views. Governance dispatch validates typed requests. The SQLite
adapter owns optimistic revisions, immutable publication history and audit,
asset relationships, binding assignments, and pinned conversation/run manifests.
Schema migration 007 adds these records. Draft text and asset-relative files are
stored together in the local authoring database. Publication and rollback advance
a pointer transactionally. There is no cross-filesystem publication transaction.

Context preview and conversation pinning share the resolver for the selected or
default profile and exact binding IDs. It traverses a bounded graph, resolves
active English revisions and defers skill/reference bytes. The receipt pins
revision metadata and digests. It does not attest that an external engine loaded
the bytes. Unsupported execution and authenticated review remain gated. Adapter
probes use a bounded worker and test only a fixed loopback endpoint.
