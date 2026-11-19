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

The bundle includes `cirnet-work-prompt-authoring`. Select it with
`-Skills 'cirnet-work-prompt-authoring'` when installing and verifying, along
with any other skills you want in the selection. For a requested handoff, an
agent using the checkout opens the [English template](../templates/chat-to-work-prompt.md)
directly. An agent using the installed Skill needs separate read access to
the same template in an approved checkout. The template stays outside the
bundle. If that access is missing, the Skill reports `PROMPT_BLOCKED`.

## Task Continuity

Run the PowerShell 7 manager from a reviewed `cirnet-rules` checkout. Set
`Repository` to the target Git worktree root and use the same lowercase `TaskId`
and `AgentFamily` (`codex` or `claude-code`) throughout the task. The target
repository owns its `.cirnet/tasks/` store. It must already ignore that path in
Git. Use `CheckpointRoot` in every call to select a different ignored, untracked
path or an owned directory outside a worktree.

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

From the toolkit checkout, set `$repo`, `$taskFile`, `$progressFile`, and
`$referencesFile` to the selected paths. This example targets a Rust project:

```powershell
$manager = './scripts/task-continuity.ps1'
$task = @{ Repository = $repo; TaskId = 'toolkit-fix'; AgentFamily = 'codex' }
& $manager save @task -TaskFile $taskFile -ProgressFile $progressFile -ReferencesFile $referencesFile -RequiredCheck 'build'
& $manager run @task -EvidenceId 'build' -Kind build -Executable cargo -Arguments @('build', '--offline', '--locked')
& $manager verify @task
& $manager resume @task -ConfirmScope
& $manager close @task -ConfirmScope -ConfirmAcceptance
```

`run` checkpoints before and after a controlled edit, build, test or handoff
command. Pass `-InputPath` for relevant ignored inputs or dependency files
within the worktree so changes invalidate prior evidence. Use `status` to
review saved state. If Git has drifted, review the differences before saving
with `-AcceptGitDrift -ConfirmScope`; run affected checks again. Read the pinned
references and confirm current authorization before resuming. Confirm acceptance
only after reviewing the task criteria and fresh required build or test evidence.

`export -ConfirmScope -OutputFile <new-file>` produces a metadata-only handoff
receipt. On the receiving machine, use `verify -HandoffFile` with the expected
`-AgentFamily` and the trusted `-ExpectedSha256` returned by export. Resuming
requires access to the local checkpoint and the relevant worktree. See the
[Task Continuity specification](../specs/cirnet-rules/task-continuity/spec.md).

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
