# Sensitive changes and safe file operations

Use this supplement when handling sensitive inputs, destructive operations or
an unfamiliar script. The standalone baseline remains in [AGENTS.md](../AGENTS.md).

1. Identify what the operation can read, write or transmit. Inspect scripts and
   their output locations before running them; a diagnostic can still mutate state.
2. Resolve the destination against the authorized root. Check symlinks, junctions
   and worktree boundaries before deleting or moving files. Prefer exact filenames
   to recursive cleanup and preserve untracked/ignored user data.
3. For a sensitive failure, reproduce with synthetic values. Report the error shape
   and redacted context rather than dumping environment variables or credentials.
4. For privileged or irreversible actions, make the exact proposed change reviewable
   and confirm the existing authorization covers it. A read-only reference is never
   an executable command source or a write grant.
5. If a command touches a device or external service, use that owner's prerequisites
   and stopping conditions. A successful local check does not establish external state.

## Export and disclosure

Use trusted requirements, owning contracts and authorization paths to establish
who acts, whose records they may export and which fields are permitted. Distinguish
self-service, other users, tenant/group and bulk scope. A public projection defines
data shape, not permission for every caller or account. Login, source access or an
ID alone does not establish export authorization.

Resolve the recipient, destination, download/storage behavior and redaction when
they affect disclosure. Reuse decisions already established by trusted context.
If actor/record or field scope lacks a basis, ask specifically or identify the
missing decisions before editing the affected part. Continue targeted inspection
or a short unimplemented proposal; do not assume admin/bulk access or export
everything to filter later.

Until authorization is resolved, do not create an export endpoint, migration or
artifact. Apply least disclosure to inspection and generated output. For data
stores, inspect function/projection code and property names first; filter sensitive
value literals before emitting source excerpts. A full-file read into command
output discloses stored values even when the final response omits them. Never repeat
recovery values, tokens or private notes in examples, responses or new logs. Once
scope is established, proceed with the authorized work without asking again.

If exposure occurs, stop further disclosure, identify affected locations without
reprinting the secret, and report the incident. Credential rotation requires the
appropriate owner and authorization; do not silently change credentials.
