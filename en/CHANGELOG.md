<!-- doc-id: CIRNETRULES-CHANGELOG -->
<!-- revision: 3 -->
<a id="unreleased"></a>

# Changelog

## Unreleased

### Added

- The native Configuration workspace manages bilingual drafts, text attachments, review, publication and rollback through the Rust runtime and SQLite.
- Published assets can be assigned to repository bindings and included in a deterministic context preview. New conversations and automation runs pin governance metadata.
- Read-only probes inspect the pinned OpenHands and OpenCode version endpoints. Import inspection reports the supported subset of selected upstream formats.

### Changed

- The current CIRNET RULES Desktop uses the shared `application-identity.json` manifest, private IPC revision 3, and the new runtime and storage names.
- The Dashboard can upgrade a previous local store through a separate verified backup and resumable journal.

- The existing maintenance workflow can recover a reviewed task as an ungranted proposal with durable source provenance.

### Fixed

- The identity migrator upgrades and verifies the staged schema before reporting readiness. A storage denial leaves a resumable journal and retains verified backups.

Earlier changes in this unreleased cycle


- The desktop, runtime, resources and application-owned namespaces use OPENRULES. The release title remains `cirnet-rules-0.1.0`.
- An existing legacy SQLite store requires preview, consent, verified backup and migration before use. The retained source is for offline recovery.
- Private Desktop IPC uses protocol 2 and the `openrules.` method prefix. Earlier clients and runtimes require matching application binaries.
