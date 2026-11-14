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
