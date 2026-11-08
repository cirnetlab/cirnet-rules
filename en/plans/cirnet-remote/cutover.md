<!-- doc-id: CUT-REMOTE -->
<!-- revision: 1 -->
<a id="cutover"></a>

# cirnet-remote cutover proposal

`CUT-REMOTE` · `REL-01` · `PLANNED` · [Audit snapshot](../../specs/cirnet-remote/audit.md)

## Dependency and boundary

Prerequisites [CUT-CONTRACTS](../cirnet-contracts/cutover.md), [CUT-HARDWARE](../cirnet-hardware/cutover.md), [CUT-INTERFACE](../cirnet-interface/cutover.md), [CUT-SIMULATOR](../cirnet-simulator/cutover.md), [CUT-FPGA](../cirnet-fpga/cutover.md).
Affected source `Cargo.toml`, `integration/release-set.json`, `native/src/openapi.rs`, `native/src/websocket.rs`, `services/api/pyproject.toml`, `scripts/install_release.py`.

Own public operations, scheduler, storage and one release-set. Replace production Python and product release logic under the Rust tooling policy. Use explicit offline migration or fresh data, then select exact clean artifacts only.

## Acceptance and rollback

Rehearse exact provenance, native browser, clean Windows, rollback and legacy rejection. Stop safely if no approved rollback bundle exists. Owner gate `clean_windows`. Record the exact source, contract revision, artifact digest and tool versions. Producer changes require product owner authorization. Restore only an approved complete combination and matching data. See [release policy](../../policies/governance.md) and [ADR 0001](../../references/adr-0001.md).
