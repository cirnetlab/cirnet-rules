<!-- doc-id: AUDIT-FPGA -->
<!-- revision: 1 -->

# cirnet-fpga product audit

[Tiếng Việt](../../../vi/specs/cirnet-fpga/audit.md) · [Cross-repository evidence](../../references/product-audit.md)

<a id="snapshot"></a>

## Snapshot and identity

| Field | Evidence |
| --- | --- |
| Repository | `https://github.com/cirnetlab/cirnet-fpga.git` |
| Time and scope | 2026-11-08 UTC, Windows local read-only audit |
| Local and hosted branch | Local `master` and hosted default `master` matched. Cached `origin/HEAD` points to `main`. |
| Commit SHA | `bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0` |
| Root tree SHA | `be702be824405c099d5ac80fb4f03260e366a46a` |
| Tracked inventory | 56 files. SHA-256 of raw Git tree listing is `1b258fcc5af2fe13afae0ae93ab51fb8a2d8c3bbbf51fee04535f874980cc983`. No gitlinks. |
| Working tree | Clean at the audit snapshot. |
| Local tags | v0.1.1, v0.1.2 |
| Selected historical hosted tag | `v0.1.2` resolves to `d27c818aca0eb95e9ee8d9b5ba6bda3182909914`. |
| Target publication | Hosted Git returned no matching `v0.2.4` or `v0.2.2` ref. Public GitHub release lookup for `v0.2.4` returned HTTP 404. Private drafts were outside the observed surface. |

<a id="coverage"></a>

## File coverage

Inspected `VERSION`, the nested `fpga/AGENTS.md`, packaging and capability metadata in `scripts/build-source.mjs`, `integration/source-provenance.json`, `integration/contracts-version.json` and `integration/toolchain-lock.json`. Reviewed packaging and verification workflow steps, entry paths `fpga/experiment-controller/rtl/top.v` and `fpga/instrument/rtl/top.v`, bench descriptions in `fpga/simulation/`, and English/Vietnamese scaffold and verification guidance. The entire tree includes actual RTL/testbenches and historical reference paths. No synthesis, timing closure, bitstream generation, board flash, live transport or RTL execution was attempted. No root `AGENTS.md` is tracked at this commit.

<a id="findings"></a>

## Findings and impact

| ID and status | Evidence at this commit | Impact and handling |
| --- | --- | --- |
| AUD-FPGA-01 · OBSERVED · BLOCKER | `VERSION` is 0.2.2. The Node packager constructs `cirnet-fpga-scaffold-0.2.2-native-source.tar.gz`, with source inventory and receipt. Remote's historical lock selects 0.1.2 source. | If admitted to the 0.2.4 train, select a verified source receipt from a clean revision. Neither the archive nor its schema is firmware or a HIL result. |
| AUD-FPGA-02 · OBSERVED · HIGH | `scripts/build-source.mjs` records `physicalAvailable: false`, executable logical envelope `1.0-scaffold` and reserved register map `1.0-reserved`. Contracts' `2.0-scaffold` example labels an unavailable design target. | Keep domain IDs separate and review family 0.2 with Contracts and Hardware. Reject unsupported logical/physical peers before enabling permits. |
| AUD-FPGA-03 · OBSERVED · HIGH | Source-only packaging uses Node. The RTL verification lane uses pinned Python test tooling, reviewed Contracts resources, slang and Icarus/vvp. Its documented behavioral gates require `LAB_FIRST_RTL_PASS` and `PERMIT_MATRIX_RTL_PASS`. | Keep HDL and the hardware toolchain. Classify Python helper scope as test provisioning. Actual board capabilities need a separate permit/safety review and evidence on the target board. |

<a id="readiness"></a>

## Readiness and handoff

**BLOCKED.** Depend on Contracts' reviewed envelope and interface review with Hardware. Produce a verified clean source archive, bind RTL and contract revisions and execute both behavioral gates against the selected toolchain. Keep `physicalAvailable: false` until separate evidence closes physical transport, power, timing and board verification. Governance did not run or alter any hardware.
