# Common rules rebuild implementation plan

Date: 2026-11-14. Migration record only; not a reusable task process.

**Goal:** A small, portable common rules repository with no runtime dependency.
**Architecture:** One standalone baseline, optional topic guides and skills,
manual setup, dated product references.
**Tools:** Markdown, existing PowerShell/Git and available local validators.
**Spec:** [Rebuild specification](../../../specs/cirnet-rules/rules-common/spec.md).

## Constraints and review focus

Write only the authorized `rules-common` worktree; leave changes unstaged.
Do not install tools, run old runtime commands, change other repos or global settings.
Check standalone copying, existing user preferences, nested FPGA instructions,
candidate versus selected releases, and physical safety claims.

## Steps

- [x] Verify worktree, branch and backup; capture initial status and refs.
- [x] Inspect the six local products read-only and record source evidence.
- [x] Replace root guidance and add four topic guides, three skills and local template.
- [x] Write manual setup and source map; preserve license and relevant attribution.
- [x] Remove only tracked legacy files backed up at the verified commit.
- [x] Validate all remaining files, run paper scenarios and independent review.
- [x] Recheck protected refs/product status and record results for user review.

## Baseline and decisions

Initial worktree: clean, 290 tracked files, no untracked/ignored files reported.
HEAD, `master`, `governance-cli-only`: `646afa38516bbfd276b0aecb7a138ab427df5fa9`.
Baseline checks: new adapter/setup/map/skills absent; root bootstrap depends on
the governance registry and all tasks require central grants. This does not meet
the requested standalone and lightweight behavior.

Ruling: use the already isolated worktree and supplied design; no new worktree,
approval loop or commits. The user's scope and direct execution request govern.
Ruling: keep paired migration records in the existing central task directories;
they are historical evidence and are not part of global guidance.
Ruling: static validation and paper scenarios fit this Markdown-only deliverable;
native client execution remains a separate, explicitly unrun check.

[Vietnamese review copy](../../../../vi/plans/cirnet-rules/rules-common/plan.md)
