# CIRNET Lab-First — Lab System Architecture

> **Scope:** Virtual and Physical Electronics Lab across the six Lab-First repositories.  
> **Related documents:** [Ecosystem Architecture](cirnetlab-architecture-en.md) · [FPGA Architecture](../cirnetlab-FPGA-architecture-en.md) · [Tiếng Việt](cirnet-architecture-lab-first-vi.md)  
> **Canonical architecture name:** **CIRNET Lab-First** (no version suffix in the architecture name).  
> **Technical direction:** **Rust core services + C++ Simulator · Contract-driven · Virtual-to-Physical · Safety-gated · Lab-first**.  
> **Source document:** `cirnet-Remote_Architecture_Lab-First.md`; restructured and standardized for six repositories, incorporating boundaries identified in the existing codebase.  
> **Baseline review date:** 2026-11-15. The implementation observations below describe that baseline. Production or electrical safety qualification requires separate evidence for the selected profile.

## 0. Executive Summary

**CIRNET** is a remote electronics laboratory platform that uses **one design workflow** for both **virtual simulation** and **experimentation with real circuits**. Students build circuits in a Web workbench; electrical connections and data are governed by shared contracts; a Mini PC orchestrates execution; and an FPGA handles deterministic, timing-critical control and measurement operations.

The non-negotiable philosophy is:

> **Computer checks SAFETY → a demonstrably safe, executable circuit may run → physics produces observations → software evaluates and teaches.**

**SAFETY ≠ CORRECTNESS.** A circuit that produces an incorrect answer is not automatically blocked. Conversely, software **must not infer** that an unidentified circuit is electrically safe. Energization is permitted only when evidence establishes compliance with electrical limits, functioning hardware protection, and routability of the required endpoints.

| Topic | Target architecture | Baseline observation |
|---|---|---|
| Source organization | Six independent repositories with clearly assigned responsibilities; Contracts defines communication standards | Primary boundaries are already established |
| Default processing technologies | **Rust** for Backend, Contracts runtime and Hardware Service; **C++** for Simulator | Rust Remote/Contracts/Hardware candidates exist; the Simulator has a pinned C++ worker |
| Python | **Only for testing, benchmarking, parity and offline verification tools** | Historical Python runtime paths remain in a pinned release; migration must pass acceptance gates |
| User interface | Vue 3 + TypeScript + Pinia + Three.js | Web foundation exists |
| Virtual Digital | E2E: Web → Rust Remote → Rust Hardware → Simulator → Web | Must be accepted against one jointly pinned set of artifacts |
| Virtual Analog | C++ adapter to a SPICE-compatible engine; prepared for mixed-signal integration | No verified operational analog solver; must not advertise `SUPPORTED` |
| Physical Digital | Safety-gated, topology scanning, endpoint resolution, infrastructure routing and real-circuit observations | Physical Hardware/FPGA has not completed full HIL and electrical safety validation |
| Physical Analog | Future expansion via capabilities, adapters and hardware | Outside the initial Physical Lab scope |
| Instruments | Logic Analyzer, 2CH Scope, 2CH Generator; ADC/DAC/AFE/SDRAM external to FPGA | FPGA interfaces are currently scaffolds, not qualified instruments |
| User load | 30–50 connected clients. One execution slot per qualified shared electrical fabric. | Target requiring load tests on the actual target hardware |

**Reporting rule:** `TARGET`, `DESIGNED`, `IMPLEMENTED`, `TESTED`, `VERIFIED`, `RELEASED`, `UNAVAILABLE`, and `BLOCKED` must be used precisely. Code that executes in a test is not equivalent to a published release or a verified physical electrical capability.

---

## 1. Goals, Principles, and Constraints

### 1.1 Goals

1. Deliver a unified Web-CAD/Workbench for **design → simulation → physical experiment → observation → assessment**.
2. Make **CircuitGraph** the source of truth for the **logical design**; the 3D scene is only a projection of that design.
3. Keep data contracts independent of FPGA revisions, backend implementations, browsers and simulator types.
4. Complete **Virtual Digital end-to-end first**, then extend to Virtual Analog, Physical Digital and real instruments.
5. Allow students to wire real circuits at locations different from the instructor's: detect actual electrical networks, resolve endpoints from observations and **never repair incorrect student wiring in software**.
6. Keep **electrical safety**, **topology comparison** and **functional assessment** separate.
7. Execute backend, hardware and simulation workloads on the Mini PC; render 3D in the client's browser; use FPGA for deterministic and timing-critical operations.
8. Support concurrent sessions with well-defined ownership, leases, recovery and resource release.
9. Replace **Python in the production execution path** with Rust for Backend/Contracts/Hardware and C++ for Simulator, using compatibility evidence and rollback; continue using Python for tests and benchmarks when appropriate.

### 1.2 Out of scope for the initial architecture

- Do not build Kubernetes, Kafka, RabbitMQ, Redis Cluster, a full microservice architecture or server-side 3D rendering without evidence that they are necessary.
- Do not execute arbitrary analog circuits through the physical FPGA without independently designed analog routing and protection.
- Do not treat the instructor's physical breadboard placement as the grading answer.
- Do not claim that electrical scanning can identify every IC.
- Do not migrate the C++ Simulator to Rust; **C++ remains the default Simulator language**, with a pinned, verified worker.
- Do not modify UI or Contracts to conceal unimplemented capabilities (for example, advertising an oscilloscope without an actual AFE/ADC).

### 1.3 Mandatory architectural principles

- **Safety-gated, not correctness-gated.** Do not use the topology matcher as a Run Gate.
- **Observed physical truth.** The wires and components the student actually placed constitute the physical circuit to be executed.
- **One source of truth per domain.** Frontend, Hardware and Simulator must not each invent independent copies of the schemas.
- **Contract-first, compatibility-first.** Changes to public schemas/ABIs require an owner, migrations and cross-repository fixtures.
- **Rust core services + C++ Simulator, Python verification-only.** Do not introduce new Python dependencies into production API handlers, station processes, solver workers or control logic.
- **Explicit capabilities.** Unavailable functionality must report `UNAVAILABLE` or a zero-valued capability rather than fabricated measurement samples.
- **Smallest verified change.** Prefer verifiable increments; do not adopt microservices or new frameworks without a demonstrated need.
- **Search Before Create → Update Existing → Consolidate When Appropriate → Verify Documentation.** Keep documentation concise and avoid multiple reports for the same problem.

---

## 2. CIRNETLAB Organization and Source-Code Ownership

| Repository | Owned domain | Target runtime technology | Must not own |
|---|---|---|---|
| [`cirnet-remote`](https://github.com/cirnetlab/cirnet-remote) | Lab API/WS, authorization, technical Lab assignments, experiment management, scheduling/leases, capture persistence, deployment and **Lab release lock**. Owns Lab-local identity and project records in standalone scope. | **Rust** (`axum`, `tokio`, `tonic`, PostgreSQL adapter) | Direct MUX/FPGA register manipulation, simulator core, shared records after their cutover |
| [`cirnet-interface`](https://github.com/cirnetlab/cirnet-interface) | Vue Workbench, CircuitGraph editor, Pinia UI state, Three.js, instrument UI, typed SDK consumer | Browser **TypeScript** | Authoritative electrical safety decisions, hardware routing or authoritative server-side simulation |
| [`cirnet-contracts`](https://github.com/cirnetlab/cirnet-contracts) | Canonical CircuitGraph/Assignment/Physical Lab schemas, catalog, RPC, NCAP, wire protocols, code generation and SDK | **Rust** runtime and generator; generated TS bindings | Business API route registration instead of Remote; station or user management |
| [`cirnet-hardware`](https://github.com/cirnetlab/cirnet-hardware) | `HardwareStation`, local-only RPC, station lifecycle, safe Stop, FPGA transport/driver, physical protection integration and supervised simulator worker | **Rust** (`tonic`, `tokio`) | User authorization or durable leases/business databases |
| [`cirnet-simulator`](https://github.com/cirnetlab/cirnet-simulator) | Deterministic simulation engine, digital event model, bounded DC, SPICE adapter boundary and worker protocol | **C++ is the default language**; pinned, supervised and verified worker | Browser access, user permissions, physical FPGA or application database |
| [`cirnet-fpga`](https://github.com/cirnetlab/cirnet-fpga) | RTL, timing, triggers, protected control, FPGA protocol decoder, digital routing controller, instrument/SDRAM interfaces and testbenches | **HDL/RTL** (Verilog/SystemVerilog), not replaced with Rust | Auth, REST, database or business graph compiler |

**Deployment profiles.** Standalone Lab uses Remote's local principals, projects and Lab assignments. For a scope that has completed cutover, Platform provides identity and tenant context, Project owns project revisions and ACLs, and LMS owns academic assignments, submissions and grades. Remote continues to own Lab authorization, leases, technical configuration and experiment evidence. Follow the scoped migration and rollback rules in [Ecosystem §5.1](cirnetlab-architecture-en.md#51-standalone-lab-and-integrated-ownership).

**Interface governance.** The owner publishes the canonical source as a versioned artifact. Consumers use those artifacts at their selected versions. Remote selects the exact versions and digests of the Lab release set.

**Technology note:** Rust is the default for Backend/Contracts/Hardware, **C++ is the default for Simulator**; TypeScript remains on the browser and HDL on the FPGA. The C++ worker must be pinned and verified; Python serves only testing, benchmarking and historical comparison lanes and is not part of new production request processing.

### 2.1 Published baseline versus source candidates

At review time, the ecosystem had a **historical system release lock** and several **source candidates** evolving independently (for example, Rust Remote, Rust Contracts, Rust Hardware, and C++ Simulator). Their presence together on `master` does **not** make them a jointly published system release.

Every release document must distinguish:

```text
Published, pinned release set
        !=
Current master branches / candidate builds
        !=
Target architecture / future capability
```

Historical tags must remain available for rollback. CIRNETLAB's newer tag/release naming convention is `<repo_name>-<version>`; apply it **going forward** without renaming previously published, pinned tags or assets.

---

## 3. Overall Logical Architecture

```text
┌────────────────────────── WEB BROWSER ───────────────────────────┐
│ cirnet-interface: Vue 3 / TypeScript / Pinia / Three.js          │
│ CircuitGraph Editor · Projects · Instruments · WebSocket Client  │
└──────────────────────────────┬───────────────────────────────────┘
                               │ HTTPS REST + authenticated WebSocket
                               ▼
┌────────────────────────── MINI PC / EDGE ────────────────────────┐
│ Nginx: static assets + same-origin reverse proxy                 │
│                                                                   │
│ ┌────────── cirnet-remote · RUST MODULAR MONOLITH ─────────────┐ │
│ │ Auth · Projects · Assignment · Circuit Validation             │ │
│ │ Experiment Manager · Scheduler/Lease · Capture · PostgreSQL   │ │
│ │ Safety/Topology/Functional results · WS/Binary Gateway        │ │
│ └───────────────────────────┬───────────────────────────────────┘ │
│                             │ localhost gRPC / Protobuf           │
│ ┌───────── cirnet-hardware · RUST SERVICE PROCESS ──────────────┐ │
│ │ Virtual/Physical HardwareStation · admission/stop             │ │
│ │ Driver · capability · protection interface · transport        │ │
│ └─────────────────┬─────────────────────┬───────────────────────┘ │
│                   │                     │                         │
│         Virtual execution         Physical execution             │
│                   ▼                     ▼                         │
│     cirnet-simulator worker      FPGA host transport              │
│     C++ engine (default)         (Serial/USB-SPI/USB3)            │
│     CNP1/CNP2 pinned worker            │                         │
│     SPICE adapter (future)             │                         │
└────────────────────────────────────────┼──────────────────────────┘
                                         ▼
                                cirnet-fpga · RTL/HDL
                             FPGA + switching/routing fabric
                             + safe power / ADC / DAC / AFE
                                         │
                                         ▼
                               STUDENT PHYSICAL CIRCUIT

      cirnet-contracts = canonical types, schema, RPC, capture and SDK
      shared by every boundary; selected artifacts pinned by Remote.
```

**Control and data paths are distinct:** UI does not decide when to energize; public APIs do not control FPGA pins; Hardware Service does not grade student answers; FPGA does not manage users or projects.

### 3.1 Services and processes

- **Nginx:** Static frontend delivery, HTTPS and REST/WS reverse proxy.
- **Rust Remote:** Modular monolith with a single authoritative scheduler for shared stations; do not split into microservices initially.
- **Rust Hardware Service:** Separate process, isolating USB/FPGA/simulator crashes and timeouts.
- **Simulator Worker:** Supervised process; CPU-bound simulation must not block Tokio/HTTP or Stop operations.
- **PostgreSQL:** Durable Lab sessions, leases, experiments and capture metadata, plus Lab-local principal records in the standalone profile. Store large waveforms outside the database.
- **Python:** Must not be launched from the new production API, Hardware Service or Simulator worker; use separate test/benchmark tools only.

---

## 4. Rust Services + C++ Simulator Technology Standard

### 4.1 Runtime

| Boundary | Standard choice | Notes |
|---|---|---|
| HTTP/WS | Rust + `axum` + `tokio` | Favor the existing Rust Remote stack; avoid competing frameworks |
| Local Hardware RPC | Rust + `tonic` / Protobuf | Bind only to loopback or a private socket with authentication/authorization |
| Database | Rust + `sqlx` / PostgreSQL | SQLite is limited to development/tests or controlled migration paths |
| Serialization/validation | Rust `serde` + Contracts canonical schemas | Do not build two incompatible validators |
| Experiment domain | Typed Rust state machine and immutable plans | Side effects controlled by lifecycle |
| Virtual Digital | **C++** discrete-event simulator/worker by default | C++ worker pinned by SHA-256, supervised and verified through Hardware Service |
| Analog | C++ adapter to native SPICE-compatible solver | Return `UNAVAILABLE` until a trustworthy solver exists |
| FPGA runtime | Synthesized RTL running in FPGA | Rust handles host drivers/transport; it does not replace HDL |
| Testing & benchmark | Rust tests, TypeScript tests, Python pytest/benchmark | Python is permitted in offline verification, not production runtime |

The crate/dependency names above describe a direction consistent with the existing code; they **do not** imply a version-upgrade commitment or a requirement to rewrite everything.

### 4.2 Migration rules for moving away from Python

1. Do not add a new Python service, Python request handler, Python simulator worker or Python dependency to the production runtime.
2. Replace an execution path only after obtaining **contract parity, functional parity, error parity, performance evidence and a rollback path**.
3. Keep historical Python implementations in a separate lane for pinned-release comparisons; never delete artifacts/releases/fixtures needed for verification.
4. Do not change wire contracts or persisted data merely because the implementation moves to Rust.
5. Keep the **C++ Simulator** as the long-term default backend under a **hash-pinned worker** with provenance, timeout, safe Stop and compatibility verification; no Rust replacement is required.
6. Do not run `ngspice` or SPICE from production Python; the C++ Simulator integrates a native solver through a versioned adapter responsible for cancellation, limits, validation and error handling; Rust Hardware supervises its worker process.
7. Record `CURRENT`, `TARGET`, `VERIFIED`, `NOT_RUN` and rollback conditions for each migration; unit-test success alone does not establish completion.

### 4.3 Performance and memory-safety rules

- Use asynchronous I/O for HTTP/RPC/WS; separate CPU-bound workers into processes with bounded queues.
- Do not hold mutexes across `await`; avoid mutable global state per user/experiment.
- Bound queues and captures with backpressure, timeouts and cancellation.
- Stream large waveforms as bounded binary chunks rather than serializing the entire capture as JSON.
- Do not optimize through `unsafe` without benchmarks, documented invariants and tests.
- Every buffer, worker and connection must have an owner, cancellation token, cleanup procedure and deterministic retry semantics.

---

## 5. Sources of Truth and CIRNET Contracts

### 5.1 Three levels of design and execution

```text
DesignGraph          (user-editable, logical intent)
    │ normalize + schema/domain validation
    ▼
ValidatedGraph       (logical consistency, signal domains, references)
    │ compile for selected backend/station
    ▼
ExecutionPlan        (derived, immutable, never edited by browser)
    ├── VirtualExecutionPlan
    └── PhysicalExecutionPlan / InstrumentationExecutionPlan
```

- **DesignGraph:** The editable logical design represented by the canonical `CircuitGraph` contract. It describes circuit instances, ports, nets, properties and placement. Project lifecycle and editor history have their own owners and contracts.
- **ValidatedGraph:** Valid references, signal widths/domains, net rules and component-model availability; **incorrect answers are not technical validation errors**.
- **ExecutionPlan:** Backend mappings, bounded event schedule or resource allocation, with owner, contract version, fingerprint and scope-specific expiry.
- **Three.js Scene:** A view of DesignGraph only; it must not infer electrical connectivity from intersecting 3D wires or nearby coordinates.

**Two distinct sources of truth must remain separate:**

```text
Logical design truth     = DesignGraph
Observed physical truth  = DetectedTopologyGraph + measured evidence
```

Incorrect wires placed by students belong to **observed truth**. The system may warn and mark the answer incorrect, but **must not** turn it into the reference circuit in the Physical Compiler.

### 5.2 Contract ownership and independent version domains

`cirnet-contracts` owns the canonical forms and compatibility tests for:

- `CircuitGraph` and the existing 1.0 / 1.1 / 1.2 versions; legacy documents must remain readable.
- `SignalDomain`: `DIGITAL`, `ANALOG`, `POWER`, `GROUND`, `CLOCK`, `MIXED`; versioned namespace; `UNKNOWN` is not equivalent to DIGITAL.
- `AssignmentDefinition`, `LabEndpoint`, instructor/reference profiles and assessment policy.
- `DetectedTopologyGraph`, `ObservedPinBinding`, `TopologyResolutionResult` and `ResolvedEndpointBinding`.
- `SafetyVerificationResult`, `TopologyVerificationResult`, `FunctionalVerificationResult` and `ExecutionDecision`.
- `StationCapabilities`, `HardwareStation` RPC boundary and error/status codes.
- Shared `SimulationRequest/Result/Error` and `VirtualExecutionPlan` schemas.
- Shared logical FPGA control contracts and conformance mappings, verified against the FPGA-owned board interface with Hardware.
- `ExperimentResult`, `CaptureMetadata`, NCAP v1/v2 and streaming envelopes.

Simulator owns the private worker framing and fixtures with Hardware as its consumer. FPGA owns the board interface and physical frame ABI. Hardware owns the driver and transport integration.

**Do not collapse versions:** Repository, toolkit, CircuitGraph, Assignment, Hardware RPC, FPGA frame, firmware, SDK, NCAP and selected system release versions are separate domains. Record each separately in a compatibility matrix.

### 5.3 Rules for changing Contracts

1. Use one canonical model; generate/validate TypeScript and Rust DTOs/validators from the same source.
2. Do not patch generated files in consumer repositories to fix a source defect; fix the owner and regenerate.
3. Even additive changes may break strict consumers; test old-reader/new-writer and new-reader/old-writer compatibility as applicable.
4. Preserve unknown optional metadata during save/load when the contract allows it; do not silently discard document fields.
5. Breaking contracts require a new version, migration, compatibility window and fixtures; do not silently modify frozen versions.
6. **Remote** registers public API operations, while Contracts owns the handoff/schema/SDK; private worker frames must not be confused with a public API.
7. All Schema IDs, package digests and resource paths must be backed by a source/manifest; do not dynamically fetch network content when executing a pinned release.

---

## 6. Virtual Digital — Mandatory End-to-End Path

This is the **first acceptance priority** for the Rust-services + C++-Simulator architecture. The goal is not merely to pass an isolated simulator test, but to **show real browser results from a chain of mutually compatible processes and artifacts**.

### 6.1 Main flow

```text
USER (Web Workbench)
  │ edit CircuitGraph; choose Virtual Digital station
  ▼
CIRNET INTERFACE (Vue/TS)
  │ typed REST: Create/Save/Validate/Run + request_id/idempotency
  ▼
CIRNET REMOTE (Rust)
  │ authenticate/authorize; validate selected Contracts
  │ establish experiment/session; select station capability
  │ compile/validate immutable VirtualExecutionPlan
  ▼
CIRNET HARDWARE (Rust, local-only gRPC)
  │ VirtualHardwareStation; admission; worker supervision
  │ matching selected simulator protocol and artifact digest
  ▼
CIRNET SIMULATOR (C++ default; pinned worker artifact)
  │ deterministic event execution; bounded samples/cancellation
  ▼
CIRNET HARDWARE
  │ validate worker identity, status, event budget and capture
  │ encode selected NCAP / typed result via Contracts
  ▼
CIRNET REMOTE
  │ persist experiment state + capture metadata
  │ publish authorized event stream / binary capture
  ▼
CIRNET INTERFACE
  │ display 0/1/X, LED, timing, Logic Analyzer and result
  └─► user observes actual result, not fabricated UI sample
```

**Separate Stop/cancel flow:** `Browser Stop → Remote permission + lease → Hardware supervisor → Simulator cancellation → safe acknowledgement → Remote release/quarantine → UI state`. `Stop acknowledged` **must not** mean merely that a UI button was pressed; actual cleanup must be confirmed.

### 6.2 Minimum data workflow

1. **Editor:** Save a valid DesignGraph, preserving graph identity and revision; scene representation does not alter net connectivity.
2. **Remote:** Check user permissions, ownership, CSRF/Origin, versioned DTOs, station capability, idempotency and quota.
3. **Contracts:** Verify wire/schema, pinned source/SDK artifacts, backend identity, signal domains and level of support.
4. **Plan:** Compiler produces an immutable VirtualExecutionPlan with model IDs, net resolution, timing parameters, bounded event budget and instrument selection.
5. **Hardware:** Create one virtual station execution; supervise worker, deadline, cancellation, expected protocol and digest.
6. **Simulator:** Read validated requests, run event scheduler, return samples/traces and explicit status; never fabricate samples on unsupported requests or errors.
7. **Hardware → Remote:** Hardware validates worker frames, status and capture and returns a gRPC response. Remote persists experiment and capture metadata and stores the capture payload.
8. **Remote → Web:** Separate Run acknowledgement from final results; WS events carry stream identity/sequence and resynchronization behavior; binary capture is permission-gated.
9. **UI:** Display `QUEUED/RUNNING/COMPLETE/FAILED/UNAVAILABLE`; loading/pending does not imply PASS.

### 6.3 Minimal reference experiment

```text
SW1 ──┐
      AND (74HC08 logical model) ── LED1
SW2 ──┘

Run vectors: 00, 01, 10, 11
Expected settled AND: 0, 0, 0, 1
```

The end-to-end test must record **requests/responses at every process boundary**, event-timestamp mapping, 0/1/X, NCAP/raw bytes, persisted metadata and UI state. Coverage can then expand to NOT, OR, 7-segment, flip-flop, counter, clock, reset, power-aware HC08/LED, within the actual supported model scope.

### 6.4 Single artifact set and pinning requirements

Remote owns the manifest that selects the **Lab system release**. Resolve its path in the selected Remote revision and profile. For example, the historical Python reference lock is under `migration/python-reference/integration/release-set.json` in the layout described by Remote's system documentation. Candidate tests use a separate pinned test-session manifest and preserve historical locks.

A selected E2E set must contain at least:

| Artifact | Required pinned fields |
|---|---|
| `cirnet-interface` | Git revision, package/build hash, selected Contracts SDK digest |
| `cirnet-remote` | Rust binary/package SHA-256, source revision, API/WS contract identity |
| `cirnet-contracts` | Rust crate/source revision, schema fingerprint, SDK artifact digest, RPC descriptor hash |
| `cirnet-hardware` | Rust service digest, selected RPC version, station profile/capability set |
| `cirnet-simulator` | C++ worker executable SHA-256, source revision, private worker protocol version |
| `cirnet-fpga` | Source/RTL revision when testing FPGA; Virtual Digital must not require a physical board |
| System | Release-set identity, test profile, OS, toolchain, selected backend and data fixture |

Do not use dynamic `latest`, branch HEAD, an unpinned executable or manually copied SDKs for acceptance. Validate SHA-256 against the **actual artifact bytes**. Digests ensure integrity but do not replace trust/signature or permission review.

**Invariant:** Source candidates from the six repositories are not a selected release; the artifact set must be resolved before E2E execution and must not change mid-run.

### 6.5 Acceptance gates: Virtual Digital E2E

| Gate | Verification | PASS condition |
|---|---|---|
| VD-01 | Contract + SDK compatibility | Browser/Rust DTOs/RPC descriptors match selected versions; canonical fixtures pass |
| VD-02 | Real process chain | Web → real Remote → Hardware → Worker → Remote → Web; do not mock any boundary claimed to be real |
| VD-03 | Functional parity | Truth tables, timing/event order, 0/1/X, powered/unpowered states follow the published model |
| VD-04 | Persistence/capture | Metadata and NCAP bytes can be retrieved by the correct owner; sample counts/timestamps are valid |
| VD-05 | Failure isolation | Bad CRC/frame, wrong digest, crash, timeout and malformed graphs produce explicit errors, never fabricated samples |
| VD-06 | Cancellation/Stop | Timeout, disconnect and Stop terminate the correct worker; ownership/lease is reliably released or quarantined |
| VD-07 | Concurrency/resource bounds | Bounded worker pool, admission, backpressure, queue overflow, latency and memory are measured |
| VD-08 | Real browser | UI shows correct Run acknowledgement, progress and results; reconnect does not replay commands |
| VD-09 | Release provenance | Manifest pins source/artifact/compiler/backend; logs retain Run ID and exact SHA |
| VD-10 | Cross-platform target | Windows integration and Linux Mini PC readiness are distinguished; production passes only after target-environment testing |

Report **Virtual Digital E2E verified** only when all ten gates have passed for the stated artifact set and environment. Record failures and unexecuted or blocked checks against their gates. Browser–API–Hardware–Worker evidence is required for the end-to-end claim.

---
## 7. C++ Simulator and Analog Expansion

### 7.1 Target digital engine

```text
VirtualExecutionPlan
  ├── Device Model Registry
  ├── Net Resolver (0 / 1 / X; optional Z only with explicit rules)
  ├── Event Priority Queue (timestamp + stable tie-break)
  ├── Propagation Delay / Clock Scheduler
  ├── Bounded Simulation Worker
  ├── Virtual Logic Analyzer
  ├── Virtual Generator (only if implemented)
  ├── Virtual Oscilloscope (only if implemented)
  └── Unified Observation / NCAP encoding boundary
```

The algorithm is **discrete-event**, not a system-wide fixed timestep.

```text
Input change at t=0
   → enqueue affected device
   → calculate output according to validated model
   → schedule propagation at t + Δ
   → update dependent net and probes
   → stop at time/event/sample limit or cancellation
```

- Use deterministic ordering for simultaneous events; record model input and random seed for reproducibility.
- Distinguish ideal gates from power-aware devices; one set of models cannot establish correctness for every physical IC.
- Unsupported models must return an explicit `UNEXECUTABLE`/`UNAVAILABLE` result.
- Bound event count, sample count, duration, queue depth, CPU/memory and maximum capture independently.
- Simulated time and wall-clock time are different domains; do not label an event timestamp as a measured FPGA clock.
- `CNP1/CNP2` are pinned private C++ worker contracts; protocol changes require version handshakes, compatibility tests and explicit migration.

### 7.2 Maintaining and improving the C++ Simulator

The C++ worker is the long-term default Simulator implementation; a migration to Rust is not required. The project should:

1. Pin the C++ worker by SHA-256, source revision, protocol version and accepted fixture set.
2. Extend the C++ worker through its existing contract; when a breaking change is needed, introduce a new protocol version and compatibility adapter.
3. Run regression/parity fixtures for event ordering, bounded DC, error/status, Stop/cancel, NCAP and resource limits.
4. Compare C++ workers from different pinned candidates/releases through offline tests; **do not** run multiple workers for one physical experiment.
5. Measure latency, memory, throughput and stability under identical profiles; Python benchmarking belongs only in the verification lane.
6. Retain C++ as the selected backend in an approved manifest; preserve older C++ artifacts for rollback.

### 7.3 Virtual Analog and Mixed-Signal

```text
ValidatedGraph
  ├── DIGITAL ── C++ discrete-event engine ──┐
  ├── ANALOG  ── C++ SPICE adapter ───────────┼─→ Unified Result Model
  └── MIXED   ── Explicit A/D & D/A bridges ─┘
```

- Supporting `SignalDomain` in schemas and UI **does not** automatically enable solver capabilities.
- A SPICE-compatible engine (for example, `ngspice`) is a **native dependency with its own version, license and digest**, integrated by the C++ Simulator through an appropriate adapter/API and isolated worker.
- The C++ adapter owns netlist generation, model provenance, parameter bounds, resource/time limits, cleanup, waveform parsing and error mapping.
- Mixed-signal support requires defined timebases, thresholds, interpolation, convergence, synchronization and anti-oscillation policies; do not substitute a fictitious bridge.
- Set `virtualAnalog=true` only after a real solver, end-to-end fixtures, exact artifacts and release acceptance are in place.
- An existing bounded educational DC model **is not equivalent to full analog SPICE**.
- Supporting Virtual Analog must not implicitly enable Physical Analog Routing.

---

## 8. Physical Lab: Design Versus Observed Electrical Topology

### 8.1 Data layers

```text
AssignmentDefinition                DesignGraph (student's virtual design)
   │ reference/expected behavior     │
   └─────────────────┬───────────────┘
                     │ only hints / assessment, never rewiring authority
                     ▼
PhysicalStation + BreadboardNodeMap + Scan Front-End
                     │
                     ▼
DetectedTopologyGraph  (student's ACTUAL wires and observed nodes)
                     │
                     ▼
TopologyResolutionResult + ResolvedEndpointBinding
                     │
                            ├── Executability/Routing   → candidate plan
                            ├── Safety Verification     → gate for the exact plan
                     ├── Topology Verification   → diagnostics only
                     └── Functional Verification → AFTER measured execution
```

`ReferenceTopologyGraph` represents the instructor's reference circuit; `DetectedTopologyGraph` records observations at the student's breadboard. Their identifiers and coordinates **must not** be assumed identical. If the instructor places an IC at E10/F10 while the student uses E35/F35, the topology may still be equivalent electrically.

### 8.2 Breadboard Node Map and active scanning

An 830-hole breadboard with a custom PCB needs a fixed mapping for each electrically connected group of holes:

```text
A1,B1,C1,D1,E1 → BreadboardNodeID N001
BreadboardNodeID → Scan Matrix / Sense Front-End → RoutingResource
```

Scanning procedure:

1. Hold the authorized lease and confirm `SAFE_STATE` with DUT power and experiment stimuli disabled and routes default-open.
2. Confirm interlock and station readiness and safe residual or external energy conditions before enabling any scan excitation. Keep scanning disabled when the pre-check cannot establish those conditions.
3. Scan permitted points under a **separate scan permit** bounded by verified voltage, current and duration limits. Keep DUT power and experiment stimuli disabled. Inhibit scan excitation on timeout, Stop or fault.
4. Measure with a high-impedance front-end, accounting for leakage, charge and cross-coupling.
5. Construct observed nodes/continuity, pin contacts, signatures, confidence and measurement provenance.
6. Use an instructor profile or signature library to suggest IC orientation/identity **only when the electrical conditions are demonstrably safe**.
7. Disable scan excitation and confirm the safe state before route application. Create `DetectedTopologyGraph`, retaining incorrect, missing, unidentified or ambiguous branches and the scan's station/profile, generation, coverage and validity.
8. Send the observations to separate Safety Verification, Topology Resolution and topology assessment.

**Do not claim** continuity measurements reliably identify VCC/GND or every IC pin function: many components exhibit nonlinear behavior, protection diodes, hidden pins, supply-dependent thresholds and stimulus-dependent states. If safety cannot be established → `INDETERMINATE` and **BLOCK**.

### 8.3 Component resolution modes

| Mode | Meaning | Use |
|---|---|---|
| `AUTO_DISCOVERY` | Infer location/identity from scans and signatures within supported scope | Capability-dependent; never overstate confidence |
| `INSTRUCTOR_PROFILE` | Instructor supplies package, VCC/GND, pin direction/function and expected IC | Useful when the assignment limits valid IC choices |
| `HYBRID` | Instructor provides expected pin semantics; scanner locates actual placement; signatures provide supporting evidence | **Recommended default** for the first Physical Digital implementation |

If a component cannot be resolved, represent it as `OPAQUE_PHYSICAL_COMPONENT`, `identity = UNKNOWN`, retaining available real observations. Do not copy the `ExpectedComponentSet` into the `observed identity`.

### 8.4 Resolve LabEndpoints against the **student's actual circuit**

`LabEndpoint` describes an authorized point where the system may stimulate or measure:

```text
INPUT_SOURCE | OUTPUT_SENSE | CLOCK | VCC | GND |
GENERATOR | SCOPE | LOGIC_ANALYZER
```

An endpoint may refer to:

- `fixed_node_id`: A fixed, verified physical node of the **actual station**.
- `reference_node_id`: A node in the instructor/reference circuit, which **must be resolved** to an observed student node before routing.
- `design_port`: Logical mapping in Virtual mode, which cannot be used directly as a physical node.
- Legacy `node_id`, when still supported, retains only the explicitly defined fixed-placement meaning and must have migration notes.

**Mandatory resolution process:**

```text
ReferenceEndpoint + AssignmentDefinition + InstructorPinProfiles
    + DetectedTopologyGraph + StationProfile
                 │
                 ▼
TopologyResolver (conservative, bounded)
                 │
                 ▼
TopologyResolutionResult:
  RESOLVED | PARTIAL | AMBIGUOUS | UNRESOLVED
  endpoint_bindings[]: reference → observed student node
  matched components / uncertainty / evidence provenance
                 │
                 ▼
Physical Compiler reads RESOLVED observed bindings only
```

Safety invariants:

1. Treat `reference_node_id` and `observed_node_id` as separate namespaces. Require independent mapping evidence even when their text values coincide.
2. Matching coordinates or net labels do not prove electrical continuity.
3. Every `ObservedPinBinding` must reference an existing observed node; pin/connection IDs must not be duplicated; malformed evidence → `DRIVER_ERROR` and safe fallback.
4. Do not assign pin roles or identity simply because the reference assignment expects a particular IC.
5. If a **required endpoint** is unresolved or unroutable → `BLOCK_UNEXECUTABLE`.
6. If ambiguity affects **grading only**, while fixed endpoints and safety are established, guarded execution may proceed under policy; `AMBIGUOUS_TOPOLOGY` alone is not a blocker.
7. `confidence=0` or missing evidence must not silently become 100% confidence or an invented electrical safety threshold.
8. Observed terminals on the same net must be considered jointly during resource allocation; greedy collapsing must not hide conflicts or feasible routes.
9. Topology matching must be bounded by graph size, mapping attempts and time; return `AMBIGUOUS` when the budget is exhausted rather than allowing untrusted input to trigger unbounded search.

---

## 9. Physical Compiler Routes **Infrastructure Only**; It Does Not Repair Circuits

### 9.1 Architectural prohibition

> **Physical Compiler MUST NOT repair, recreate, or replace student IC/jumper wiring using FPGA routing.**

This applies even when the system knows the instructor's correct schematic. Reconnecting logical nets inside the FPGA to “rescue” incorrect student wiring would turn the experiment into a measurement of a **software-reconstructed circuit**, not the circuit physically assembled by the student.

### 9.2 Physical Compiler inputs

```text
DetectedTopologyGraph                  (actual observed wiring)
+ TopologyResolutionResult             (actual observed endpoint bindings)
+ AssignmentDefinition / allowed LabEndpoints
+ Pre-check evidence and verified protection limits
+ StationCapabilities / HardwareProfile
+ Available protected routing/instrument resources
                       │
                       ▼
            LabExecutionCompiler (Rust domain)
                       │
                       ▼
        InstrumentationExecutionPlan (immutable candidate)
```

Output may allocate only:

- **Protected VCC/GND paths**, input stimuli, clocks, generators and output sensing.
- Scope/Logic Analyzer channels, triggers, captures, timing and test vectors.
- MUX/crosspoint/resources for **infrastructure connections** to authorized observed endpoints.
- Runtime interlocks, power policies, protection budgets and resource-release paths.

Assess safety and executability on the exact immutable candidate before applying routes or requesting any experiment energy (§10.4). The compiler **must not** create a route from the reference AND output to the LED if the student did not wire them together. FPGA measures actual points, and `FunctionalVerificationResult` may report FAIL/NO_RESPONSE/PARTIAL based on actual behavior.

### 9.3 Example that must be tested

```text
Reference:  SW1 → AND1.A, SW2 → AND1.B, AND1.Y → LED1
Observed:   SW1 → AND1.A, SW2 → AND1.B, AND1.Y --X-- LED1

Forbidden:
  Route AND1.Y → LED1 through FPGA to repair the missing wire.

Permitted if safe and executable:
  Route authorized input stimuli to observed SW endpoints.
  Route sense probes to observed AND1.Y / LED nodes.
  Run guarded and record actual signals.
  Return functional FAIL / NO_RESPONSE with topology diagnostics.
```

Additional test cases: wrong but safe IC, missing IC, opaque IC, dangerous short, missing endpoint, two competing outputs, ambiguity confined to unrelated branches, resource exhaustion, stale scan/plan, and driver disconnect during guarded power-up.

### 9.4 Routing Solver

- Solve constrained graph routing for **infrastructure endpoints**, never for the student's logical nets.
- Inputs: observed endpoint bindings, protected capabilities, resource graph, voltage/domain/driver constraints, reservations and instrument allocation.
- Algorithms may use BFS/Dijkstra/backtracking and conflict checks, provided execution is deterministic and bounded.
- Prevent output contention, incompatible voltage domains, feedback shorts, protection bypass and route reuse across leases.
- The plan carries `hardware_profile_id`, `scan_epoch`, `lease_id` and `plan_hash`. The driver reports `applied` separately, with evidence for the matching plan and execution generation.

---

## 10. Safety, Topology, and Functional Verification

### 10.1 Three independent result types

| Result | Meaning | May block Run? |
|---|---|---|
| `SafetyVerificationResult` | `SAFE / UNSAFE / INDETERMINATE`; shorts, contention, polarity, current, voltage, protection, stale evidence | **Yes**: UNSAFE/INDETERMINATE blocks |
| `TopologyVerificationResult` | `MATCH / MISMATCH / UNKNOWN / AMBIGUOUS`; missing/extra nets/components, orientation, pin map, reference confidence | **No**: diagnostics and grading only; cannot by itself grant or revoke Run permission |
| `FunctionalVerificationResult` | `PASS / FAIL / PARTIAL / NO_RESPONSE / NOT_EVALUATED`; measured versus expected truth table/timing/waveform | Appears **after** execution or as NOT_EVALUATED; never a pre-Run gate |

### 10.2 Execution Decision policy

```text
BLOCK_SAFETY
BLOCK_UNEXECUTABLE
RUN_NORMAL
RUN_GUARDED
```

| Proven condition | Decision |
|---|---|
| VCC–GND short, overcurrent, dangerous output contention, protection failure | `BLOCK_SAFETY` |
| `SafetyVerificationResult = INDETERMINATE` | `BLOCK_SAFETY` (fail closed) |
| Missing/unroutable required endpoint, unavailable station/driver, resource conflict | `BLOCK_UNEXECUTABLE` |
| Safe + wrong IC/wiring, but all required endpoints are routable | `RUN_NORMAL` or `RUN_GUARDED`, depending on profile |
| Safe + opaque component + proven executability/protection | `RUN_GUARDED` |
| Safe + topology mismatch confined to grading content | **Not blocked automatically** |
| Analog physical request to an unsupported digital station | `BLOCK_UNEXECUTABLE` |

**Important:** `SAFE + WRONG → RUN` is shorthand for **SAFE + EXECUTABLE + AUTHORIZED + protection verified → RUN**. It does not waive executability, permissions, lease state or interlocks.

Both Run modes require verified protection and guarded physical power-up. The station enforces separate scan, DUT power and experiment stimulus permits with their own enable conditions.

### 10.3 Electrical Safety Envelope

**Actual hardware protection** required before enabling `physicalDigital=true`:

```text
Supply → eFuse/current limit → current sense → fast cutoff → breadboard
FPGA drive → protected buffer/level interface → current limiting → route → node
```

Minimum requirements:

- Current-limited supply, calibrated sensor, overcurrent trip and independent fast cutoff.
- Default power OFF, route OPEN, drive high-impedance and no unintended startup pulse.
- Guarded staged power-up with active monitoring and fault latching.
- FPGA heartbeat, watchdog, per-output enable, contention detection and safe reset/Stop.
- HardwareProfile with voltage/current ranges, power domains, maximum drive and pin/route constraints, grounded in schematics and measurements.
- Interlocks must not depend solely on a software boolean from a browser/server.
- Drivers unable to prove electrical safety must return `UNAVAILABLE`/`INDETERMINATE`, not fabricated SAFE results.

**Without schematics/datasheets/AFE/eFuse/measurements → `TBD_BY_HARDWARE_DESIGN`**, and physical capability remains false/zero. RTL simulation does not certify board power circuitry or fault-response time.

### 10.4 Physical state machine

```text
OFFLINE → IDLE → RESERVED → SAFE_STATE → PRE_CHECK → BOUNDED_SCAN
        → SCAN_OFF → RESOLVE_ENDPOINTS → FREEZE_CANDIDATE_PLAN
        → SAFETY_AND_EXECUTABILITY_CHECK → EXECUTION_DECISION
          ├── BLOCK_SAFETY / BLOCK_UNEXECUTABLE → SAFE_CLEANUP
          └── RUN_NORMAL / RUN_GUARDED → APPLY_AND_VERIFY_ROUTES
              → GUARDED_POWER_UP → READY → ARMED → RUNNING
              → STOPPING → SAFE_CLEANUP
SAFE_CLEANUP → confirmed SAFE_STATE → RELEASE → IDLE
             → unconfirmed safe state → QUARANTINE
```

This diagram describes control order. Safe Stop proceeds independently of capture and functional assessment. Apply and verify routes with DUT power and experiment stimuli disabled. Before power-up, Hardware verifies the fresh scan, exact approved plan and applied routes against the current lease, station/profile and generation. Wiring or configuration changes that invalidate this evidence require a safe stop and fresh preparation. Faults and permit loss inhibit the affected energy sources and start safe cleanup. Keep the station fenced in quarantine when safe cleanup is unconfirmed.

---

## 11. HardwareStation and Capability Discovery

### 11.1 Shared abstraction

```text
HardwareStation
  ├── VirtualHardwareStation
  └── PhysicalHardwareStation
```

The application calls a shared service contract (exact method names are defined by canonical Contracts):

```text
get_capabilities()
validate_configuration()
enter_safe_state()
scan_topology()            # physical; virtual may report NotApplicable
read_runtime_safety()      # physical-only evidence
prepare_execution()
apply_routing()            # physical; virtual may be N/A
configure_clock()/generator()/trigger()
guarded_power_up()         # physical-only
arm_capture()
start_execution()
stop_execution()
read_capture()
reset()
```

Do not force VirtualStation to fabricate scan or electrical safety evidence. Typed `NotApplicable`/`NOT_RUN` outcomes are acceptable.

### 11.2 StationCapabilities is authoritative server/driver data

```text
virtualDigital
virtualAnalog
physicalDigital
physicalAnalogRouting
physicalExecutionSlots
logicAnalyzerChannels
oscilloscopeChannels
generatorChannels
supportedSignalDomains
supportedComponentModels
supportedFpgaProtocols
topologyScanSupported
guardedRunSupported
safetyGateSupported
componentResolutionModes
maxCaptureSamples / maxSampleRate (when measured)
firmwareVersion / hardwareProfileId
```

Frontend **must not** hard-code channel/slot counts or enable controls when capability is zero. `virtualAnalog` does not become true merely because the schema defines ANALOG. `physicalDigital` does not become true just because an RTL scaffold exists.

Public capabilities **must not** expose private MUX addresses, FPGA pins, registers, USB endpoints or detailed routing matrices.

---

## 12. FPGA, Transport, and Instrument Architecture

### 12.1 Mini PC versus FPGA responsibilities

**Mini PC/Rust:** API, authentication, projects, simulator orchestration, compiler, resolver, safety policy, scheduler, waveform conversion, persistence, telemetry and user-level status.

**FPGA/RTL:** Deterministic timing control, clock, trigger, timestamps, digital I/O/route switching, scan coordination, acquisition, DDS and SDRAM/CDC. FPGA does not implement users/authentication/databases/WebSocket.

### 12.2 Development hardware milestones

| Stage | Hardware | Scope |
|---|---|---|
| Prototype | **EP4CE6** | Safe-state control, protocol skeleton, GPIO/clock, basic protected digital routing, small captures; **do not assume SDRAM is available** |
| Target instrument platform | **EP4CE10 + external SDRAM** | Logic Analyzer, 2CH Scope, 2CH Generator, SDRAM arbiter, trigger, CDC; requires actual board/datasheets/pins |
| Future Physical Analog | Analog switches/relays/crosspoints + dedicated AFE/protection | Digital FPGA baseline must not be treated as already supporting analog routing |

Analog instruments require external hardware:

```text
Oscilloscope CH1/CH2: Probe → AFE → external ADC → FPGA capture → SDRAM
Generator CH1/CH2:  FPGA DDS → external DAC → filter/output/protection
Logic Analyzer:     Digital inputs → FPGA trigger/sampler → SDRAM
```

RTL interfaces alone **do not** establish the presence of an ADC, DAC, validated sample rate or enabled channels. Publish capability only after real measurements and calibration.

### 12.3 SDRAM/CDC/trigger

- Scope capture, Logic Analyzer and Host Readback share **one SDRAM Arbiter**; do not give three independent masters direct control of DRAM.
- Separate clock domains: System, Acquisition, DDS, SDRAM, USB/host; cross using verified CDC synchronizers/FIFOs/handshakes.
- Reset release, PLL lock, timing constraints, setup/hold and resource budgets require Quartus/timing reports on the target part.
- Overflow/backpressure must return errors or partial status according to contract; never silently truncate a waveform and report complete.
- Do not publish maximum sample rates/channels without schematics and actual measurements.

### 12.4 Transport and protocol

```text
FpgaTransport
  ├── SerialTransport      (prototype, when built)
  ├── UsbSpiTransport      (prototype, when built)
  ├── Usb3FifoTransport    (high-throughput target)
  └── EthernetTransport   (future option)
```

Conceptual frame:

```text
SOF | ProtocolVersion | Command | Sequence | Length | Payload | CRC | EOF
```

Command family: `GET_CAPABILITY`, `SET_CLOCK`, `SET_ROUTE`, `WRITE_OUTPUT`, `CONFIGURE_TRIGGER`, `ARM_CAPTURE`, `START_CAPTURE`, `READ_CAPTURE`, `STOP`, `RESET`, `SAFE_STATE`.

**Mandatory:** Sequence/generation, timeout, response/status, CRC/integrity, command authorization, version negotiation, replay rejection and fail-safe disconnect. Physical bytes, bit ordering and CRC polynomial **must be frozen** only after a firmware/board implementation has been verified; an RTL scaffold cannot prove a working physical ABI.

---
## 13. Browser Workbench and Graphics Performance

### 13.1 Frontend modules

```text
WorkspaceShell
├── CircuitEditor / Graph Commands / Undo-Redo
├── ComponentLibrary / ProjectManager / AssignmentWorkspace
├── ThreeScene (SceneManager, WireLayer, Picking, Gizmo, Camera)
├── InstrumentPanel (Logic Analyzer, Oscilloscope, Generator)
├── APIClient (typed SDK, auth, idempotency)
├── WebSocketClient (seq, reconnect/resync)
└── ResultPanel (Safety / Topology / Functional, separately)
```

State separation:

| State | Owner |
|---|---|
| UI/panels/forms | Pinia |
| Graph semantics, commands, history | CircuitGraph engine |
| Camera/selection/picking/transient graphics | Three.js SceneManager |
| Remote/auth/operation progress | Typed client stores + backend authority |
| Hardware safety, lease, applied plan | **Server/Hardware is authoritative** |

Pinia must not replace the simulator/database/FPGA. Three.js must not create a net just because visual wires intersect. `DesignGraph` changes only when the user commits an editor action; a placement ghost is not an existing component.

### 13.2 Asset and UX performance

- GLB → Meshopt → KTX2/Basis → browser cache; retain asset ownership/provenance.
- Use InstancedMesh for breadboard holes and repeated pins; lazy-load/LOD models that are not yet needed.
- Do not re-render when graph/camera have not changed; pause when the canvas is hidden or has zero size; handle WebGL context loss.
- Initially bound Undo/Redo snapshot size/count; a completed drag is one entry, not one snapshot per animation frame.
- Virtual and Physical modes share panels/waveform bindings; distinguish them through capabilities, status, and measured/provenance labels.

---

## 14. Authentication, Authorization, Assignments, and Data Access

### 14.1 Authentication and session control

- Roles: `Student`, `Instructor`, `Administrator`; RBAC and ownership are separate checks.
- In standalone scope, Remote authenticates Lab-local principals. In an integrated scope after cutover, Remote verifies Platform identity and applies Lab permissions. Use password hashing where local credentials exist, Secure and HttpOnly cookies for browser sessions, and Origin/CSRF controls for mutations. Production uses HTTPS and PostgreSQL.
- Bind the controlling tab/client identity to a session/lease for Run/Stop control; viewers may access only authorized data.
- The API must distinguish request accepted, command running, measurement complete and safe Stop acknowledged.
- Reconnect/resynchronization may observe previous state but **must not** automatically replay `Run`/`Apply`/`Create`.
- Permission to edit source code or view a project **does not** automatically grant permission to export somebody else's data. Before an ambiguous export, establish the actor, owner/scope, allowed fields and destination.

### 14.2 AssignmentDefinition

```text
assignmentId
ReferenceDesignGraph? / ReferenceTopologyGraph?
ExpectedComponentSet?
InstructorPinProfiles?
LabEndpoints
ExpectedBehavior? (truthTable/timing/waveforms)
AssessmentPolicy (STRICT_TOPOLOGY / FUNCTIONAL_ONLY / TOPOLOGY_AND_FUNCTION)
DisclosurePolicy (observed_only / instructor-approved detailed)
```

Students may see only assignment content and feedback approved for disclosure. Reference schematics, mappings and secret answers must not leak through REST, WebSocket, capture, SDK or error details. Disclosure policies change **what information may be shown**, not safety algorithms or grading rules.

### 14.3 Learning evaluation

```text
Safety:   SAFE
Topology: MISMATCH
Function: FAIL
```

This is a **valid state** when a circuit was allowed to run safely but the student wired it incorrectly. The system reports actual signals and educational feedback instead of fabricating outputs that match the expected answer.

---

## 15. Scheduler and 30–50-User Load

Target capacity:

```text
30–50 concurrent connected users
       !=
30–50 simultaneous physical executions
```

- Virtual sessions use a bounded worker pool, with workloads measured against CPU/RAM/capture; do not run CPU-bound engines directly in the API event loop.
- A qualified shared physical breadboard/routing fabric has **one physical execution slot by default**. Unqualified profiles expose zero slots. Other authorized users may edit, queue and read results while that fabric has a single controller.
- Admission: FIFO + per-user rate limits + maximum runtime + lease TTL; instructor priority may be introduced only under an explicit, bounded policy.
- Serialize even under simultaneous bursts; do not rely on the assumption that students rarely press Run at the same time.
- Disconnect, lease expiry, Stop or crash → attempt safe cleanup; on failure → quarantine. Never allocate resources to another user until safe release is confirmed.
- Set `physicalExecutionSlots > 1` only when each partition is **electrically isolated**, with its own safety/routing/resource profile and HIL validation; changing a JSON number is not enough.
- All queues, workers and waveform streams need backpressure; benchmark 30–50 connected users using measured throughput, tail latency and failure recovery.

---

## 16. Capture, Waveforms, and Storage

### 16.1 Unified Result Model

```text
ExperimentResult
├── experimentId / operationId / stationId
├── executionState / executionDecision
├── safetyVerification? / topologyVerification? / functionalVerification?
├── digitalSignals / analogCaptures
├── instrumentStatus / timestamps / captureMetadata
├── warnings / errors / rawObservations reference
└── artifact/protocol/profile provenance
```

Depending on Virtual/Physical mode, optional `?` fields may be `NOT_APPLICABLE`/`NOT_EVALUATED`; do not infer SAFE or PASS from absence.

### 16.2 Waveform storage and transport

- Store large captures in file/object storage; PostgreSQL holds only ownership, path/ID, sample metadata, checksums and retention data.
- Use `WebSocket Binary` or authorized streaming/downloads for large waveforms; do not place raw ADC/NCAP samples into enormous JSON payloads.
- Minimum header: `capture_id`, `instrument`, `channel`, `sample_rate`, `sample_count`, `timestamp`, `encoding`, `protocol_version`.
- Contract-dependent payloads: `Int16`, `Uint16`, `Float32`, digital events/RLE; never decode as a different type on version mismatch.
- Enforce quotas, storage retention, access control, signed/authorized reads, resume/resync and cleanup.
- Distinguish NCAP v1 digital from v2 sampled; codec support does not imply a producer already emits sampled analog captures.

---

## 17. Edge Deployment

### 17.1 Target Mini PC

| Component | Target |
|---|---|
| CPU | Intel Xeon W-1370 |
| RAM | 16 GB (increase only when load testing justifies it) |
| SSD | Initial 256 GB; 512 GB–1 TB as captures/assets grow |
| Production OS | Linux |
| Proxy | Nginx |
| Application | Rust Remote + Rust Hardware, supervised simulator worker |
| Database | PostgreSQL; SQLite for tests/prototypes |
| Service manager | systemd |
| Python | Testing/benchmarking tools outside production runtime only |

### 17.2 Services

```text
nginx.service
postgresql.service
cirnet-remote.service
cirnet-hardware.service
[supervised cirnet-simulator worker processes]
```

Use systemd hardening/permissions, local-only gRPC, health/readiness, controlled startup order and fail-safe service restart. Do not expose private Hardware RPC to the WAN. Browsers render Three.js on users' devices rather than receiving server-rendered 3D from the Mini PC.

Docker Compose is appropriate for development/integration tests but is not mandatory in production. Configure endpoints/credentials according to environment/service policy, and do not commit secrets. Database migrations/backups and capture consistency form part of release/rollback.

### 17.3 Observability

Every request/experiment/worker command carries `correlation_id`, `principal_id` (where permitted), `station_id`, `lease_id`, `operation_id`, `plan_hash`, release-set SHA and endpoint/safety generation. Logs must not expose passwords, tokens, instructor answers or private waveforms. Metrics include worker queue, Run latency, capture bytes, cutoff/fault events, timeout, quarantine, WS lag, database health and resource utilization.

---

## 18. Release Engineering, Provenance, and Integration of Six Repositories

### 18.1 Release naming

General `cirnetlab` convention for **new releases**:

```text
Git tag            = <repo_name>-<version>
GitHub Release     = <repo_name>-<version>
Example            = cirnet-remote-0.2.4
```

`0.2.4` is **only an example of formatting**, not a claim that this version has been released. Preserve historical tags using older conventions.

### 18.2 System release set

Remote owns the selected artifact manifest; each candidate/release set should contain:

```text
systemReleaseId
apiArtifact: repository + sourceRevision + binarySHA256 + version
interfaceArtifact: sourceRevision + staticBundleSHA256 + sdkDigest
contractsArtifacts: canonicalSchemaFingerprint + Rust/SDK package digests
hardwareArtifact: binarySHA256 + hardwareRpcVersion + profile
simulatorArtifact: executableSHA256 + protocolVersion + backendKind
fpgaArtifact: sourceRevision + firmwareDigest? + boardProfile? + hardwareEvidence?
compatibility: API / Graph / RPC / NCAP / FPGA / migration constraints
rollback: matching backups + safe drain/quarantine + no command replay
```

Optional `?` fields are absent or `UNAVAILABLE` in a simulator-only release; never invent a firmware hash when only an RTL scaffold exists.

### 18.3 Multi-repository release workflow

1. Contracts owner publishes verified schemas/SDK/RPC/fixtures.
2. Simulator, Hardware and Interface build against exact pins, including the SDK for Interface. Verify build digests, worker protocol, cancellation and error parity.
3. Remote assembles the candidate manifest and pins all artifacts for the declared test scope.
4. Interface and Remote run the real-browser tests against that same candidate set.
5. An FPGA physical release requires bitstream/board/timing/HIL/safety evidence; never label scaffold-only source as hardware-supported.
6. Complete the acceptance matrix for the pinned candidate, distinguishing Windows and Linux target coverage according to each claim. Remote selects the release set after the applicable gates pass.
7. Package artifacts, manifest and migration/rollback notes; obtain separate authorization before tagging/publishing.
8. Roll back only to a compatible artifact set with database/capture backup, stop admission, confirm safe cleanup/quarantine and do not automatically replay Start.

Selecting the “right branch” does not replace selecting the right artifact. Do not upgrade SDKs/workers independently without checking consumer pins and release locks.

---

## 19. Verification Strategy and Definition of Done

### 19.1 Test layers

| Layer | Owner | Coverage |
|---|---|---|
| Unit/domain | Each repository | Typed errors, state machines, bounded resource behavior and validators |
| Contract/conformance | Contracts + consumers | Schemas, generated SDK, gRPC descriptors, NCAP and ABI/version compatibility |
| Worker integration | Hardware + Simulator | Real worker process, digest pin, CNP1/CNP2 or successor, Stop, timeout and corrupt frames |
| API/service integration | Remote + Hardware | HTTP/gRPC/DB/WS, ownership/lease, operation identity and capture |
| Browser E2E | Interface + Remote | Real workbench editing, authorization, Run, results/reconnect and waveform |
| Physical control simulation | FPGA | RTL tests for reset, watchdog, faults, permit loss and default-open routing |
| Hardware-in-the-loop | Hardware + FPGA | Board-level electrical safety, timing, fault injection and real transport/ADC/DAC |
| Load & soak | Remote + all | 30–50 connected clients, workers/queues, Stop latency, storage/capture stability |
| Release acceptance | Remote integration owner | Exact selected artifact set, Linux target, migration, rollback and provenance |

### 19.2 Do not conflate evidence types

```text
RTL simulation PASS   != physical electrical safety PASS
Contract check PASS   != real end-to-end service PASS
Windows candidate PASS != Linux production PASS
Digital/DC model PASS != full analog SPICE PASS
Source HEAD available != published release artifact available
```

Python benchmarking scripts run only in test environments. Do not open a production path that invokes Python to “get test data,” and do not mock FPGA/capture while claiming physical verification.

### 19.3 Definition of Done for a feature

- Contract owner and compatibility boundary are identified.
- Code changes belong to the correct repository, with no unintended cross-repo modifications.
- Happy-path and negative tests cover the actual boundary and selected versions.
- Failure/cancel/Stop plus security/authorization have been tested.
- Evidence records exact source/artifact/release-set identities, logs/exit codes and environment.
- Update canonical EN/VI documentation and appropriate changelogs without unnecessary Markdown duplication.
- Public status is `VERIFIED` only in the environment genuinely tested; other environments remain `NOT_RUN`.

---
## 20. Development Roadmap: Optimized Priorities and Acceptance Gates

The stages below **do not** require implementing every abstraction simultaneously. Begin with a measurable vertical slice, then expand in dependency order.

| Phase | Goal | Primary owner | Exit gate |
|---|---|---|---|
| **A — Virtual Digital contract baseline** | Stabilize the graph, assignment, virtual plan, capabilities, RPC, results and NCAP needed for B, with Rust/TS SDK parity in that scope. | Contracts + Remote + Interface | Generated fixtures, old/new compatibility and exact selected-version tests PASS |
| **B — Virtual Digital E2E (Rust services + C++ Simulator)** | Web → Rust Remote → Rust Hardware → Worker → Web with digital gates/LED/clock/NCAP | All except physical FPGA | Gates VD-01…VD-10 PASS on a pinned artifact set; no mock at a boundary claimed to be real |
| **C — Expand and optimize C++ Simulator** | Extend the C++ worker already used in B. Preserve protocols, parity and artifact pins. | Simulator + Hardware | Functional, error, Stop, performance and rollback evidence |
| **D — Virtual Instruments** | **Virtual** Logic Analyzer, generator/scope with accurate capabilities | Simulator + Interface + Contracts | Typed samples, bounded captures, UI and E2E tests |
| **E — Virtual Analog** | C++ SPICE adapter + native solver, analog waveforms and mixed bridges when ready | Simulator + Contracts | Real analog solver fixtures and licensing/provenance; no capability overclaim |
| **F — Edge multi-user qualification** | Extend the baseline admission, scheduler and Linux coverage exercised in B to the target edge deployment and load. | Remote + Hardware | Load of 30–50 connected clients, recovery and safe Stop in the virtual lane |
| **G — FPGA Emulator & EP4CE6** | FPGA protocol emulator, RTL control and small digital resources | FPGA + Hardware | CRC/timeout/Stop/RTL parity; physical power stays unavailable without board evidence |
| **H — Physical Safety & Discovery** | Breadboard PCB/scan matrix/protected drive, HYBRID resolver, physical endpoint bindings | FPGA + Hardware + Contracts + Remote | Board HIL demonstrates SAFE/INDETERMINATE, fault cutoff and observed-physical routing |
| **I — Physical Digital Lab** | Protected routing, guarded execution, real IC captures and three separate verifications | Hardware + FPGA + Remote + Interface | Safe-but-wrong circuits may run; unsafe blocked; no automatic rewiring; HIL + teaching tests |
| **J — EP4CE10 Instruments & Scale** | SDRAM, Logic Analyzer, 2CH Generator, 2CH Scope, CDC/AFE calibration, multi-user queue | FPGA + Hardware + Remote | Actual sample/trigger/timing/electrical evidence and resource/load gates |
| **K — Future Physical Analog** | Analog routing/protection/physical compiler adapters | Hardware + FPGA + Contracts | Only after analog electrical safety design and measured HIL |

**First acceptance milestone:** Stabilize the contracts needed for B and deliver the real C++ Virtual Digital slice with §6.5 evidence, including resource bounds and the target OS for each claim. C, D, E and F expand that baseline according to their own acceptance gates. Physical work follows its own prerequisites, from protocol/board preparation (G) through protection and discovery (H) to Physical Digital (I) and instrument scale (J). Virtual Analog (E) can proceed independently of that physical path. Physical Run requires the relevant safety hardware, endpoint contracts and verified station profile.

### 20.1 Most important teaching acceptance scenario

A student's circuit has a different placement from the instructor's and contains an intentionally incorrect wire, yet protection proves safety and all required endpoints remain executable:

1. Scan obtains the actual observed graph and **does not copy reference wiring**.
2. Topology: `MISMATCH`; Safety: `SAFE`.
3. `ExecutionDecision`: `RUN_GUARDED` or `RUN_NORMAL`, according to profile.
4. Hardware routing connects only FPGA stimuli/sensing infrastructure to observed endpoints.
5. FPGA measures the real behavior of the incorrectly wired circuit; Functional: `FAIL`, `PARTIAL` or `NO_RESPONSE`, depending on observations.
6. UI displays separate Safety/Topology/Function outcomes and only the information instructors authorize for disclosure.
7. Stop/timeout → `SAFE_STATE`; resources are released only after confirmation.

This is the **core acceptance scenario** for Lab-First. If the system uses a MUX to reconnect the circuit according to the reference schematic, this scenario **FAILS**, even if the UI displays PASS.

---

## 21. Architectural Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Continuity scan cannot reliably identify IC/pins | Incorrect mapping, dangerous power application | HYBRID, evidence confidence, profile limits and fail-closed `INDETERMINATE` |
| Reference endpoint confused with observed node | Misrouting and fictitious experiments | Versioned bindings, reference≠student tests and required-binding gate |
| Compiler automatically repairs student wiring | Destroys the learning objective | Infrastructure-only plan; negative fixtures proving no automatic rewiring |
| Software announces SAFE without interlocks | Electrical hazard | Independent hardware envelope, HIL-measured cutoffs, capability unavailable until verified |
| Unit tests or Windows-only tests treated as production PASS | Regression on Linux deployment | Exact-artifact E2E and Linux target/load acceptance |
| C++ Simulator upgrades change timing/error behavior | Incorrect educational results | Regression/parity, pinned workers, staged upgrades and explicit rollback |
| Virtual Analog advertised before SPICE works | Misleading expectations | `virtualAnalog=false`, `ANALOG_UNAVAILABLE`, actual solver tests |
| SDK/Contracts/worker mismatch | Crash or corrupted data | Artifact/digest pinning, handshakes, conformance matrix and fail-at-startup |
| Multi-user physical contention | Route conflicts and electrical danger | Leases, single execution slot, FIFO, interlocks and quarantine after uncertain cleanup |
| Instructor answers or another user's waveforms leak | Unauthorized data disclosure | RBAC + ownership + disclosure projections + export authorization |
| FPGA resource/timing overclaims | Instruments malfunction | Part-specific synthesis/timing, SDRAM arbiter and ADC/DAC/AFE bench measurements |
| Multiple contradictory copies of documentation | AI Agents use the wrong authority | Canonical architecture, links, update-existing policy, source evidence and duplicate review |

---

## 22. Architectural Decision Records (ADR)

### ADR-CIRNET-001 — Lab-First: Safety is a hard gate; correctness is not

**Decision:** Block if unsafe/indeterminate or not executable; wrong-but-safe circuits may run under the protection policy. Topology is only for diagnostics/grading; functional assessment uses measured behavior after Run.

**Consequence:** Require independent hardware interlocks and `SafetyVerificationResult`; do not merge correctness checks into execution admission.

### ADR-CIRNET-002 — Rust core services and C++ Simulator

**Decision:** Rust is the default for Backend, Hardware and Contracts runtime; **C++ is the default for Simulator**. TypeScript serves Web, HDL serves FPGA and Python is reserved for testing/benchmarking. The pinned, supervised C++ worker remains the official primary implementation.

**Consequence:** C++ Simulator upgrades must not silently change public behavior or erase historical releases; parity measurements and rollback are required.

### ADR-CIRNET-003 — CircuitGraph versus DetectedTopologyGraph

**Decision:** User DesignGraph is the truth of logical intent; the observed physical graph is the truth of the wired circuit. Neither may be transformed into the other without supporting measurement/resolution evidence.

**Consequence:** Frontend cannot confirm physical connectivity from coordinates; Physical Compiler cannot fix student wires.

### ADR-CIRNET-004 — HYBRID resolution as the first Physical Digital default

**Decision:** Instructor profile provides pin semantics; scanning detects actual placement; signatures assist only when electrical safety is established.

**Consequence:** Students can position ICs freely without a false claim that every IC can be identified.

### ADR-CIRNET-005 — Endpoint binding requires observed physical evidence

**Decision:** Reference IDs and observed IDs belong to separate namespaces. Bind required endpoints to actual observed nodes before routing. Ambiguity confined to grading can coexist with proven safety and executability.

**Consequence:** Compiler consumes `TopologyResolutionResult`, not a reference graph as a shortcut for routing.

### ADR-CIRNET-006 — Infrastructure-only physical routing

**Decision:** FPGA routing connects only **protected control/stimulus/sense/power infrastructure** to resolved endpoints; it never repairs the student's internal circuit.

**Consequence:** Functional results reflect the correctness of the actual wires; software-corrected topology violates the architecture.

### ADR-CIRNET-007 — Contract-driven, pinned cross-repository integration

**Decision:** Contracts owns the canonical source; Remote owns the pinned release set; E2E tests/releases select exact artifacts and exercise real boundaries.

**Consequence:** Repository HEAD is not a selected system; generated consumer bindings must not be patched in place.

### ADR-CIRNET-008 — Capability flags express measured support

**Decision:** Physical Digital, Virtual Analog, Scope/Generator, FPGA channels and execution slots are enabled only with matching implementation and verification.

**Consequence:** Interface adapts to capabilities rather than displaying fabricated samples or assuming physical safety.

### ADR-CIRNET-009 — Bounded multi-user physical execution

**Decision:** Many users may connect or queue; the baseline shared physical fabric has only one execution slot, expandable solely when electrically isolated partitions are demonstrated.

**Consequence:** Scheduler, leases and quarantine are mandatory; an early message broker is not required.

### ADR-CIRNET-010 — Versionless architecture name, versioned artifacts

**Decision:** The Lab architecture is named **CIRNET Lab-First** with no version suffix. Each repository, schema, RPC, protocol, hardware profile and release is versioned independently.

**Consequence:** Subordinate documents link to the canonical architecture; do not create conflicting competing “official architectures” in each repository.

---

## 23. Canonical Documentation and Cross-Repository Update Process

This EN/VI pair is the [canonical reference](../../en/README.md#architecture) for Lab execution and electrical safety. The [Ecosystem Architecture](cirnetlab-architecture-en.md) governs Organization boundaries and cross-domain integration. Owner documentation links to the applicable authority and records the contract or artifact revision it implements.

- Lab architecture: this document and the paired [Vietnamese edition](cirnet-architecture-lab-first-vi.md).
- Implementation/API/system release authority: `cirnet-remote`.
- Canonical schemas/SDK/RPC/NCAP: `cirnet-contracts`.
- Browser UI: `cirnet-interface`.
- Runtime stations/transport: `cirnet-hardware`.
- Workers/models: `cirnet-simulator`.
- Board/RTL/timing/bench: `cirnet-fpga`.

For cross-repository changes:

```text
Read common CIRNET Lab-First architecture
  → identify owning repository and canonical contract
  → inspect currently selected artifact/version
  → write scoped plan + acceptance criteria in owner's existing convention
  → implement smallest change
  → verify consumer/producer compatibility and safety
  → sync EN/VI docs/changelog at owner
  → update release-set only after integrated acceptance
```

Prefer updating existing documentation. Do not create `architecture-final.md`, `architecture-v2.md` and `architecture-latest.md` simultaneously for the same scope. Archive an earlier version only when necessary, preserving provenance. Repositories may retain frozen release/architecture histories; naming standardization must not compromise traceability.

---

## 24. Architecture Approval Criteria and Open Issues

This document standardizes **design decisions**; it does not itself prove that the product is complete. The following items remain `OPEN / TBD / NOT_RUN` until supported by evidence:

1. **Default C++ Simulator:** Needs artifact-pinned E2E acceptance covering Digital/DC, cancellation and benchmarks of the selected release set.
2. **Virtual Analog:** Native SPICE solver has not been qualified in the selected runtime; model licensing, adaptation and E2E remain open.
3. **Physical station:** Needs board schematics, voltage/current profiles, eFuse/current sensing/fast cutoff, protected drive and actual hardware validation.
4. **Automatic Topology Discovery:** Needs PCB node maps, scan front-end and real measurements; universal identification of every IC is not realistic.
5. **FPGA transport/bitstream:** Needs actual part/pin assignment, physical protocol ABI, clock/reset constraints, synthesis/timing and HIL.
6. **EP4CE10 instruments:** Need a selected ADC/DAC/AFE/SDRAM BOM, CDC, analog accuracy, calibration and measurement.
7. **Linux Mini PC acceptance:** Needs 30–50-client load and physical lease/Stop latency measured in the target environment.
8. **Exact cross-repository release:** Needs a new manifest pinning the exact Rust/C++/TS artifacts and real integration; historical release locks must not be silently modified.

### Implementation architecture closure criteria

- Architectural decisions are reflected in Contracts and each owning repository.
- A **Virtual Digital E2E vertical slice** passes with a real browser and selected real processes, pinned artifacts and tested rollback.
- Every hardware capability has evidence of the appropriate type, with no mixing of **Safety/Topology/Functional** outcomes.
- Rust is the verified default for Backend/Contracts/Hardware; C++ Simulator is the default and must be pinned and verified against the selected release set.
- Python is absent from new production execution paths, except for isolated historical reference lanes used for verification.
- EN/VI documentation and release notes do not contradict each other regarding capabilities or provenance.

---

## 25. Summary of Development Priorities

Deliver the contract-driven Virtual Digital slice with a pinned C++ worker first. Use §20 for the separate Simulator, analog, load and physical milestones. Each capability is enabled within the scope of its verified evidence.

> **Conclusion:** **CIRNET Lab-First** is the architecture shared by six repositories, built around Rust core services and **C++ as the default Simulator**, unified contracts, virtual-to-physical operation and safe experimentation. Virtual Digital E2E is the first verification milestone. When moving to real circuits, the system routes only **protected measurement/control infrastructure** to verified endpoints on **the student's actual circuit**; it **must not repair student wiring using FPGA routing**. Real measured behavior, rather than software-corrected outputs, provides the basis for learning assessment.

---

## Appendix A — Mapping of Inherited Source Decisions

| Topic in the source document | Standardized location in this document |
|---|---|
| Web-first, Lab-first, Safety versus Correctness | §0, §1, §10, ADR-001 |
| DesignGraph / ValidatedGraph / ExecutionPlan | §5 |
| Domain separation, CircuitGraph versus Three.js | §5, §13 |
| Physical placement and active electrical topology scanning | §8 |
| AssignmentDefinition, LabEndpoint, Instructor Profile | §8, §14 |
| Safety/Topology/Functional Verification and Run policy | §10 |
| Physical compiler and routing solver | §9 |
| HardwareStation, capabilities and Hardware Service | §11, §3 |
| FPGA protocol, EP4CE6, EP4CE10, SDRAM, Scope, Generator | §12 |
| Digital/Analog simulation and worker pool | §6, §7 |
| API/WS/capture/DB/auth | §3, §14, §16, §17 |
| 30–50 users / leases / scheduler | §15 |
| Roadmap and priority teaching experiment | §20 |
| Architectural decisions / ADR | §22 |

## Appendix B — Verification Boundaries and References

- **Primary content source:** `cirnet-Remote_Architecture_Lab-First.md`, supplied by the user (the earlier Lab-First proposal before Organization-wide standardization).
- **Repository status verification source:** `master` branches of six `cirnetlab` repositories reviewed on 2026-11-15, especially `cirnet-remote/docs/en/architecture/lab-first-v3-1.md`, `system.md` and `native-migration-status.md`; and README/architecture documents of Interface, Contracts, Hardware, Simulator and FPGA.
- **Deliberate differences from the old proposal:** Removed the former product name and any version suffix from the architecture name; migrated Backend/Contracts/Hardware from Python to Rust while keeping C++ as the default Simulator language; Virtual Analog is not yet VERIFIED; added observed endpoint resolution and prohibited the Physical Compiler from repairing student wiring; required artifact-pinned, real Virtual Digital E2E.
- **No automatic changes:** Source repositories, CI, bitstreams, releases, schemas, SDK artifacts and deployed Mini PC installations are not modified by this document. It establishes development direction and acceptance criteria; implementation requires separate PRs, tests and releases in the relevant owning repositories.
