<!-- doc-id: CGACP-DESKTOP-PERFORMANCE-LIFECYCLE-SPEC -->
<!-- revision: 1 -->
<a id="acceptance"></a>

# Desktop performance, data lifecycle and Settings

Keep CGACP v1 and application 0.1.0. Use only disposable workspaces and
SQLite directories for verification. Report each acceptance case by tier.

- [x] Record comparable baseline and follow-up evidence.
- [x] Verify the bounded Qt transport and cancellable storage worker.
- [x] Verify SQLite lifecycle actions, restart and active-run protection.
- [x] Verify the implemented Settings controls and their authority.
- [x] Synchronize the operator documentation and record unresolved gates.
- [ ] Close the remaining cases in the 54-case acceptance matrix.

The outstanding cases include owned file cleanup, bulk deletion, native
configuration writes, the remaining SQLite-specific services, and additional
fault-injection and resource tests. Release readiness is a separate gate.
