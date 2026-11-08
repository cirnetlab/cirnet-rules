<!-- doc-id: AUDIT-HARDWARE -->
<!-- revision: 1 -->

# cirnet-hardware product audit

[Tiếng Việt](../../../vi/specs/cirnet-hardware/audit.md) · [Cross-repository evidence](../../references/product-audit.md)

<a id="snapshot"></a>

## Snapshot and identity

| Field | Evidence |
| --- | --- |
| Repository | `https://github.com/cirnetlab/cirnet-hardware.git` |
| Time and scope | 2026-11-08 UTC, Windows local read-only audit |
| Local and hosted branch | Local `master` and hosted default `master` matched. Cached `origin/HEAD` points to `main`. |
| Commit SHA | `2139cccc8b36dc403c2f7182dfb670a97513858a` |
| Root tree SHA | `e59bab4b1750b19e203ca36e0aa9a0a761e47268` |
| Tracked inventory | 91 files. SHA-256 of raw Git tree listing is `b0d52180ef62b38738db4a3e43e97105d679a03824b885c147f6accfbbf8a80c`. No gitlinks. |
| Working tree | Clean at the audit snapshot. |
| Local tags | v0.1.1, v0.1.2 |
| Selected historical hosted tag | `v0.1.2` resolves to `9a8935b6a596abe2d21ce90416ba3b914e9f36be`. |
| Target publication | Hosted Git returned no matching `v0.2.4` or `v0.2.2` ref. Public GitHub release lookup for `v0.2.4` returned HTTP 404. Private drafts were outside the observed surface. |

<a id="coverage"></a>

## File coverage

Inspected dependency metadata in `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `pyproject.toml` and `integration/`. Reviewed native entry and readiness, station state, worker supervision and frames in `native/src/main.rs`, `native/src/station.rs`, `native/src/simulator.rs`, `native/src/simulator/protocol.rs`, `native/preview.proto` and `native/tests/simulator_parity.rs`. Checked workflow package/build passages, `scripts/check_installed.py` and the English/Vietnamese station boundaries and verification documentation. The tracked Python `hardware_service/` implementation and existing physical test fixtures remain in inventory. No station, controller, lease, worker, firmware or physical transport was exercised.

<a id="findings"></a>

## Findings and impact

| ID and status | Evidence at this commit | Impact and handling |
| --- | --- | --- |
| AUD-HW-01 · OBSERVED · BLOCKER | The Rust crate/executable is 0.2.2. Build and runtime dependencies select Contracts `=0.1.2` at `e640f80b1199779659d3db7817fee16d13cc24c1`. `pyproject.toml` uses isolated reference `0.2.2.dev0` with published Contracts and Simulator 0.1.2 pins. | For release train 0.2.4, select exact producer bytes from a clean revision, the reviewed crate/schema and the compatible worker. Retain the reference lane only as non-product evidence. |
| AUD-HW-02 · OBSERVED · HIGH | Hardware's local canonical RPC is `cirnet.hardware.v2.HardwareService`, with RPC identity 2.0. The private `cirnet.hardware.internal.v1.SimulatorPreview` has Protobuf field 1 for the unsigned Describe reply and bytes for request/response JSON. The worker uses CNP1/CNP2; product digest is verified separately. | Review the exact RPC and IPC representation under family 0.2. Do not relabel protobuf syntax, reuse retired fields or treat ABI/wire tags as SemVer. |
| AUD-HW-03 · OBSERVED · HIGH | Native launch accepts the C++ simulation backend. The TCP readiness branch prewarms Preview. Source tracks admission, generation, cancellation and cleanup, and can reject safe-state acknowledgment. The verification guide treats physical support separately from virtual station evidence. | Verify reject-before-energization, guarded start, watchdog, Stop/ack and quarantine. Preserve circuit correctness versus safety separation and independent electrical evidence. |
| AUD-HW-04 · VERIFIED · HIGH | `git diff --shortstat v0.1.2 HEAD -- hardware_service pyproject.toml setup.py MANIFEST.in scripts/check_installed.py` returned five changed paths, including packaged `rpc.py` and station code. The retained selected wheel's ZIP `METADATA` identifies `cirnet-hardware-service` 0.1.2. HEAD's `.github/workflows/hardware-service.yml` has read-only permissions and builds the unpublished `0.2.2.dev0` wheel for verification. | The inspected HEAD build path separates changed source from published 0.1.2 identity. No wheel publication step was found in the inspected workflows. Keep the historical identity immutable and require a new approved producer identity if a different distributable is later authorized. |

<a id="readiness"></a>

## Readiness and handoff

**BLOCKED.** Depend on Contracts and Simulator. Bind the exact Rust/C++ pair by reviewed receipt, cover both IPC frames and unsupported versions, then verify application integration with Remote. Require loaded Stop and ownership/quarantine evidence at the selected target, and require HIL before exposing any physical capability. Existing source and previous parity reports are evidence of design and candidate scope.
