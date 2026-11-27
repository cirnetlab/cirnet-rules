---
name: cirnet-release-changelog
description: Use when proposing a version or preparing changelog, upgrade and rollback notes for a candidate.
---

# Release and changelog

Use for release preparation. Use implementation when the task only changes
code. Publication requires its own authorization.

Minimum inputs: Last published version, owner policy, user-visible changes, tested artifacts and open limitations.

1. Classify user impact and identify the version domain affected. Propose a bump consistent with owner policy.
2. Describe changes, breakage, migration steps and rollback risk. Keep published history and selected release facts intact.
3. Record tested revision/artifact, candidate state and unexecuted release or platform checks.

Maintain the English major-version file under `docs/changelogs/`. Keep the
root and directory indexes as navigation. Use Unreleased and full Semantic
Versioning entries with a published date or an evidenced status. See the
documentation policy in the approved local Rules checkout.

Deliver a version proposal, changelog and upgrade/rollback notes with verification limits.

Example: A skill name changes. List old/new names and a safe migration for installations of unknown ownership. Leave tagging to the authorized release action.
