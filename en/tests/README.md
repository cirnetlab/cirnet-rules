<!-- doc-id: TESTS -->
<!-- revision: 2 -->
<a id="verification"></a>

# Governance tests

Run the [local verification commands](../references/verification.md) from
the governance root. The Rust suite covers policy and translation validation,
task grants, binding identity, lease fencing, evidence freshness, scoped
configuration export and MCP STDIO. The CI gate has a separate reviewed
[candidate evidence procedure](../references/shared-task-coordination.md).

Use disposable fixtures for test writes and follow the
[testing rule](../rules/testing.md) when recording results.
