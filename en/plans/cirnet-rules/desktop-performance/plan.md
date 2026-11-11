<!-- doc-id: CIRNETRULES-DESKTOP-ASYNC-PLAN -->
<!-- revision: 1 -->
<a id="steps"></a>

# Desktop async performance implementation plan

[Acceptance criteria](../../../specs/cirnet-rules/desktop-performance/spec.md)

**Goal** Keep selection and editing responsive while preserving the ordering and authority of saved operations.

**Architecture** Use the existing Qt event loop for widgets and bounded response handling, with a broker for view reads. Reuse the existing storage worker and scheduler and isolate interactive SQLite reads from slow dispatch. Use measured parse and render costs to choose the transport and transcript changes.

**Stack** Qt Widgets and C++20, pinned Rust, SQLite WAL and private stdio JSON-lines v3.

1. Build Release at the baseline revision. Collect native conversation paint and heartbeat samples and compare concurrent read latency during an independently reproduced runtime stall. Record the data shape and SQLite query plans.
2. Add a latest-intent path for detail and poll requests in `runtime_client.*` and `conversations.cpp`. Verify rapid switches and A→B→A before and after, including replies after a page change.
3. Route explicitly classified reads in `src/bin/cirnetrules-runtime.rs`. Give the workers separate connections through the storage factory. Preserve FIFO for existing writes and exercise cancellation, queue limits and slow output.
4. Return the event watermark and bound conversation results in `src/dashboard/sessions.rs`. Exercise event-after-snapshot and paging on SQLite fixtures. Verify that a preview and apply use the same authority.
5. Bound transcript formatting and preserve message mapping, drafts and scroll. Capture local timing and resource evidence. Run complete verification and record the remaining gates.

Review cross-process write contention, closure during a late reply, retry of a committed mutation, and navigation back to a conversation whose earlier read is still in flight.
