<!-- doc-id: OPENRULES-IDENTITY -->
<!-- revision: 1 -->
<a id="identity-cutover"></a>

# OPENRULES identity and local data upgrade

The application package is OPENRULES 0.1.0. The repository and proposed
release retain the cirnet-rules and cirnet-rules-0.1.0 identities. The
Desktop launches its staged, checksum-verified sibling runtime. Private stdio
uses protocol version 2 and requires the openrules. prefix for every method.
The version and namespace are checked before any request is dispatched.
The package is openrules and the two application executables are
openrules-desktop and openrules-runtime. The existing cirnet-governance
executable remains a maintenance tool.

## Database upgrade

Desktop inspects the old and current data locations before opening a store.
If only the previous database exists, close the previous application and
runtime, review both paths, and choose **Back up and upgrade**. The runtime
requires an exclusive SQLite connection, checks database integrity and
migration history, writes a verified backup, and stages the copy. It records
the verified file digest before activating the new filename. Subsequent
schema migrations validate the historical checksums and preserve record IDs,
timestamps, preferences, evidence, and idempotency keys.

If activation is interrupted, reopen Desktop. An incomplete candidate can
be rebuilt from the retired source. A candidate already installed is
verified against the durable receipt and retained backup before the
runtime opens it. If both independent databases exist, Desktop stops for an
explicit decision. **Use current data** keeps both files and selects the
current database. To review or recover the previous store, cancel startup
and retain both copies. The previous database and its existing backups
remain at their previous paths for offline recovery.

The current runtime writes only the OPENRULES database after cutover. Keep
the retired application closed to avoid writing to the archived store.
Filesystem task records in the legacy checkout remain separate. The
maintenance executable blocks local task operations while that legacy
store exists. The archive port preserves those records as evidence without
activating tasks or transferring permissions.

<a id="identity-namespaces"></a>

## Legacy identifiers

The prior branding is retained only in data discovery and archival code,
guards preventing access to the legacy private state directory, migration
fixtures and historical task or evidence references. Historical SQLite
migration files and user-authored text keep their original contents.
