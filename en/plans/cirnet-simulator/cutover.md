<!-- doc-id: CUT-SIMULATOR -->
<!-- revision: 1 -->
<a id="cutover"></a>

# cirnet-simulator cutover proposal

`CUT-SIMULATOR` · `REL-01` · `PLANNED` · [Audit snapshot](../../specs/cirnet-simulator/audit.md)

## Dependency and boundary

Prerequisites [CUT-CONTRACTS](../cirnet-contracts/cutover.md).
Affected source `CMakeLists.txt`, `native/worker.cpp`, `pyproject.toml`, `cirnet_sim/backends.py`.

Keep C++ CMake worker and the Hardware IPC boundary. Separate Python reference models from the selected product, remove old backend selection and namespace aliases from product packaging.

## Acceptance and rollback

Prove both peers agree on framing and bounds, rejection before simulation side effects, deterministic waveforms, timeouts and worker exit. Owner gate `simulator_worker`. Record the exact source, contract revision, artifact digest and tool versions. Producer changes require product owner authorization. Restore only an approved complete combination and matching data. See [release policy](../../policies/governance.md) and [ADR 0001](../../references/adr-0001.md).
