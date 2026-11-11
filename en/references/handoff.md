<!-- doc-id: HANDOFF -->
<!-- revision: 3 -->
<a id="decision"></a>

# Handoff between operator and coding agent

1. Confirm the repository, branch, worktree binding and task revision. Read
   the selected English instructions and review the exact read and write
   scope in `cnr task context --task ID --session TOKEN`.
2. Open the bound worktree in Codex Desktop, Claude Code or Antigravity.
   Use the [repository instructions and skills](../adapters/README.md) or the
   approved portable session. To connect the client to the shared task store,
   review the [MCP launcher](mcp.md) and the selected access mode.
3. For a coordinated task, use the prepared session to claim the lease and
   submit the structured report. Hand off the task ID and relevant receipt
   references through the [shared coordination workflow](shared-task-coordination.md).
   Keep the session file private.
4. The verifier checks the bound diff and evidence, then records the result
   through the [operator workflow](cirnetrules-local.md). The next agent reads
   the current task status and context before taking over.

<a id="record-templates"></a>

## Handoff records

| Record | Fields to include |
| --- | --- |
| Design decision | Task ID, constraints, decision, rationale, affected scope, owner and approval reference. |
| Task handoff | Task ID and revision, repository and binding, expected result, dependency task IDs, current status, outstanding work and next action. |
| Verification evidence | Source and governance revisions, command, tool versions, platform, time, exit code, receipt digest and limits. |
| Scope change proposal | Requested files or actions, reason, owner, expiry and approval reference once granted. |

Use the [governance policy](../policies/governance.md) for scope changes and
the [verification guide](verification.md) when recording results.
