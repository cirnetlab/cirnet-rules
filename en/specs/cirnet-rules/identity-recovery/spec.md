<!-- doc-id: CIRNETRULES-IDENTITY-RECOVERY-SPEC -->
<!-- revision: 1 -->
<a id="recovery-acceptance"></a>

# Identity recovery acceptance

- [x] CR-23 recovers a reviewed legacy task with stable identity, durable provenance and no inherited authority or execution state. The original bytes and evidence stay accessible. Drift and conflicts fail safely, and fresh bindings and caches use the current paths.
- [x] CR-26 verifies a real Windows ACL denial during SQLite migration. The runtime reports a storage error, retains the source, backup and journal, then resumes after access is restored.
- [x] Verify the Rust and Desktop suites, documentation parity and scope. Test only disposable data.
