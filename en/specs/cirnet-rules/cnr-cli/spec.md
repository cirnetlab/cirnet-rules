<!-- doc-id: CNR-CLI-SPEC -->
<!-- revision: 1 -->
<a id="cnr-acceptance"></a>

# CNR implementation acceptance

- [x] A separate binary shares the existing governance core. Maintenance command envelopes and Desktop IPC retain compatibility.
- [x] The terminal menu is usable with keyboard controls and supplies plain help outside a TTY. The CLI validates option scopes, returns clean JSON and supports completion.
- [x] Projects can initialize, select profiles and exact tool versions, lock native inputs, plan and synchronize environments, and export or import configuration.
- [x] A single provider inventory drives discovery and installation plans. Mutations require authority, stay within project ownership, support dry runs, and leave recoverable records.
- [x] Build, run, test, exec and package commands apply environment and execution policy. Verify bounded output, timeouts, cancellation, version conflicts and Windows argument safety.
- [x] Verify Rust, governance, translations and Desktop build compatibility. Report platform and provider cases without fixture or native evidence.
