<!-- doc-id: DASHBOARD-RESTORE-CI-SPEC -->
<!-- revision: 1 -->
<a id="acceptance"></a>

# Dashboard restore CI acceptance

The restore fixture must prepare a private destination and reopen the copied SQLite snapshot through the runtime. The runtime's directory security policy remains in force.

- [x] Identify the Ubuntu test failure and the Windows job outcome from the Actions logs.
- [x] Verify the restore and governance checks with Rust 1.99.1 on Windows.
- [x] Verify the restore and governance checks with Rust 1.99.1 on Linux.

The Ubuntu job failed with `PERMISSION_DENIED` while reopening the restored snapshot. The Windows matrix job was cancelled during its build. Windows local verification passed formatting, Clippy, build, all 67 tests and bilingual validation. [Ubuntu log](https://github.com/cirnetlab/cirnet-rules/actions/runs/42495833286/job/128777867294), [Windows log](https://github.com/cirnetlab/cirnet-rules/actions/runs/42495833286/job/128777867386).

An isolated Linux container reproduced exit 101 at the Actions source revision with Git 2.56.0. With the test patch applied, formatting, Clippy, build, all 68 tests and the validator passed. Hosted verification of the patched revision is pending.
