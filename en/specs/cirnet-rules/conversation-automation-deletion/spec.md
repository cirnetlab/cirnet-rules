<!-- doc-id: CGACP-CONVERSATION-AUTOMATION-DELETION-SPEC -->
<!-- revision: 1 -->
<a id="acceptance"></a>

# Conversation and Automation deletion

- [x] Full deletion of a run-generated Conversation deletes its originating Automation from the catalog and stops its schedule. Existing sibling Conversations and terminal run metadata remain available.
- [x] The preview binds the exact Automation ID and revision. An active or leased run blocks deletion.
- [x] A message deletion, archive, or unrelated same-name record leaves its Automation state intact.
- [x] A detached Automation can be deleted explicitly by ID. Retries and restart preserve the deletion, and Qt refreshes the visible catalog.
