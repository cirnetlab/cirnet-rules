# Repository ownership and architecture routing

This index follows [Ecosystem Architecture §4](../../docs/tai-lieu-kien-tuc-he-thong/cirnetlab-architecture-en.md).
Use it to find the relevant owner, then verify the live contract, selected
artifacts and availability in that checkout. The table records target ownership
and integration paths. Verify implementation in the
[product evidence](../docs/toolkit-verification.md#lab-first-product-evidence).

Every row uses `ecosystem`. The six core Lab-First repositories also use
`lab-first`. `fpga` applies to Lab board implementation. Select further overlays
when the task crosses one of those boundaries. Arrows show the direction of
the API call, data or artifact exchange between direct participants. See the
[architecture policy](../policies/architecture.md) for controlling scope.

## Applications

| Repository | Architecture | Bounded context | Owned interfaces and artifacts | Direct exchanges | Protected boundary |
| --- | --- | --- | --- | --- | --- |
| `cirnet_web` | `ecosystem` | Public site | Static site, approved content, labeled demos | Approved publishers → public readers | Keep private learner data, instructor answers, secrets and Lab controls outside the site. |
| `cirnet-interface` | `ecosystem`, `lab-first` | Lab workbench | CircuitGraph editor, Three.js breadboard, instrument panels, typed Lab client | Contracts and Remote → Interface → Studio and lab users | Graph semantics define connections. Remote and Hardware authorize actions and FPGA access. |
| `cirnet-studio` | `ecosystem` | Engineering workspace composition | Shell, navigation, project explorer, tool layout | Project, Interface, IDE and Analyze → Studio | Compose owner tools. Interface, IDE, Numerics and LMS keep their engines and domain logic. |
| `cirnet-ide` | `ecosystem` | Source editing and device workflow UI | Editor, LSP/debug UI, terminal, serial monitor, typed commands | Blocks or Project → IDE → Toolchains and Devices | Call pinned build and authorized device adapters. Toolchains owns compiler setup. |
| `cirnet-blocks` | `ecosystem` | Visual programming | Block AST/IR, type checks, source export, diagnostic maps | Blocks → IDE, Project and Toolchains | Source export gives no flash or privileged execution authority. |
| `cirnet-analyze` | `ecosystem` | Analysis experience | Notebooks, plots, reproducible jobs and derived measurements | Remote, Simulator or IoT plus Numerics → Analyze → Project | Preserve raw evidence, units and provenance. Numerics owns reusable kernels. |
| `cirnet-learn` | `ecosystem` | Learning experience | Coursework, feedback and progress UI | LMS → Learn, with links to Project and Lab | LMS owns gradebook, rubrics and academic decisions. Enforce allowed disclosure. |
| `cirnet-desktop` | `ecosystem` | Native host | Signed shell, scoped file/USB/serial bridge, update/offline support | Interface, Studio or IDE ↔ scoped Desktop bridge | Scope native privileges by action. Owner apps retain business logic. |
| `cirnet-app` | `ecosystem` | Mobile and tablet experience | Mobile client, notifications, scoped offline cache | LMS, Project and IoT → App | Confirm mobile capabilities. Device actions and safety stay with owners. |

## Platform and education

| Repository | Architecture | Bounded context | Owned interfaces and artifacts | Direct exchanges | Protected boundary |
| --- | --- | --- | --- | --- | --- |
| `cirnet-platform` | `ecosystem` | Shared identity and tenant context | IAM and tenant APIs, OIDC sessions, scoped claims, audit context | Platform → Project, LMS, Remote and other domains after tested cutover | Domain owners authorize resources, leases, grades and physical actions. |
| `cirnet-contracts` | `ecosystem`, `lab-first` | Canonical shared cross-repo contracts | Versioned schemas, DTOs, API/RPC/NCAP descriptors, Rust/TS SDKs, conformance fixtures | Domain owners → Contracts → Interface, Remote, Hardware and other pinned consumers | Domain owners retain business semantics. Generate SDKs from canonical schemas. |
| `cirnet-project` | `ecosystem` | EngineeringProject lifecycle and collaboration | Project API, immutable revision IDs, ACLs, artifact references | Project → Studio, Lab, LMS and Format-Project | Project authorizes references. Producers own raw artifacts and their access. Format-Project owns the codec. |
| `cirnet-format-project` | `ecosystem` | Portable project serialization | Package format, manifest, readers/writers, integrity and migration fixtures | Project and domain artifacts → Format-Project → Studio and CLI | Validate bounded paths, sizes and integrity. Resource owners authorize export and import effects. |
| `cirnet-database` | `ecosystem` | Data infrastructure operations | Provisioning, migration runner, retention, tested recovery procedures | Database → Remote, LMS, Project and other data owners | Domains retain business schemas and migrations. Secure backups and recovery. |
| `cirnet-lms` | `ecosystem` | Authoritative academic lifecycle | Courses, enrollment, assignments, submissions, rubrics, gradebook APIs and events | Project and Remote supply references → LMS → Learn and instructors | Scope records and disclosure by class and tenant. Remote retains Lab controls and captures. |

## Lab, simulation and instruments

| Repository | Architecture | Bounded context | Owned interfaces and artifacts | Direct exchanges | Protected boundary |
| --- | --- | --- | --- | --- | --- |
| `cirnet-remote` | `ecosystem`, `lab-first` | Lab control plane | Authorized API/WS, experiments, planning, leases, operation events, capture metadata, Lab release set | Interface → Remote → Hardware. Remote → authorized clients. | Check Lab action, lease and disclosure. Hardware owns drivers, LMS owns grades, Platform owns global identity after cutover. |
| `cirnet-hardware` | `ecosystem`, `lab-first` | Virtual and Physical HardwareStation | Private Rust RPC, worker supervision, capabilities, safe Stop, station protection and transport | Remote → Hardware → Simulator or FPGA. Hardware → Remote status and capture. | Keep station RPC private and enforce local protection. Confirm safe state or quarantine. |
| `cirnet-simulator` | `ecosystem`, `lab-first` | Electronic simulation | Default C++ digital and bounded DC worker, private protocol, SPICE adapter and simulated results | Hardware → pinned Simulator worker → Hardware. Optional native SPICE package → Simulator. | Bound execution and label simulated results. Physical FPGA actions and user rights belong elsewhere. |
| `cirnet-embedded-hardware` | `ecosystem`, `lab-first`, `fpga` | Lab station RTL and board operation | Protected routing, timing, trigger/capture, SDRAM/CDC, ICD, bitstreams and timing/HIL artifacts | Hardware driver ↔ FPGA. FPGA supplies verified board artifacts. | Preserve trusted Lab control, local protections and infrastructure-only routes. Web APIs, IAM and scheduling belong elsewhere. |
| `cirnet-nhspice` | `ecosystem` | Conditional native SPICE packaging | Independently pinned build/patch, packages, model/license provenance and compatibility evidence | Optional package → Simulator | Simulator owns the worker protocol, adapter and execution. Confirm the packaging and naming decision before changes. |

## Developer tools, computing and extensions

| Repository | Architecture | Bounded context | Owned interfaces and artifacts | Direct exchanges | Protected boundary |
| --- | --- | --- | --- | --- | --- |
| `cirnet-cli` | `ecosystem` | Command-line experience | Versioned commands and binary | CLI → Project, Toolchains, Devices and Lab APIs as applicable | Delegate to domain APIs. Calling a command gives no device or Lab authority. |
| `cirnet-toolchains` | `ecosystem` | Pinned build environments | Compiler/SDK registry, build adapters/specs, manifests, outputs and logs | IDE and CLI with source and Devices profile → Toolchains → build artifacts | Bound and isolate untrusted builds. Separate device authorization for flashing. |
| `cirnet-devices` | `ecosystem` | Target type, board profiles and adapters | Board catalog, pin/power capabilities, upload/debug adapters | Devices profiles → Toolchains. IDE and CLI → Devices operations. | Check target and action permissions. IoT owns fleet instances. Remote owns Lab leases and Hardware owns station protection. |
| `cirnet-numerics` | `ecosystem` | General scientific kernels | Library APIs, linear algebra, FFT, statistics, ODE, precision and benchmarks | Numerics → Analyze, Robotics and compatible jobs | Analyze owns the experience and Simulator owns electronic circuit semantics. Preserve units and precision. |
| `cirnet-integrations` | `ecosystem` | External-provider adapters | Versioned LMS, Git, EDA and data connectors, compatibility fixtures | External providers ↔ Integrations ↔ matching CIRNET domain | Scope connector credentials and synchronization. Internal domain owners retain authority. |
| `cirnet-extensions` | `ecosystem` | Extension host contract and lifecycle | SDK/ABI, manifests, package validation, trust, capability and sandbox policy | Extension publishers → Extensions → supported host apps | Enforce explicit permissions and isolation. Protected services retain approval. |

## Advanced engineering domains

| Repository | Architecture | Bounded context | Owned interfaces and artifacts | Direct exchanges | Protected boundary |
| --- | --- | --- | --- | --- | --- |
| `cirnet-robotics` | `ecosystem` | Robot modeling and control | Models, kinematics, motion/control and experiment contracts, simulation adapters | Numerics and Devices → Robotics → Studio or authorized targets | Require independent motion safety. Numerics owns common kernels and Toolchains owns builds. |
| `cirnet-iot` | `ecosystem` | Managed device instances and fleets | Provisioning, credential lifecycle, twins, telemetry, command APIs and alerts | Enrolled devices ↔ IoT → authorized App and Analyze clients | Tenant scope and credentials gate commands. Devices owns the catalog and Hardware owns protected Lab stations. |
| `cirnet-silicon` | `ecosystem` | Educational ASIC workflows | RTL-to-ASIC, synthesis/STA/EDA/PDK workflows and verified design artifacts | Project and pinned Toolchains → Silicon → Analyze or LMS references | Protect PDK licenses and sign-off evidence. FPGA owns Lab controller RTL. |
| `cirnet-ai` | `ecosystem` | Advisory tutoring and engineering assistance | Permission-scoped retrieval, suggestions, explanations and audit data | Authorized domain sources → AI → human or client review | Grade, flash, power and routing approvals remain with their owners. Scope retrieval to actor and resource. |

Ecosystem §3 also assigns organization duties to `cirnet-rules` for shared agent
guidance and distribution, `about-cirnetlab` for profile, branding and approved
public documentation, and `.github` for GitHub organization configuration and
automation metadata. Product APIs and physical approvals remain with their
product owners.

For CAD model authoring and asset preflight, use the shared
[CAD engineering rule](../rules/cad-engineering.md). Interface owns its
Three.js asset contract and product acceptance. Use the
[engineering workspace UI rule](../rules/engineering-workspace-ui.md)
for interactive selection, editing and persistence in the workbench.
