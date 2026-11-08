<!-- doc-id: CGACP-DESKTOP-ARCHITECTURE -->
<!-- revision: 2 -->
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
timeout. Schema 4 adds presentation preferences and first writes an
integrity-checked snapshot of an existing schema 3 store into its private
`backups/` directory. An upgrade stops if the snapshot cannot be prepared.
The Qt client uses asynchronous `preferences.get` and `preferences.set`
requests and debounces layout writes. An expected revision prevents a
second window from silently overwriting a newer update. Preferences contain
layout and appearance data, with no task permissions or repository paths.
SQLite services are internal to the Rust backend. PostgreSQL
migration requires a repository contract covering scoped queries,
optimistic updates, unique occurrences, claim fencing, outbox and rollback.
The adapter and its contract suite remain pending. Use an offline export and
verified import into a separate database, compare IDs and digests, then
switch the runtime in a maintenance window. Preserve the old store until
the new database and recovery path are proven.

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
