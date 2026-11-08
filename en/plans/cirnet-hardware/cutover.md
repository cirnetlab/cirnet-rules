<!-- doc-id: CUT-HARDWARE -->
<!-- revision: 1 -->
<a id="cutover"></a>

# cirnet-hardware cutover proposal

`CUT-HARDWARE` · `REL-01` · `PLANNED` · [Audit snapshot](../../specs/cirnet-hardware/audit.md)

## Dependency and boundary

Prerequisites [CUT-CONTRACTS](../cirnet-contracts/cutover.md), [CUT-SIMULATOR](../cirnet-simulator/cutover.md), [CUT-FPGA](../cirnet-fpga/cutover.md).
Affected source `Cargo.toml`, `native/src/main.rs`, `native/src/station.rs`, `native/src/simulator/protocol.rs`, `pyproject.toml`.

Integrate exact Contracts crate and C++ worker. Keep local RPC and guarded station lifecycle. Move the Python service and wheel out of target product closure. Reject old peer profiles before power.

## Acceptance and rollback

Verify RPC/Preview and worker interoperability, fail-closed starts, quarantine and formal loaded Stop with the selected bundle. Owner gate `hardware_safe_stop`. Record the exact source, contract revision, artifact digest and tool versions. Producer changes require product owner authorization. Restore only an approved complete combination and matching data. See [release policy](../../policies/governance.md) and [ADR 0001](../../references/adr-0001.md).
