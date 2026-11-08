<!-- doc-id: CUT-FPGA -->
<!-- revision: 1 -->
<a id="cutover"></a>

# cirnet-fpga cutover proposal

`CUT-FPGA` · `REL-01` · `PLANNED` · [Audit snapshot](../../specs/cirnet-fpga/audit.md)

## Dependency and boundary

Prerequisites [CUT-CONTRACTS](../cirnet-contracts/cutover.md).
Affected source `VERSION`, `scripts/build-source.mjs`, `fpga/experiment-controller/rtl/top.v`, `fpga/instrument/rtl/top.v`.

Keep HDL and source-only packaging. Agree on logical envelope and register metadata with Contracts and Hardware. Treat the physical board path as unavailable without separate evidence.

## Acceptance and rollback

Prove reviewed logical fixtures and pinned RTL tools. Reject unsupported target metadata and record no physical capability from source/simulation. Owner gate `fpga_scaffold`. Record the exact source, contract revision, artifact digest and tool versions. Producer changes require product owner authorization. Restore only an approved complete combination and matching data. See [release policy](../../policies/governance.md) and [ADR 0001](../../references/adr-0001.md).
