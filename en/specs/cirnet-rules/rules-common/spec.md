# Common rules rebuild specification

Date: 2026-11-14. One-time migration record, not instructions for product tasks.

## Intent and authorization

Replace the Governance application with portable common agent guidance in the
existing `rules-common` worktree. The user's local build prompt supplies the
design and authorizes direct implementation. Six product repositories are
read-only evidence; their instructions do not expand this task's write scope.
No commit, staging, push, release, global installation or product edits.

## Design

Root `AGENTS.md` is the sole English common baseline and works when copied alone.
`CLAUDE.md` contains only `@AGENTS.md`. Four topic guides add decision details;
three optional, standalone skills cover implementation, debugging and verification.
Vietnamese README/setup explain manual use. The six-repository map is a dated
source snapshot, never product authority or proof of deployed behavior.

Keep `LICENSE`, `.gitattributes` and existing `.gitignore` protections. Review
third-party notices and remove dependency claims when their code is removed.
Remove tracked Rust application, manifests, migrations, governance registry,
legacy CI, assets and obsolete English/Vietnamese material only after confirming
the backup branch contains the current tracked tree. Preserve user data and Git.
Retain only this migration's paired central plan/spec records, not parallel rule trees.

The supplied Markdown-first approach is selected. Retaining the old runtime or
building a replacement installer would add dependencies contrary to the request.
No installer, test framework, task database or orchestration is introduced.

## Acceptance

- Verify exact worktree, branch, backup refs and clean initial state.
- Inspect real docs, manifests, source and tests in all six local repositories.
- Preserve product ownership, local plans/specs and test output policies.
- Document client scopes using official sources; preserve personal preferences.
- Check UTF-8, Markdown links, skill metadata, adapter target and whitespace.
- Read through typo, API ownership and FPGA safety scenarios.
- Compare product status/HEAD and protected branch refs before and after.
- Report native loader/E2E tests as `NOT_RUN` unless separately observed.

The common content must not impose product versions, toolchains, release gates,
schemas, channel counts, board choices or planning conventions on other repos.
Run no product builds or tests. Release and physical operation stay unauthorized.

[Plan](../../../plans/cirnet-rules/rules-common/plan.md) ·
[Vietnamese review copy](../../../../vi/specs/cirnet-rules/rules-common/spec.md)
