<!-- doc-id: PRODUCT-PROFILES -->
<!-- revision: 1 -->
<a id="profiles"></a>

# Product architecture profiles

| Profile | Ownership and dependency direction | Evidence |
| --- | --- | --- |
| Rust service/library | Domain library behind application adapters. Contracts inputs flow into Remote and Hardware. RPC remains local. Separate startup, persistence, tests and package outputs. | Locked build, API/RPC and Stop verification. |
| C++ worker | Bounded models and framed IPC under Hardware supervision. Keep source, CMake build tree, fixtures and binaries separate. | CNP1/CNP2 and malformed input, deterministic output and exit. |
| Frontend | Keep Vue, TypeScript and Vite with schema-driven SDK input. Browser owns UX, does not own station safety or release selection. | Locked SDK identity, typecheck, application tests and real browser on exact native pairing. |
| Contracts/codegen | Source contracts, schemas and generation precede downstream binding artifacts. Record generator revision. Keep generated files reproducible. | Canonical handoff digest, operation IDs and parity across languages. |
| FPGA RTL | RTL and benches own logical scaffold. Hardware consumes approved ABI; board, calibration and timing evidence stay separate. | Parser, RTL simulation, synthesis/timing and board evidence only when claimed. |

Do not copy product tests or schemas into governance. Keep artifacts external.
[Product audit](../references/product-audit.md)
