<!-- doc-id: CGACP-CONVERSATION-AUTOMATION-DELETION-PLAN -->
<!-- revision: 1 -->
<a id="steps"></a>

# Conversation and Automation deletion plan

[Scope and verification](../../../specs/cirnet-rules/conversation-automation-deletion/spec.md)

1. Resolve Automation ownership from the originating run and show the affected definition in the delete preview.
2. Retire the definition in the same SQLite transaction as full Conversation deletion. Add an explicit Automation delete action for older detached cards.
3. Verify stale previews, active-run blockers, unrelated names, retry, restart, and native UI behavior with disposable data.
