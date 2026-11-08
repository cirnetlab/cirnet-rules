<!-- doc-id: DASHBOARD-RESTORE-CI-PLAN -->
<!-- revision: 1 -->
<a id="steps"></a>

# Dashboard restore CI plan

[Scope and acceptance](../../../specs/cirnet-rules/dashboard-restore-ci/spec.md)

1. Trace the failing restore to the directory created by the integration test.
2. Create the restore directory with Unix owner-only permissions before copying the SQLite snapshot.
3. Run governance verification with the pinned toolchain and record the available platform results.
