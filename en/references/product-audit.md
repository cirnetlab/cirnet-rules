<!-- doc-id: AUDIT-PRODUCTS -->
<!-- revision: 1 -->
# Six-product audit for release train 0.2.4

<a id="method"></a>

## Scope and reproducibility

The local evidence snapshot was taken on 2026-11-08 UTC on Windows with Git 2.52.0.windows.1 and PowerShell 7.6.9. All six local worktrees reported clean and used branch `master`. A public Git query independently identified `master` as the default branch and returned the same commits. No public Git refs for `v0.2.4` or `v0.2.2` appeared in that query. The public release endpoint for `v0.2.4` returned HTTP 404 for each repository. Private or draft release records and private CI evidence were outside the observed scope.

| Repository | Commit | Tracked paths | Selected public historical tag |
| --- | --- | ---: | --- |
| [cirnet-contracts](../specs/cirnet-contracts/audit.md#snapshot) | `4bfd0950f564aaf24abaf6b8b82399c2d54d0e39` | 334 | `v0.1.2` |
| [cirnet-remote](../specs/cirnet-remote/audit.md#snapshot) | `1a38c1331a07bdc3b5644c70eebb40590ae53452` | 629 | `v0.1.4` |
| [cirnet-hardware](../specs/cirnet-hardware/audit.md#snapshot) | `2139cccc8b36dc403c2f7182dfb670a97513858a` | 91 | `v0.1.2` |
| [cirnet-simulator](../specs/cirnet-simulator/audit.md#snapshot) | `c05be23bef71de44a9027319af945d3e635d0882` | 73 | `v0.1.2` |
| [cirnet-interface](../specs/cirnet-interface/audit.md#snapshot) | `f1ff70b9f22554c0680b92679e9e44a5544247ff` | 609 | `v0.1.4` |
| [cirnet-fpga](../specs/cirnet-fpga/audit.md#snapshot) | `bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0` | 56 | `v0.1.2` |

Each repository audit records the real root tree ID and raw inventory digest. The six initial lookup IDs are commit objects. The root tree records paths and entry identities; a blob identifies one Git object. An artifact SHA-256 is computed over artifact bytes independently of those Git IDs.

Run these commands inside a product repository. For the inventory digest, hash the unmodified byte stream from the final Git command, including NUL separators, before parsing path records. Use the pinned commit instead of `HEAD` to reproduce this audit later. Do not hash a PowerShell-decoded pipeline.

```text
git --no-optional-locks status --porcelain=v1 --untracked-files=all
git rev-parse HEAD HEAD^{tree}
git --no-optional-locks -c credential.helper= ls-remote --symref origin HEAD refs/heads/master refs/tags/v0.2.4 refs/tags/v0.2.2 refs/tags/v0.1.4 refs/tags/v0.1.2
git --no-optional-locks ls-tree -r -z --full-tree HEAD
```

All 1,792 tracked paths were inventoried. The six audits list human-reviewed source, documentation and verification passages and classes that were only inventoried. Reviewed `README.md`, `AGENTS.md`, changelog and relevant Git history for each checkout. FPGA's Agent entry is scoped under `fpga/AGENTS.md`. No product build, clean-room run, cloud CI or physical test was executed in this governance audit. Retained artifact bytes and receipt hashes were checked separately.

<a id="identity"></a>

## Identity and dependency order

| Domain | Observed identity | Baseline treatment |
| --- | --- | --- |
| System release | Remote owns published selection 0.1.4. Target release train 0.2.4 remains blocked. | Only the owner activates a tested, complete selection. |
| Component packages | Remote native and Interface are 0.2.4. Contracts, Hardware, Simulator and FPGA prepare 0.2.2. | Coordinate CIRNET-owned component releases toward the requested 0.2.4 train. Record the actual approved per-component version with clean source commit, target and artifact receipts. A version-only edit cannot close a gate. |
| CIRNET compatibility family | Legacy representations and pins coexist in the inspected sources. | `0.2` is the proposed governing family. Each exact representation revision needs conformance. |
| Contract revision | Published lock selects Contracts 0.1.2. Remote and Hardware Rust both pin `=0.1.2` at `e640f80b1199779659d3db7817fee16d13cc24c1`. Interface still pins a Contracts 0.1.2 npm archive. | Select exact owner revision and raw/generated artifact digests. Treat any source, wire or behavioral difference as a reviewed change. |
| Governance version | `cirnet-rules` governs selection and evidence. Its own package and policy metadata version their delivery. | Record the exact governance commit and version used for each decision. |
| Toolchain version | Rust toolchain pin 1.99.1 and MSRV 1.90 are distinct. Simulator uses C++20 with CMake minimum 3.21; Interface CI uses Node 26. FPGA uses its own tool lock. | Record verified compiler, target and lock identities per artifact. |

Resolve the candidate Remote API semantics and handoff with Contracts first. Freeze and review the export at an exact Remote revision, then produce the selected Contracts resources, Rust crate and SDK. Validate the C++ Simulator worker and FPGA source against their reviewed boundaries. Use the selected Contracts and exact worker in Rust Hardware, then verify the native Remote and Interface pairing. Remote remains the release-set authority. CI acceptance, safety gates, disposition of old data and final activation follow the integration evidence.

<a id="contract-inventory"></a>

## Contract inventory and missing conformance

**Target for every row is family `0.2`.** The listed source and repository snapshot identify the exact observed revision. The proposed cutover revision for each domain is pending the owner's selection and paired conformance. Schema dialects, protobuf syntax and format versions remain separate.

| Domain and representation | Authority and producer → consumer | Observed identity, source/generation and conformance to obtain |
| --- | --- | --- |
| HTTP and OpenAPI JSON | Remote operations and export → Contracts handoff/SDK → Interface or HTTP clients | Contracts `contracts/api/openapi.json` is OAS 3.1.0, `info.version` 0.1.2, 63 distinct IDs. Native `native/src/openapi.rs` projects its runtime version and retains 46 operations. Check SDK generation against the exact native export and prove retained IDs, retired routes, authentication, response shapes and stable error semantics. |
| WebSocket JSON | Contracts schema → Remote producer → Interface event client | `contracts/public/realtime.schema.json`, Remote `native/src/events.rs` and Interface `src/services/websocket/client.ts` implement snapshot protocol 1.0. Check hello, sequence gaps, bounded frames, disconnect and resync via authorized snapshots. Commands must never replay. See `native/tests/websocket_protocol.rs` and `tests/websocket-reconcile.test.ts`. |
| Binary capture | Contracts `contracts/capture/` → Remote writer/API → Interface decoder | NCAP digital v1 and sampled v2 use independent binary layouts; HTTP canonical MIME is `application/vnd.cirnet.capture; version=1`. Native rejects a legacy-only Accept. Verify parser bounds, golden waveform and negotiation, including `native/tests/runtime_captures.rs` and `tests/capture-golden.test.ts`. |
| Hardware RPC and Preview | Contracts canonical proto → Hardware/Remote; Hardware private proto → Remote | `contracts/hardware/v2/rpc_v2.proto` is local RPC 2.0, package `cirnet.hardware.v2`. `native/preview.proto` in Hardware defines private `cirnet.hardware.internal.v1.SimulatorPreview` with Describe version 1 and JSON bytes. Generate descriptors/bindings from selected inputs. Verify bounds, unsupported version, reserved numbers, timeout and unavailable paths. |
| Simulator worker IPC | Hardware codec ↔ Simulator C++ worker | `native/src/simulator/protocol.rs` in Hardware and the Simulator native worker encode CNP1/CNP2 via little-endian length framing, request/generation identities and bounded frames. CNP2 adds powered DC. Run Hardware's `native/tests/simulator_parity.rs` with the exact selected worker, including malformed frames and cancel/timeout. |
| CircuitGraph and stored data | Contracts schema → Interface serialization → Remote persistence | `contracts/circuit-schema/` contains 1.0/1.1 and 1.2 JSON forms. Remote `native/src/db/migrations/` and `native/tests/db_migrations.rs` govern storage migration ordering. Validate rejected legacy inputs and explicit migration or fresh data. Back up, restore and validate SQLite and PostgreSQL as selected. |
| Execution, results and capabilities | Contracts registry and lab/public schemas → Remote and Hardware → Interface | `contracts/manifest.json` records individual identities, including 1.0 plans, decisions, results and capabilities. `contracts/physical-lab/lab.schema.json` and `contracts/public/api.schema.json` feed validation and bindings. Verify units, bounds, ordering, nullability, permit and ownership, and the difference between implemented, validated and available. |
| FPGA logical and registers | Contracts logical schema/register map → FPGA verification and Hardware/Remote selection | `contracts/fpga-protocol/control.schema.json` enforces `1.0-scaffold`; `register-map.md` documents `1.0-reserved`. `2.0-scaffold` is a design target. Test RTL guards and reject unsupported peers; physical availability requires a separate approved hardware verification. |
| Release manifest JSON | Remote selection → verification, install and packagers | `integration/release-set.json` is schema version 1. `scripts/system_release_manifest.py` binds it to historical system 0.1.4 and `scripts/native-release-inputs.mjs` checks native selection inputs separately. Verify selected source, digest, platform, version, capability and evidence; keep previous authority immutable. |

<a id="historical-evidence"></a>

## Historical release and candidate evidence

The [published selection as inspected](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/integration/release-set.json) uses `schemaVersion: 1`, `releaseVersion: 0.1.4` and distribution `github-release`. The observed checkout-byte SHA-256 is `90a8ef4d30ddc19d3024dcb3790fef5d5e041f45fa47922bfd5e34c2fd19c419`. The observed compatibility envelope names Contracts 0.1.2, Hardware RPC 2.0, FPGA `1.0-scaffold` / `1.0-reserved`, same origin, one API scheduler and local-only RPC. Rehashing the 15 retained assets by repository role, filename and byte size matched every selected digest. This covers Remote wheel/sdist/deploy, Contracts wheel/sdist/npm/dev-wheel/dev-npm, Hardware and Simulator wheel/sdist, Interface source/static and FPGA source. Hosted asset bytes were not downloaded afresh.

<a id="candidate-evidence"></a>

## Windows evidence at three distinct identities

| Evidence | Scope and decision |
| --- | --- |
| Published system 0.1.4 | The historical Python-based combination remains immutable and separate from the candidate train. |
| Clean Windows functional evidence | The retained host summary reports VERIFY and lifecycle PASS in offline Windows 11 x64 VirtualBox with no installed Python, including authenticated WebSocket hello, reference NCAP waveform, Restart and exact restored capture. The rehashed archive is `b7f529820dee37680bb909881cc8de331988f893d15ddf3e250a526fef9015d4` and manifest is `a40209a417d5b838b27c0686fac9c46563a5ba7b9ffef9502e76c4d0588878ea`. The manifest identifies executable version 0.1.2. VERIFY and lifecycle receipt hashes match the retained host summary. |
| Formal loaded Stop | The same host summary reports FAIL in two qualification attempts: Hardware `DRIVER_ERROR`, Remote `SAFE_STATE_FAILED`, and quarantine retained. Both raw Stop receipt digests match the host summary. The 600-sample requirement was unfulfilled. The recorded incomplete cleanup phase was worker exit or I/O thread termination. Source and candidate identity require fresh qualification before claiming closure. |
| 0.2.4 development assembly | Rehashed archive `6325074ccd869a23ae1e0b6dca30e5fe00e9eda57b27a218d396fd0cfd249489`, manifest `eea887c864ecbd301c712928f8b7483a43c541d0a172bd2554098a105282f711` and static input `25b5ddec0922e076a1472d629e12a4c54dd60da2bcf9ce6a4f4240421f7bf4c9` match Remote's record. All six recorded source overlays are dirty. Its SDK bound the older export `e1304e7d38581aa21455f6267e45c87f448fd136804c9b6fbf66ff758978d540`; Remote records a different MIME-corrected export `10da9731ce85297eadb145d9910d0923e9d17cadfb82708f00eacc8cdac319dc` and rejection of the older static producer metadata. A separate development archive with the same basename hashes to `3af63459111bcdee501d9e1a92c559607dbfa0088a4fa473aeb6815128647b93`. Select by digest and receipts. [Owner's source/target/input evidence](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/docs/en/operations/release-version-policy.md#observed-candidate-and-release-decision) |

The audit compared recorded receipts and local bytes. It did not rerun the VM or elevate an earlier functional PASS to the new candidate.

**PRODUCT_0.2.4_CUTOVER_READINESS · BLOCKED.** The release owner needs clean six-product selection, selected contract conformance, verified legacy rejection and data disposition, reproducible packaging and provenance. On the exact new candidate, rerun the Python-free clean Windows VERIFY covering WebSocket, waveform, Restart and backup/restore, qualify loaded Stop under the existing safety bounds and rehearse rollback. Physical capabilities require separate safety evidence.
