<!-- doc-id: TASK-LOCK-SPEC -->
<!-- revision: 1 -->
<a id="acceptance"></a>

# Task lock regression acceptance

- [x] Reproduce retained locks with a duplicated file handle.
- [x] Release a completed task lock even while a duplicate remains open.
- [x] Preserve denial of concurrent writers and revision conflicts.
- [x] Verify the workflow checks on Windows and synchronize the guidance.
- [ ] Verify the updated workflow on the Linux runner.
