# Task Continuity implementation plan

Implement the [Task Continuity specification](../../../specs/cirnet-rules/task-continuity/spec.md) in the common-rules checkout. The PowerShell entry point serves an explicitly selected Git worktree. Repository owners choose a Git-ignored checkpoint location or local storage outside a checkout. Existing Plan/Spec bytes and ownership stay with their repositories.

- [x] Add a PowerShell integration suite in `tests/continuity/` using disposable Git repositories and the public script. The initial run without the entry point was not observed in this build.
- [x] Implement read-only Git capture and pinned document references in `scripts/continuity/`. Check staged, unstaged, untracked, HEAD, branch, worktree and index drift. Check authorized references, including related checkouts.
- [x] Implement complete snapshots and atomic current-pointer publication. Exercise corrupted snapshots, failed publication prerequisites, concurrent access and same-family resume. Verify the target index and existing Plans/Specs by hash.
- [x] Add the controlled milestone runner and lifecycle commands in `scripts/task-continuity.ps1`. Exercise actual process exit codes, interrupted runs, input changes and closure with fresh evidence. Verify the original v1 metadata-only export and reject altered packages. The distribution guide describes the current portable format.
- [x] Document checkout usage in the [distribution guide](../../../docs/distribution.md#task-continuity), pair the English and Vietnamese documentation, and add the suite to the Windows/Ubuntu CI matrix.
- [x] Record the successful Ubuntu run in the [revision-specific CI results](../../../docs/toolkit-verification.md#task-continuity-source-verification).
- [ ] Demonstrate native recovery and handoff between separate accounts in each supported agent family before claiming full specification acceptance.

## Local verification

Observed from the `cirnet-rules` root on Windows for the initial implementation recorded in `c2a7671`:

- PASS: `cargo build --offline --locked`, `cargo fmt --all -- --check` and
  `cargo clippy --offline --locked --all-targets -- -D warnings`.
- PASS: `cargo test --offline --locked` (61 passed, one ignored Windows
  directory symlink test).
- PASS: `pwsh -NoProfile -File ./tests/continuity/task-continuity.tests.ps1`.
  The suite uses disposable Git repositories and actual local child processes.
- PASS: `cargo run --offline --locked --bin check-docs --` with `--new-file`
  for both language versions of the new Plan and Spec,
  `./tests/validation/check-changed-docs.tests.ps1`, and `git diff --check`.
  The docs checker reports advisory warnings for the existing architecture variants.
- BLOCKED: Ubuntu run. WSL and the Docker Linux engine were inaccessible locally.
- NOT_RUN: Native quota exhaustion and handoffs between separate accounts.
  The suite exercises Codex and Claude Code family checks with fixtures.

## GitHub Actions follow-up

The [Ubuntu job for c2a7671](https://github.com/cirnetlab/cirnet-rules/actions/runs/44495650591/job/135462022825)
printed the suite's PASS message and exited with code 1. The runner forwards
`$LASTEXITCODE`, which the final, intentionally rejected `verify` set to 1.
The suite now asserts that rejection code and returns 0 after successful
assertions and cleanup.

The GitHub Actions PowerShell wrapper reproduced exit 1 on Windows before
the fix and returned 0 afterward. An injected fixture setup failure still
returned 1. Hosted CI was pending at that point. The linked CI record includes the fix at `67594ec`.

## Checkpoint hardening source changes

- Separate integrity, readiness, verification and completion. Recognize
  unmerged index entries and allow explicitly authorized corrective edits
  after worktree and reference reconciliation. Keep declared agent-family
  matching separate from runtime identity.
- Add v3 reviews tied to evidence artifacts by SHA-256, completed-work review,
  declared environment fingerprints, and explicit progress reconciliation
  against a reviewed checkpoint ID. Retain v1/v2 history and verify the
  portable package separately from live evidence.
- NOT_RUN during the source phase recorded in `6ca1dcf`: local build and test suites, CI, benchmarks and
  Native interruption recovery and handoff. Earlier results above describe
  earlier revisions. Acceptance remains open.

## Current Native acceptance

The [2026-11-27 Codex acceptance](../../../../reports/task-continuity/native-codex-acceptance.md#2026-11-27-native-acceptance) records
**PASS — User-Attested Native Acceptance** for quota-exhaustion recovery,
handoff between Codex accounts with a generic resume instruction, and reuse
of the existing checkpoint and Build Output session. Claude Code account
handoff remains `NOT_RUN`.
