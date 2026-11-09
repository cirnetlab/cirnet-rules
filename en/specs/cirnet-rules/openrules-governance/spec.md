<!-- doc-id: OPENRULES-GOVERNANCE-SPEC -->
<!-- revision: 2 -->
<a id="acceptance"></a>

# OPENRULES governance acceptance

Only the cirnet-rules checkout is writable for this task. Test writes use a disposable workspace and database. The attached OPENRULES specification defines the full OR-01 through OR-64 scenarios. Record their evidence separately and keep unchecked gates pending.

- [x] Verify OPENRULES identity, legacy migration and conflict handling without exposing user data.
- [x] Verify an offline native authoring round trip for English and Vietnamese, including review, apply and rollback.
- [x] Verify import, attachments, revision conflicts and dependency restrictions.
- [ ] Verify assignment and effective context boundaries, including the difference between preview and execution.
- [ ] Verify pinned adapter contracts and separately establish live execution evidence for both engines.
- [ ] Verify shared task lineage, controlled execution, independent review and automation snapshots.
- [ ] Verify recovery, UI regression, performance workloads and the requested accessibility matrix.
- [ ] Record all acceptance results, capability evidence and release readiness.

The verified scope uses isolated fixtures. Native QTest exercises the bilingual editor, attachment, review, apply, history and rollback. Rust fixtures cover the local SQLite migration, JSON bundle, stale revisions and dependency checks. The full OR acceptance matrix remains separate.
