<!-- doc-id: CIRNETRULES-IDENTITY -->
<!-- revision: 3 -->
<a id="identity-cutover"></a>

# CIRNET RULES identity and local data upgrade

[Application metadata](../../application-identity.json) defines the display name,
technical ID, organization, repository identity and private IPC revision. Rust
and Qt derive the runtime basename and namespace from the same metadata. Cargo
and `VERSION` agree on application 0.1.0. The planned repository release is
`cirnet-rules-0.1.0`. IPC revision 3 uses `cirnetrules.`. The Desktop validates
its sibling runtime checksum, then verifies the product and protocol before
enabling operations. This cutover affects only private Desktop IPC. External
engine protocols and SQLite schema revision 7 have independent identities.

## Upgrade from the Dashboard

1. Close the prior Desktop and all of its runtime processes. Start CIRNET RULES
   and review the source generation, source database and WAL sizes, and destination.
   The default roots are OS local application data under `CIRNETLAB`. A custom
   `--data-dir` uses the same directory and a new database filename.
2. Choose **Back up and upgrade**. The runtime locks the new data directory
   and obtains exclusive SQLite access to the source. It uses an online backup
   to include committed WAL changes. It keeps the original backup and
   upgrades the schema in a separate staged copy. It verifies the upgraded
   database, foreign keys, migration history and candidate digest before
   publishing it. On a permission, disk space or SQL error, resolve the
   reported condition and reopen Desktop to retry. An unpublished candidate
   remains outside the active store.
3. Pause before activation if needed. Reopen Desktop to resume. A copy that
   was never installed is rebuilt from the locked source. A copy installed
   before an interrupted activation is checked against its journal and backup.
   Loss of a destination after activation requires recovery.
4. If independent databases exist, Desktop shows a conflict. **Use current
   data** is available only for an existing verified destination. The sources
   remain separate. If only two independent predecessors exist, close Desktop
   and resolve ownership of the data before retrying.

The current database is `cirnetrules.sqlite3`. The migration discovers
`openrules.sqlite3` and `cgacp.sqlite3` in the known roots or explicit data
directory. When both predecessors exist, the earlier activation receipt and
verified backup must establish that the CGACP source belongs to the previous
OPENRULES cutover. The new `identity-cutover-v2` journal records its own
source and destination generation, paths, schema revision and snapshot digest.
It compares the older database with the earlier snapshot before activation to
reject divergent writes after the prior cutover.

Database migration preserves user text, imported content, IDs, evidence, and
revisions. It retains source databases, historical backups and old receipts
for offline recovery. Keep old runtimes closed after activation to avoid
divergent writes. Published history and earlier migration bytes stay intact.

Filesystem task stores `.openrules` and `.cgacp` retain their original bytes.
The explicit legacy archive preserves reviewed task and binding evidence in
SQLite. The [maintenance recovery workflow](cirnetrules-local.md#recover-maintenance-task)
can restore an individual task as a proposal with the same ID and a source
receipt. Fresh grants and verified Git bindings are required before execution.
Disposable caches are rebuilt under `.cirnetrules`.

<a id="identity-namespaces"></a>

## Legacy identifiers

The [inventory allowlist](../../identity-legacy-allowlist.json) records exact
exceptions for migration, guards, fixtures, immutable history and provenance.
