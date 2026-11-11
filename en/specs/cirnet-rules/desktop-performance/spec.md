<!-- doc-id: CIRNETRULES-DESKTOP-ASYNC-SPEC -->
<!-- revision: 2 -->
<a id="acceptance"></a>

# Desktop request scheduling and performance

Verify the Release checkout on Windows with disposable conversation, Git and SQLite fixtures. Preserve the existing desktop, CLI, IPC v3 envelope, data directory, authorization, revision checks and mutation receipts.

A conversation switch shows selection and loading immediately and replaces superseded detail and event reads. An optional runtime capability permits cancellation of those reads. A snapshot watermark covers events visible in the same SQLite read transaction. The GUI renders bounded transcript pages with stable message identity and leaves drafts and mutation receipts under their existing lifecycle.

The runtime admits interactive reads separately from work that can wait on Git, filesystem or SQLite writes. A bounded response path and a shutdown deadline contain slow readers. Connection owners preserve foreign keys, WAL, preview authority and writer ordering.

- [x] Capture comparable native before and after traces with workload, sample count and percentiles.
- [x] Verify rapid navigation, stale replies, snapshot/event races, typing and incremental transcript paging.
- [x] Verify cancellation, slow work isolation, SQLite paging and preservation of write receipts.
- [x] Measure resources after warm-up and record the bounds and cleanup paths.
- [x] Run the pinned build, Rust and Qt suites, CLI and governance checks. Record skipped cases separately.
- [ ] Meet the mixed-load heartbeat and apply targets on the verification workload.

[Verification and remaining evidence](verification.md)
