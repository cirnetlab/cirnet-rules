# Bundle and upgrade

`VERSION` is the toolkit version. The Cargo checker and first release entry
in each language changelog use the same version. Product versions are separate.
Version 0.2.0 changes the default skill names and introduces the local bundle
format; existing installations need a reviewed migration.

## Release naming and preflight

CIRNETLAB Git tags and GitHub Release titles use `<repo_name>-<version>` with
the exact repository name and approved version, without an automatic `v` prefix.
For this toolkit the tag and release title are both `cirnet-rules-0.2.0`.
Keep `VERSION` and `bundle.json.version` as `0.2.0`. The generated archive
may include a short source SHA for provenance; its filename is not the release title.

Before publishing, verify the clean reviewed `master` commit, matching
`VERSION`/Cargo/changelog, successful Windows and Linux CI for that commit,
license/notices, and the checked bundle with install/restore tests. Confirm the
tag and release names are unused. See the [toolkit verification record](toolkit-verification.md)
for known limits: hosted checks do not prove native client discovery.

Create a tag only after the final reviewed commit is on `master`, and only
with release authorization. Package the clean checkout using
`-Revision cirnet-rules-0.2.0`, verify the extracted bundle, and attach the
archive and reviewed notes to a GitHub Release titled `cirnet-rules-0.2.0`.
Do not publish a tag or release as part of routine packaging.

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
