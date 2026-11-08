<!-- doc-id: LANG-01 -->
<!-- revision: 1 -->
<a id="LANG-01"></a>

# Language and integration

Use Rust for CIRNET services, domain logic and governance tooling, C++ for the Simulator core, and Python for testing or benchmark only. Keep the existing Interface stack and RTL. Wrappers delegate to native tools. Keep worker IPC unless an ADR compares faults, throughput and ownership. If FFI is justified, specify ABI types, allocator/free, lifetimes, panic/exception handling, threading, cancellation and deadlines. Verify with source dependency and packaging review plus both boundary peers.

[Policy](../policies/governance.md)
