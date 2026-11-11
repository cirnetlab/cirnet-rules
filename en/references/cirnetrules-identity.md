<!-- doc-id: CIRNETRULES-IDENTITY -->
<!-- revision: 6 -->
<a id="identity-cutover"></a>

# CIRNETRULES identity and data recovery

[Application metadata](../../application-identity.json) defines the package,
display name, organization and repository. The pinned Rust build checks the
version and public executable identity.

The default CIRNETRULES database path on Windows is
`%LOCALAPPDATA%\CIRNETLAB\CIRNETRULES\cirnetrules.sqlite3`. For an earlier
installation or a custom data directory, pass the original database path
to `--source`. The [archived recovery guide](https://github.com/cirnetlab/cirnet-rules/blob/52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c/en/references/cirnetrules-identity.md#identity-cutover)
records the earlier default paths. Keep the original database, WAL and
backups. Stop writers before moving or restoring the original data.

```sh
cnr governance migrate inspect --source PATH_TO_DATABASE --json
cnr governance migrate export --source PATH_TO_DATABASE --destination NEW_ARCHIVE --json
cnr governance migrate validate --archive NEW_ARCHIVE --json
cnr governance migrate import --archive NEW_ARCHIVE --dry-run --preview-out REVIEW_DIR --json
cnr governance migrate import --archive NEW_ARCHIVE --yes --expected-digest PLAN_DIGEST --json
cnr audit migration --receipt PLAN_DIGEST --json
cnr governance migrate rollback --expected-digest PLAN_DIGEST --yes --json
```

Export uses SQLite's backup API for an active WAL view with readable WAL
and shared-memory files. If the stopped database was fully checkpointed
and both sidecars are absent, export verifies a staged byte copy. Inspect
directs the operator to export for this case. An incomplete sidecar pair
or inaccessible WAL view blocks the operation.

Keep the archive in a private parent outside Git or in ignored local state.
It preserves the complete database, including private records. Schema 7
archives include governance rows and history. Verified schemas 1 through 6
can be archived for recovery. Staging a proposal requires schema 7.
Unsupported schemas or failed integrity checks block export. Keep such
sources in place for recovery with the original application.

The optional `--preview-out` saves the reviewable proposal in a new private
directory. Without it, dry-run reports the mapping on stdout. Confirmed
import saves the proposal in the ignored migration store.
Use `--additional-archive` to review a second source without overwriting the first.
Compare the proposed assets, translations, dependencies and repository
mappings with the registry before adopting anything. Confirming the digest
saves the proposal and receipt for review. Git changes and task grants need
the usual review and authorization. Rollback only retires the matching
proposal and keeps the archived source data.

For legacy filesystem tasks, keep the original store and follow the
[maintenance recovery workflow](cirnetrules-local.md#recover-maintenance-task).

<a id="identity-namespaces"></a>

## Compatibility identifiers

The [identity inventory](../../identity-legacy-allowlist.json) lists exact
exceptions for recovery, guards, fixtures and historical provenance.

<a id="adoption"></a>

## Confirm adoption in Git

The archive preserves legacy bytes and the staged proposal records reviewed
mapping. Apply approved files and catalog entries in a separately authorized
Git change. Validate the resulting authority and record the reviewed commit.

```sh
cnr governance migrate adoption --expected-digest PROPOSAL_DIGEST --authority-commit REVIEWED_COMMIT --actor OPERATOR --json
```

The adoption receipt checks tracked content and catalog provenance. A staged
proposal and a partially adopted proposal have distinct states. The command
writes the local receipt and keeps existing grants and authority files intact.
