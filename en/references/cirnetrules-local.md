<!-- doc-id: CIRNETRULES-LOCAL -->
<!-- revision: 3 -->
<a id="operator-workflow"></a>

# CIRNETRULES local operator workflow

Build once with `cargo build --locked`. Run the verified binary from the
governance checkout or its workspace. The default sibling discovery uses
tracked governance entry files and the actual Git root. For an alternative
governance worktree, pass its path with `--governance` or set
`CIRNETRULES_GOVERNANCE` for the process. A worktree binding is local and stored
under ignored `.cirnetrules/`. Discovery and ordinary work are offline.

Task updates release their file lock when the operation ends, even if a
duplicated file handle is still open. Concurrent updates remain exclusive and
must use the current revision.

1. Run `workspace-discover`. Review the repo entry and bind one checkout
   using `workspace-bind --repo NAME --path WORKTREE --actor OPERATOR`.
2. Write the English spec and plan in their central task directories. Keep
   their Vietnamese counterparts in the same change. Supply a local JSON
   descriptor conforming to the [task schema](../schemas/cirnetrules-task.schema.json)
   with `task-create --descriptor descriptor.json`. Keep operator input in
   the ignored local state directory when it is session-specific.
   Use `_workspace` for a cross-repo parent and give each child a separate
   repository, scope and acceptance. A parent cannot close with a required
   child still pending.
3. Inspect `route --task ID` and `task-context --task ID --session TOKEN`.
   Advance with `task-transition --task ID --revision N --actor ACTOR --to STATE`.
   Starting captures dirty paths and the worktree identity. Overlapping
   unowned edits block the start. Use `task-resume` for state and next action.
   Submit a bounded JSON checkpoint via `task-checkpoint --checkpoint FILE`.
   Revision conflicts require reload. Changed authority requires
   `task-revise` and renewed approval.
4. The execution harness edits only its approved scope. Run `diff-check`
   and the policy-configured checks. `grant-evaluate`, `run-step` and
   `cancel-step` work on the managed task and named step. The runner supports
   fixed read-only Git operations and built-in JSON and UTF-8 checks.
   It supplies an executable and argv directly. Repository scripts, general
   build execution, network access and external runtime containment require
   a separate authorized harness.
5. After IMPLEMENTED, use `task-verify --task ID --revision N --actor ACTOR`.
   The runner records checks against the current diff and test bytes.
   `review-context` exposes spec, selected test inputs, bounded tracked
   diffs and results for review. Sensitive or oversized diffs require a
   separate local review.
   `task-review` records a distinct local reviewer assertion. Elevated
   boundaries remain blocked for acceptance until authenticated review is
   available. `task-accept` requires a fresh PASS and the authorization
   principal. Implementer-triggered checks are recorded as self-checks.
6. After acceptance, preview documentation with `docs-preview --task ID --proposal FILE`.
   The JSON object contains a targets array with repo, path, locale, base_sha256,
   content and optional section. Use the SHA-256 of current UTF-8 with LF and
   an exact heading when replacing a section. Rebase conflicts against current
   bytes. The preview supplies a patch when write permission is unavailable.
   An unbound target can be bound and accepted again before any write. An
   already accepted worktree cannot be silently replaced.
   `docs-apply` applies each granted destination separately and records
   pending entries. `docs-check` verifies the resulting bytes. `task-close`
   requires complete docs or the accepted docs_not_required reason. Local
   application, merge and publication are distinct.
7. Prepare a portable Agent handoff with `task-bootstrap --task ID --agent codex --session DIRECTORY --dry-run`.
   Select claude or antigravity for the other adapters. Use an owner-exclusive
   external session. Report native discovery, hook execution and task
   execution separately from the prepared bundle.

<a id="recover-maintenance-task"></a>

## Recover a maintenance task

Stop older writers and keep the original file store. To preserve a reviewed
archive of task evidence and leases, use the
[legacy archive](dashboard.md#legacy-archive). For an individual recovery,
prepare a new descriptor with the original task ID, current central spec and
plan, a current authorization owner, empty actions and no expiration.
Review the original evidence separately.

1. Use `task-create --descriptor descriptor.json --legacy-review preview`.
   Review the source path, SHA-256, state and revision alongside the proposed
   descriptor and the current spec, plan and governance it references.
2. Repeat the same command with the returned `review_sha256` in place of
   `preview`. If inputs changed, review the new preview. An ID in multiple
   legacy stores or an unrelated current record with that ID blocks recovery.
   Recover dependencies before tasks that reference them.
3. Use `task-show --task ID` to confirm `PROPOSED` and the source receipt.
   Use `workspace-bind` to verify current Git bindings and rebuild context
   from current sources. Use `task-revise` for a fresh grant, then follow the
   normal readiness and execution gates.

The original task, approvals, checkpoints, leases and evidence stay in the
retired store and are outside active authority. Recovery can be repeated for
other reviewed tasks. Regular task creation stays guarded while a retired
store remains in the checkout.

For release, use an explicitly authorized RELEASE descriptor and
`release-check --task ID --actor ACTOR`. The existing `check-release`
command remains an explicit offline verifier for the legacy workspace layout.
Both read train and family from policy and verify the exact selection and
reviewed evidence. The existing Windows software profile and recognized gate
types constrain what this implementation verifies. No release mode is run as
a side effect of WORK, VERIFY or ACCEPT. Older snapshots remain historical.

The JSON envelope returns exit 0 for PASS, 1 for FAIL, and 2 for BLOCKED or
NOT_RUN. A PASS from a local runner establishes the recorded checks only.
Path guards and hashes constrain managed operations. The external harness
owns OS isolation, operator authentication and hardware interlocks.
