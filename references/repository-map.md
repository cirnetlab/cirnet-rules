# Six-repository source map

Inspected locally on **2026-11-14**. This is reference data, not product authority
or a required organization-wide bootstrap. Read only the owning repo and direct
dependencies relevant to a task; recheck current local instructions and selections.

All six checkouts were read-only. Each was on `master` with clean
`git status --short` at inspection. No product build, test, install or runtime
operation was run. **SOURCE_REVIEW** below means code/document observation;
runtime reproduction is **NOT_RUN**. Missing external artifacts are **NOT_VERIFIED**.
Version numbers here describe this snapshot only; they are not common policy.

## Local identities

Repositories are direct children of the user-selected `<repo-root>`.
Paths below are relative to each product root. Links pin the inspected Git revision,
rather than claiming that a moving branch or historical release is current.

| Repository | Local branch | Full HEAD | Initial working tree |
| --- | --- | --- | --- |
| cirnet-remote | master | `1a38c1331a07bdc3b5644c70eebb40590ae53452` | clean |
| cirnet-interface | master | `f1ff70b9f22554c0680b92679e9e44a5544247ff` | clean |
| cirnet-contracts | master | `4bfd0950f564aaf24abaf6b8b82399c2d54d0e39` | clean |
| cirnet-hardware | master | `2139cccc8b36dc403c2f7182dfb670a97513858a` | clean |
| cirnet-simulator | master | `c05be23bef71de44a9027319af945d3e635d0882` | clean |
| cirnet-fpga | master | `bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0` | clean |

## Selected system versus current source

Remote's [release selection](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/integration/release-set.json)
still selects **system 0.1.4 / Interface 0.1.4**, with **API, Contracts, Simulator,
Hardware and FPGA scaffold 0.1.2**. Current native source/candidate versions below
do not supersede that selection. Neither a version label nor a recorded candidate
PASS proves deployment, complete release acceptance or physical readiness.

## cirnet-remote

**Owns:** HTTP/realtime application semantics, authentication, project/capture
persistence, scheduling, integration/deployment and **system release selection**.
Consumes canonical Contracts and local Hardware RPC; does not own their schemas.

**Observed:** release-set selects system and Interface **0.1.4**, with API,
Contracts, Simulator, Hardware and FPGA scaffold **0.1.2**. The selected runtime
is Python FastAPI plus separate local gRPC Hardware and browser Interface.
Current Rust source is **0.2.4**, with native Contracts still pinned to **0.1.2**;
source version is not an activation or replacement of that selected release.
Native migration docs record scoped candidate results, including different
Windows bundles with different remaining gates. Exact-release acceptance,
Linux/Xeon loaded operation and physical HIL are separately recorded as not run.
External native bundle/manifest bytes were not available for inspection here.

**Source evidence:** HTTP routes consume shared Contracts; OpenAPI export checks
operation inventory; scheduler authority is acquired before recovery and retained
through drain. Tests contain unsafe/indeterminate blocking, safe topology mismatch,
control loss cleanup and quarantine cases. Their existence is not a fresh PASS.

**Docs inspected:** [README.md](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/README.md) · [AGENTS.md](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/AGENTS.md) · [docs/en/architecture/system.md](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/docs/en/architecture/system.md) · [docs/en/architecture/lab-first-v3-1.md](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/docs/en/architecture/lab-first-v3-1.md) · [docs/en/architecture/native-migration-status.md](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/docs/en/architecture/native-migration-status.md) · [docs/en/contributing/verification.md](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/docs/en/contributing/verification.md) · [docs/en/operations/native-candidate.md](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/docs/en/operations/native-candidate.md).

**Manifests:** [Cargo.toml](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/Cargo.toml) · [integration/release-set.json](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/integration/release-set.json) · [integration/contracts-version.json](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/integration/contracts-version.json).

**Source inspected:** [native/src/http_api.rs](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/native/src/http_api.rs) · [native/src/openapi.rs](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/native/src/openapi.rs) · [native/src/hardware.rs](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/native/src/hardware.rs) · [native/src/db/authority.rs](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/native/src/db/authority.rs) · [services/api/app/main.py](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/services/api/app/main.py).

**Test source inspected, not executed:** [native/tests/scheduler_authority.rs](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/native/tests/scheduler_authority.rs) · [services/api/tests/test_lab_lifecycle.py](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/services/api/tests/test_lab_lifecycle.py).

## cirnet-interface

**Owns:** Vue 3/TypeScript browser workbench, document commands/history, graph
editing, Three.js scene projection and browser tests. The **logical graph is truth**;
wire geometry, crossings and camera state do not create electrical connectivity.
Contracts owns canonical schemas, generated DTOs/validators and device catalog.

**Observed:** package source is **0.2.4**, with compatibility marked
`UNRELEASED_NATIVE_HANDOFF` targeting Remote 0.2.4 and Contracts 0.2.2.
The actual dependency/artifact selection still locks Contracts SDK **0.1.2**.
This is not the Interface 0.1.4 selected in Remote's system release.
Default virtual profile does not expose Scope/Generator hardware actions;
desktop/mobile are proposed, not implemented capability claims.

**Source evidence:** catalog and validators import `@cirnet/contracts`;
graph commands validate candidate edits before committing; SceneManager projects
graph IDs and logical ports. Tests cover stable graph identity, history, invalid
edit preservation and generated boundary diagnostics.

**Docs inspected:** [README.md](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/README.md) · [AGENTS.md](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/AGENTS.md) · [docs/en/architecture/workbench.md](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/docs/en/architecture/workbench.md) · [docs/en/contributing/verification.md](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/docs/en/contributing/verification.md).

**Manifests:** [package.json](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/package.json) · [integration/compatibility.json](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/integration/compatibility.json) · [integration/contracts-artifacts.json](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/integration/contracts-artifacts.json).

**Source inspected:** [src/data/editorCatalog.ts](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/src/data/editorCatalog.ts) · [src/contracts/structural.ts](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/src/contracts/structural.ts) · [src/domain/document/editorDocument.ts](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/src/domain/document/editorDocument.ts) · [src/domain/document/graphCommands.ts](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/src/domain/document/graphCommands.ts) · [src/three/SceneManager.ts](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/src/three/SceneManager.ts).

**Test source inspected, not executed:** [tests/document-core.test.ts](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/tests/document-core.test.ts) · [tests/generated-boundary.test.ts](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/tests/generated-boundary.test.ts).

## cirnet-contracts

**Owns:** canonical schemas, wire formats, device catalog, generated bindings,
public SDK, validation resources and compatibility fixtures. Consumers must not
redefine canonical schemas or patch generated SDK output as the source of truth.
Remote owns HTTP operation semantics and system selection.

**Observed:** Rust package and public SDK source versions are **0.2.2**; Remote's
selected Contracts remains **0.1.2**. Contract identities have independent versions,
including Hardware RPC **2.0** and FPGA scaffold/reserved interfaces. Manifest
visibility separates public browser contracts from internal execution/control
plans. Schema/codec support does not establish an available runtime producer.

**Source evidence:** canonical protobuf retains explicit unavailable/unsafe errors
and virtual/physical capability separation. Validation resolves offline resources;
public bundles reject private or unmanifested references. Fixtures test registry
coverage, descriptor parity, optional values and separate safety/topology decisions.
The source tree does not establish that a new SDK has been published or selected.

**Docs inspected:** [README.md](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/README.md) · [AGENTS.md](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/AGENTS.md) · [docs/en/contract-reference.md](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/docs/en/contract-reference.md) · [docs/en/verification.md](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/docs/en/verification.md) · [docs/en/native.md](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/docs/en/native.md).

**Manifests:** [contracts/manifest.json](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/contracts/manifest.json) · [Cargo.toml](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/Cargo.toml) · [packages/public-sdk/package.json](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/packages/public-sdk/package.json).

**Source inspected:** [contracts/hardware/v2/rpc_v2.proto](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/contracts/hardware/v2/rpc_v2.proto) · [native/src/lib.rs](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/native/src/lib.rs) · [native/src/validation/mod.rs](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/native/src/validation/mod.rs) · [native/src/validation/bundle.rs](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/native/src/validation/bundle.rs).

**Test source inspected, not executed:** [native/tests/hardware_v2_parity.rs](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/native/tests/hardware_v2_parity.rs) · [tests/contracts/test_contract_registry.py](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/tests/contracts/test_contract_registry.py) · [tests/contracts/test_lab_first.py](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/tests/contracts/test_lab_first.py).

## cirnet-hardware

**Owns:** local Hardware RPC 2.0 implementation, station lifecycle, guarded
start/stop and cleanup, worker supervision and private FPGA transport.
Contracts owns the RPC contract. Remote owns admission, durable leases, quarantine,
capture persistence and the system-level safe-release decision.

**Observed:** Rust **0.2.2** candidate is **virtual-only** and still consumes native
Contracts 0.1.2. The selected system uses historical Python Hardware **0.1.2**.
The historical simulator selection is `legacy`; the native candidate supervises a
hash-pinned C++ worker. These are separate lanes.
Physical transport/acquisition is **UNAVAILABLE**, not established by virtual tests.

**Source evidence:** loopback/local-socket service exposes the canonical RPC.
Safe handling fences generation and waits for confirmed cleanup; unconfirmed
cleanup retains fault/ownership. Physical requests return `OFFLINE`,
`HARDWARE_UNAVAILABLE`, `hardware_applied=false`. Python reference guards fresh
safety/topology, preparation, routing, power and capture; Serial/USB-SPI transport
classes are unavailable placeholders. Tests cover cleanup acknowledgement, abandoned
Start, unavailable devices and quarantine on missing safe acknowledgement.

**Docs inspected:** [README.md](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/README.md) · [AGENTS.md](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/AGENTS.md) · [docs/en/architecture/station-boundaries.md](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/docs/en/architecture/station-boundaries.md) · [docs/en/contributing/verification.md](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/docs/en/contributing/verification.md).

**Manifests:** [Cargo.toml](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/Cargo.toml) · [integration/simulator-version.json](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/integration/simulator-version.json).

**Source inspected:** [native/src/main.rs](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/native/src/main.rs) · [native/src/station.rs](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/native/src/station.rs) · [hardware_service/station/physical.py](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/hardware_service/station/physical.py) · [hardware_service/fpga_transport.py](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/hardware_service/fpga_transport.py).

**Test source inspected, not executed:** [native/src/station/tests.rs](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/native/src/station/tests.rs) · [native/tests/test_rpc_lifecycle.py](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/native/tests/test_rpc_lifecycle.py) · [tests/test_transport.py](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/tests/test_transport.py) · [tests/test_physical_station.py](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/tests/test_physical_station.py).

## cirnet-simulator

**Owns:** deterministic digital/bounded DC simulation and private CNP1/CNP2 worker
framing. Hardware owns process supervision and station lifecycle; Contracts owns
canonical circuit/capture formats; Remote owns REST orchestration.

**Observed:** native C++ worker source is **0.2.2**, with CNP1 and CNP2 frame
layouts distinct from worker/schema/system versions. Python **0.2.2.dev0** is the
current reference/verification lane; selected historical Python Simulator is
**0.1.2**. `ngspice` is **UNAVAILABLE**, with zero channels, no samples and no
fallback. Bounded DC support does not mean a SPICE backend has been implemented.

**Source evidence:** worker bounds frames/events/samples and rejects malformed
input without partial capture. Tests contain transitions, unknown logic values,
CNP2 bounded DC, malformed frames, separate limits, cancellation and protocol reuse.
The unavailable backend descriptor is enforced in Python test source.

**Docs inspected:** [README.md](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/README.md) · [AGENTS.md](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/AGENTS.md) · [docs/en/architecture/backends.md](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/docs/en/architecture/backends.md) · [docs/en/contributing/verification.md](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/docs/en/contributing/verification.md) · [native/README.md](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/native/README.md).

**Manifests:** [CMakeLists.txt](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/CMakeLists.txt) · [pyproject.toml](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/pyproject.toml) · [integration/contracts-version.json](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/integration/contracts-version.json).

**Source inspected:** [native/worker.cpp](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/native/worker.cpp) · [native/worker_limits.hpp](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/native/worker_limits.hpp) · [cirnet_sim/backends.py](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/cirnet_sim/backends.py).

**Test source inspected, not executed:** [tests/test_cpp_worker.py](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/tests/test_cpp_worker.py) · [tests/test_backends.py](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/tests/test_backends.py) · [tests/native_cpp_backend_cases.py](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/tests/native_cpp_backend_cases.py).

## cirnet-fpga

**Owns:** Verilog RTL scaffold, simulation/testbench and logical control/safety
permits. Hardware owns station transport/driver; Remote owns system orchestration.

**Observed:** source version **0.2.2**, compared with selected scaffold **0.1.2**.
Packaging declares source-only, logical `1.0-scaffold`, register map
`1.0-reserved`, and `physicalAvailable: false`. Physical firmware, transport,
board programming, acquisition, timing closure and electrical readiness are
**UNAVAILABLE or unverified**, not certified by this scaffold.

**Source evidence:** instrument availability, capture-valid and output enables are
hardwired off. Controller safe_state disables modeled routing/drive. Scan readiness
does not establish topology correctness. Physical decoder, measurement, switching
matrix/protection circuits, exact board pins and measured limits are missing inputs.
Verification requires actual simulation behavior and both owner-defined pass markers;
compile/elaboration alone is insufficient. Stored provenance is a migration record,
not the current Git HEAD or worktree status.

**Discovery:** root `AGENTS.md` is absent. Instructions are in
`fpga/AGENTS.md`; explicitly read them for FPGA work rather than assuming a client
opened at repo root automatically loads that nested file.

**Docs inspected:** [README.md](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/README.md) · [fpga/AGENTS.md](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/fpga/AGENTS.md) · [docs/en/scaffold.md](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/docs/en/scaffold.md) · [docs/en/verification.md](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/docs/en/verification.md) · [fpga/experiment-controller/README.md](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/fpga/experiment-controller/README.md) · [fpga/constraints/README.md](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/fpga/constraints/README.md) · [fpga/quartus/README.md](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/fpga/quartus/README.md).

**Manifests:** [VERSION](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/VERSION) · [integration/toolchain-lock.json](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/integration/toolchain-lock.json) · [integration/source-provenance.json](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/integration/source-provenance.json).

**Source inspected:** [fpga/experiment-controller/rtl/top.v](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/fpga/experiment-controller/rtl/top.v) · [fpga/instrument/rtl/top.v](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/fpga/instrument/rtl/top.v) · [scripts/build-source.mjs](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/scripts/build-source.mjs) · [scripts/check_rtl.py](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/scripts/check_rtl.py).

**Test source inspected, not executed:** [fpga/simulation/tb_permit_matrix.v](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/fpga/simulation/tb_permit_matrix.v) · [tests/build-source.test.mjs](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/tests/build-source.test.mjs) · [tests/fpga/test_fpga_scaffold.py](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/tests/fpga/test_fpga_scaffold.py) · [tests/fpga/test_lab_first_control.py](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/tests/fpga/test_lab_first_control.py).

## Domain safety boundary

In Remote/Hardware/FPGA, **SAFETY != CORRECTNESS**: topology mismatch alone is not
a reason to block an electrically safe, executable experiment. Unknown or unsafe
electrical state does not authorize power. Observed wiring must not be replaced by
expected wiring to make a check pass. These are product-domain distinctions; read
the current owners' specifications instead of imposing them on every repository.

Repository-specific toolchains, output allocators, plan/spec directories, release
gates, schema versions, channel counts and board targets remain local.
This map neither changes those rules nor grants write or physical-operation authority.
