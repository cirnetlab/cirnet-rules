<!-- doc-id: CIRNETRULES-IDENTITY-RECOVERY-PLAN -->
<!-- revision: 1 -->
<a id="recovery-steps"></a>

# Close the identity recovery checks

[Acceptance](../../../specs/cirnet-rules/identity-recovery/spec.md)

1. Reproduce the blocked maintenance task and a Windows ACL denial in isolated fixtures.
2. Add a reviewed recovery path for individual task records through the existing maintenance executable. Preserve archived evidence and require new authority for execution.
3. Verify source and authority drift, collisions, restart, fresh bindings and cache rebuild. Verify the SQLite journal resumes after the ACL is restored.
4. Run the existing verification gates, synchronize operator documentation and record the acceptance results.
