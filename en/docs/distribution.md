# Bundle and upgrade

`VERSION` is the toolkit version. The Cargo checker and first release entry
in each language changelog use the same version. Product versions are separate.
Version 0.2.0 is a candidate because it changes the default skill names and adds
the local distribution format.

## Prepare a local bundle

Run the toolkit checker first. Use PowerShell 7, Git and an output directory you own.
For a release bundle, select a reviewed revision equal to the clean checkout
HEAD. The second example creates a development bundle from a dirty checkout.
The scripts default to preview. Add `-Apply` to create the archive.

```powershell
./scripts/package.ps1 -Output $output -Revision $reviewedRevision -Apply
./scripts/package.ps1 -Output $output -AllowDirty -Apply
```

`$output` is the archive destination directory and `$reviewedRevision` is the
full 40-character Git commit ID or a local tag. Choose the command that matches your source. A
development manifest records the actual revision and dirty state. Inspect
`Archive` from the command result, extract to another directory, then set
`$bundle` to the extracted directory containing the manifest.

```powershell
./scripts/package.ps1 -Verify -Source $bundle
```

Format 1 includes file paths and SHA-256 hashes, toolkit version, Git revision
and dirty state. The payload is the baseline, license and notices, and English
skills with linked local resources. A matching checksum detects content changes.
Use a bundle from a source you trust. The manifest describes payload bytes.
ZIP timestamps and container metadata can differ between builds.

Architecture is read on demand. For Codex, Claude Code or Antigravity, provide
an approved toolkit checkout or an approved extracted architecture artifact.
Start at [architecture policy](../policies/architecture.md); keep the source revision
and verify the needed bytes against
[`docs/architecture-sources.tsv`](../../docs/architecture-sources.tsv). The
bundle installs guidance and leaves the architecture sources outside the
installed payload. If an approved reference is unavailable, record that limit
before making architecture decisions.

## Work Prompt authoring

Select the bundled Skill with `-Skills 'cirnet-work-prompt-authoring'` when
installing and verifying, along with any other selected Skills. For a requested
handoff, open the [English template](../templates/chat-to-work-prompt.md) in an
approved checkout. The template stays outside the bundle, so an installed
Skill needs separate read access. Without it, the Skill reports `PROMPT_BLOCKED`.

## Task Continuity

Run `scripts/task-continuity.ps1` and its modules with PowerShell 7 from an
approved `cirnet-rules` checkout at a reviewed revision. The installed bundle
supplies the baseline and Skills. Grant separate read access to the checkout.
Set `Repository` to the owning worktree root and use the current
`AgentFamily` (`codex` or `claude-code`). Use the original lowercase
`TaskId` for operations on an identified task.

Use the owner's existing local storage convention. Before using the default
`.cirnet/tasks/`, confirm it is untracked and ignored, using an owner approved
repository ignore rule or the checkout's local Git exclude file. Set
`CheckpointRoot` on every call for a different ignored path or an owned directory
outside a worktree. Keep input notes and handoff files out of tracked paths and
restrict access to authorized operators.

Under `<CheckpointRoot>/<task-id>/`, `current.json` identifies the published
snapshot. Its `snapshots/<checkpoint-id>/` directory contains `task.md`,
`progress.md`, `state.json` and the checksum manifest `manifest.json`.

### Initialize and record work

Prepare UTF-8 task and progress notes with level-two headings (`##`). Keep only
authorized, redacted task information. The required headings are:

- Task: `Original Task`, `Objectives`, `Acceptance Criteria`, `Architecture & Rules`,
  `Scope & Authorization`, `Plans & Specs`.
- Progress: `Completed Work`, `Pending Work`, `Decisions & Blockers`, `Next Action`,
  `Re-verification`.

The references JSON has `documents` and `relatedRepositories` arrays. Each
document has `kind` (`plan`, `spec`, `architecture`, or `rules`), `repository`
(`primary` for the target worktree) and a repository-relative `path`. Include the
applicable architecture and rules and any existing Plans/Specs. For a related
repo, record its `id`, full Git `revision` and approved `access` (`read` or `edit`)
and pass `-RelatedRoot` with an ID-to-worktree mapping when using the checkpoint.

List each acceptance criterion on a separate bullet or numbered line under
`## Acceptance Criteria`. A single paragraph can represent a single criterion.
The acceptance JSON is an array in the same order. Each record has a stable `id`
and a `checks` array of required build or test evidence IDs. For example:

```json
[
  {"id": "builds", "checks": ["build"]},
  {"id": "ownership-reviewed", "checks": []}
]
```

An empty `checks` array still requires an authorized review with evidence. Set
the file variables below to the prepared paths. Initialize the checkpoint before
edits. The example wraps a Rust project's authorized build:

```powershell
$manager = './scripts/task-continuity.ps1'
$task = @{ Repository = $repo; TaskId = 'toolkit-fix'; AgentFamily = 'codex' }
& $manager save @task -ConfirmScope -TaskFile $taskFile -ProgressFile $progressFile -ReferencesFile $referencesFile -AcceptanceFile $acceptanceFile
& $manager run @task -ConfirmScope -EvidenceId 'build' -Kind build -Executable cargo -Arguments @('build', '--offline', '--locked')
$recovery = & $manager resume @task -ConfirmScope
```

`run` saves state before and after a controlled edit, build, test or handoff
command. Declare relevant ignored files and dependency inputs in the worktree
with `-InputPath` and nonsecret environment inputs with `-EnvironmentName`.
The manager records hashes for declared environment values. Review undeclared
or external dependencies separately. For edits outside the wrapper, review the
changes and save with `-AcceptGitDrift -ConfirmScope`. These integrations cover
explicit tool calls.

Update progress and decisions with `save -ProgressFile -ConfirmScope` and the
reviewed `-ExpectedCheckpointId`, including before handoff. Append to the
existing completed work, pending work, decisions, blockers and re-verification
sections. The same save can replace `Next Action`. To revise earlier text in
the other sections, add `-ConfirmProgressChange` with
`-ProgressReason`. The confirmed snapshot history retains the previous notes.
For v1, also record that reason in the new Decisions & Blockers section.

### Find and recover an unfinished task

For a short request such as “Resume unfinished task,” use the approved owning
worktree and checkpoint root. With authorized read access, run this from the
reviewed toolkit checkout:

```powershell
$manager = './scripts/task-continuity.ps1'
$scope = @{ Repository = $repo; AgentFamily = 'codex'; ConfirmScope = $true }
$recovery = & $manager bootstrap @scope -Compact
```

Use `claude-code` for Claude Code. Add `CheckpointRoot` to `$scope` for an
approved nondefault location. A relative root is relative to `Repository`.
Discovery examines direct task directories in that root. It checks the
published snapshot, owning repository and worktree, and agent family.
`discover @scope` returns IDs and selection status without task notes or a
live Git inventory. `bootstrap` selects and recovers the task in one call.
Without a session key, automatic selection requires a complete scan with
exactly one eligible task in the approved worktree and agent family. If a
worktree binding exists, it must match that task. A supplied session key
requires an exact session binding for automatic selection, even with one
candidate. For `AMBIGUOUS`, ask the operator to select from `Candidates` and
call `bootstrap @scope -TaskId <id>`. For `NONE`, verify the worktree, family
and storage location.

Each automatic scan allows up to 128 direct entries and 16 MiB of snapshot
payload. An unreadable or damaged candidate or an exceeded limit gives
`BLOCKED`. A verified explicit Task ID or exact session binding lets the
operator recover one task without scanning unrelated candidates.

For an authorized, durable session binding, add `-SessionKey <local-label>`
to the first `save` or use `bind @scope -TaskId <id> -SessionKey <local-label>`
for an existing task. Use the same nonsecret operator-approved label with
`discover` and `bootstrap`. Omit the key if the session has no reliable
label. `bind @scope -TaskId <id>` records a worktree binding for keyless
recovery. The selected binding must resolve to an open task with the same
stable task key. Resolve stale or damaged bindings with an approved Task ID
and an authorized `bind -ReplaceBinding`. If initialization saves a checkpoint
but binding publication fails, recover by Task ID and bind separately.
Bindings live under `<CheckpointRoot>/.bindings/` as local, access-controlled
data. The workflow requires the baseline and access to the approved toolkit.

Use the recovered `Task`, `Progress`, `References`, `Git.Changes` and check
summaries to read only what's needed for the next action. The Git summary
includes the worktree and the recorded and current branch and HEAD. Review
current local Rules, applicable pinned Plans/Specs and live reconciliation.
`-Compact` retains the original task and progress, references, acceptance
criteria and effective check results. Use ordinary `bootstrap` to inspect full
process and review evidence.
After resolving blockers and independently establishing current authorization,
repeat `bootstrap` with `-ConfirmContinuation -ExpectedCheckpointId
$recovery.CheckpointId`, using the same scope and explicit Task ID if one was
selected. Require `CanContinue` before ordinary edits. Bootstrap rechecks
implicit selection after Git and evidence checks. Changes to the candidate set,
checkpoint or binding require a fresh read. A local session label can be
reused across accounts when its authorized storage is available. The label
identifies a local binding. The agent's runtime identity and permissions still
come from the active session.

### Inspect and resume

`status` reports Git drift, pinned references and evidence. Local
`verify` exits nonzero for `FAIL`, `NOT_RUN` or `BLOCKED`.
`resume -ConfirmScope` returns the sealed notes for read-only recovery even
when verification is stale, a command was interrupted or Git has drifted.
Read `Blockers`, `Git.Changes`, `Checks` and `Acceptance` before taking action.
Reopen the pinned references and verify present implementation authorization.
Then repeat local `resume` with `-ConfirmContinuation` and require
`CanContinue` before ordinary edits. A stale observation can leave
`CanContinue` false while `CanCorrect` permits an authorized repair using
`run -Kind edit -Corrective -ConfirmScope`. Both paths require the original
worktree, reconciled Git state without unmerged paths, and current pinned
references. If a milestone was left running, confirm its process and children
stopped before using `-ConfirmStopped` with `save` or `run`. Its observation
remains `NOT_RUN`.

If the current pointer or its snapshot is damaged, use local
`resume -ConfirmScope -RecoverPrevious`. On each replacement the manager saves
the last confirmed pointer in `previous.json`. Recovery checks its manifest,
payload hashes, worktree and declared agent family. When it returns that older
snapshot, `RecoveredPrevious` is true and verification is `BLOCKED`. The owner
must reconcile the missing interval and restore the current pointer from
trusted evidence before further writes. A first save or an older store may
have no previous pointer. Snapshot directories alone cannot establish publication.

`IntegrityStatus` describes the sealed checkpoint, `Ready` describes technical
readiness, and `Status` describes overall verification, which can be incomplete
even when `Ready` is true.
`CompletionStatus` requires a recorded closure and current evidence.
`AgentFamilyMatched` compares the caller's declaration to the saved family.
The tool does not independently verify client runtime identity or account
authorization. Confirm both through the authorized session.

### Review and close

For an authorized human review, prepare `$reviewFile` as a JSON object with
`criteria` and `completedWork`. Each criterion entry has `id`, `status` and
`evidence`. The completed work entry has `status` and `evidence`, or use
`null` when that review is unchanged. For `PASS` or `FAIL`, include at least
one approved evidence record of the form `{"path":"relative/artifact.txt",
"sha256":"<reviewed SHA-256>"}`. Use a regular worktree file outside the
checkpoint store. An evidence file is separate from the review request.
The manager rechecks the file's hash for identity and freshness, and records
review provenance separately from agent notes and observed process exits.
The authorized reviewer assesses what the file proves.

Changes to source, references or linked results invalidate the corresponding
review. Changes to the completed work section invalidate its review. Close
only after completed work and every criterion have current evidence:

```powershell
& $manager review @task -ConfirmScope -ConfirmHumanReview -ExpectedCheckpointId $checkpointId -ReviewFile $reviewFile
& $manager close @task -ConfirmScope -ConfirmAcceptance
```

Version 1 and 2 checkpoints remain readable. Before closing an open version 1 task,
use `save -ConfirmScope -AcceptanceFile` to map its original criteria and
retain its required checks. This writes a new snapshot, keeping the original
task and snapshots. Writes to open v2 checkpoints adopt v3. Legacy review
statements and v1/v2 closures remain historical evidence and require separate
re-verification before any completion claim.
An ambiguous legacy acceptance section remains readable. Get an owner approved
criterion breakdown before attempting to upgrade or claim acceptance.

### Export and read a handoff

Confirm the recipient's authorization. Review the task and progress sections
and relative document references for secrets, session data and source excerpts.
Use `export -ConfirmScope -ConfirmExport -OutputFile <new-file>` outside the
entire checkpoint directory. The v2 package includes approved context, repository
fingerprints and evidence summaries. It omits source files, raw commands, logs,
local Git paths and changed file names. Filtering rejects recognized sensitive
content. The operator must check the remaining free text.

The same agent family can use `verify -HandoffFile` with the expected
`-AgentFamily` and `-ExpectedSha256` obtained from export through a trusted
channel. For reading context explicitly, use `resume -HandoffFile` with those
options and `-ConfirmScope`. A portable read can report `IntegrityStatus: PASS`
while live verification stays `NOT_RUN`. Live Git, authorization, references
and evidence freshness require the original worktree and its local checkpoint.
For native interruption recovery and handoffs between authorized accounts in
both families, see the [acceptance criteria](../specs/cirnet-rules/task-continuity/spec.md#11-verification--acceptance-criteria).

## Update and restore

Keep the previous reviewed bundle and the receipt returned by apply.
Use the same install command for a new reviewed bundle, preview its destinations,
then apply and verify. Repeating an identical bundle and selection is a no-op.
Install ownership covers managed files. A selection change leaves previously
installed skills in place. An obsolete linked file within an updated skill stays
on disk and is reported for review. Review ownership before manually retiring legacy
`implementation`, `debugging` and `verification` to avoid duplicate workflows.

Use the reported `Backup receipt` path as `$receipt` and match the original agent,
home and Codex home choices. Preview restore before apply.

```powershell
./scripts/restore.ps1 -Backup $receipt -Agent $agent
./scripts/restore.ps1 -Backup $receipt -Agent $agent -Apply
```

A receipt records pre/post bytes by hash for the affected files. Restore checks
the current post-install state before restoring. If a user edited a destination
after apply, review the conflict and recover from the backup manually. Updates and
restores operate across multiple files. A failing write reports any rollback
problem so you can inspect the named files. Receipts and empty directories may
remain after a successful restore.

Read the [changelog](changelog.md) for changes and [setup](setup.md) for client
destinations. Select a stable reviewed revision for regular use and retain a
tested previous bundle for recovery.
