# CIRNET — STEM & Engineering Learning Ecosystem

> **Document:** `cirnetlab-architecture-en.md` · **Language:** English  
> **Scope:** Organization-wide target architecture for [`cirnetlab`](https://github.com/orgs/cirnetlab/repositories).  
> **Architecture name:** **CIRNET — STEM & Engineering Learning Ecosystem**, without a version suffix.  
> **Related documents:** [Lab-First Architecture](cirnet-architecture-lab-first-en.md) · [FPGA Architecture](../cirnetlab-FPGA-architecture-en.md) · [Tiếng Việt](cirnetlab-architecture-vi.md)  
> **Evidence snapshot:** GitHub repository inventory and supplied architecture reviewed on **2026-11-15**.  
> **Status:** **PROPOSED TARGET ARCHITECTURE**; not an implementation, performance, production, electrical-safety or semiconductor sign-off certificate.  
> **Organization structure:** 33 existing repositories: 30 product/engineering/tool repositories, `cirnet-rules`, and two documentation/organization repositories. **No development phases are prescribed by this document.**

---

## 0. Executive architectural statement

CIRNET is a **multi-platform, modular, contract-driven, interoperable STEM and engineering learning ecosystem**. Learners and engineers can study, model, design, program, simulate, compute, build, deploy to authorized hardware, observe real or simulated systems, analyze results, submit reproducible evidence, and receive structured educational feedback within a unified Engineering Project.

Educational institutions can manage courses, classrooms, assignments, lab access, submissions, rubrics, competency records and learning analytics without making one simulator or one web app the sole system of record. CIRNET seeks broad engineering-workflow integration comparable in ambition to established scientific-computing ecosystems; it **does not claim MATLAB-language compatibility, parity with proprietary toolboxes, or capabilities that have not been implemented and verified**.

```text
ONE IDENTITY + ONE ENGINEERING PROJECT + VERSIONED CONTRACTS
       + MULTIPLE DOMAIN APPLICATIONS + VERIFIED ENGINES
       + AUTHORIZED VIRTUAL / PHYSICAL EXECUTION
       + IMMUTABLE EVIDENCE + EDUCATIONAL ASSESSMENT
```

**Non-negotiable Lab-First invariant:** **Safety is not correctness.** A wrong-but-demonstrably-safe and executable student circuit may run under approved protection to expose its actual behavior. The Physical Compiler may connect **protected measurement, stimulus, clock and supply infrastructure to observed physical endpoints**; it **must not repair, recreate or replace the student's IC or jumper wiring** with FPGA routes.

### 0.1 Status and evidence vocabulary

| Label | Meaning |
|---|---|
| `TARGET` | Intended architectural capability. Record implementation status separately. |
| `DESIGNED` | Documented design without a verified implementation claim |
| `IMPLEMENTED` | Actual corresponding source exists |
| `TESTED` | Specific tests completed with identifiable evidence |
| `VERIFIED` | Capability verified in the declared environment and scope |
| `RELEASED` | Traceable, versioned artifacts have been published. Record deployment selection separately. |
| `UNAVAILABLE` | Feature cannot currently be offered by this target/runtime |
| `BLOCKED` | A precondition, permission, protection or necessary dependency is unmet |

An existing GitHub repository is not evidence of an implementation. A `master` branch is not an integrated, compatible system release. Source, build, contract, runtime, device profile and exact artifact digests must be distinguished. Architectural ownership below is a **target** where legacy code resides elsewhere; transfers require tested migration, not competing authorities.

---

## 1. Product experiences and engineering domains

| CIRNET experience | User purpose | Primary UI repository | Consumed domains |
|---|---|---|---|
| **CIRNET Studio** | Multi-domain engineering workspace, tools, models, project navigation | `cirnet-studio` | Project, Interface, IDE, Analyze, Platform |
| **CIRNET Lab** | Virtual/physical circuit design, electronics, instruments, measured behavior | `cirnet-interface` | Remote, Hardware, Simulator, FPGA, Contracts |
| **CIRNET IDE** | Source editing, LSP, debugging, build, firmware upload, serial monitoring | `cirnet-ide` | Toolchains, Devices, Projects, CLI |
| **CIRNET Blocks** | Typed block-based programming and generated source | `cirnet-blocks` | IDE, Toolchains, Devices, Contracts |
| **CIRNET Analyze** | Scientific notebooks, plots, signal processing, numerical experiments | `cirnet-analyze` | Numerics, Project, Contracts |
| **CIRNET Learn** | Learning paths, coursework, submissions, feedback | `cirnet-learn` | LMS, Platform, Projects, authorized Lab |
| **CIRNET Desktop** | Privileged local integrations and desktop delivery | `cirnet-desktop` | Authorized UI components and native adapters |
| **CIRNET App** | Mobile/tablet study, projects, observation and notifications | `cirnet-app` | Platform, LMS, Project and authorized telemetry |
| **CIRNET Robotics** | Robot modeling, control and experimentation | Studio/IDE/Analyze | Robotics, Numerics, Devices, Toolchains |
| **CIRNET IoT** | Device fleet, telemetry, networked embedded experiments | Studio/App/Analyze | IoT, Devices, Platform, Projects |
| **CIRNET Silicon** | Digital logic, RTL, FPGA/ASIC education and tool integration | Studio/IDE/Analyze | Silicon, FPGA, Toolchains, Devices |
| **CIRNET AI Assistance** | Advisory explanations and productivity support | Authorized application surfaces | AI, Contracts, Platform policies |
| **Public website** | Product information and approved demonstrations | `cirnet_web` | Public-only content |

A **product experience is not necessarily a separate deployed service**. Applications may compose reusable components and APIs without copying domain engines or introducing a microservice per screen.

---

## 2. Logical architecture

```text
┌──────────────────────── CLIENT EXPERIENCE ───────────────────────────┐
│ Web: Studio · Lab Interface · IDE · Blocks · Analyze · Learn        │
│ Desktop: trusted native shell + approved file/USB/serial bridges     │
│ Mobile/Tablet: adapted UX, scoped capabilities and offline cache    │
│ Public website: cirnet_web (public content only)                     │
└──────────────────────────────┬───────────────────────────────────────┘
                               │ HTTPS / WS / versioned typed SDK
┌──────────────────────────────▼───────────────────────────────────────┐
│ CROSS-APPLICATION PLATFORM AND DOMAINS                                │
│ cirnet-platform: identity · tenant · organization · shared policy      │
│ cirnet-project: project · revision · artifact references               │
│ cirnet-lms: courses · enrollment · assignments · grading               │
│ cirnet-remote: Lab API · experiments · lease · capture metadata         │
│ cirnet-iot: fleet · device lifecycle · telemetry                       │
│ cirnet-ai: advisory workflows under authorization                      │
│ cirnet-contracts: public/private typed boundary contracts              │
│ cirnet-database: DB infrastructure and recovery tooling                 │
└─────────────┬──────────────────────────────────────┬──────────────────┘
              │ authorized jobs and artifacts        │ Lab control plane
┌─────────────▼──────────────────────────┐  ┌────────▼──────────────────┐
│ ENGINEERING AND DEVELOPER ENGINES      │  │ LAB AND EDGE              │
│ Toolchains: compiler/SDK/build jobs    │  │ Hardware: Rust station    │
│ Devices: target profiles and adapters  │  │ Simulator: default C++    │
│ Numerics: numeric kernels              │  │ FPGA: RTL, protected IO   │
│ Robotics / Silicon: domain workflows   │  │ Instruments + physical DUT│
│ Extensions / Integrations: adapters    │  │ Safety: independent gates │
└───────────────────┬────────────────────┘  └──────────┬────────────────┘
                    └────────────────┬────────────────┘
                                     ▼
               VERSIONED, IMMUTABLE ENGINEERING ARTIFACTS
         project revisions · source · builds · firmware · bitstreams
        models · waveforms · measured evidence · academic references
```

The central diagram describes **domain boundaries**, not a mandatory distributed deployment. A modular monolith, isolated workers, on-prem edge services and independent client distributions may coexist. Kafka, Kubernetes, a service mesh, a monorepo or a general orchestration framework is **not a default requirement**.

### 2.1 Dependency rules

1. Client apps consume authorized public APIs and generated/versioned SDKs, never database tables or FPGA registers directly.
2. Domain services depend on explicit contracts; no private-source imports or hand-maintained copies of another repository's schema.
3. Contracts define inter-boundary agreements, not every domain's business implementation.
4. Engine/plugin/build/notebook workloads are bounded and isolated from privileged API processes.
5. The shared Platform provides identity and policy; it does **not** decide electrical safety or route an FPGA.
6. Database infrastructure coordinates provisioning, backup and migrations; each domain owns its business schema and invariants.
7. APIs and workers use typed errors, capability negotiation, correlation IDs, limits, cancellation and provenance.

---

## 3. Existing repository inventory and classification

On 2026-11-15 the accessible GitHub inventory contained **33 repositories**. This specification assigns responsibility to the **30 application, platform and engineering/tool repositories** below. The other three are:

| Repository | Organizational role | Runtime status |
|---|---|---|
| `cirnet-rules` | Baseline AI Agent rules, task-specific Skills, policies and verification/distribution guidance | Development governance only; not a CIRNET business runtime |
| `about-cirnetlab` | Organization profile, brand and approved high-level documentation; possible canonical architecture host after approval | Not a product runtime |
| `.github` | GitHub organization/profile/configuration/automation metadata | Not a CIRNET product runtime |

The repositories `cirnet-platform`, `cirnet-toolchains`, `cirnet-devices`, `cirnet-numerics`, `cirnet-integrations`, `cirnet-extensions`, `cirnet-robotics`, `cirnet-iot`, `cirnet-silicon`, and `cirnet-ai` **already exist**. At the checked revision their files were limited to README and LICENSE: the responsibilities here are **future owners, not deployed features**. Several other recent repos, including Studio, Learn, Analyze, Project, Format-Project, LMS and Database, are also repository placeholders.

The user reports local clones under `E:\ALL_PROJECTS\LUAN_VAN_TOT_NGHIEP\REPO`. This is a **workspace root with independent clones**, not a unified Git repository or a filesystem we have inspected remotely. Local HEADs may differ from GitHub.

---

## 4. Repository Responsibility and Boundary Matrix

Every repository has one primary bounded context. **Outputs are target interfaces/artifacts**, not automatic implementation claims.

### 4.1 Applications and user experiences — 9 repositories

| Repository | Exclusive responsibility | Published output / clients | Must not own |
|---|---|---|---|
| [`cirnet_web`](https://github.com/cirnetlab/cirnet_web) | Public site, approved content, demos clearly marked as demos | Static website, public content and SEO | Private learner records, Lab control, secrets, instructor answers |
| [`cirnet-interface`](https://github.com/cirnetlab/cirnet-interface) | CircuitGraph editor, 3D breadboard/Three.js Lab UI and instrument panels | Reusable Lab editor/workbench, typed Lab client | Electrical-safety authority, 3D-coordinate inferred connectivity, direct FPGA writes |
| [`cirnet-studio`](https://github.com/cirnetlab/cirnet-studio) | Cross-domain workspace, project explorer, navigation and application composition | Workspace shell and shared UI composition | Copies of Circuit Editor, IDE engine, numerical kernels or LMS domain logic |
| [`cirnet-ide`](https://github.com/cirnetlab/cirnet-ide) | Editor, LSP/debug UI, terminal, build/flash controls, serial monitor | IDE interfaces and typed editor commands | Compiler/SDK installation authority, unrestricted USB privilege, own board compiler forks |
| [`cirnet-blocks`](https://github.com/cirnetlab/cirnet-blocks) | Visual typed language, block AST/IR, code generation and diagnostics maps | Block IR and source export with test fixtures | Raw flashing or privileged execution of generated code |
| [`cirnet-analyze`](https://github.com/cirnetlab/cirnet-analyze) | Numerical-analysis experience, notebooks, charts, signal visualization | Analysis documents, reproducible jobs and plots | Canonical numeric kernels, alteration of raw measurement history |
| [`cirnet-learn`](https://github.com/cirnetlab/cirnet-learn) | Student/instructor learning UX, coursework, feedback and progress | LMS client surfaces | Authoritative browser-side grading, enrollment/gradebook database |
| [`cirnet-desktop`](https://github.com/cirnetlab/cirnet-desktop) | Native desktop shell, scoped OS file/USB/serial bridge, update/offline integration | Signed app and privileged native bridge | Cloned Lab/Studio/IDE business engines; arbitrary renderer-to-native calls |
| [`cirnet-app`](https://github.com/cirnetlab/cirnet-app) | Mobile/tablet UX, learning/projects/device observation, scoped offline cache | Mobile client and permitted notifications | Universal FPGA/compiler/flash promises; hardware safety authority |

### 4.2 Platform, shared data and education — 6 repositories

| Repository | Exclusive responsibility | Published output / clients | Must not own |
|---|---|---|---|
| [`cirnet-platform`](https://github.com/cirnetlab/cirnet-platform) | Common identity, institution/tenant, organization membership, SSO/OIDC, sessions, cross-app policy and quotas | IAM/tenant APIs, scoped claims, audit context | Grades, Lab leases, electrical safety, project file codecs |
| [`cirnet-contracts`](https://github.com/cirnetlab/cirnet-contracts) | Canonical cross-repo DTOs, API/RPC/NCAP protocols, generated Rust/TypeScript SDKs and conformance fixtures | Versioned schemas/SDKs and wire descriptors | Domain business routes, simulation or hardware logic |
| [`cirnet-project`](https://github.com/cirnetlab/cirnet-project) | EngineeringProject aggregate, owner references, revisions, artifact links and collaboration lifecycle | Project API, immutable revision IDs, artifact registry references | Portable file codec, LMS grades, raw Lab capture bytes |
| [`cirnet-format-project`](https://github.com/cirnetlab/cirnet-format-project) | Portable project package/manifest, serialization, import/export, versioned migration and integrity validation | Package specification, readers/writers and fixtures | Project authorization, durable DB, UI, grading |
| [`cirnet-database`](https://github.com/cirnetlab/cirnet-database) | DB topology/provisioning, migration conventions/coordination, backup/restore and retention tooling | Infrastructure code, runner, tested recovery procedures | Single ownership of all business data tables and domain logic |
| [`cirnet-lms`](https://github.com/cirnetlab/cirnet-lms) | Courses, classes/enrollments, learning assignments, submissions, rubrics, gradebook and competency records | LMS APIs, assessment events, learning records | Source-project implementation, raw FPGA capture, global IAM, direct Lab hardware control |

### 4.3 Lab, simulation and instruments — 5 repositories

| Repository | Exclusive responsibility | Published output / clients | Must not own |
|---|---|---|---|
| [`cirnet-remote`](https://github.com/cirnetlab/cirnet-remote) | **Lab domain control plane**, including admission, experiments, technical Lab assignment configuration, scheduling/leases, APIs/WS, plans, capture persistence and **Lab release-set selection**. See §5 for standalone ownership. | Authorized Lab APIs, operation events and pinned Lab manifest | Shared identity or project lifecycle after the respective cutover, hardware drivers, academic gradebook |
| [`cirnet-hardware`](https://github.com/cirnetlab/cirnet-hardware) | Rust Virtual/Physical HardwareStation, local RPC, worker supervision, safe Stop and FPGA transport | Station RPC, capabilities, protection/driver status | Business IAM, grading, bypassing protection |
| [`cirnet-simulator`](https://github.com/cirnetlab/cirnet-simulator) | **Long-term default C++** discrete-event digital simulation, bounded DC, private worker protocol and SPICE adapter | Pinned worker and honest simulation result/waveform | User rights, physical FPGA control, unsupported analog claims |
| [`cirnet-fpga`](https://github.com/cirnetlab/cirnet-fpga) | RTL, FPGA protocol implementation, timing/trigger/capture, protected routing, SDRAM/CDC and testbench | RTL sources, verified board bitstreams, timing/HIL artifacts | Web API, global IAM, scheduler or project database |
| [`cirnet-nhspice`](https://github.com/cirnetlab/cirnet-nhspice) | **Conditional owner** for independently versioned native SPICE build/patch/packaging, model provenance and licensing | SPICE native dependency package and compatibility evidence | A second competing public Simulator API or worker authority |

**Open ownership decision:** the `cirnet-nhspice` README title currently reads `cirnet-core-spice`, unlike its repository name. Decide whether independent SPICE packaging/patches are needed, then align naming and licensing. The simulator still owns solver adapters and execution. Do not rename/delete a repository or promise full SPICE until its role and actual implementation are verified.

### 4.4 Developer tools, computing and extensions — 6 repositories

| Repository | Exclusive responsibility | Published output /clients | Must not own |
|---|---|---|---|
| [`cirnet-cli`](https://github.com/cirnetlab/cirnet-cli) | User-facing project/dev CLI and command UX | Versioned `cirnet` commands and binary | Implementing compilers, board drivers, domain engines inside CLI |
| [`cirnet-toolchains`](https://github.com/cirnetlab/cirnet-toolchains) | Compiler/SDK registries, pinned tool environments, sandbox build adapters and deterministic outputs | Toolchain manifests, build specs, binary artifacts and logs | Unapproved flashing, IAM, all target-board definitions |
| [`cirnet-devices`](https://github.com/cirnetlab/cirnet-devices) | MCU/SBC/FPGA BoardProfiles, chip/pin/power/debug/flash target adapters and capabilities | Versioned board catalog, upload/debug adapters | Cloud IoT fleet, Lab shared station lease, academic grading |
| [`cirnet-numerics`](https://github.com/cirnetlab/cirnet-numerics) | Reusable scientific/numerical kernels: matrices, FFT, statistics, ODE, optimization | Numeric library/APIs, precision and benchmark contracts | UI notebooks, digital circuit simulator ownership |
| [`cirnet-integrations`](https://github.com/cirnetlab/cirnet-integrations) | External LMS, Git, EDA, data-provider interoperability with scoped adapters | Versioned connectors and compatibility tests | A duplicate internal LMS, IAM or hardware driver |
| [`cirnet-extensions`](https://github.com/cirnetlab/cirnet-extensions) | Plug-in SDK/manifest, lifecycle, signature/trust, version and sandbox/permission policy | Extension format, host API and validation tools | Unrestricted arbitrary code execution or safety bypass |

### 4.5 Advanced engineering domains — 4 repositories

| Repository | Exclusive responsibility | Published output / clients | Must not own |
|---|---|---|---|
| [`cirnet-robotics`](https://github.com/cirnetlab/cirnet-robotics) | Robot model, coordinates/kinematics, motion/control, simulation adapters and experiments | Robot models, planning/control/evidence contracts | General numerical kernels, every board compiler, assumption that electrical safety covers motion safety |
| [`cirnet-iot`](https://github.com/cirnetlab/cirnet-iot) | Managed IoT device instances/fleet, provisioning, telemetry, device twins and command lifecycle | Device APIs, time-series/event streams and alerts | Board catalog (`devices`), protected Lab station authority (`hardware`) |
| [`cirnet-silicon`](https://github.com/cirnetlab/cirnet-silicon) | RTL-to-ASIC learning, synthesis/STA/EDA/PDK and physical-design workflows | Verified EDA artifacts, design-rule and sign-off evidence references | Lab FPGA control firmware, generic toolchain installation or uncontrolled PDK distribution |
| [`cirnet-ai`](https://github.com/cirnetlab/cirnet-ai) | Advisory tutoring, engineering assistants, retrieval and human-review workflows | Permission-scoped suggestions/explanations and audit data | Final academic grades, physical safety permits, autonomous flash/power/route approvals |

**Count:** 9 + 6 + 5 + 6 + 4 = **30 product/tool repositories**, plus the three organization/rules repositories = **33**.

---

## 5. Mandatory non-overlap boundaries

| Potential overlap | Architecture decision |
|---|---|
| Studio vs Interface | Studio composes tools; Interface owns CircuitGraph/Three.js Lab editing and electrical UX semantics |
| Learn vs LMS | Learn owns experiences; LMS owns courses, submissions, rubrics, academic grade data |
| Project vs Format-Project vs Contracts | Project owns lifecycle; Format owns portable serialization; Contracts owns versioned inter-boundary DTOs |
| Platform vs Remote | Platform becomes the shared identity and tenant authority after tested cutover. Remote retains Lab authorization, admission, leases and safety policy. |
| Project / LMS vs Remote | Project owns shared project lifecycle after cutover. LMS owns academic assignments and grades. Remote owns technical Lab configuration, experiment records and evidence. |
| Database vs domain schemas | Database owns provisioning/recovery conventions; each domain owns its business schema/migrations |
| IDE vs Toolchains vs Devices vs CLI | IDE/CLI are interfaces; Toolchains build; Devices identify/flash/debug targets with separate authorization |
| Analyze vs Numerics vs Simulator | Analyze visualizes/jobs; Numerics computes general kernels; Simulator owns C++ electronic simulation semantics |
| Simulator vs nhspice | Simulator owns adapter/worker contract; native SPICE packaging is optional independent dependency ownership |
| FPGA vs Silicon vs learner bitstreams | FPGA controls Lab RTL/board; Silicon owns ASIC/EDA learning workflows; untrusted learner bitstreams never replace trusted Lab Control FPGA |
| Hardware vs IoT vs Devices | Hardware owns protected shared Lab stations; IoT owns enrolled fleet instances; Devices owns target type/board adapters |
| Extension vs Integration | Extensions define host plug-in capability/security; Integrations implement external-provider adapters through those or regular typed APIs |
| Rules vs runtime | `cirnet-rules` governs development agents/doc quality; it does not approve student runs or implement product policies |

No repository may become a catch-all for every missing feature. Split by **independently meaningful bounded context**, security boundary or independently distributed SDK, not by endpoint/CRUD table. An owner's public API is not permission for consumers to reimplement its internals.

### 5.1 Standalone Lab and integrated ownership

| Records or decisions | Standalone Lab or before cutover | Integrated scope after cutover |
|---|---|---|
| Identity and tenant context | Remote issues Lab-local sessions for its own principals. | Platform supplies shared identity and tenant context. Remote verifies that context and applies Lab-specific permissions. |
| Project identity and revisions | Remote owns the Lab-local project records used for circuit editing and runs. | Project owns shared project IDs, revisions and ACLs. Remote references authorized revisions and keeps the exact inputs and evidence of each experiment. |
| Assignments and assessment | Remote owns local Lab assignments, disclosure and technical feedback. | LMS owns academic assignments, submission, grading and academic disclosure. Remote owns versioned technical Lab configuration, permitted endpoints, technical results and evidence access under Lab policy. LMS references the assigned configuration and authorized evidence. |
| Lab operation and physical protection | Remote authorizes admission, leases and plan execution. Hardware and FPGA enforce station permits, interlocks and safe Stop. | The same authorities apply. |

Record one authority for writes and permission decisions on each resource in each tenant or workspace and record type. Before switching, map IDs and permissions, reconcile data, and test consumers and rollback. At cutover, fence legacy writes, switch to the designated owner and revoke superseded credentials where applicable. Account for writes made after cutover before rolling back.

---

## 6. Universal EngineeringProject model

The CircuitGraph is canonical for *electronic circuit intent* but cannot be the only representation for mechanical robots, numerical work, source trees or ASIC projects. Define a domain-neutral **EngineeringProject** aggregate:

```text
EngineeringProject
 ├─ identity / owner / tenant / permissions / collaborators
 ├─ immutable ProjectRevision + working branch/commits
 ├─ artifact references
 │   ├─ CircuitGraph / Schematic / 3D model / PCB interchange
 │   ├─ source code / typed block IR / RTL / netlist
 │   ├─ robotics or IoT model / notebook / dataset
 │   ├─ compiled firmware / FPGA bitstream / EDA output
 │   └─ measurements / captures / simulations / reports
 ├─ execution and build references
 │   ├─ BuildSpec / toolchain lock / target board profile
 │   ├─ SimulationRun / LabExperiment / HardwareExecution
 │   └─ provenance / status / errors / resulting evidence
 └─ optional LMS assignment/submission reference (not gradebook ownership)
```

**One source of truth per layer:** `project` controls aggregate and revisions; `format-project` controls portable on-disk/transfer encodings and migrations; `contracts` controls cross-service schemas; each domain controls its own model semantics; storage handles immutable bytes under authorized references. A 3D scene view does not define electrical connectivity.

### 6.1 Artifact envelope and reproducibility

At minimum, artifacts need stable ID, type, format/schema version, content digest, owner/tenant, generation source and timestamp, access labels, retention rules and references to parent revisions. Build/run records additionally include exact input digests, toolchain/engine version, execution environment, board/device identity where applicable, limits, outcomes, failure/partial status and evidence quality.

Do not overwrite raw measurements to improve educational scores or notebook appearance. Label distinct evidence as **MEASURED**, **SIMULATED**, **COMPUTED**, **GENERATED** or **INFERRED**. Digests protect integrity but are not proof of trusted authorship without an explicit signing/review policy.

### 6.2 Safe imports and exports

Portable packages require bounded parsing, schema/version validation, content-size limits, path traversal/symlink defense, file integrity, and strict handling of external links/secrets. Imports cannot grant ACLs, hardware permissions, arbitrary code execution or trust in bundled binaries. Exports require explicit owner/field/destination authorization.

---

## 7. Contract-driven integration and runtimes

### 7.1 Runtime technology decisions

| Boundary | Standard or target |
|---|---|
| Core backend, identity, project services, Lab Remote and Hardware | **Rust** by default; async APIs/typed domain models, isolated workers |
| Canonical runtime contracts, RPC/SDK generation | **Rust** implementations plus generated typed language clients |
| Browser apps | **TypeScript**, Vue-based UI and Three.js where 3D is appropriate |
| Electronics Simulator | **C++** long-term default, pinned supervised workers and versioned protocol |
| FPGA target and logic | **Verilog/SystemVerilog RTL**, vendor tooling and target-specific constraints |
| Desktop/native adapters | Selected audited native shell/bridge, respecting platform permissions |
| Scientific kernels | Rust by default where justified; native C/C++ FFI allowed with test/license/benchmark evidence |
| Platform verification | Rust/TS tests and **Python for testing, benchmark and comparison**, not for new platform production request handlers |
| Learner code | Language/toolchain dependent; Python/MicroPython can be supported only as **isolated user workloads or board runtimes**, a separate product decision |

Do not rewrite the C++ Simulator in Rust. Do not assume that providing Python to learners licenses Python-based privileged production backend services.

### 7.2 API and execution rules

- Public REST/WS/event APIs are owned by the relevant domain; SDK/descriptor/schema versions are maintained by Contracts.
- Hardware RPC is local-only and requires authenticated, authorized local access. Worker frames stay between Hardware and the supervised Simulator worker. Simulator owns that private protocol. Contracts owns shared RPC and logical schemas.
- An accepted request is not a completed operation; represent QUEUED, RUNNING, COMPLETE, FAILED, CANCELED, PARTIAL and UNAVAILABLE accurately.
- Explicit idempotency, replay protection, correlation IDs, bounded queues, cancellation and timeouts apply to mutating or long-running operations.
- API, schema, RPC, worker ABI, firmware, project-format and release versions are independent and need compatibility negotiation and migration plans.
- Generated consumers must be rebuilt from canonical owner changes, not hand-patched in each repository.

### 7.3 Execution, authorization and isolation

Compiling a project, flashing a target, running a physical experiment, starting a robot, grading a submission and publishing a model are **distinct privileged actions**. Check actor, tenant, project/device ownership, requested operation, destination and current safety/lease state. Jobs handling untrusted user source, notebooks, HDL, third-party models or extensions require isolation, resource limits, audit and network/secret restrictions.

---

## 8. CIRNET Lab-First — retained specialist architecture

[Lab-First](cirnet-architecture-lab-first-en.md) governs electronics execution, endpoint binding and electrical safety. [FPGA Architecture](../cirnetlab-FPGA-architecture-en.md) specifies board and RTL behavior within those limits. This Ecosystem document governs how Lab integrates with the other domains.

### 8.1 Required real Virtual Digital end-to-end chain

```text
Browser Lab Interface (TypeScript/Three.js)
  → Rust Lab Remote: authentication, experiment/lease, immutable plan
  → Rust HardwareStation: supervised execution and bounded worker I/O
  → C++ Simulator: digest-pinned, deterministic digital engine
  → HardwareStation → Remote → authorized Web results/capture
```

Use the [VD-01–VD-10 acceptance gates](cirnet-architecture-lab-first-en.md#65-acceptance-gates-virtual-digital-e2e) for the selected artifact set and tested environment.

### 8.2 Actual student wiring, topology and endpoint resolution

```text
Authorized lease → verified safe state → bounded scan under its own permit
                         ↓
Observed topology + assignment/profile → resolve required endpoints
                         ↓
Prepare infrastructure-only plan → verify exact plan and safety
                         ↓
Apply and verify routes with DUT power and experiment stimuli disabled
                         ↓
Guarded power-up → authorized stimuli → actual measurement and assessment
```

- `reference_node_id` is **not** automatically `observed_node_id`. Missing required observed bindings → `BLOCK_UNEXECUTABLE`.
- `SafetyVerificationResult` is a hard gate: `UNSAFE` or `INDETERMINATE` prevents energization.
- `TopologyVerificationResult` is diagnostic/grading evidence; a mismatch alone must not deny a safe, executable run.
- `FunctionalVerificationResult` comes from real behavior after execution; UI must never fabricate a passing waveform.
- The Physical Compiler **MUST NOT** repair student IC/jumper wiring using FPGA routing.
- Hardware needs independent current/voltage protection, fail-safe startup/reset, watchdog, measured cutoff and board HIL before physical capability is enabled.
- A qualified shared fabric offers **one physical execution slot** until electrically isolated partitions are verified. Unqualified profiles advertise zero physical slots. Unconfirmed safe cleanup triggers quarantine and prevents reassignment.
- FPGA RTL tests, synthesis and timing analysis alone do not prove board electrical safety.

### 8.3 FPGA dual-role protection

Separate a **trusted Lab Control FPGA** from a **learner-programmable FPGA target**. Restrict learner bitstreams to isolated targets with verified BoardProfiles and applicable upload permissions. Reserve Lab Control FPGA programming for authorized, qualified station firmware.

### 8.4 Analog/measurements

`cirnet-simulator` owns the default C++ simulator and any SPICE adapter. The optional `cirnet-nhspice` boundary concerns native dependency packaging only. `virtualAnalog=true` requires a real native solver, model provenance/licensing, typed errors/cancellation and acceptance for the exact E2E artifact set. Bounded DC, Virtual Analog and physical analog routing have separate criteria. Oscilloscope 2CH, Generator 2CH, Logic Analyzer, SDRAM and ADC/DAC/AFE are hardware **targets** until measured and validated for a specific board.

---

## 9. IDE, blocks, toolchains and target devices

```text
IDE source / BlockProgram / HDL project
  → typed validation and versioned source/IR
  → BuildSpecification + exact toolchain/SDK lock + BoardProfile
  → sandboxed compile/simulate/synthesize/link
  → content-addressed BuildArtifact + diagnostics
  → separate authorization for Flash/Debug/Deploy
  → selected target with compatible power/boot/debug constraints
  → verified operation logs / telemetry / Project artifact references
```

`cirnet-toolchains` owns compiler and SDK selection, reproducible job execution and binary provenance. `cirnet-devices` owns board/target profiles and upload/debug adapters. `cirnet-ide` and `cirnet-cli` provide controls and diagnostics, not competing compilers; `cirnet-blocks` provides typed AST/IR and deterministic source generation, not direct privileged flashing.

BoardProfile covers vendor, chip family/part, flash/boot/debug protocols, pin/power/memory capabilities, target toolchains, supported SDKs, compatible binaries and validation status. ESP32, STM32, Arduino-compatible MCU boards, PIC, Raspberry Pi/Orange Pi SBCs and FPGA are **different target families**, not interchangeable flashing workflows. Build success is not deployment or physical safety approval.

Source maps should link generated code diagnostics back to block positions. For FPGA/ASIC learning, toolchains, silicon flows and board programming need separate evidence and authorization.

---

## 10. Scientific computing and engineering analysis

`cirnet-numerics` supplies reusable numerical kernels (linear algebra, statistics, FFT, signal processing, integration/ODEs, optimization and precision/units policies). `cirnet-analyze` supplies the application experience: plots, datasets, notebooks, interactive analysis, experiment comparison and reports. The C++ electronics Simulator remains the domain owner of discrete-event circuit semantics and its SPICE adapter.

Every notebook/job must pin engine, environment/dependencies, input digests, limits and output, and must isolate arbitrary learner code from privileged services. Propagate units, sample-rate/clock quality, calibration/uncertainty, missing samples, clipping and provenance rather than silently presenting simulated, calculated and measured results as equivalent. External libraries and licensed toolboxes are subject to license, trust, ABI and reproducibility review.

---

## 11. CIRNET Learn and LMS — evidence-driven education

In the integrated profile (§5.1), Platform owns institution, identity, common membership and policy. LMS owns academic assignments, submissions, rubrics, grades and progress. Learn provides the interfaces. Project owns engineering revisions and artifact references. Lab Remote retains technical experiment records and capture persistence, with measurements produced by the station.

```text
Instructor → LMS assignment/rubric + allowed lab/device capabilities
Student → Learn → EngineeringProject revision
        → Studio / IDE / Blocks / Lab / Analyze
        → immutable submission reference + exact evidence digests
LMS → permitted checks + instructor review + academic audit
    → authorized feedback and grade publication
```

Automated evidence checks, instructor grading, and AI-generated suggestions are separate; AI assistance is never automatically the final grading authority. Instructor answers/reference circuits require disclosure controls across REST, WebSocket, captures, exports and diagnostics. Learner data, including minors' records where applicable, needs tenant segregation, retention/consent policies, grade change audit and human appeal/review. External LMS interoperability should use appropriately versioned adapters (for example LTI/roster integration if adopted) rather than copying the entire gradebook into CIRNET.

---

## 12. Advanced domains: Robotics, IoT, Silicon, AI

### 12.1 Robotics

`cirnet-robotics` owns robot models, frames/joints/kinematics, control, motion planning, sensor simulation and selected robotics interfaces. It consumes Numerics, Devices, Toolchains, Project and Analyze. Physical motion requires **mechanical safety**: motion envelope, actuator limits, emergency stop, collision/pinch hazards, watchdog and operation authorization. The Lab electrical-safety permit alone cannot prove motion safety.

### 12.2 IoT and edge computing

`cirnet-iot` owns enrolled *device instances*, provisioning/credential lifecycle, twins, telemetry/time series, commands and alerts. Distinguish **board type/catalog** (`devices`), **managed field device instance** (`iot`) and **protected shared Lab station** (`hardware`). Telemetry access and remote commands must respect tenant ownership, transport credentials, quotas and audit; public UI or an AI agent cannot invoke raw privileged device controls.

### 12.3 Semiconductor and EDA learning

`cirnet-silicon` owns the educational HDL→synthesis→STA→physical design→DRC/LVS workflow, PDK adapters and sign-off evidence references. Reuse `cirnet-fpga` where genuine RTL/testbench or board-specific capability belongs, `toolchains` for exact tools, `devices` for board programming and `project` for artifacts. FPGA simulation/synthesis success **does not** imply ASIC tapeout readiness; PDK rights, corner analysis, STA, physical rule checking, licensed tooling and review must be evaluated separately.

### 12.4 AI learning and engineering assistance

`cirnet-ai` may explain code, propose fixes, search authorized course/project documents and suggest model experiments, subject to data minimization, privacy, cost/rate policy, citations and human review. AI output is **advisory**. It must not bypass hardware interlocks, grant itself board privileges, flash firmware, issue source permits or silently grade/publish learner results.

---

## 13. Multi-platform application strategy

- **Web:** primary broad-access engineering/education environment, typed SDKs, accessible responsive UX and client-side 3D rendering when appropriate.
- **Desktop:** reuse shared app modules while adding **explicitly permissioned** local filesystem, USB/serial/debug and offline tooling; isolate renderer from native privilege with an audited bridge.
- **Tablet/Mobile:** focus on learning, projects, submissions, observation, lightweight design/simulation and permitted device management. Do not assume OS-level access to every compiler/programmer.
- **Shared UI design:** reusable design tokens, navigation, authentication and common widgets; one app may consume a package published by another owner, but source copies and divergent business logic are forbidden.
- **Offline:** local drafts must retain revision IDs and conflict semantics; stale offline state cannot command hardware, refresh safety evidence or bypass live authorization.

---

## 14. Identity, privacy, data governance and security

### 14.1 Authority separation

In an integrated deployment, **Platform identity** establishes principal and tenant context, **LMS membership** establishes the educational relationship and **Project ACLs** govern engineering content. For every Lab deployment, Remote checks Lab permissions and the active lease. Hardware and independent protection enforce physical actuation conditions. Each check is required within its own scope.

### 14.2 Security controls

- TLS and appropriate session/token policy; service-to-service auth where necessary; deny-by-default scopes for mutating APIs.
- Accurate Origin/CSRF/cookie protections for browser session flows; device and worker interfaces do not become WAN-accessible by default.
- Quotas for compute, storage, concurrent jobs, capture streaming and learners' own code; sandbox and secret/network controls for untrusted workloads.
- Data isolation among organizations/classes; restrict export of projects, datasets, waveforms and instructor solutions according to owner and destination.
- Provenance/audit for firmware uploads, hardware stops, regrading, plugin installation, external-data connector actions and release selection.
- Retention/backup/data portability policies distinguish raw measurements, grades, student identities, public materials and restricted PDK/IP assets.
- Explicitly treat signed code, checksums, license conformance and vulnerability review as separate evidence dimensions.

### 14.3 Learning analytics

Learning metrics are derived from authorized LMS records and immutable project/experiment references. Avoid implicit surveillance or covert AI profiling. Algorithms, thresholds, disclosures and the meaning of competence scores require institutional review. An analytics recommendation is not a verified physical experiment result.

---

## 15. Deployment and operational topology

Supported target patterns can include browser+cloud application, school-managed on-prem, a hybrid installation and a **local Mini PC/Edge Lab** with directly connected instruments. Exact installation claims depend on target OS, network, storage, hardware and measurements.

```text
School / Cloud control and learning services
  ├─ Platform / Project / LMS / registered domain APIs
  ├─ DB + authorized immutable artifact storage + backup
  ├─ async/sandboxed build and analysis workers
  └─ authorized Lab Remote / gateway
               ↓ scoped, authenticated operation
Lab/Edge host (Linux target, as verified)
  ├─ Rust Remote / Rust HardwareStation
  ├─ pinned C++ Simulator worker(s)
  └─ protected FPGA instrumentation and physical station
```

No direct WAN access to private FPGA register paths or Hardware RPC. Nginx/systemd/PostgreSQL/worker supervision and bounded local IPC are suitable examples for edge deployments; container orchestrators are not required by default. Operational readiness includes health, cancellation, Stop, backups, schema migrations, recovery, physical fault quarantine, power and resource monitoring.

---

## 16. Releases, artifact pinning and compatibility

### 16.1 Naming convention

For **new** CIRNETLAB repository releases:

```text
Git tag        = <repo_name>-<version>
GitHub Release = <repo_name>-<version>
Example       = cirnet-fpga-0.2.2   # format illustration only
```

Repository, SDK, schema, API, RPC, worker protocol, project-format, board/firmware, simulation model, PDK/toolchain and integrated system versions remain **distinct domains**. Never rename a historic published tag or silently alter pinned release artifacts.

### 16.2 Integrated release manifest

Each selected application or Lab system release set records exact repository/source revisions, built artifact SHA-256, compiler/SDK/dependency versions, schema/API/RPC/NCAP/FPGA compatibility, target OS, board profiles where relevant, database migration bounds and rollback/restore prerequisites. Never use moving `latest`/branch HEAD as a substitute for a verified digest.

For Lab, `cirnet-remote` owns release-set selection. Resolve the manifest path against the selected revision and deployment profile. See [Lab-First §6.4](cirnet-architecture-lab-first-en.md#64-single-artifact-set-and-pinning-requirements). An ecosystem-wide composition manifest can reference the exact Lab release lock. Its own location and owner remain open decisions.

### 16.3 Promotion and rollback

Contract owner publishes compatible schemas/SDKs; engines/toolchains/devices build testable artifacts; consumers rebuild; selected integration tests execute across real boundaries; deployment release set is then locked. Never promote based only on matching version labels. A rollback must account for database/artifact/capture compatibility and must drain or quarantine physical stations without replaying prior Run/Start commands.

---

## 17. Verification and architecture conformance

| Verification layer | Required evidence |
|---|---|
| Repo-local | Unit/lint/type tests, bounded errors, source/lock/license checks |
| Contract | Generated Rust/TS SDKs, wire formats, compatibility fixtures and old/new reader/writer tests |
| Engineering build | Selected compiler/SDK, BoardProfile and exact artifact digests; deterministic build tests |
| Simulation | Real pinned C++ worker, event/timing/error parity, limits, Stop and explicit unsupported status |
| Platform/project/LMS | Auth, tenancy, membership, revisions, submissions, grading/audit and data export rules |
| Browser/desktop/mobile | Real client integration, permissions, reconnect/offline/version mismatch and correct capabilities |
| Physical Lab | Measured electrical envelope, HIL interlock/fault injection, actual endpoint binding, no-auto-rewiring |
| FPGA/EDA | RTL, CDC/reset, synthesis/fitter/timing, board measurements; ASIC verification separate |
| Security/privacy | Tenant isolation, job sandbox, injection/secret defenses, plugin trust, audit and retention |
| Load/operations | Resource boundedness, tail latency, DB/capture recovery and measured target configuration |
| Release | Exact artifact manifest, cross-repo compatibility, migration, rollback and provenance |

Distinguish `RTL simulation PASS` from electrical safety, `Contract tests PASS` from a real E2E, `Windows candidate PASS` from Linux production and `digital/DC model PASS` from full SPICE. The status `VERIFIED` applies only to actually tested conditions. Attach exact source/asset hashes and command exit codes; an unexecuted test is `NOT_RUN`.

---

## 18. Repository development, documentation and agent policy

The GitHub organization maintains **independent repositories**. Local clones under `E:\ALL_PROJECTS\LUAN_VAN_TOT_NGHIEP\REPO` do not constitute a monorepo or implicitly grant an agent permission to modify all descendants.

1. Read organization-wide CIRNET architecture; then read the relevant specialist architecture (`Lab-First`, FPGA), the repository `AGENTS.md` and task-specific rules/Skills from `cirnet-rules`.
2. Identify the owning bounded context, exact current selected contracts/artifact versions and permitted change scope.
3. Prefer the smallest maintainable change in the owning repository; do not duplicate schemas, engines or source files in consumers.
4. Write cross-repo change/compatibility tests before claiming an integrated capability.
5. Apply **Search Before Create → Update Existing → Consolidate When Appropriate → Verify Documentation**. Keep a canonical source, EN/VI correspondence and historic test evidence.
6. Preserve student/instructor data, ignored/untracked files, frozen releases and hardware evidence. Do not delete/migrate/force-reset another repository to tidy the workspace.
7. Review diff, license impacts, API compatibility, provenance and test output. Git commit/push/merge/tag/release is separately authorized.
8. Report actual `IMPLEMENTED`, `TESTED`, `VERIFIED`, `NOT_RUN` and `BLOCKED`; never represent an AI-proposed architecture as deployed software.

### 18.1 Architecture document ownership

This EN/VI pair is the toolkit's [canonical Ecosystem reference](../../en/policies/architecture.md) for organization boundaries and integration. Approval and the long-term Organization publication location remain open (§21). Domain repositories link to this reference and the applicable Lab-First or FPGA architecture. Keep the paired editions aligned by meaning and retain historical revisions in Git.

---

## 19. Representative cross-repository workflows

### 19.1 A digital electronics assignment

In the integrated profile, an instructor links an LMS assignment and rubric to a versioned technical configuration in Remote with approved Lab endpoints. A learner opens Learn and a Project revision in Studio/Interface, authors CircuitGraph, and runs a compatible Virtual Digital experiment through Remote → Hardware → pinned C++ Simulator. Waveforms, results and provenance return to the authorized viewer. Analyze may process the evidence. LMS grades against exact references and disclosure policy. A physical exercise additionally requires observed topology, safety, executability and actual measurement.

### 19.2 Embedded firmware design

IDE or Blocks creates source linked to a Project revision. Toolchains selects a pinned compiler/SDK, Devices selects a validated ESP32/STM32/Arduino/PIC/SBC BoardProfile, and sandboxed build produces an artifact with checksum. Flashing is an **independent, authorized** operation against the identified target, with verify/logs stored for the project and optionally referenced by an LMS submission. A README-only target adapter must not appear as supported.

### 19.3 Scientific data analysis

Hardware, Simulator or IoT produces typed capture/telemetry with provenance. Project registers immutable inputs; Analyze runs Numerics/plot jobs, retaining units, input digests and uncertainties. Reports may be published or submitted under authorization without modifying raw measurements.

### 19.4 FPGA and semiconductor learning

IDE/Project stores RTL, constraints and target profile. Toolchains executes pinned HDL/EDA jobs; FPGA owns Lab controller RTL where appropriate; Silicon owns ASIC/PDK learning workflows. Analyze displays logs/waveforms and LMS references evidence. A learner's FPGA bitstream never crosses the trust boundary into the Lab's protected control fabric.

### 19.5 Robotics and IoT

Robotics models and numerical controls are compiled for Device targets. IoT manages enrolled instances, credentials and telemetry; Analyze displays results. Physical actuation requires an independent motion/device safety policy, not an inferred permission from course enrollment.

---

## 20. Architectural decision records

| ID | Decision | Non-negotiable consequence |
|---|---|---|
| CIRNET-ECO-001 | Ecosystem governs Organization boundaries and integration | Lab-First governs Lab execution and electrical safety |
| CIRNET-ECO-002 | Multi-platform apps share one EngineeringProject and versioned contracts | Do not clone business data/engines for each UI |
| CIRNET-ECO-003 | One source of truth per bounded context | Platform, Project, LMS, Remote, Format and Database roles stay separate |
| CIRNET-ECO-004 | Rust-first services, **C++ Simulator default**, FPGA RTL | No forced Rust Simulator rewrite or new privileged Python services |
| CIRNET-ECO-005 | Contract-first and compatibility-first | Migrate canonical owners and generated consumers together |
| CIRNET-ECO-006 | Build, flash, run, deploy and grade are distinct authorities | UI, agent and extensions cannot bypass privileged approval |
| CIRNET-ECO-007 | Global IAM, academic authority and physical permits are distinct | Instructor role or AI advice does not prove hardware SAFE |
| CIRNET-ECO-008 | Immutable artifacts and reproducible evidence | Exact input/output/protocol/tool/environment digests |
| CIRNET-ECO-009 | Capability flags reflect real tested support | Do not manufacture solver, measurement or device results |
| CIRNET-ECO-010 | Physical routing is infrastructure-only | Never fix students' physical wiring with FPGA |
| CIRNET-ECO-011 | Modular ownership, deployment only as needed | No forced microservices/monorepo or excessive repository splitting |
| CIRNET-ECO-012 | Sandboxed builds/extensions and human-visible side effects | Untrusted code cannot access privileged production services |
| CIRNET-ECO-013 | Tenant/academic/IP confidentiality is a primary requirement | Separate data and export policies per domain |
| CIRNET-ECO-014 | Stable naming, independent version domains | `<repo_name>-<version>` for new releases; preserve historical pins |
| CIRNET-ECO-015 | Canonical documentation and preserved evidence | Update existing, avoid duplication, maintain EN/VI equivalence |
| CIRNET-ECO-016 | Trusted Lab Control FPGA is separate from learner FPGA targets | Student HDL never gains Lab electrical permits by build or upload alone |

---

## 21. Open decisions at Organization level — not a development schedule

1. **Canonical document location:** approve the exact repository/path for this global architecture, specialist Lab-First and FPGA references and EN/VI sync policy.
2. **Identity, Project and LMS migrations:** follow §5.1 and verify the actual issuers, writers and consumers before each scoped cutover or rollback.
3. **Project versus format versus contracts:** freeze the precise EngineeringProject revision, portable format and public DTO compatibility rules.
4. **SPICE dependency:** decide whether `cirnet-nhspice` will be independently built/licensed/distributed or whether a pinned external engine from Simulator is sufficient; resolve the README-title discrepancy.
5. **Real Lab E2E:** pin Rust Remote, Rust Hardware, C++ worker, interface and Contracts on a verified test set; physical FPGA still needs HIL.
6. **Language policy:** Python stays out of new privileged platform production services; learner Python/MicroPython requires a separate sandbox/user-code decision.
7. **Shared design system:** decide which packages/owners provide cross-app UI primitives without copying source across clients.
8. **Deployment/data residency:** define school tenancy, on-prem/cloud/hybrid identity, artifact store, backup and disaster recovery.
9. **External licensing:** verify compiler SDKs, SPICE libraries, EDA/PDKs, ROS-related tools, numerical kernels and extensions before redistribution.
10. **Academic governance:** institutional policy for consent, minors, grading audits, retention, access and external LMS interoperability.
11. **AI controls:** data sharing, model-provider permission, cost quota, explanation and human oversight, especially around side effects.
12. **Truthful runtime status:** README-only repositories must not be labeled as implemented or released capabilities.

Each bounded context can evolve at its own cadence while respecting compatibility, security, evidence and existing consumer obligations. The [Lab-First](cirnet-architecture-lab-first-en.md) acceptance milestones apply within the Lab domain.

---

## 22. Long-term architectural summary

**CIRNET = EngineeringProject + Shared Platform + Domain Applications + Verified Engineering Engines + Learning Evidence.** The six Lab-First repositories remain the electronics virtual-to-physical core. The larger ecosystem adds IDE/Blocks/Toolchains/Devices, Analyze/Numerics, Learn/LMS, Projects/Formats/Database, Studio/Desktop/Mobile, Robotics/IoT/Silicon/AI and shared extension/integration boundaries.

The GitHub inventory already supplies **30 bounded product/tool repositories plus CIRNET Rules and two organization documentation/configuration repositories**. Additional repositories are not necessary merely to communicate ambition. The durable architecture is determined by precise ownership, independently versioned contracts, a real selected release set, secure and reproducible execution, education-data integrity, and physical safety backed by appropriately scoped evidence.

A learner should be able to move **Learn → Studio/IDE/Blocks → Lab/Analyze → Submission/Assessment** within the same EngineeringProject while never confusing an editor, a simulator, a physical instrument, an LMS gradebook or an AI suggestion with a shared source of authority.

> **This document records target architecture, not proof of existing implementation, production maturity, physical electrical safety, EDA sign-off or educational outcomes.** Any externally advertised capability must reference actual code, exact selected artifacts and relevant tests/measurements.

---

## Appendix A — Source and provenance

1. **User-provided Lab-First architecture:** `cirnet-architecture-lab-first(3).md` (~1,240 lines), particularly Contracts, Virtual Digital E2E, Physical Endpoint Resolution, Safety, HardwareStation, FPGA, release and verification sections.
2. **Existing Organization-wide architecture draft:** `cirnetlab-architecture.md` (~735 lines, Vietnamese), used as the translation/adaptation basis for this English edition.
3. **GitHub inventory:** [`cirnetlab`](https://github.com/orgs/cirnetlab/repositories) accessible list of **33 repositories** as checked on 2026-11-15. Source maturity and deployment are not inferred from repository presence.
4. **FPGA specialist document:** `cirnetlab-FPGA-architecture-vi.md`, rewritten from the user-provided legacy FPGA hardware design, retaining safety, clock/CDC, routing, capture, SDRAM and instrument constraints while adopting the Ecosystem ownership model.
5. **Local clone context:** Windows workspace root provided by the user; no direct operation was performed on their local repositories by generating these documents.
6. **Architectural proposals:** Role separation, Engine/IDE/LMS/Project integration, multi-tenant authorization, numerical computing, robotics/IoT/semiconductor and AI control models describe desired ownership, not verified features.

## Appendix B — Glossary

| Term | Meaning |
|---|---|
| Bounded context | Domain scope with explicit owner, semantics, data and interfaces |
| Canonical contract | Versioned cross-boundary agreement under one source owner |
| EngineeringProject | Aggregate of engineering identity, revisions and linked artifacts |
| Artifact | Source/model/binary/measurement/report with identity, checksum and provenance |
| Provenance | Trace of inputs, versions, builders, devices, runs and environments |
| Capability | Explicit, validated support advertised by a service or target |
| Virtual Lab | Simulation lane with bounded model assumptions, not physical measurement |
| Physical Lab | Protected experiment with actual devices and measured evidence |
| Release set | Compatible collection of pinned artifacts deployed/tested together |
| Source of truth | Unique authority for a domain datum or decision |
| HIL | Hardware-in-the-loop evidence at a specified board configuration |
| PDK | Process Design Kit for semiconductor technology constraints and models |
