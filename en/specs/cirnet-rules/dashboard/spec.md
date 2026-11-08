<!-- doc-id: CGACP-DESKTOP-SPEC -->
<!-- revision: 1 -->
<a id="scope"></a>

# CGACP Dashboard scope

CGACP v1 has one Rust runtime and a Qt Widgets operator interface. Local repository identity, selected paths and activity are managed by the runtime through SQLite. Repository selection never creates a write grant. Plan and spec content stays in Git. The target application package is 0.1.0.

Desktop, catalog, conversation and scheduler verification use temporary workspaces. Publication, native provider execution and elevated approvals require separate evidence.
