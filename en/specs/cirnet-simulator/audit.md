<!-- doc-id: AUDIT-SIMULATOR -->
<!-- revision: 1 -->

# cirnet-simulator product audit

[Tiếng Việt](../../../vi/specs/cirnet-simulator/audit.md) · [Cross-repository evidence](../../references/product-audit.md)

<a id="snapshot"></a>

## Snapshot and identity

| Field | Evidence |
| --- | --- |
| Repository | `https://github.com/cirnetlab/cirnet-simulator.git` |
| Time and scope | 2026-11-08 UTC, Windows local read-only audit |
| Local and hosted branch | Local `master` and hosted default `master` matched. Cached `origin/HEAD` points to `main`. |
| Commit SHA | `c05be23bef71de44a9027319af945d3e635d0882` |
| Root tree SHA | `3c6dac7c4163a36b26df9cc29de31310f3945021` |
| Tracked inventory | 73 files. SHA-256 of raw Git tree listing is `5557996d208554b6cd16e9884718ed807b104563994687c145a555594f136e83`. No gitlinks. |
| Working tree | Clean at the audit snapshot. |
| Local tags | v0.1.1, v0.1.2 |
| Selected historical hosted tag | `v0.1.2` resolves to `9ec36fa2772ba8227e15ae33fec9710943a9a775`. |
| Target publication | Hosted Git returned no matching `v0.2.4` or `v0.2.2` ref. Public GitHub release lookup for `v0.2.4` returned HTTP 404. Private drafts were outside the observed surface. |

<a id="coverage"></a>

## File coverage

Read the product/version inputs in `CMakeLists.txt`, `pyproject.toml` and `integration/contracts-version.json`. Inspected worker framing and implementation excerpts in `native/worker.cpp` and Hardware's codec, Python comparison selectors in `cirnet_sim/backends.py`, packaged identity checks in `scripts/check_installed.py`, both CI workflows and English/Vietnamese backend and verification guidance. The complete tree also contains Python graph compilation and execution modules, reference aliases, native headers, protocol tests, and comparison fixtures. Other lines and payloads are inventory only. No build, frame exchange, CPU benchmark or model result was run.

<a id="findings"></a>

## Findings and impact

| ID and status | Evidence at this commit | Impact and handling |
| --- | --- | --- |
| AUD-SIM-01 · OBSERVED · BLOCKER | `CMakeLists.txt` builds the 0.2.2 worker using C++20 and static MSVC runtime settings on Windows. `pyproject.toml` builds the isolated Python reference `0.2.2.dev0` with Contracts 0.1.2. The published 0.1.4 profile selects the 0.1.2 Python wheel. | The 0.2.4 train needs hash-pinned worker bytes built from a clean revision. The pre-baseline product path is historical after activation. Keep only bounded comparison tests and benchmarks. |
| AUD-SIM-02 · OBSERVED · HIGH | Worker reads a little-endian 32-bit length and a bounded payload, with CNP1/CNP2 magic, request and generation IDs. Source bounds frame size at 16 MiB and probes at four. CNP2 extends the data model for bounded DC/powered gates. Windows native workflow includes smoke checks for both tags. | Verify exact representations under family 0.2 with Hardware. Prove length, malformed/wrong tag/ID, timeout, worker lifecycle and capture parity. IPC provides process isolation. No evidence justified replacing it with FFI. |
| AUD-SIM-03 · OBSERVED · HIGH | `cirnet_sim/backends.py` retains reference selectors and `netcircuit_sim/` retains a comparison import alias. Docs distinguish unavailable `ngspice` from the bounded implemented simulation. | Remove pre-baseline selectors and aliases from the target product dependency closure. Retain a source archive or test-only references explicitly. Document unsupported analog capability without claiming physical support. |
| AUD-SIM-04 · VERIFIED · HIGH | `git diff --shortstat v0.1.2 HEAD -- cirnet_sim pyproject.toml setup.py MANIFEST.in scripts/check_installed.py` returned six changed paths, including new packaged `cpp_backend.py` and `cpp_dc.py`. The retained selected wheel's ZIP `METADATA` identifies `cirnet-simulator` 0.1.2. HEAD's `.github/workflows/simulator.yml` has read-only permissions and builds the unpublished `0.2.2.dev0` wheel for verification. | The inspected HEAD build path separates changed source from published 0.1.2 identity. No wheel publication step was found in the inspected workflows. Keep the historical identity immutable and require a new approved producer identity if a different distributable is later authorized. |

<a id="readiness"></a>

## Readiness and handoff

**BLOCKED.** Receive the reviewed contract and frame profiles, build the exact worker, record target/toolchain/executable hash and run positive and negative framing, timing and lifecycle tests with Hardware. Establish the approved performance/safety acceptance conditions before reporting measurements. No runtime or benchmark gate was closed here.
