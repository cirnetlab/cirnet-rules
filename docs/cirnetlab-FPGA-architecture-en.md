# CIRNET FPGA — Hardware, RTL, and Engineering Education Architecture

> **Architecture name:** CIRNET FPGA Architecture (no version suffix in the architecture name).  
> **Organization:** [`cirnetlab`](https://github.com/orgs/cirnetlab/repositories) · **Owning repository:** [`cirnet-fpga`](https://github.com/cirnetlab/cirnet-fpga).  
> **Document language:** English · **File:** `cirnetlab-FPGA-architecture-en.md`.  
> **Ecosystem architecture:** `cirnetlab-architecture-en.md` (**CIRNET — STEM & Engineering Learning Ecosystem**).  
> **Domain-specific electronics specification:** `cirnet-architecture-lab-first.md` (**CIRNET Lab-First**).  
> **Source review date:** 2026-11-15. **Status:** target architecture, NOT a certification that the hardware or RTL has passed acceptance testing.

## Purpose and scope

This specification brings the earlier FPGA architecture into the **CIRNET** ecosystem: FPGA is an engineering capability that learners can study, simulate, program, test, and measure through CIRNET Studio/IDE/Blocks/Lab/Learn/Silicon. At the same time, the **FPGA controlling the physical laboratory** must continue to honor every electrical and safety invariant established in CIRNET Lab-First. These two FPGA applications share artifact/provenance models but **do not share authority over physical electrical operations**.

**Document boundary:** FPGA fabric architecture, peripherals connected directly to the FPGA, protected I/O, routing/scanning, instruments, capture/memory/transport, RTL verification, and interfaces to other repositories. This document does not redefine educational business backends, public APIs, grading rubrics, or user permissions.

### Current implementation context that must be respected

- On `master`, `cirnet-fpga` currently contains a **Lab-First RTL scaffold**, including `fpga/experiment-controller/rtl/top.v`, `fpga/instrument/rtl/top.v`, testbenches under `fpga/simulation/`, verification tooling, lockfiles, and EN/VI documentation. Its README describes a **0.2.2 source scaffold candidate**; it does **not** claim board-ready firmware.
- `cirnet-hardware` owns the **Rust HardwareStation, private transport, Stop, station lifecycle, and worker supervision**; `cirnet-remote` owns authorization, scheduler/leases, experiments, and system-release selection.
- `cirnet-contracts` owns canonical schemas/RPC/ICD/SDK. **External plan names must not be changed unilaterally inside RTL**; `FpgaExecutionImage` in this document is merely a **proposed FPGA-internal structure**.
- EP4CE6, EP4CE10, 64 MiB SDR SDRAM, Logic Analyzer, 2-channel Oscilloscope, and 2-channel Function Generator are **targets/candidates**, not demonstrated capabilities. Real firmware/bitstream artifacts require evidence for the exact board revision, timing, pins, electrical protection, and Hardware-in-the-Loop (HIL) validation.
- Python is allowed for **testing, benchmarking, and parity verification** within CIRNET infrastructure; production services use Rust, the default circuit simulation engine uses C++, the Web uses TypeScript, and FPGA RTL uses Verilog/SystemVerilog. **Learner-authored programming languages** belong to a separate IDE/Toolchains scope with appropriate sandboxing and target selection.

### Inviolable rules

> **Safety ≠ Correctness.** FPGA executes real signals within demonstrated electrical limits; it does not decide whether a student's answer is correct, fabricate measurements, or **repair student wiring/ICs through FPGA routing**. The Physical Compiler routes only **protected power/stimulus/measurement infrastructure** to actual endpoints resolved from observations of the physical student circuit.

> **Build ≠ Verified hardware.** RTL compilation/simulation is not electrical measurement, timing closure, or certification of an instrument.

| Classification | Meaning |
|---|---|
| `TARGET`/`DESIGNED` | An established design intention; working code is not implied |
| `IMPLEMENTED` | Corresponding RTL/source exists |
| `TESTED` | Specific tests and source/artifact-linked evidence exist |
| `VERIFIED` | Verified on the stated target/board/environment |
| `RELEASED` | An artifact is pinned, published, and consistent with its compatibility manifest |
| `UNAVAILABLE`/`BLOCKED` | A capability is absent or cannot be operated safely |

### Reference hardware scope

| Component | Planned scope (not a published specification) |
|---|---|
| FPGA prototype | EP4CE6E22C8N; protected control/scan/drive/sense; no assumption of external SDRAM |
| Lab instrument board | EP4CE10, package selected only after pin/clock/resource closure |
| SDRAM target | **64 MiB** (= 67,108,864 bytes = 512 Mibit), subject to confirmation of the actual part/BOM |
| Instruments | Logic Analyzer; 2-channel Oscilloscope; 2-channel Function Generator, requiring external ADC/DAC/AFE |
| Shared physical lab | One active electrical execution context on one shared fabric |
| Physical analog routing | **Must not be inferred** merely from having oscilloscope/generator ADC/DAC/AFE |

### Reference conventions

- **[ECOSYSTEM]**: the general architecture `cirnetlab-architecture-en.md`.
- **[LF]**: `cirnet-architecture-lab-first.md`; domain authority for observed topology, safe Run, endpoint resolution, and the prohibition on repairing student wiring.
- **[CONTRACTS]**: schemas/SDK/Hardware RPC/ICD **selected by real artifacts** from `cirnet-contracts`.
- **[F0]**: the historical FPGA design draft supplied for this work; it informs technical detail but **does not** define the current implementation authority or wire ABI.
- **[H1]–[H5]**: manufacturer references in the appendix; they do not establish that a CIRNET board has achieved those specifications.

Every `MUST`/`MUST NOT` expresses a proposed architectural requirement for implementation. It must not be recorded as implemented without test/bench evidence.

---

# 1. Hardware invariants to preserve

**Inherited from:** CIRNET Lab-First [LF] and the foundational FPGA design [F0].

1. `SAFETY != CORRECTNESS`: a topology mismatch is not an input to energy-permit logic.
2. `UNKNOWN_COMPONENT` is not automatically `UNSAFE`, but neither is it automatically `SAFE`.
3. `SafetyVerificationResult.status = UNSAFE` or `INDETERMINATE` must not grant permission to energize.
4. `BLOCK_SAFETY`, `BLOCK_UNEXECUTABLE`, `RUN_NORMAL`, and `RUN_GUARDED` retain their meanings. The FPGA may veto an unsafe/unexecutable action; it must not promote `RUN_GUARDED` to `RUN_NORMAL`.
5. Every executed circuit, including `RUN_NORMAL`, requires guarded power-up and runtime protection.
6. Scanning returns raw observations; a single electrical sample does not conclusively identify an IC.
7. One shared breadboard/fabric has one active execution context; instruments within that context may operate concurrently when admission permits.
8. Capture must not silently lose samples, change sample rate/encoding, or truncate data while reporting completion.
9. Clock loss, reset, lost configuration, or host loss must not leave power/drive enabled indefinitely.
10. Safe shutdown must not wait for SDRAM, DMA/readback, command completion, or a host ACK.
11. No master outside the memory subsystem issues arbitrary physical SDRAM addresses during normal operation.
12. Capability must reflect the actual build, board profile, and validation; planned channel counts are not evidence of availability.

---

# 2. Six-plane overview and boundaries outside the FPGA

The six-plane arrangement from the foundational FPGA architecture is retained, but the planes are **responsibility boundaries**, not six sequential processing layers.

```text
                       Host hardware interface
                   commands / status / observations
                                 |
               +------------------v--------------------+
               | Transport adapter + protocol endpoint |
               | bounded control path | chunked bulk   |
               +------------+--------------------------+
                            |
       +--------------------v-------------------------------------+
       |                    FPGA FABRIC                           |
       |                                                          |
       | CONTROL: execution FSM, image banks, resource admission   |
       |       |                                                  |
       | SAFETY: permits, fault latch, watchdog, bounded shutdown  |
       |       |          dedicated inhibit/status wires          |
       | LAB I/O: scan sequencer, route sequencer, drive, sense     |
       |       |                                                  |
       | INSTRUMENT: LA | Scope 2CH | DDS 2CH | time/trigger        |
       |       |           |             +-> DAC [local LUT]      |
       |       +-----------+                                      |
       |             capture FIFOs                                |
       |                    |                                     |
       | MEMORY: region manager -> bounded arbiter -> SDR ctrl     |
       |                    |                                     |
       | DATA: sealed descriptors, readback, raw scan, telemetry    |
       +--------------------+-------------------------------------+
                            |
                         SDR SDRAM

External safety island:
   current limiter / voltage guard / supervisor / fault latch
        | hard inhibit (independent of FPGA clock or USB)
        +--> digital OE, scan source, generator disconnect, rail cutoff

Electrical interfaces:
   FPGA <-> protected level interface <-> endpoint routing <-> DUT
   DUT  -> protected scope AFE -> ADC -> FPGA
   FPGA -> DAC -> reconstruction/output AFE -> protected disconnect -> DUT
```

**Selected optimization:** the basic DDS does not access SDRAM. Only a separately verified AWG extension may add a memory-read client. Safety cutoff must not traverse the shared register bus.

## 2.1 Module ownership

| Module | Owns | Must not own |
|---|---|---|
| Protocol Endpoint | Framing, sequencing, packet-level command admission | Energy permission or physical-pin bypass |
| Execution Controller | Active image, lifecycle, execution token | Full topology matching |
| Safety Core | Permit gating, fault evidence, stop deadlines | Scores, expected truth table |
| Route Controller | Shadow/active routes and switch sequencing | Repairing a student's circuit |
| Scan Engine | Test-source sequencing and observations | IC identity/classification |
| Drive/Sense | Bounded I/O and measurement frontend | Raw, unprotected access to FPGA pins |
| Time/Trigger | Timebase, sample anchors, event distribution | Assuming all clocks are phase aligned |
| Instruments | Sample production or DAC codes | Physical SDRAM addresses |
| Memory Subsystem | Region bounds, arbitration, SDR timing | Authorization to energize |
| Data Plane | Descriptor readback, packetization | Delaying hard inhibit |

---

# 3. Contract conformance: what is preserved and what remains unverified

## 3.1 Traceability matrix at the FPGA boundary

| ID | Source contract/constraint | Hardware obligation in this document | Review status |
|---|---|---|---|
| CT-01 | `PhysicalExecutionPlan` [LF] | Accept primitives from an already compiled plan; use an internal image, not a raw graph | Architecturally aligned; requires conformance with selected [CONTRACTS] |
| CT-02 | `InstrumentationExecutionPlan` [F0] | Do not silently alias CT-01; versioned mapping required | OPEN-GATE |
| CT-03 | `LabEndpoint` [LF] | Resource IDs, drive/sense direction, approved node bindings | Preserved; requires selected [CONTRACTS] tests |
| CT-04 | `SafetyVerificationResult` [LF] | Host approval cannot override hard limits/interlocks | Preserved; requires selected [CONTRACTS] tests |
| CT-05 | `TopologyVerificationResult` [LF] | No topology-correctness gate in RTL | Preserved; requires selected [CONTRACTS] tests |
| CT-06 | `FunctionalVerificationResult` [LF] | Return measurements/quality; do not grade in hardware | Preserved; requires selected [CONTRACTS] tests |
| CT-07 | `ExecutionDecision` [LF] | Preserve all four decisions; guarded restrictions may only tighten | Preserved; requires selected [CONTRACTS] tests |
| CT-08 | Active Electrical Topology Discovery [LF] | Distinguish observation, bounded continuity, and signature; output evidence | Clarified hardware modes |
| CT-09 | `BreadboardNodeID` [LF] | Use a NodeMap revision fixed for a PCB | Preserved; requires selected [CONTRACTS] tests |
| CT-10 | `HardwareStation` [LF] | Defined configure/arm/run/stop/readback effects | Semantic mapping in §5 |
| CT-11 | `GET_CAPABILITY` [LF] | Snapshot features/limits/health, not only compile flags | Clarified payload; ABI not frozen |
| CT-12 | `SET_ROUTE` [LF] | Validate → shadow → commit under energy inhibit | Clarified transaction |
| CT-13 | `ARM_CAPTURE` / `START_CAPTURE` [LF] | Separate arm, acquisition, start, and trigger; do not energize automatically | ICD must be finalized |
| CT-14 | `STOP`, `RESET`, `SAFE_STATE` [LF] | Electrical safety, invalidate old permissions, retain evidence per policy | Clarified reset classes |
| CT-15 | Frame sequence/length/CRC [LF] | No side effect before complete validation | Envelope semantics retained |
| CT-16 | `StationCapabilities` [LF] | Report actual resources/configuration constraints | Do not silently rename external fields |
| CT-17 | Scope 2CH [LF] | Measure two-channel acquisition and alignment | Target retained; not yet measured |
| CT-18 | Generator 2CH [LF] | Independent DDS/control channels and external DAC/AFE | Target retained; not yet measured |
| CT-19 | Shared SDRAM arbiter [LF] | One memory controller, bounded service per client | Retained and specified in more detail |
| CT-20 | Physical slots = 1 [LF] | One hardware owner, not 30–50 physical contexts | Retained |
| CT-21 | Capture result [LF] | Map sample rate/count/timestamp/encoding and quality | Schema not yet verified |
| CT-22 | EP4CE6 → EP4CE10 [LF] | Portable core, target wrappers, stable protocol semantics | Retained |

## 3.2 Three meanings of “matching” must remain distinct

- **Architectural semantics:** can be evaluated against the supplied Markdown documents.
- **Schema/wire ABI compatibility:** requires the actual selected Contracts/ICD sources; not yet demonstrated.
- **Hardware implementation conformance:** requires RTL simulation, fitting/timing, and bench evidence; not yet demonstrated.

A documentation checklist cannot substitute for the latter two levels. Do not claim ABI compatibility or hardware closure without the appropriate artifacts and tests.

---

# 4. Execution image: an integrated configuration immutable after approval

## 4.1 Required content at the hardware boundary

`FpgaExecutionImage` is a proposed internal structure. The FPGA is not required to parse JSON/Protobuf or hold an entire application plan.

| Group | Minimum logical contents |
|---|---|
| Identity | Image revision, plan reference, link/boot session, execution epoch |
| Binding | Hardware-profile revision, NodeMap revision, scan-evidence generation |
| Policy | `RUN_NORMAL` / `RUN_GUARDED`, bounded duration, approved safety envelope |
| Routing | Resource IDs, endpoint-to-node bindings, exclusive resource masks |
| Digital drive | Allowed channels, direction/mode, initial values, update limits |
| Power/scan | Allowed rails/test sources, validated profile IDs, staged timeouts |
| Clock/generator | Frequencies/tuning words, phases, waveform IDs, output limits |
| Trigger/capture | Arm dependencies, source/consumer masks, pre/post depth, formats |
| Memory | Descriptor requests, capacities, workload profile ID, permitted concurrency |
| Integrity | Complete-image length, configuration checksum, sealed flag |

Do not include IC identity, expected truth tables, or topology-match status in `energy_permit` computation.

## 4.2 Three levels of limits

```text
Effective hardware limits
  = intersection(
      physical protection limits,
      validated board/build envelope,
      approved execution restrictions
    )
```

A runtime request may reduce privileges, never expand the existing board/build limits. A hardware-profile ID selects an already validated configuration; it does not let the host arbitrarily overwrite absolute electrical limits.

## 4.3 LOAD → SEAL → APPLY → ARM

1. Load into `SHADOW_IMAGE`; active outputs do not change.
2. Validate complete length, ranges, resource masks, profile identity, descriptor requests, and cross-field constraints.
3. Seal one specific image revision. After sealing, changing route/power/direction/capture layout, or parameters outside approved runtime updates, invalidates the seal and relevant approval. Updates within the preapproved envelope and atomic-update mode follow §4.4 and do not enlarge permission.
4. Plan-level approval must bind the exact sealed image. Safety review before compilation cannot by itself approve routes/drives produced later.
5. Apply infrastructure while DUT power is disabled; a successful route commit does not enable outputs.
6. Recheck live health, evidence validity, and resource reservations before power-up; verify memory admission before capture acquisition begins.

A configuration checksum can detect an unintended or wrong image; it is **not an authentication signature and does not replace protection**.

## 4.4 Narrowly scoped runtime updates

`WRITE_OUTPUT` changes values only on an already authorized mask. It cannot change direction, route ownership, electrical domain, or output envelope. Multibit updates require atomic latching/timestamp boundaries when the image specifies synchronized updates.

Route changes, power-domain changes, AFE-range changes, and memory-layout changes are rejected during a run under the V1 semantics. Generator parameter updates are supported only when an advertised atomic update mode exists; otherwise execution must stop and be prepared again.

---

# 5. Semantic command contract — do not invent a new external ABI

**Inherited:** Lab-First [LF] contract operations, subject to verification against selected `cirnet-contracts`. **Clarified here:** completion, side effects, and internal decomposition.

| Source operation | Hardware obligation | Completion rule |
|---|---|---|
| `GET_CAPABILITY` | Snapshot build/profile/resources/health/version | No execution-state mutation |
| `SET_CLOCK` | Stage clock settings within the allowed envelope | Does not automatically generate a clock |
| `SET_ROUTE` | Stage, validate, and apply a route transaction in an allowed state | `APPLIED` only after the sequencer and required verification complete |
| `WRITE_OUTPUT` | Latch valid value/mask into the drive engine | Cannot exceed channel ownership or permits |
| `CONFIGURE_TRIGGER` | Stage trigger configuration; latch atomically before arm | Does not trigger or run automatically |
| `ARM_CAPTURE` | Reserve a buffer, arm local engines, confirm acquisition readiness | Does not energize the DUT or experiment source |
| `START_CAPTURE` | Permit acquisition/forced start under the configured mode | ICD must specify semantics; not implicitly identical to `run()` |
| `READ_CAPTURE` | Read a descriptor or bounded range of sealed data | No arbitrary SDRAM write permission |
| `STOP` | Stop experiment and place electrical systems in a safe state within bounded time | Does not wait for complete capture upload |
| `SAFE_STATE` | Revoke all energy permits, including scan stimulus | Receipt ACK differs from confirmed electrical completion |
| `RESET` | Published reset class; invalidate previous run permissions | No automatic resume or clearing of active faults |
| `configure_generator()` | Configure two channels through station abstraction | Corresponding binary primitive is not defined in [LF] |
| `run()` | Deterministic execution start after all-required-ready barrier | Cannot skip scanning, safety, power, or admission |

Names such as `LOAD_ROUTE_SHADOW`, `COMMIT_ROUTE`, `CONFIG_GENERATOR`, `READ_STATUS`, `PREPARE`, `ARM`, and `START` from the historical FPGA draft can be **internal micro-operations**. Expose them as wire commands only after the ICD defines opcodes, payloads, versions, and compatibility.

### Semantics to finalize in the ICD

Each command needs command identity, sequence, context token where applicable, bounded payload, accepted states, side effects, timeout, completion status, and error class. An asynchronous operation may return `ACCEPTED` followed by `APPLIED`/`FAILED`; a command merely queued must not return final success.

`SAFE_STATE`/emergency inhibit is idempotent. Commands that emit pulses, single-step, or start execution require duplicate suppression: retrying the same sequence must not cause another action.

---

# 6. Execution and instrument lifecycles are separate

## 6.1 Execution FSM

```text
RESET_HOLD
    |
    v
SAFE_IDLE <----------------------------------------------+
    |                                                     |
    +-> SCAN_ACTIVE -> SAFE_IDLE [evidence generation]     |
    |                                                     |
    +-> PREPARING -> PREPARED -> ROUTE_APPLY               |
                              |                           |
                              v                           |
                       GUARDED_POWER_UP                   |
                              |                           |
                              v                           |
                            READY                         |
                              | all-required-ready        |
                              v                           |
                            ARMED                         |
                              | deterministic START       |
                              v                           |
                           RUNNING                        |
                              |                           |
                              v                           |
                           STOPPING ----------------------+

Critical fault from any state -> ENERGY_INHIBIT -> FAULT_LATCHED
```

`SCAN_ACTIVE` is not `SAFE_IDLE`: even bounded continuity scanning injects energy into the DUT. A possible `SCAN_READY` status means only that observations are available, not that topology is correct or safety has passed.

Execution does not have to enter `CAPTURING` as an exclusive state after `RUNNING`. Captures may begin before, during, and after start to record pre-trigger activity or power-up transients.

## 6.2 Per-instrument capture FSM

```text
DISABLED -> CONFIGURED -> ALLOCATED -> ARMED/PRETRIGGER
                                     |
                                  TRIGGERED
                                     |
                                 POSTTRIGGER
                                     |
                                  DRAINING
                                     |
                          SEALED_COMPLETE / SEALED_PARTIAL
                                     |
                                  RELEASED
```

`ARMED/PRETRIGGER` may write data before a trigger when the mode requires it. Bandwidth reservations therefore take effect when pre-trigger acquisition begins, not only at experiment `START`.

## 6.3 Synchronization barrier

Start is permitted only after routing, power, required capture engines, generator configuration, and trigger consumers have confirmed readiness. Every CDC acknowledgment must carry the generation of that arm operation. A timed-out consumer aborts the arm; execution must not partially start without an explicit indication.

A `run()` without capture is possible if its profile permits it; internal execution arming must not force the host to invent an `ARM_CAPTURE` request.

## 6.4 Mapping outward

FPGA states are implementation details. When exporting status through the hardware boundary, define explicit mappings to the existing Contracts lifecycle; do not silently substitute RTL enums for public enums.

---

# 7. Safety island and energy-permit tree

## 7.1 Two independent response layers

```text
Fault/current/voltage/comparator/supervisor
         |
         +--> EXTERNAL LATCHED INHIBIT --> physical enables OFF
         |
         +--> FPGA synchronized status --> fault record / lifecycle cleanup
```

The external safety island must function when the FPGA clock stops, the FPGA is unconfigured, a brownout occurs, or control becomes stuck. An operational FPGA records faults and coordinates cleanup; it is not required for the electrical inhibit path itself to work.

An internal FPGA watchdog clocked by the clock that stopped cannot count down to timeout. Therefore an independent-timebase external watchdog/supervisor, or a proven equivalent, is required to cut off power on FPGA clock loss. This is an **additional architectural requirement**, not a claim that the current board includes it.

## 7.2 Separate permit classes without circular dependencies

Do not use one `global_drive_enable` for scanning, rail power, and experiment outputs.

```text
base_health_permit =
    external_interlock_ok AND configuration_ready
    AND supervisor_alive AND no_latched_fault

scan_source_permit =
    base_health_permit AND scan_context_valid
    AND scan_routes_ready AND bounded_scan_timer_alive
    AND scan_mode_allows_stimulus AND normal_sources_inhibited

rail_enable_permit =
    base_health_permit AND sealed_plan_approved
    AND run_watchdog_alive AND rail_route_safe
    AND state_allows_guarded_power

normal_source_permit =
    base_health_permit AND sealed_plan_approved
    AND execution_context_valid AND route_verified
    AND run_watchdog_alive AND power_stage_ready
    AND state_allows_requested_source
```

`rail_enable_permit` must not depend on `power_stage_ready` of the very rail that has not yet been enabled. `scan_source_permit` must not depend on approval of an execution plan that can only be produced after scanning. All effective permits remain subordinate to the external inhibit; these formulas express semantics, not a recommendation to wire uncontrolled multiclock combinational RTL directly.

## 7.3 Channel-level enforcement

Each source channel has a requested enable, ownership, route-ready state, allowed mode, and envelope. The effective enable is the intersection of these conditions with the permit for that source class. Writing an enable bit cannot bypass this logic.

Board profiles must define hard upper bounds for current, voltage, duration, and frequency consistent with physical hardware. Arbitrary host-supplied numbers are not evidence of safety.

## 7.4 Safety of unknown components

Guarded operation is permitted only when **supply, source/sense paths, endpoint binding, and runtime protection are sufficient to demonstrate the required electrical envelope**. Knowing the IC's name is not necessary to assess every such condition; however, merely having an eFuse cannot prove safety for every IC type.

Unknown VCC/GND/polarity, unisolated external sources, uncontrolled injection-current paths, or unproven pin domains require an indeterminate safety verdict. These are actual electrical constraints, not topology-correctness gates. [LF]

---

# 8. Safety staging and evidence freshness

## 8.1 Correct hardware sequence

```text
SAFE electrical state
  -> infrastructure self-check + passive voltage observation
  -> bounded continuity scan under a separate scan permit
  -> raw observations + validity/coverage/generation
  -> [boundary: candidate route/drive/power configuration]
  -> freeze candidate execution image
  -> plan-level safety approval bound to the exact image revision
  -> hardware limits/resource checks
  -> de-energized route apply + required route verification
  -> guarded power-up + runtime qualification
  -> readiness barrier -> execution
```

Any change introduced after approval must be reassessed or rejected. This staging does not replace the `ExecutionDecision` contract; it specifies hardware prerequisites that cannot be bypassed before energizing.

## 8.2 Passive prechecks do not prove all shorts

Observing 0 V on two unstimulated nodes is insufficient to determine whether they are shorted or disconnected. A precheck reports only what it actually measures: external voltage, interlock status, rail feedback, disabled sources, and frontend readiness.

Continuity/impedance testing requires bounded test excitation, even if the applied voltage/current is very small. Such measurements must not be described as “injecting no energy.”

## 8.3 Evidence binding

A scan result must be linked at least to NodeMap/profile revision, scan generation, power conditions, coverage mask, completion/abort status, and timestamp validity. An image that uses evidence from another revision is rejected.

Wiring may change after a scan. Do not promise that a software token detects every cable insertion or removal. V1 requires an operational hardware condition validated for the intended fault model: interlocked physical access, suitable board-change detection, or a procedure mandating rescan before every energization, together with runtime protection against undetectable changes.

An evidence time-to-live is only an age limit, **not a topology-change sensor**. On a board-change/interlock event or suspected stale evidence, inhibit and rescan.

---

# 9. Faults, STOP, SAFE_STATE, and recovery

## 9.1 Two-stage shutdown independent of data processing

**Stage 1 — energy inhibit:** disable digital/clock drive, scan stimulus, and generator output stages; demand rail cutoff through the hardware path. Latch the fault cause. Do not wait for a request bus or FIFO drain.

**Stage 2 — isolation and cleanup:** once electrical conditions permit switching, the route controller returns infrastructure to default-open; capture stops collecting samples, seals partial data when possible, and retains fault/status evidence.

A memory failure or lost acquisition clock must not prevent Stage 1 or route isolation. Closing descriptors is never a prerequisite for opening routes.

## 9.2 Residual voltage and external sources

`power_enable = 0` does not prove that a DUT rail has reached 0 V. A validated discharge policy, feedback, or bounded discharge interval is required. When an external source cannot be controlled, remain inhibited/faulted; do not falsely report `SAFE_COMPLETE`.

Emergency isolation must be suitable for actual fault-interruption conditions. If energy does not decrease within the deadline, keep the fault latched and hard-inhibited; do not make new routes. Protective earth/reference ground must not be indiscriminately switched in an “open every switch” sequence.

## 9.3 Normal STOP

STOP initiates bounded quiescence: stop clock/generator at a valid boundary when possible within the deadline, disable normal drive, revoke rail permit, wait for the electrical-safe criterion, and isolate routing. Capture draining continues independently within its allowance.

If the quiescence deadline expires or a protection fault occurs, transition immediately to hard inhibit. A fault may truncate a pulse; record the safety action rather than preserving an aesthetically clean waveform at the expense of safety.

## 9.4 Reset classes

| Type | Electrical effect | Data/context effect |
|---|---|---|
| Execution abort / `SAFE_STATE` | All experiment/scan sources OFF; route cleanup | Invalidate run token; retain sealed captures if memory remains healthy |
| Controller soft reset | Inhibit first; reset FSM/command state | Do not reuse execution epoch; publish descriptor policy |
| Full FPGA configuration/reset | External hardware holds enables OFF | Close link, invalidate descriptors without a retention guarantee, reinitialize memory |
| Board power loss | External hardware must fail safe | No assumption of SDRAM retention or timestamp continuity |

`RESET` does not automatically clear an active physical fault latch. Recovery requires the fault input to disappear, stable live health, new evidence/plan when needed, and explicit recovery. Never auto-restart after power restoration, PLL relock, or reconnection.

## 9.5 Fault evidence

Retain the first-fault code, subsequent-fault mask, execution identity, route/image revision, requested/effective enables, and timestamp-quality flag in registers/small on-chip memory. If the timebase is no longer valid, report `timestamp_valid = false`; do not fabricate a precise timestamp.

---
# 10. Watchdogs and execution identity

## 10.1 Watchdogs for distinct purposes

- **Host-liveness watchdog:** refreshed only by valid heartbeats from the active link/context, not by every packet.
- **Execution duration limiter:** the maximum energization duration cannot be extended indefinitely by heartbeats.
- **Scan dwell limiter:** bounds each stimulus duration and the overall scan, independently of host observation reads.
- **External FPGA-liveness watchdog:** detects stopped clock/control within a validated fault model and directly inhibits electrical outputs. Its service signal must reflect the monitored control conditions/progress, not merely a free-running divider that continues refreshing while control logic is stuck. This watchdog cannot guarantee detection of arbitrary logic faults.

Replayed heartbeat/sequence values must not extend a timeout. Losing the host during a scan must also disable stimulus; scanning is not exempt from watchdog protection.

## 10.2 Layered identity

Every side-effecting command must validate a tuple equivalent to:

```text
(link_boot_session, execution_epoch, image_revision, command_sequence)
```

`execution_epoch` changes upon preparing a new execution, completing an execution with STOP, an explicit abort, a fault, or plan invalidation. The counter must not wrap and be reused within one session; near wraparound, inhibit and establish a fresh session.

An FPGA counter resetting to the same value after a hard reset **does not** prevent stale commands. Full reset must close the transport session, remove stale data in the bridge/FIFOs/parser, and accept side-effecting commands only after fresh-session binding.

## 10.3 Boot freshness is a mandatory closure gate

Board integration must select and prove one of these mechanisms:

- A boot generation from an independent supervisor/persistent source that cannot repeat within the replay window to be prevented; or
- Verified reset-and-drain of the complete transport boundary, combined with a new session identity and a handshake that rejects old bytes/transactions before binding.

Do not assume EP4CE6/EP4CE10 has a built-in persistent monotonic counter or entropy source. If the bridge cannot safely purge/bind a session, reset-replay closure cannot be claimed; outputs remain inhibited until the link is cleanly established.

This protects against unintended delayed/replayed operations; it is not an FPGA trust/authentication subsystem.

## 10.4 At-most-once side effects

The baseline allows one outstanding mutating transaction plus a high-priority inhibit path. Cache the most recent response or a finite replay window according to the ICD. Duplicates of completed commands return status without re-executing the pulse/START/COMMIT.

An execution-scoped `STOP` must not let a stale stop packet terminate a later run. A valid global `SAFE_STATE` remains an idempotent energy-reduction command, but framing/integrity checks cannot be bypassed by searching bulk payload data for opcode-like byte patterns.

---

# 11. Routing fabric: connect infrastructure only to the real circuit

## 11.1 Physical path classes

```text
R_SCAN   : bounded stimulus / high-impedance scan sense
R_DIGITAL: protected source, clock, LA/digital sense endpoints
R_SCOPE  : approved measurement path to scope AFE
R_GEN    : approved generator output path
R_POWER  : current-limited supply distribution, if supported by the board
```

These paths may share some switch hardware, but resource exclusivity must be explicit. Power is not routed automatically through digital-signal MUXes. Scope/generator analog paths must not use an uncharacterized digital switch.

A full N×N crosspoint between all breadboard nodes is not required. The **baseline favors endpoint-to-node routing** and banked scan selection, preserving the student's own wiring as the actual circuit. This proposed hardware choice reduces switch counts/pin usage and avoids repairing the assignment.

If a current authoritative design defines another mode that explicitly synthesizes a circuit-assembly net, that mode requires independent authorization; do not infer or activate it under Lab-First student-wiring semantics.

## 11.2 Resource table

Every route resource has an ID, electrical class, allowed directions, exclusive group, supported endpoints/nodes, on/off bounds, voltage/current envelope, power-off behavior, and verification method. FPGA needs small lookup/mask checks rather than a general-purpose routing solver.

A passive sense bus must not close switches onto different nets and accidentally short them together. Sense fan-out is valid only for electrically isolated buffered inputs or an already verified common net. Source fan-out is permitted only when the image has explicitly approved the electrical connections it creates.

## 11.3 Node mapping

`BreadboardNodeID` identifies an electrical group on the PCB, not each plastic socket individually. NodeMap must document split rails, fixed paths, and reserved endpoints. Do not hard-code the assumption that every “830-hole breadboard” has identical connectivity. [LF]

---

# 12. Route transactions: atomically published state, not instantaneous physical switching

## 12.1 One baseline transaction sequence

```text
Validate complete shadow route + ownership
  -> assert ALL affected-source inhibits
  -> remove DUT rail permit under V1 policy
  -> confirm electrical conditions permit switching
  -> OPEN old switch paths
  -> wait validated break interval
  -> program new switch configuration
  -> wait settle interval
  -> readback/electrical verification if required
  -> publish active route revision
  -> keep normal outputs OFF
```

V1 does not permit live rerouting. Disabling FPGA outputs alone is insufficient if real ICs are powered and driving outputs. Route-changing transactions must account for DUT rails, generators, and all external energy sources.

## 12.2 Meaning of “atomic commit”

A shadow table is published as one active revision only after completion. External FPGA-controlled switches may be updated sequentially; safety is achieved through isolation and sequencing, not an assertion that every physical contact changes on the same clock edge.

Configuration checksum/readback proves only that switch registers received their data; it cannot prove that electrical contacts are not stuck on. Required verification depends on the fault model for the route class. Where readback is absent, capability must say so and use another method if safety requires confirmation.

## 12.3 Failure handling

Timeouts, incomplete tables, illegal simultaneous closures, or verification failures do not produce a new active route. Inhibits remain active, and the route revision is marked invalid. Do not automatically restore previous drive after a failed commit.

A simple invalid/stale request rejected before any side effect does not need to destroy a previously valid active route. This is an intentional change from the historical FPGA document [F0]; see the fault taxonomy in §27. Failures that make actual electrical state untrustworthy still require hard inhibit.

---

# 13. Scan architecture: three explicit electrical modes

## 13.1 Mode contract

| Proposed internal mode | Injects test energy? | Preconditions | Result |
|---|---|---|---|
| `PASSIVE_OBSERVE` | No active injection | Appropriately rated/protected input paths | Present voltage/logic and external-source evidence |
| `LIMITED_ENERGY_CONTINUITY` | Yes, bounded V/I/dwell/charge | Validated scan envelope, normal sources OFF | Continuity/impedance observations, not automatic component identity |
| `ACTIVE_SIGNATURE_SCAN` | Yes, controlled power/vectors | Sufficient pin/power mapping evidence plus scan-specific approval | Raw response of a supported test profile |

“Active Electrical Topology Discovery” in [LF] remains the name of the overall approach. `PASSIVE_SCAN` in the historical FPGA material also described stimulus; therefore, its name must not be taken to mean zero energy. Legacy mapping to the new modes requires confirmed ICD semantics.

`ACTIVE_SIGNATURE_SCAN` need not succeed before every Run. An unknown component may be unsuitable for signature scanning yet still run guarded when electrical and endpoint routability constraints have been independently established.

## 13.2 Scan transaction

```text
Acquire scan resource lock
  -> inhibit normal sources / verify power condition
  -> select one stimulus path and valid sense bank
  -> establish bounded test setting
  -> settle
  -> sample / accumulate
  -> disable stimulus
  -> discharge/settle as required
  -> emit observation record
  -> next test or terminate
```

Do not keep stimulus active while the observation FIFO is full or the host has stopped reading. The sequencer needs a safe pause behavior: turn stimulus OFF before waiting; if it cannot pause at a valid boundary, abort and mark the scan incomplete.

## 13.3 Observation record

Minimum logical fields include scan ID/generation, step index, stimulus node/mode/profile, sense-node range/bitmap, raw ADC/logic result, measurement range, power conditions, sample timing, saturation/fault/quality flags, and completed coverage.

Do not return a Boolean “connected/not connected” matrix as absolute evidence if the frontend only measures voltage through semiconductor structures. IC protection paths, leakage, and parallel branches can make an observation differ from a metallic jumper. Resolution must consider test conditions and recorded evidence. Switch power-off/injection specifications require part-by-part review. [H2]

## 13.4 Optimize for electrical-node count, not hole count

Let `N` be the number of electrical nodes and `P` the number of sense nodes that can actually be measured in parallel. A full stimulus-by-node scan takes approximately `N × ceil(N/P)` sense-bank steps before valid optimizations. This is a cost model, not a promise to discover every IC.

```text
T_scan ≈ sum_over_steps(
    t_select + t_stimulus_settle + t_measure + t_release
)
```

Use banked frontends, scan masks, calibration baselines, and FIFO streaming to reduce pin and memory usage. There is no need to store a full N×N matrix in M9K. Skip steps only when permitted by the test profile, and always report actual coverage.

---

# 14. Voltage-domain boundaries, power, and protection

## 14.1 Every DUT path must be evaluated

The boundary includes more than digital outputs: scan sources, high-impedance sense, Logic Analyzer inputs, scope inputs, generator outputs, trigger inputs, and supply paths all cross electrical domains.

An FPGA GPIO is not automatically tolerant of 5 V or arbitrary DUT voltage. Use the exact Cyclone IV part's operating limits, vendor tables, and appropriate external protection/level translation. A series resistor alone is not a universal solution. [H1, H5]

## 14.2 Disabled does not necessarily mean isolated

An unpowered switch/buffer may back-power through a signal path; power-off protection cannot be inferred from a disabled OE pin. Fail-safe control logic and power-off isolation of the signal path are distinct properties. TI documents the danger of back-power through ESD diodes when protection is inadequate. [H2]

Review both DUT → FPGA and FPGA/instrument → DUT directions when a supply rail is absent. Test external sources present at endpoints, power sequencing, and brownout.

## 14.3 Required electrical envelope

| Group | Parameters to establish by selected part/bench |
|---|---|
| Power | Voltage range, current-limit tolerance, cutoff response, capacitance/energy, discharge, reverse-current behavior |
| Digital source | VOH/VOL under load, source/sink limits, short/contention duration, OE-off behavior |
| Digital sense | Thresholds, hysteresis if present, input range, leakage, off-state behavior |
| Routing | Ron, capacitance, off leakage/isolation, charge injection, switching transients, power-off limits |
| Scan | Maximum voltage/current, dwell, injected charge/energy, measurement uncertainty |
| Scope | Input rating, loading, coupling/range, overrange behavior, common reference |
| Generator | Amplitude/offset/load envelope, source impedance, current limit, DC output under fault |

Do not describe protection only by nominal values. Account for tolerances, corners, and fault response of the weakest path. A current-limited rail does not replace per-pin injection-current limits or polarity checks.

---

# 15. Protected digital drive and digital sensing

## 15.1 Drive engine

Each channel contains a value register, allowed-direction/mode bits, requested enable, ownership, and an effective-enable gate. Prefer loading the value while OE is OFF to avoid transients from enabling before valid data.

Push-pull must be an explicit mode. Open-drain may be advertised only when the output stage truly supports it; then logic “1” means release and the pull-up must fit within the electrical envelope. Do not emulate open-drain by exposing an FPGA pin to voltage beyond its rating.

Scan, clock, generator, and digital stimulus must not concurrently own one source endpoint. Switching source classes requires stop/inhibit and a newly applied image.

## 15.2 Two distinct sense paths

```text
Protected digital inputs
     +-> control/status synchronizer -> snapshot / status FSM
     +-> measurement capture frontend -> LA / trigger / event path
```

Do not introduce status debounce/glitch filtering into the raw measurement path by default. Any measurement filter must be configurable and recorded in its descriptor.

Synchronizing one status bit does not guarantee an atomic multi-bit bus snapshot. A snapshot of an asynchronous DUT bus can show real skew/transitions; do not claim a coherent bus transaction without an appropriate handshake or synchronous-clock mode.

## 15.3 Conditions for valid capture

Published capability must describe capture clock/source, relevant input setup/hold or sampling uncertainty, detectable pulse-width envelope, and measured channel skew. Do not present nominal fabric frequency as the “logic bandwidth” of the full measurement path.

---

# 16. Experiment clock and deterministic digital timing

An experiment clock source should use output registers clocked by a constrained timebase, with dividers/phase accumulators/pulse schedulers creating enables or output transitions. Do not construct many unconstrained fabric clocks to clock internal FSMs. [F0]

Support only features actually validated: programmable period, duty quantization, finite cycle count, single-step, initial level, deterministic start, and stop policy.

`SET_CLOCK` only configures the clock. It is emitted only after image authorization, correct source ownership, and the execution readiness barrier.

Normal stop may complete a pulse at an appropriate boundary within the deadline. Emergency inhibit takes priority and may truncate a pulse; record the corresponding quality/event flag. The external safety path must not wait for a clock edge that may never arrive.

Frequency, duty accuracy, and divider/NCO jitter must be reported in terms of actual timebase/profile. “Deterministic” means behavior governed by a definite rule and clock; it does not mean zero jitter or exact generation of every arbitrary real-valued frequency.

---

# 17. Clock, reset, and CDC architecture

## 17.1 Clock plan based on real resources

Conceptual domains:

| Domain | Function | Strategy |
|---|---|---|
| `control_clk` | Protocol/control/safety housekeeping/scan control | Stable clock; clock enables for slower logic |
| `capture_clk` | Logic Analyzer and/or scope ingress | Derived from selected ADC/measurement clocking |
| `dac_clk` | DDS/output sample updates | May share a source with capture when valid |
| `sdram_clk` | SDRAM controller/arbiter | Constrained by memory timing and PCB |
| `transport_clk` | FIFO bridge | According to the external bridge clock |
| External supervisor clock | FPGA liveness/cutoff | Independent of FPGA PLL |

Do not assume a separate PLL per domain. EP4CE6/EP4CE10 have two general-purpose PLLs according to [H1]. Share PLL sources/outputs when valid; split domains only when needed for actual peripherals.

## 17.2 CDC primitives

- Stable single-bit level: timing-reviewed synchronizer.
- Pulse/event: toggle/request-acknowledge protocol or event FIFO with loss-prevention conditions.
- Multibit configuration: stable mailbox and handshake, latched atomically in the receiving domain.
- Sample stream: asynchronous FIFO with local full/empty indication.
- Counter/timestamp: snapshot handshake or constrained Gray/event mapping; never synchronize each binary bit independently.

Gray coding does not remove the need for timing/skew constraints. Vendor guidance states that multibit Gray CDC paths must preserve the intended single-bit-transition property at the sampling boundary. [H3]

## 17.3 Reset domain crossing

Asynchronous assertion when protection requires it; synchronous deassertion in each clock domain. A domain reports readiness only when its clock is valid, reset has been released safely, and local state/FIFOs have been initialized.

Resetting just one side of an async FIFO can corrupt pointer/occupancy interpretation. Coordinate reset protocols or flush/invalidate generations on both ends before reuse. Do not declare `ready` solely because a PLL's `locked` bit asserted without further qualification.

## 17.4 Clock loss

Loss of a required clock during energization triggers external/FPGA inhibit via a validated detection path; capture becomes partial/invalid depending on data integrity. Loss of SDRAM clock can break data retention when refresh stops; do not keep reporting old captures as valid merely because control-domain counters still run.

---

# 18. Timebase, triggering, and measurement-time integrity

## 18.1 A global epoch is not a chip-wide counter bus

Maintain a 64-bit global timebase for event correlation. Raw capture domains use local sample counters and anchors to map to the timebase; ordinary ADC samples do not need individual 64-bit timestamps.

```text
SampleDomainAnchor:
  timebase_generation
  source_clock_id
  source_sample_counter
  reference_timestamp
  sample_rate_numerator / denominator
  alignment_uncertainty
```

Configuration/timestamp snapshots crossing clock domains must be atomic. With clocks lacking a phase relationship, global trigger events reach consumers with different latencies and uncertainties. Document this in descriptors; a “shared trigger” does not mean all channels sampled at the exact same absolute instant.

## 18.2 Trigger fabric

The baseline favors one trigger context per execution, a small source mask, and broadcast to selected consumers. Avoid a complex general-purpose trigger network without demonstrated need.

Possible sources include digital edge/pattern, ADC threshold, external trigger, software force, experiment start, and generator sync. Advertise only integrated, validated trigger sources.

Detect triggers in their source domain, retain sample index/timestamp anchor, and distribute via handshake/FIFO or qualified pulses. Consumers acknowledge the appropriate arm generation before start. If tighter alignment than CDC latency permits is needed, use a common-clock synchronous path or separately validated calibration/protocol.

## 18.3 Pre-trigger and no-trigger cases

Pre/post-trigger depths are explicit. Trigger wait expiry returns proposed `NO_TRIGGER`/timeout capture status, not an automatic judgment that DUT behavior is functionally incorrect. These statuses must map through the current result schema rather than silently adding a new application enum.

---

# 19. Logic Analyzer

## 19.1 Baseline datapath

```text
Protected digital capture inputs
 -> capture registers / sample counter
 -> trigger qualifier
 -> raw sample packer
 -> ingress FIFO
 -> descriptor-bounded ring/linear writer
 -> shared SDRAM
```

Raw sampling is the easiest baseline to verify. `EDGE_EVENT` and `RLE_COMPRESSED` are optional build modes; the first implementation need not support all three.

## 19.2 Packing

For `N` channels, the true packed width must be recorded in the descriptor. If sample words are aligned to 16 or 32 bits, bandwidth is computed from bytes actually stored, not from the count of active signal bits.

Channel/bit order, endianness, sample period, and endpoint-to-channel mapping remain immutable metadata for a capture.

## 19.3 RLE needs a worst-case bound

RLE reduces repeated data, but a signal that changes every sample may use equal or more bytes than raw format, depending on record encoding. Admission must use the specified worst-case record rate.

V1 should store raw and optionally compress upon readback, or use a block format with explicit raw fallback. Never change encoding mid-capture without block tags. Pre/post-trigger counts refer to logical samples, not RLE records.

## 19.4 Overflow

If the FIFO fills while the source continues sampling, latch overflow, record the valid prefix/sample index, and stop or seal partial data according to policy. A DUT/ADC that cannot be stalled cannot be “backpressured” while still claiming the original sample rate.

If capture is required for execution, the baseline stops execution safely after losing capture. Overflow is an execution/data-quality failure, not automatically an electrical `UNSAFE` fault; see §27.

---

# 20. Two-channel Oscilloscope

## 20.1 Hardware boundary

```text
CH1 input -> protection/attenuation/coupling/AFE1 -> ADC CH1 --+
                                                            +-> FPGA
CH2 input -> protection/attenuation/coupling/AFE2 -> ADC CH2 --+
```

Use a two-channel ADC or two ADCs with suitable clocking/alignment. Alternating channels on one ADC must not be advertised as two simultaneous channels without sample-and-hold and timing evidence proving the claimed equivalence.

The supplied architectural sources do not establish ADC interface, resolution, sample rate, or analog bandwidth. Do not assign a fixed measurement/sampling rate to architectural capability before AFE/ADC selection and closure.

## 20.2 Acquisition core

Ingress validation/alignment → sample counter → trigger condition → sample packer → FIFO → memory writer. Preserve or reflect ADC status/overrange bits, where available, in metadata.

Two channels have separate calibration/range metadata but a defined shared capture timing model. Changing range/coupling requires stopping acquisition or beginning a new capture revision; do not mix samples from before and after a range change into an unmarked waveform.

## 20.3 Semantically correct decimation

Taking every `D`-th sample without anti-alias filtering must not be presented as a properly bandwidth-limited, decimated waveform. V1 may retain raw samples and export a window, or use a min/max display envelope with distinct encoding. Advertise averaging/filtered decimation only after filter and metadata requirements are established.

Do not move computationally heavy signal processing into FPGA merely to save storage if it breaks resource/timing budgets. Raw capture/window readback remains the baseline.

## 20.4 Measurement validity

Descriptors need actual sample rate, trigger index, channel alignment, ADC code format, range/gain/offset profile IDs, clipping/overrange, valid sample count, and timestamp quality. ADC numerical resolution does not equal effective resolution or instrument accuracy; the latter require actual measurement.

---

# 21. Two-channel Function Generator

## 21.1 Preferred architecture

Two independent phase accumulators with small waveform lookup/arithmetic, DAC code formatting, and separate output enables. A shared sync operation may reset phases/latch updates according to a defined clock.

```text
DDS1 accumulator -> waveform core -> DAC1 codes -> external AFE1 -> isolate -> OUT1
DDS2 accumulator -> waveform core -> DAC2 codes -> external AFE2 -> isolate -> OUT2
```

The sine LUT may use shared dual-port ROM or time multiplexing if adequate throughput for both channels is demonstrated. Do not sacrifice independent-channel semantics merely to save M9K.

## 21.2 Bounded parameter updates

Stage frequency, phase, waveform, and amplitude/offset parameters, then commit at a defined boundary. Amplitude/offset may use DAC code scaling, an external analog stage, or both depending on the hardware; do not assume an FPGA already has an analog gain stage.

Enforce output swing, DC offset, load/current, waveform frequency, and routing envelope together. The fault path must disconnect/mute the external output stage; setting the DAC code to zero does not necessarily produce 0 V or high impedance.

## 21.3 DDS and memory

Baseline DDS uses a local LUT/registers and **consumes no SDRAM bandwidth**. Arbitrary waveform generation (AWG) from SDRAM is a separate feature, OFF by default. Add it only with validated read FIFO, rate admission, preload, underrun policy, and output protection.

On AWG underrun, the output must enter the state proven safe by its electrical profile. Do not assume holding the last sample is safe, since it may be a hazardous DC level.

---
# 22. Capture descriptors and ownership across lifecycles

## 22.1 Public/boundary and internal descriptors

| Public/boundary descriptor | Internal-only fields |
|---|---|
| Descriptor ID + generation; capture reference | Physical base address |
| Instrument/channel map; encoding | Allocator cursor/region ownership |
| Allocated capacity; valid bytes/sample count | SDRAM bank/row/column |
| Pre/post-trigger and trigger position | Outstanding write counters |
| Sample rate/clock/timebase anchors | DMA/write-ring pointers |
| Termination reason and quality flags | Memory-controller scheduling state |
| Sealed/completion state | Hardware-specific stride/alignment details not needed externally |

The host never requests a base address. Capacity and read offsets are logical bounded quantities; normal operations must not provide unrestricted raw-memory access.

## 22.2 Allocation optimized for one execution slot

Use one capture arena with a bounded number of page/burst-aligned descriptors, **allocated before acquisition**. The baseline uses contiguous/bump allocation per execution rather than compaction or a general-purpose heap during capture.

Scope/Logic Analyzer regions are sized to the image's needs; do not permanently hard-partition the full 64 MiB. A general-purpose multitenant virtual-memory manager is unnecessary inside the FPGA.

If old unreleased captures occupy space needed by the next execution, return resource unavailable or wait for release; never silently overwrite them. V1 may require draining/releasing before reuse instead of implementing concurrent multi-experiment capture storage on the FPGA.

## 22.3 Completion and readback

`SEALED_COMPLETE` is valid only after the producer finishes, all FIFOs/outstanding writes commit, and final metadata is published atomically. `valid_bytes` must not exceed `allocated_bytes`.

Baseline readback is limited to sealed data. Reading an actively written region is an optional snapshot/segmented feature requiring its own protocol. `STOP`/`SAFE_STATE` must not automatically erase preserved sealed capture; otherwise run → stop → read_capture would lose its data.

## 22.4 Generations prevent unsafe reuse

A descriptor handle contains ID plus allocation generation. When its region is reused, the generation changes, and stale queued readback handles are rejected. Expiration of a run token does not necessarily invalidate separately authorized read-only handles for retained descriptors; these lifetimes are distinct.

A full FPGA/memory reset invalidates data unless retention is guaranteed. Do not promise persistence across power loss.

## 22.5 Pre-trigger ring

Record the oldest-valid offset, write position at trigger, actual valid pre-trigger count, and actual post-trigger count. Readback must linearize samples chronologically from the oldest valid sample instead of exposing raw physical memory order and expecting a consumer to infer ring wrap.

---

# 23. SDR SDRAM subsystem

## 23.1 Capacity and organization

[F0] used “64 MB” without selecting an exact part. This specification proposes **64 MiB** as the precise memory-address contract target, equal to 512 Mibit. If the final part/BOM differs, publish actual `capacity_bytes`; do not present a target capacity as installed hardware.

An x16 data bus is a candidate, not an invariant. Row/column/bank geometry, chip count, CAS, refresh, and frequency must come from the actual part/revision. Do not infer them from capacity alone.

## 23.2 Controller responsibilities

Power-up timing, initialization, precharge, mode setting, active/read/write, burst termination, bank/row tracking, refresh deadlines, and bus turnaround. Derive all timing counters conservatively from datasheet values and actual clock frequency.

The SDR interface needs both output and input constraints, clock phase/board delay, I/O standards, and bank-voltage review. Logic simulation of a controller does not replace physical timing closure of the external memory bus.

## 23.3 Three checks for a valid address

```text
instrument logical offset
  -> region bounds / owner / generation check
  -> physical address translation
  -> SDR geometry + timing controller
```

The memory controller must reject out-of-region requests even if an instrument malfunctions. Offset/length arithmetic must use enough bits to prevent wraparound before bounds checking.

## 23.4 Health and corruption

Run memory BIST before enabling deep-capture capability; stress refresh/arbitration using appropriate profiles. BIST does not automatically provide ECC for all runtime samples. Where ECC is absent, capabilities must make that explicit; packet CRC protects transfers but does not prove memory contents are always error-free.

Essential fault/control records belong in registers/M9K, not exclusively in SDRAM. If SDRAM fails, deep capture becomes unavailable; independent functions may continue only if the safety and build profile explicitly allow that separation.

---

# 24. Memory bandwidth, latency, and admission

## 24.1 Capacity and throughput are different

```text
B_peak = data_bus_bits / 8 × f_memory

R_scope = active_scope_channels × stored_bytes_per_sample × f_scope
R_la    = stored_bytes_per_la_sample × f_la
R_awg   = stored_bytes_per_awg_sample × f_dac_stream  [only if AWG exists]
```

Use real packed-byte sizes. A 12-bit scope sample stored as 16 bits consumes 2 bytes/channel/sample. For RLE, use worst-case encoded rates, not average compression ratio.

## 24.2 Choose exactly one overhead model

**Model A:** `B_usable = B_peak × eta`, where `eta` already includes refresh, precharge/activate, turnaround, and modeled overheads.

**Model B:** `B_usable` is sustained payload bandwidth measured on representative workloads/corners while real refresh and controller overheads are active.

Do not subtract refresh from measured payload bandwidth and then count the same refresh again as a payload client. Include separately any metadata/control overhead absent from the measurement, and document the accounting.

```text
sum(R_realtime_payload) + R_extra_payload
    <= beta × B_usable
```

`beta < 1` is a project margin, not a datasheet specification. Admission passes only if **bandwidth, maximum service latency, buffer capacity, routing/clock/trigger exclusivity, and electrical limits** all pass together.

## 24.3 Hypothetical example — not an advertised capability

Assume x16 SDR at 100 MHz: peak bandwidth is 200 MB/s. Assume `eta = 0.65` and `beta = 0.80`, yielding an admission payload budget of 104 MB/s.

- Two scope channels, each 25 MS/s and 2 bytes/sample: 100 MB/s.
- A 16-channel Logic Analyzer at 20 MS/s and 2 bytes/sample: 40 MB/s.
- Total 140 MB/s: **admission must fail**, even with 64 MiB memory.

Reducing to two scope channels × 20 MS/s and LA16 × 10 MS/s yields 100 MB/s, which passes only the sum-of-bandwidth check. FIFO/service latency, timing, and AFE feasibility still require proof. These example values are not board specifications.

## 24.4 FIFO sizing from worst-case service gaps

For client `i`:

```text
D_i_bytes >= R_i_max × L_i_max + burst_i_bytes + margin_i_bytes
```

`L_i_max` includes refresh, longest non-preemptible burst, turnaround, arbitration, and CDC visibility delay. Aggregate MB/s alone cannot guarantee no overflow.

Continuous ADC/LA inputs cannot be rescued with host backpressure. Without an upper bound on service gaps, finite FIFOs cannot guarantee lossless acquisition.

## 24.5 Bounded arbiter is preferred for V1

Use one SDR controller with burst-limited weighted round-robin/time-slot scheduling across a **finite set of workload profiles**. Refresh has a deadline; real-time clients receive reserved service; high-water intervention is bounded so other clients cannot starve.

Host readback is best effort and may be paused during acquisition in the baseline. After capture, readback uses the remaining bandwidth. There is no need for a general-purpose deadline scheduler or online FPGA optimization engine.

Record maximum observed service gaps, FIFO high-water marks, and overflow for validation. Finite bench tests produce evidence, while analytical bounds/RTL invariants are still necessary for upper-bound claims.

## 24.6 Capacity admission

```text
sum(allocated_capture_bytes)
  + optional_AWG_reserved_bytes
  + reserved_memory_bytes
  <= usable_capacity_bytes
```

Depth, pre/post windows, and formats must fit before acquisition. Slow memory readback does not change sampling frequency; it affects buffer release time and when the next execution can acquire resources.

---

# 25. Transport endpoint and control latency

## 25.1 Bridge-independent architecture

Keep USB-UART/USB-SPI options for the prototype and a USB3 FIFO for the high-throughput target. FPGA implements the FIFO/UART/SPI-facing adapter; the bridge's USB PHY/controller sits outside the FPGA. [LF]

Use a small control queue and a separate bulk queue. Bulk chunks are bounded; the serializer/arbiter inserts control between chunks. Readback backpressure must not block RX command parsing or the emergency shutdown path.

## 25.2 Latency limits require an explicit scope

```text
T_local_control <=
    maximum_nonpreemptible_chunk_time
  + bounded_queue_delay
  + parse_validate_time
  + dispatch_time
```

This bound is meaningful only if the link/bridge is servicing requests and the control packet has reached the defined hardware boundary. Do not promise a hard end-to-end bound from the host application through USB/OS scheduling when the host may stall or the link may disconnect.

Safety under a stalled host/link relies on **local watchdogs and external inhibit**, not an assumption that `STOP` always arrives in time. Measured transport performance is not a substitute for a validated safety-cutoff budget.

## 25.3 Bridge width and pin budget

FTDI documents 16/32-bit FIFO interface options in the FT600/FT601 family; this is an example of the bus-width tradeoff, not an approved BOM. A wider bridge does not automatically increase sustained SDRAM throughput and may exceed available E144 pins. [H4]

Select the bridge alongside ADC/DAC/SDRAM interfaces before freezing the FPGA package. Do not add USB3 after the schematic is complete without reserving pins, clocks, and power domains.

---

# 26. Protocol integrity and side-effect control

## 26.1 Envelope

Retain the meanings of SOF/version/command/sequence/length/payload/CRC/EOF from [LF]. Byte encoding, escaping, opcodes, CRC polynomial, and endianness belong in a versioned/frozen ICD; this specification does not invent their values.

A packet may cause side effects only after length/version/integrity/sequence/context/state/permissions are all valid. Receiving route payload bytes must not simultaneously change external switches.

## 26.2 Version compatibility

Distinguish protocol version, feature set, image schema revision, HardwareProfile revision, and firmware/build revision. Incompatible major semantics require safe rejection; absent optional features report unsupported.

Adding a token or image that changes wire layout requires a negotiated extension or a new documented protocol version. Retaining the same command names alone does not prove binary backward compatibility.

## 26.3 Conservative parser

Enforce bounded frame size, bounded staging buffers, partial-frame timeout, and a defined resynchronization rule. Invalid payloads must not overflow configuration RAM. Bad CRC packets must not refresh the watchdog.

Do not search for the byte pattern `SAFE_STATE` inside raw capture streams as an emergency shortcut: measurement data can contain arbitrary bytes. Emergency electrical cutoff remains independent of the protocol.

## 26.4 Internal bus

A small register/command fabric with request-valid/ready, response-valid/status, fixed arbitration, and bounded slave response is sufficient. A timed-out slave must not block execution shutdown. Avoid adding a soft CPU, full AXI infrastructure, or generic multimaster DMA merely for control when a simple register bus meets requirements.

---

# 27. Fault taxonomy: distinguish command rejection from lost electrical safety

This intentionally revises the earlier rule that every invalid command/protocol error/stale plan must immediately open routes. Rejecting a command before any side effect need not invalidate an already approved active configuration, unless that error itself makes the trusted safety state uncertain.

| Class | Example | Proposed hardware behavior |
|---|---|---|
| Benign request rejection | Bad read offset, unsupported optional feature, stale no-side-effect command | NACK, preserve active outputs/routes, do not refresh watchdog |
| Transport corruption | CRC/length error before commit | Drop/NACK/resync; retain safety watchdog; persistent loss eventually triggers timeout shutdown |
| Configuration/admission rejection | Resource conflict, image mismatch, rate above validated profile | Do not apply/arm; release reservations; do not energize new plan |
| Electrical critical fault | Overcurrent, overvoltage, dangerous contention, interlock fault | Immediate external inhibit + latched fault + isolation |
| Trusted-state loss | Control-state corruption, inconsistent active image, loss of required clock | Inhibit, invalidate context, explicit recovery |
| Measurement failure | Capture overflow, missing samples, lost ADC alignment | Invalid/partial capture; stop execution if capture is required; do not falsely label electrical `UNSAFE` |
| Required subsystem unavailable | SDRAM not ready for an image requiring capture | Block configuration/execution; never fabricate capability |

Hardware status reports detailed causes and severity. Mapping to `BLOCK_SAFETY`/`BLOCK_UNEXECUTABLE` and external error enums must follow actual Contracts; do not replace public enums with internal labels.

**Revised invariant:** an invalid command can never enable a new source, expand privileges, or change a protected active configuration. This differs from “every packet error must interrupt an otherwise safe experiment.”

---

# 28. Capabilities: hardware truth, health, and availability

## 28.1 Do not silently change external schema

[LF] uses fields such as `logicAnalyzerChannels`, `oscilloscopeChannels`, `generatorChannels`, and `physicalExecutionSlots`. Older material also illustrated nested/snake_case fields; treat those as internal representations. Explicitly map to version-tested external fields.

FPGA does not advertise component-resolution algorithms implemented elsewhere. It reports scan primitives and test-profile support; `AUTO_DISCOVERY`, `INSTRUCTOR_PROFILE`, and `HYBRID` remain external contract modes that consume FPGA evidence.

## 28.2 Three status layers

```text
IMPLEMENTED: module/peripheral is integrated into the build
VALIDATED  : concrete profile passed tests/timing/electrical validation
AVAILABLE  : current health/resources permit its use now
```

A validated two-channel instrument that is temporarily faulty need not permanently erase its channel count and pretend to be a different device. Prefer separate stable capability and dynamic health/availability fields when supported by schema. If only one capability snapshot exists, define clearly whether it represents installed capacity or usable-now resources.

## 28.3 Minimum logical hardware manifest

Build ID, device/package, protocol/features; board/profile/NodeMap revision; digital source/sense counts; instrument channel counts; supported sample/clock modes; actual SDRAM capacity and validated workload profiles; routing/trigger/memory exclusions; electrical-envelope IDs; timestamp metadata; and validated watchdog/cutoff bounds.

A single `maxSampleRate` does not describe every simultaneous Scope/Logic Analyzer/Generator combination. Expose validated configuration profiles or per-instrument limits plus concurrency constraints through versioned contracts.

## 28.4 Calibration and identity

Hardware profiles/AFE calibration data need revisions, integrity checks, and validity status. Missing calibration may allow explicitly supported raw-code measurements, but not fabricated calibrated-voltage accuracy. Calibration is never permission to exceed absolute hardware limits.

---

# 29. Resource and package strategy: EP4CE6 → EP4CE10

## 29.1 Reference silicon limits

According to Cyclone IV Device Handbook tables 1-1 and 1-3 [H1]:

| Resource | EP4CE6 | EP4CE10 |
|---|---:|---:|
| Logic elements | 6,272 | 10,320 |
| Embedded memory | 270 Kbit | 414 Kbit |
| Embedded 18×18 multipliers | 15 | 23 |
| General-purpose PLLs | 2 | 2 |
| User I/O in E144 package | 91 | 91 |
| User I/O in U256/F256 packages per reference table | 179 | 179 |

These are device resources, not the number of pins/RAM/logic elements remaining free on a particular board. “144 package pins” must not be confused with “144 user I/O”; dedicated/multipurpose/configuration pins and bank constraints still apply.

## 29.2 Pin pressure may limit a design before logic elements do

**Hypothetical example illustrating a tradeoff, not a schematic:**

- x16 SDRAM with 13 address, 2 bank, 2 DQM, 4 command/control, CKE and CLK: 39 signals.
- Two parallel 12-bit ADCs: at least 24 data pins.
- Two parallel 12-bit DACs: at least 24 data pins.
- x16 USB3 FIFO: at least 16 data pins.

Those signals alone total **103**, before peripheral clocks/control, scan, Logic Analyzer, safety, or digital drives. This illustrative set cannot fit the 91 user I/Os in an E144 package. It does not prove every EP4CE10 E144 configuration impossible: serial/shared interfaces may reduce pin counts, but change throughput and complexity.

## 29.3 Order of hardware decisions

1. Define required simultaneous instrument configurations and electrical envelopes.
2. Select compatible candidate ADC/DAC/SDRAM/bridge interfaces.
3. Budget pins/banks/clocks and memory service for the **complete final configuration**, even if some RTL instruments are implemented later.
4. Assess E144; use serialized/banked control only for appropriately low-rate signals.
5. If E144 is infeasible, evaluate EP4CE10 U256/F256 where all other constraints fit; otherwise consider a different device.

Never silently drop promised channels or critical safety pins merely to retain a package. Any target change must be reflected in capability and decision records.

## 29.4 Design budgets are not synthesis results

A project target of approximately 70–75% LE utilization may be retained from earlier proposals, but meeting it does not guarantee timing. Budget M9K according to **actual allocated blocks after width/depth packing**, rather than only dividing FIFO bytes by total embedded-memory bits.

Prioritize resources in this order: safety/control → I/O/scan/routing → ingress FIFO/trigger → memory controller → required instruments → optional compression/AWG/debug. When resources are tight, reduce optional functions first, not essential safety gating.

No measured utilization table is provided for the current design. Fit claims must come from the Fitter running against the exact target and revision.

---

# 30. Chosen architectural optimizations and rejected alternatives

| Baseline decision | Reason | Accepted tradeoff |
|---|---|---|
| One active image/context | One shared electrical fabric | No concurrent physical executions on one fabric |
| Six planes and a small command bus | Clear isolation and responsibility | No generic SoC inside FPGA |
| External hard inhibit | Independent of clock/USB | Requires external safety hardware |
| No live rerouting | Avoid source contention while DUT is powered | Topology changes require stop/prepare |
| Scan sequencer with raw evidence | Small RTL, no speculative component identification | Resolution outside FPGA |
| Banked endpoint routing | Fewer pins/switches; preserves student wiring | No full arbitrary N×N net assembly |
| Paired scope stream + Logic Analyzer stream | Limits the number of real-time memory clients | Scope pair shares a capture timing model |
| Local-LUT DDS, AWG OFF | Avoids unnecessary SDRAM read pressure | Arbitrary waveform generation is a separate extension |
| Capture arena allocated during idle | Simple bounds, no runtime fragmentation | Reuse after release; no general-purpose heap |
| Raw capture baseline | Straightforward capacity/quality proofs | Compression cannot justify unsafe rates |
| Workload profiles + bounded arbiter | Verifiable admission/latency on a small FPGA | Not every arbitrary combination of rates is allowed |
| Chunked readback after sealing | Immutable data and simple coherence | No promise of continuous raw streaming |
| Portable RTL core + board wrappers | Lower migration risk | Pin/timing review still required for each board |

Simply adding modules to the historical design would not resolve ambiguities around permits, epochs, partial commits, and readback lifetimes. Replacing the FPGA immediately would not fix them either. Clarify transaction/safety/data semantics before increasing device size.

---

# 31. Portable RTL organization

This is a **target modular structure**; current `cirnet-fpga` source has a different layout. Do not move files or rename public RTL interfaces automatically to match this example. Compare against real paths in §37.1.

```text
fpga/
  rtl/
    common/          # small bus, FIFO wrappers, CDC, counters
    control/         # execution FSM, image banks, admission
    safety/          # permits, fault record, watchdog interfaces
    lab_io/          # routing, scan, digital drive/sense, clock
    timing/          # timebase, anchors, trigger distribution
    instruments/
      logic_analyzer/
      scope_2ch/
      generator_2ch/
    memory/          # regions, ring writers, arbiter, SDR controller
    transport/       # protocol endpoint, control/bulk arbitration
  family/
    cyclone_iv/      # PLL, M9K, I/O primitive wrappers
  targets/
    ep4ce6_dev/      # board top, pin map, QSF/SDC, build manifest
    ep4ce10_lab/     # candidate/final target separated by revision
  verification/
    unit/
    integration/
    assertions/
    timing_cdc/
    bench_specs/
  contracts/
    hardware_icd/   # approved mapping/ABI references, not a new app schema
```

Wrap vendor FIFO/PLL primitives rather than scattering board-specific IP instances throughout the portable core. Do not claim that particular simulation tools or source layouts are currently in use until they are checked against the repository.

---

# 32. Verification: architectural acceptance properties

## 32.1 Safety invariants

| ID | Required property |
|---|---|
| V-01 | Unconfigured/reset/brownout/stopped-clock state → hazardous enables OFF by external hardware |
| V-02 | Fault assertion → physical inhibit within a measured bound, independent of protocol/memory |
| V-03 | No scan stimulus in `SAFE_IDLE` |
| V-04 | Scan dwell timeout works even if host or scan-data FIFO stalls |
| V-05 | Approval of image A cannot energize image B; critical changes after sealing invalidate approval |
| V-06 | No route commit before affected sources/DUT energy meets switching-safety criteria |
| V-07 | Failed route commit does not automatically restore outputs |
| V-08 | Unknown/mismatch alone does not enter fault/permit cone of logic |
| V-09 | Guarded startup applies to both Run modes |
| V-10 | Fault recovery, PLL relock, reconnect cannot auto-resume |
| V-11 | Stale execution/descriptor generations cannot affect a new execution or memory region |
| V-12 | Bad CRC/length/version packets cause no side effects and do not refresh heartbeat |

## 32.2 Capture/memory properties

| ID | Required property |
|---|---|
| V-13 | Every accepted sample has correct order/encoding, or an explicit loss flag |
| V-14 | Pre-trigger ring wrap is linearized correctly and trigger index is valid |
| V-15 | Completion only after last write drains; valid bytes never exceed allocation |
| V-16 | STOP → SAFE still permits authorized readback of retained sealed captures |
| V-17 | Release/reuse of descriptors prevents queued stale reads from accessing new data |
| V-18 | Worst-case RLE or raw fallback stays inside reserved service/capacity |
| V-19 | Arbiter meets maximum service gap for each admitted real-time client |
| V-20 | Refresh/turnaround cannot be starved by other bursts |
| V-21 | Async FIFO reset at any phase never mixes stale and new-generation samples |
| V-22 | Scope/DDS/LA concurrency works per supported workload profile, not by assuming all individual maxima are simultaneous |

## 32.3 Required test coverage

Module simulation should cover FSMs, image sealing, route sequencing, scan dwell, source gates, triggers, packers, ring descriptors, allocation, and arbitration. Integration fault injection must cover every transition, particularly seal/apply/power-up/start and memory FIFO full conditions.

Use randomized clock/reset ratios for CDC, duplicate/reordered/truncated command tests, bit-flip/length-bound tests, SDR memory-model timing tests, and fitting/full timing analysis for each target.

Bench tests include controlled shorts/overvoltage within the fixture's test envelope, power-off backfeed, stuck-switch/enable fault models, clock removal, bridge reset, cable disconnect, capture overrun, no-trigger, and reset replay. Electrical-fault tests require appropriately energy-limited fixtures; do not use students or arbitrary ICs as loads for protection testing.

---

# 33. Closure gates and quantities requiring evidence

| Gate | Required closure | Status based on available documentation |
|---|---|---|
| G-01 Authority/ICD | CIRNET Lab-First + Contracts, plan-name mapping, commands/enums, ABI version | OPEN |
| G-02 Electrical envelope | Rail/I/O/scan/scope/generator limits, injection/backfeed, cutoff/discharge | OPEN |
| G-03 Pin/package | Final peripherals + pins/banks/safety/debug, Fitter feasibility | OPEN |
| G-04 Clock/CDC/RDC | Sources, PLL budget, constraints, reset/coherence/uncertainty | OPEN |
| G-05 RTL/resources | Utilization and FSM/transaction/data invariants | OPEN |
| G-06 SDR timing/service | Exact memory part, timing, refresh, service-gap, payload budget | OPEN |
| G-07 Measurements | LA acquisition envelope, scope alignment/calibration, generator output quality | OPEN |
| G-08 Transport/replay | Local control bounds, watchdog, bridge purge/session freshness | OPEN |
| G-09 Integrated bench | Simultaneous workloads and full-system fault injection | OPEN |

**OPEN does not mean technically infeasible.** It means available documentation does not yet prove closure. Do not upgrade these statuses merely because an architecture document has been written.

## 33.1 Timing budgets to populate with measurements/models

`T_fault_detect`, `T_external_cutoff`, `T_FPGA_inhibit`, `T_rail_discharge_max`, `T_route_break_min`, `T_route_commit_max`, `T_scan_dwell_max`, `T_stop_quiesce_max`, `T_host_watchdog`, `T_external_watchdog`, `L_memory_client_max`, `T_local_control_max`.

No single numeric default is safe for all components. Each bound requires load conditions, voltage/temperature corners, clock/profile, and a validation method. Do not use one generic timeout for all states.

## 33.2 Debugging/configuration must also be interlocked

JTAG/configuration/bench overrides must not unintentionally bypass protection. Test mode may drive external pins outside ordinary RTL behavior; the fixture/board must keep external inhibit active during unvalidated configuration/debug operations. Debug access never expands the student's execution envelope.

---
# 34. FPGA in CIRNET — STEM & Engineering Learning Ecosystem

## 34.1 Two FPGA roles must be isolated

| Role | Purpose | Primary owner | Authority and requirements |
|---|---|---|---|
| **Lab Control FPGA** | Scanning, protected routing, clock/trigger, measurements, instruments, and safety interlocks | `cirnet-fpga` (RTL) + `cirnet-hardware` (host/driver) | Approved bitstreams only, programming by authorized operators, proven board-specific fail-safe behavior |
| **Learner FPGA Target** | Students write HDL, build, simulate, flash, and debug on an educational board | `cirnet-ide`, `cirnet-toolchains`, `cirnet-devices`; `cirnet-silicon` owns the EDA learning workflow | Artifacts are untrusted by default; require sandboxing, board isolation, and distinct programming permissions |

**Never permit** learner-generated bitstreams to overwrite the Lab Control FPGA connected to students' protected electrical fabric. Even when both devices use the same FPGA family, they must be **separate devices or a separately proven trust/isolation boundary** with verified interlocks. Distinguish artifact namespaces, toolchain profiles, execution policies, and release channels.

### 34.2 FPGA learning workflow within the ecosystem

```text
CIRNET Learn / LMS: HDL assignment + rubric + permitted target
  -> CIRNET Studio / IDE: Verilog/SystemVerilog source + project revision
  -> cirnet-toolchains: pinned synthesis/simulation/fitter/timing toolchain
  -> cirnet-devices: exact FPGA board/part/package/pin/clock profile
  -> CIRNET Silicon: RTL / verification / synthesis / timing analysis
  -> [sandboxed RTL simulation / controlled device programming]
  -> evidence: logs + traces + build manifest + artifact checksums
  -> CIRNET Analyze: waveforms, measurements, comparisons
  -> LMS: review/grade based on published rubric and allowed evidence
```

- **RTL simulation PASS** does not equal correct operation on an FPGA board, and correct board operation does not equal **physical electrical safety**.
- CIRNET Blocks may provide HDL-oriented blocks **if** it has typed IR, semantics, and source maps; string-concatenated RTL generation cannot bypass syntax/timing validation.
- `cirnet-ai` may suggest/explain, but must not independently authorize bitstream programming, energization, evidence deletion, or grade confirmation.

## 34.3 FPGA Lab Control runtime flow

```text
CIRNET Interface / Studio (viewer, not hardware authority)
  -> cirnet-remote (Rust): identity, authorization, lease, experiment
  -> cirnet-contracts: selected version + schema/protocol identity
  -> cirnet-hardware (Rust): plan validation, transport, safe stop
  -> cirnet-fpga RTL: permit/route/scan/trigger/capture under hard interlock
  -> protected external electronics + actual student DUT
  -> real observations + capture quality + checksum
  -> Hardware -> Remote -> authorized UI/LMS/Analyze consumers
```

The backend must not accept raw route-bypass commands from a browser. Transfer `PhysicalExecutionPlan` or `InstrumentationExecutionPlan` according to a **schema verified against the selected Contracts**. The Hardware compiler emits hardware primitives/images; the FPGA does not need to parse full JSON or grade student assignments. `SafetyVerificationResult`, `TopologyVerificationResult`, and `FunctionalVerificationResult` remain distinct as required by Lab-First.

## 34.4 Instrumentation is reusable, but cannot bypass permissions

Applications such as Lab, Analyze, Silicon, Robotics, and IoT may **read** waveform/measurement artifacts subject to data permissions. Control of supplies, triggers, generators, or DUT operations requires an authorized station lease and the protected safety path. Robotics/IoT/AI modules must not call FPGA registers directly from public APIs.

---

# 35. Ownership Matrix — cross-repository responsibilities

| Repository | FPGA-related ownership | Prohibited responsibility |
|---|---|---|
| `cirnet-fpga` | RTL, FPGA module logic, target-specific SDC/QSF, testbenches, source/bitstream provenance, and board-specific timing evidence | No Auth, Scheduler, Projects, Grading, REST, or circuit simulation engine |
| `cirnet-hardware` | Rust HardwareStation, protected transport/driver, station safe-state, worker supervision, applied-image acknowledgment | Cannot redefine canonical schemas or replace hard electrical protection |
| `cirnet-remote` | Experiment admission, auth delegation, durable leases/quarantine, provenance lock, capture/API | Cannot write FPGA/MUX registers from public request handlers |
| `cirnet-contracts` | Canonical types, FPGA protocol mapping, RPC, hardware capabilities, capture envelope | Does not implement RTL or certify electrical limits |
| `cirnet-interface` | Browser circuit/lab editor, capability UI, authorized waveform display | Cannot declare a circuit SAFE or create real routes |
| `cirnet-studio` | Engineering workspace integrating editors and result views | Does not fork `cirnet-interface` or create a duplicate FPGA driver |
| `cirnet-ide` | HDL editor, LSP, build/debug UI | Does not invent compilers or a programmable bridge bypass |
| `cirnet-toolchains` | Pinned HDL simulators, synthesis, fitting, timing analysis, reproducible builds | Cannot authorize electrical safety or program an unapproved board |
| `cirnet-devices` | BoardProfile, part/package/pins/clocks/flash/debug adapters, validation metadata | Does not own shared Lab leases or grades |
| `cirnet-silicon` | HDL/RTL/ASIC learning workflow, logical synthesis/STA/PDK adapters under access policy | Cannot replace Lab Control FPGA production firmware |
| `cirnet-project` / `cirnet-format-project` | Project revisions, artifact references, portable file manifest/format migration | Must not store station secrets or unauthorized firmware |
| `cirnet-analyze` / `cirnet-numerics` | Analysis of measured waveforms, FFT/statistics/uncertainty, reproducible reports | No direct hardware-routing commands |
| `cirnet-lms` / `cirnet-learn` | Assignments/gradebook and learning UX; rubrics and learning evidence | Cannot declare `SAFE` or grant hardware permissions based on grades |
| `cirnet-platform` / `cirnet-database` | Identity, organizations/tenants/policy and persistence infrastructure | No FPGA pin management or overriding Hardware domain schemas |
| `cirnet-extensions` / `cirnet-integrations` | Signed/validated plugins/adapters and interoperability boundaries | Cannot distribute extensions that bypass safety without control |
| `cirnet-rules` | AI Agent baseline/Skills/Policies, documentation discipline | Not an execution service or automatic firmware publisher |

`cirnet-app`, `cirnet-desktop`, `cirnet-cli`, `cirnet-robotics`, `cirnet-iot`, and `cirnet-ai` consume only authorized public interfaces. USB/Serial privileges in Desktop or CLI do not, by themselves, confer authority over a Lab Control FPGA. `cirnet_web` publishes approved/public information; `.github` and `about-cirnetlab` are organization configuration/documentation outside runtime.

---

# 36. Project, BoardProfile, Image, and Artifact conventions

## 36.1 Distinguish authored source from physical evidence

```text
EngineeringProjectRevision
   |- HDL SourceArtifact (user-authored, untrusted)
   |- BuildSpecification (toolchain lock + target profile)
   |- RTLVerificationEvidence (simulation/synthesis/timing logs)
   |- BitstreamArtifact (hash, signature/trust, board compatibility)
   |- DeploymentRecord (actor, exact board, mode, permission)
   |- MeasurementArtifact (MEASURED + instrument/profile/quality)
   `- LearningSubmissionRef (immutable revision + rubric ref)
```

`cirnet-project` owns the data lifecycle; `cirnet-format-project` owns portable file packaging/import/export; shared schemas live in `cirnet-contracts`; runtime builds belong to Toolchains; target boards belong to Devices. Do not replicate each concept as competing sources of truth inside every UI.

## 36.2 Required build/bitstream manifest contents

- Source commit or content digest; project revision; generator/toolchain versions; provenance and licenses for IP cores.
- Device family, exact part/package/speed grade, board revision, pin/I/O-bank electrical standards, and memory/clock constraints.
- Binary SHA-256 computed over **actual artifact bytes**, release label, signing/trust policy where applicable, and selected compatibility set.
- Validated board limits/capabilities; do not infer PCB performance from a chip datasheet alone.
- When installing on the Lab Control FPGA: approved firmware chain, safe-state prerequisites, physical operator authorization, and a rollback/secure recovery plan.

## 36.3 Version domains

Maintain separate versions for repository tags, RTL source, bitstream, ICD/wire ABI, Contracts schema, BoardProfile, toolchain, pinout/PCB, FPGA IP core, capture encoding, and selected CIRNET system release. The common tag/Release naming convention is **`<repo_name>-<version>`**; `cirnet-fpga-0.2.2` is only a **naming example**, not a claim that the tag has been published. Do not rewrite pinned history.

---

# 37. RTL development rules and actual verification

## 37.1 Build on the actual repository layout

At review time, `cirnet-fpga` contains `fpga/experiment-controller/rtl/top.v`, `fpga/instrument/rtl/top.v`, testbenches under `fpga/simulation/`, `fpga/common`, `fpga/constraints`, `fpga/quartus`, source/verification scripts, and `integration/` lockfiles. The portable RTL structure in §31 is a **modularization goal**, not a reason to move files automatically or rename already accepted public interfaces. Extend modules in suitable locations and preserve old tests/paths while consumers depend on them.

## 37.2 Required tests and conformance evidence

| Layer | Evidence required before claiming support |
|---|---|
| Source hygiene | Lint, formatting, paths/docs, license/IP provenance, no secrets, correct locks |
| RTL unit/integration | V-01…V-22 from §32, fault injection, reset/CDC, no-rewire invariant |
| FPGA protocol | Artifact-pinned [CONTRACTS] ICD, framing/replay/sequence, timeout, Stop, safe downgrade |
| Synthesis/fitting | Exact target part/package, utilization, real pin assignments, I/O banks, DRC |
| Timing/CDC/RDC | Target-specific SDC, setup/hold, async paths, reset crossing, relevant corners |
| Electrical safety | Schematic/BOM/revision plus measured independent cutoff, power-off/backfeed, latched-fault recovery |
| Instrument bench | ADC/DAC/AFE calibration, actual sample rate, skew/jitter, trigger, output envelope, quality flags |
| Full Lab E2E | Real Rust Remote/Hardware, exact firmware/board/Contracts, authenticated Web, measured evidence |
| Learning target | RTL simulation and board programming isolated from trusted Lab Controller, exact assignment artifact |

Do not run Python test scripts inside the production FPGA control path. Python is allowed for offline tests/benchmarks, consistent with current `cirnet-fpga` files such as `pytest.ini`, `tests/`, and `requirements-verification.lock`.

## 37.3 Three distinct meanings of Stop

- **UI Stop requested:** a client action; it proves nothing about the electrical state.
- **Hardware Stop/SAFE_STATE acknowledged:** hardware has confirmed electrical/resource state according to policy.
- **External hard cutoff:** the physical safety path independent of FPGA clock, OS, and USB; must be proven on a bench for the approved fault model.

## 37.4 Important prohibitions

- Do not program learner firmware/bitstreams onto a protection-controlling Lab FPGA serving other users.
- Do not use unsafe routes, external sources, clock/FPGA resets, or debug/JTAG to bypass the external safety island.
- Do not generate HDL or FPGA routes that “repair” an incorrectly wired student circuit.
- Do not claim operating scope/Logic Analyzer/generator, SDRAM, HIL, or Physical Digital support without evidence.
- Do not let extensions/AI/LMS/CLI request side effects outside leases, board profiles, and protected RPC.
- Do not move semiconductor/EDA learning workflows into Lab Control RTL when they are independently owned educational workflows.

---

# 38. Architectural Decision Records

| ADR | Proposed architectural decision |
|---|---|
| FPGA-ADR-001 | Safety-gated, not correctness-gated; retain the four `ExecutionDecision` variants |
| FPGA-ADR-002 | `cirnetlab-architecture-en.md` covers the Organization; CIRNET Lab-First defines Lab safety; FPGA owns only hardware RTL |
| FPGA-ADR-003 | Canonical Contracts/ICD selected by artifact; `FpgaExecutionImage` is internal |
| FPGA-ADR-004 | Approval binds to a sealed image; critical configuration changes require renewed approval |
| FPGA-ADR-005 | External hard inhibit/watchdog independent of clock and transport |
| FPGA-ADR-006 | Separate noncircular permits for scanning, sources, and power |
| FPGA-ADR-007 | Explicit semantics for passive observation, limited-energy continuity, active-signature scanning |
| FPGA-ADR-008 | Route changes under electrical-safe conditions; no live rerouting in the baseline |
| FPGA-ADR-009 | One execution context on a shared fabric; independent capture lifecycle |
| FPGA-ADR-010 | Boot/session/execution freshness is distinct from descriptor readback lifetime |
| FPGA-ADR-011 | Rejected benign requests do not disrupt safe execution; loss of trusted state requires inhibit |
| FPGA-ADR-012 | Local-LUT DDS; optional SDRAM AWG with separate admission/underrun policy |
| FPGA-ADR-013 | Explicit capture quality flags; RLE does not replace worst-case resource bounds |
| FPGA-ADR-014 | Bounded capture arena/SDR arbiter, not general-purpose FPGA heap/OS |
| FPGA-ADR-015 | Host/USB control latency does not substitute for physical-cutoff deadlines |
| FPGA-ADR-016 | Freeze EP4CE10 package only after pin/clock/SDRAM/ADC/DAC/USB budget closure |
| FPGA-ADR-017 | Waveform provenance includes rate/skew/uncertainty/partial/loss/calibration |
| FPGA-ADR-018 | Capability requires verified board evidence; planned RTL/scaffolds are insufficient |
| FPGA-ADR-019 | Lab Control FPGA and Learner FPGA Target must be isolated in trust, permissions, and physical I/O |
| FPGA-ADR-020 | C++ is the default circuit simulator; Rust owns services/Hardware, RTL owns FPGA, Python is a platform verification lane |
| FPGA-ADR-021 | Toolchains/Devices/Silicon/Project/LMS consume versioned artifacts without duplicating ownership |

---

# 39. Open issues, applicability, and provenance

## 39.1 What cannot be established from Markdown alone

1. Final FPGA part/package/board/pinout, target clock/timing closure, and source-specific synthesis results.
2. eFuse/supervisor/AFE/ADC/DAC circuitry, electrical protection thresholds, fault-cutoff/discharge times, and external supply voltages.
3. Physical byte-level protocol, stale-session protection, HDL/firmware hashes, and compatible board profiles.
4. SDRAM part/geometry, BIST, refresh/service bounds, and concurrent Logic Analyzer/Scope/Generator workloads.
5. Actual measurements of oscilloscope channel synchronization, generator output quality, and sample-rate/uncertainty.
6. Full HIL and Physical Lab authorization/lease/recovery with the exact target board.
7. Isolation tests for extensions, student bitstreams, and chip debug rights.

Keep these items `OPEN/NOT_RUN/UNAVAILABLE` until supported by the appropriate evidence. Do not silently edit the selected system release set, create Git tags/releases, or equate documentation updates with completed implementation/testing.

## 39.2 Source material

- **User-provided source:** historical FPGA hardware design [F0], including six planes, 22 boundary checks, 22 verification properties, nine closure gates, SDRAM/pin calculations, and hardware safety. Sound technical principles were retained; obsolete organization branding and outdated source assumptions were removed.
- **Current architecture:** `cirnetlab-architecture-en.md` (Ecosystem), `cirnet-architecture-lab-first.md` (Lab-First).
- **Repository source observations (2026-11-15):** `cirnetlab/cirnet-fpga` branch `master`, and the READMEs of `cirnet-hardware`, `cirnet-contracts`, and `cirnet-simulator`; observing source is not HIL validation.
- **Hardware references [H1]–[H5]:** tables/links below inform component selection/review and do not prove a final BOM or board specification.

| ID | Technical reference | Intended use |
|---|---|---|
| H1 | [Cyclone IV Device Handbook](https://cdrdv2-public.intel.com/655151/cyiv-5v1.pdf) | Part resources, pins, clock, PLL, constraints; consult the exact device/package |
| H2 | [TI TMUX1575](https://www.ti.com/product/TMUX1575) | Example power-off signal-path protection and back-power hazards; not a selected BOM item |
| H3 | [Intel timing constraint guidance](https://www.intel.com/content/www/us/en/docs/programmable/683241/24-3/user-configurable-timing-constraint.html) | CDC/Gray timing principles; use documentation for the real vendor/target |
| H4 | [FTDI FT601](https://ftdichip.com/products/ft601q-b/) | Example 16/32-bit high-throughput FIFO; not a guarantee of measured throughput |
| H5 | [Intel programmable device temperature support](https://www.intel.com/content/www/us/en/products/docs/programmable/temperature.html) | Part operating envelope; do not infer 5 V tolerance for raw FPGA GPIO |

**Conclusion:** CIRNET's FPGA domain is both a **reusable engineering education capability** and a **protected Lab Control Fabric**. HDL/FPGA/ASIC learning workflows reuse source, toolchains, boards, and evidence without inheriting authorization to energize circuits or bypass interlocks. Earlier hardware design principles are preserved, cross-repository ownership is clarified across the 33 repositories, and all compatibility/electrical claims remain dependent on real verification.
