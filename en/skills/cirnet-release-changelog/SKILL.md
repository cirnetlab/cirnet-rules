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
4. For CIRNETLAB repositories, use `<repo_name>-<version>` for both the Git tag and GitHub Release title. Take the exact repository name and its approved version; do not add an implicit `v` prefix or a commit suffix to either name. Example: `cirnet-rules-0.2.0`.
5. Verify that the tag and release are unused, the tag will target the reviewed commit, and CI and packaging checks apply to that commit. Tagging and publishing require separate authorization.

Deliver a version proposal, changelog and upgrade/rollback notes with verification limits.

Example: A skill name changes. List old/new names and a safe migration for installations of unknown ownership. Leave tagging to the authorized release action.
