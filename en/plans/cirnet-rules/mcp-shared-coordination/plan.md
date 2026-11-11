<!-- doc-id: CNR-MCP-COORDINATION-PLAN -->
<!-- revision: 1 -->
<a id="implementation"></a>

# Shared coordination implementation plan

1. Verify the linked worktree baseline, legacy data locations and historical SQL digests. Keep the current Rust CLI, task records and Git authority.
2. Extend binding selection and task transactions. Test named worktrees, shared writer ownership, fencing, reports and dependency freshness with local fixtures.
3. Add a fixed-context STDIO service and reviewable client configuration. Exercise the actual protocol through a process fixture.
4. Add portable candidate evidence and a gate evaluated against separately trusted inputs. Run regression checks and record limitations in both locales.
