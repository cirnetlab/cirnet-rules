<!-- doc-id: CGACP-V3-PLAN -->
<!-- revision: 1 -->
<a id="work"></a>

# CGACP v3 implementation plan

[Specification](../../../specs/cirnet-rules/cgacp-v3/spec.md)

1. Relocate all Rust into src and establish the regression baseline.
2. Add local worktree bindings and narrow routing, then test dirty inputs,
   cache deltas and scope budgets.
3. Add task state transitions, optimistic writes and proof-bound actions.
   Verify controlled commands and diff scope with disposable fixtures.
4. Add verification, review, acceptance and resumable docs sync. Separate
   explicit release checks from ordinary work and retain historical evidence.
5. Verify the real CLI, update bilingual guides and report platform limits.
