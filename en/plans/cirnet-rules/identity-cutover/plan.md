<!-- doc-id: CIRNETRULES-REBRAND-PLAN -->
<!-- revision: 1 -->
<a id="steps"></a>

# CIRNET RULES identity cutover plan

[Acceptance](../../../specs/cirnet-rules/identity-cutover/spec.md)

1. Inventory source, build, wire, persistence and active references. Record baseline and preservation fixtures.
2. Add checked identity metadata consumed by Rust and Qt. Rename owned binaries, resources and references. Gate private IPC and startup on the new product identity and protocol.
3. Separate prior-generation discovery from the new cutover journal. Verify online backup, staged bytes, conflict and resume using disposable stores. Keep historical bytes and scoped authorizations separate.
4. Verify current authoring, storage and repository behavior. Check Windows staging, native UI, documentation hashes, exact legacy-name exceptions and the CR acceptance matrix.
