# Toolkit upgrade plan

- [x] Record worktree, baseline size and Rust results. Reduce startup context,
  extend skill discovery and add 12 skills and templates.
- [x] Build and test local bundle, install, verify and restore. Document
  versioning and upgrades, measure context and add toolkit CI.
- [x] Record bilingual acceptance evidence and limits.

## Markdown hygiene

- [x] Confirm the clean `rules-common` worktree at `5d6cc26` and inspect the checker.
- [x] Extend the existing checker and pass eight duplicate, exclusion and new-file tests.
- [x] Update existing EN/VI guidance and verify the checkout with the offline checker.

## Markdown hygiene hardening

- [x] Confirm the clean `rules-common` worktree at `66d2c8a`.
- [x] Cover duplicate severity and preserve advisory review with focused Rust tests.
- [x] Pass Git additions to the checker in CI and verify selection with local fixtures.
- [x] Check baseline size, EN/VI synchronization and retained evidence.

## Architecture compliance

- [x] Confirm `rules-common` at `004c318` and match imported snapshots to
  their source revision.
- [x] Compare architecture variants, route owners and record product evidence.
- [x] Verify offline architecture checks, EN/VI updates and isolated distribution.

[Specification](../../../specs/cirnet-rules/toolkit-upgrade/spec.md) ·
[Verification](../../../docs/toolkit-verification.md)
