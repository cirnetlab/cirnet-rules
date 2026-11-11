<!-- doc-id: CNR-CLI-ONLY-PLAN -->
<!-- revision: 2 -->
<a id="implementation"></a>

# CLI-only governance implementation plan

1. Keep the Rust governance core, task store, Git worktree bindings and
   verification gates behind CNR and the compatibility CLI.
2. Validate the versioned English registry. Resolve instructions, rules and
   skills within task scope, then bind source digests to receipts and freshness.
3. Export repository instructions and skills for independent sessions in
   Codex Desktop, Claude Code and Antigravity. Test scoped sync and rollback
   in bound Git fixtures with per-file grants.
4. Preserve existing governance records using verified SQLite archives and
   digest-reviewed proposals. Exercise migration and recovery in disposable
   fixtures.
5. Verify the Rust build, bilingual sources, CLI results and audit receipts.
   Record native client checks separately from static export checks.

[Scope](../../../specs/cirnet-rules/governance-cli-only/spec.md) ·
[Verification record](../../../specs/cirnet-rules/governance-cli-only/verification.md)
