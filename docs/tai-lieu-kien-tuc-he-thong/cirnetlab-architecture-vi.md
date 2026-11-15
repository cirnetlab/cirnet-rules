# CIRNET — STEM & Engineering Learning Ecosystem

> **Tài liệu:** `cirnetlab-architecture.md` · **Loại:** Kiến trúc tổng thể cấp Organization (Organization-wide Target Architecture)  
> **Organization:** [`cirnetlab`](https://github.com/orgs/cirnetlab/repositories) · **Tên kiến trúc:** **CIRNET — STEM & Engineering Learning Ecosystem** (không gắn số phiên bản vào tên).  
> **Kiến trúc chuyên ngành kế thừa:** **CIRNET Lab-First**, theo đặc tả `cirnet-architecture-lab-first.md` do dự án cung cấp.  
> **Ảnh chụp nguồn:** danh sách 33 Repository GitHub truy cập được và tài liệu Lab-First được rà soát ngày **2026-11-15**.  
> **Tình trạng:** đặc tả **kiến trúc mục tiêu đề xuất để phê duyệt**; không đồng nghĩa các mô-đun đã được triển khai hoặc đã nghiệm thu production.  
> **Phạm vi:** toàn bộ sản phẩm, dịch vụ, công cụ phát triển, chuẩn dữ liệu, giáo dục, kỹ thuật, thiết bị và liên kết liên Repository; **không chia thành các Phase phát triển**.

---

## 0. Tuyên bố kiến trúc

**CIRNET** định hướng là **hệ sinh thái giáo dục STEM và kỹ thuật số đa nền tảng**: người học và kỹ sư có thể học tập, thiết kế, viết chương trình, mô phỏng, tính toán khoa học, biên dịch, nạp firmware, thử nghiệm phần cứng thật, đo lường, lưu bằng chứng và đánh giá kết quả trong một không gian dự án thống nhất. Giảng viên và tổ chức giáo dục có thể quản lý học liệu, lớp học, bài thực hành, tiến trình và kết quả học tập mà không phụ thuộc vào một công cụ mô phỏng đơn lẻ.

**Định vị:** hướng đến độ rộng của một *engineering learning environment* có thể mở rộng như các hệ sinh thái công cụ kỹ thuật, **không** tuyên bố sao chép MATLAB/Simulink, tương thích ngôn ngữ MATLAB, có đầy đủ các toolbox hoặc có năng lực công nghiệp chưa được chứng minh.

**Công thức tổ chức:**

```text
ONE IDENTITY + ONE ENGINEERING PROJECT + VERSIONED CONTRACTS
       + MULTIPLE APPLICATIONS + DOMAIN ENGINES
       + AUTHORIZED VIRTUAL / PHYSICAL EXECUTION
       + REPRODUCIBLE EVIDENCE + LEARNING ASSESSMENT
```

**Tính bất biến của Lab-First:** *Safety ≠ Correctness*. Khi thực hành mạch thật, chỉ cấp năng lượng nếu chứng minh được an toàn điện, khả năng thực thi và quyền điều khiển. Mạch sai nhưng an toàn có thể được chạy để đo hành vi thực tế. **Physical Compiler chỉ định tuyến hạ tầng đo/kích/cấp nguồn có bảo vệ tới endpoint quan sát được; không được tự sửa, tạo lại hoặc thay dây nối sai của sinh viên.**

### 0.1 Phân biệt cam kết kiến trúc và trạng thái thực tế

- **TARGET:** hướng thiết kế/khả năng dự kiến; **DESIGNED:** đã có đặc tả; **IMPLEMENTED:** có code; **TESTED:** có bằng chứng kiểm thử; **VERIFIED:** đã được nghiệm thu trong điều kiện xác định; **RELEASED:** có artifact được phát hành và khóa; **UNAVAILABLE/BLOCKED:** chưa thể dùng hoặc bị chặn.
- **GitHub Repository tồn tại ≠ tính năng đã triển khai.** Nhiều Repo hiện chỉ chứa `README.md` và `LICENSE`.
- **Một branch `master` ≠ một bộ hệ thống đã kiểm chứng tương thích.** Phải chọn chính xác version, source revision, digest, toolchain và môi trường của từng artifact.
- **Hệ thống học tập ≠ hệ thống điều khiển điện.** Phân quyền học tập không được bỏ qua interlock, safety hay lease thiết bị.
- Tài liệu này xác lập **quyền sở hữu mục tiêu**; với trách nhiệm đang nằm ở Repo cũ, việc chuyển giao đòi hỏi **migration có kiểm thử**, không phát sinh hai nguồn dữ liệu có thẩm quyền cùng lúc.

---

## 1. Các năng lực sản phẩm của hệ sinh thái

| Product / Experience | Mục đích | Repo giao diện chính | Domain/engine tiêu thụ |
|---|---|---|---|
| **CIRNET Studio** | Không gian làm việc đa lĩnh vực; project, file, design, tools | `cirnet-studio` | `project`, `contracts`, `interface`, `ide`, `analyze`, `platform` |
| **CIRNET Lab** | Thiết kế mạch, mô phỏng, Virtual/Physical Lab, instruments | `cirnet-interface` | `remote`, `hardware`, `simulator`, `fpga`, `contracts` |
| **CIRNET IDE** | Viết mã, build, debug, flash, serial/monitor, project tooling | `cirnet-ide` | `toolchains`, `devices`, `cli`, `project` |
| **CIRNET Blocks** | Lập trình trực quan, chuyển khối thành mô hình/mã có kiểm chứng | `cirnet-blocks` | `ide`, `toolchains`, `contracts`, `devices` |
| **CIRNET Analyze** | Tính toán, biểu đồ, notebook, dữ liệu đo và tín hiệu | `cirnet-analyze` | `numerics`, `project`, `database`, `contracts` |
| **CIRNET Learn** | Trải nghiệm khóa học, lớp học, bài tập, nộp bài và nhận xét | `cirnet-learn` | `lms`, `platform`, `project`, `remote` |
| **CIRNET Desktop** | Công cụ native/local workspace, filesystem, USB/serial, debug | `cirnet-desktop` | Các ứng dụng và SDK tương ứng qua native bridge |
| **CIRNET Mobile/Tablet** | Học tập, quản lý project, theo dõi dữ liệu/thiết bị | `cirnet-app` | `platform`, `lms`, `project`, IoT APIs |
| **CIRNET Robotics & IoT** | Mô hình robot, tự động hóa, fleet/telemetry, thiết bị nhúng | Studio/IDE/Analyze | `robotics`, `iot`, `devices`, `toolchains` |
| **CIRNET Silicon** | RTL/FPGA/ASIC learning workflow và EDA interoperability | Studio/IDE/Analyze | `silicon`, `fpga`, `toolchains`, `project` |
| **CIRNET AI Assistance** | Giải thích, hỗ trợ học tập và coding có kiểm soát | Chèn vào ứng dụng có quyền | `ai`, `platform`, `contracts` |
| **CIRNET Public Website** | Giới thiệu, trang sản phẩm, tài liệu công khai | `cirnet_web` | Chỉ dữ liệu public/được phép xuất bản |

**Nguyên tắc:** các tên sản phẩm là *experience*; không bắt buộc một sản phẩm = một process/service. Một Repo chứa nhiều module cùng bounded context vẫn hợp lệ. Không nhân bản engine vì cần thêm một UI mới.

---

## 2. Sơ đồ kiến trúc logic tổng thể

```text
┌────────────────────── CLIENTS / EXPERIENCE ─────────────────────────┐
│ Web: Studio · Lab Interface · IDE · Blocks · Analyze · Learn        │
│ Desktop: native shell + shared UI + local privileged adapters      │
│ Mobile/Tablet: responsive/native UX + restricted device abilities  │
│ Public Site: cirnet_web (separate, public-only)                     │
└───────────────────────────────┬─────────────────────────────────────┘
                                │ HTTPS / WS / typed SDK / authorized APIs
┌───────────────────────────────▼─────────────────────────────────────┐
│ SHARED PLATFORM & PRODUCT DOMAINS                                    │
│ platform: identity · tenant/org · membership · authz · workspace ctx │
│ project: projects · revisions · artifact references · lifecycle     │
│ lms: courses · enrollments · assignment · submission · grades       │
│ remote: Lab API · experiment · scheduler/lease · captures · safety   │
│ iot: device fleet · telemetry domain   ai: advisory assistance      │
│ database: DB provisioning/migration coordination/backup tooling     │
│ contracts: canonical cross-boundary schemas, protocols, SDKs        │
└───────────────┬──────────────────────────────┬──────────────────────┘
                │ Authorized jobs/artifacts    │ Lab control plane
┌───────────────▼──────────────────────┐  ┌────▼───────────────────────┐
│ ENGINEERING & DEVELOPER TOOLS       │  │ LAB / EDGE                 │
│ toolchains: build and compiler jobs │  │ hardware: Rust Station     │
│ devices: board profiles/adapters    │  │ simulator: C++ worker      │
│ numerics: numeric kernels           │  │ nhspice: optional SPICE pkg│
│ robotics / silicon: domain engines  │  │ fpga: RTL + real instruments│
│ extensions / integrations: boundary │  │ protected physical circuit│
└───────────────┬──────────────────────┘  └────┬───────────────────────┘
                └───────────────────┬───────────┘
                                    ▼
                    IMMUTABLE ARTIFACT + EVIDENCE STORE
          project revisions · firmware · models · netlists · waveforms
             captures · grades linkage · audit/provenance · backups
```

**Lớp trung tâm không phải một “siêu-service”.** Sơ đồ thể hiện *domain ownership* và contracts. Runtime triển khai có thể khởi đầu dạng modular monolith, worker process hoặc service tùy độ tin cậy, tài nguyên và security boundary, không mặc định xây microservices, Kafka hay Kubernetes.

### 2.1 Quy tắc phụ thuộc

1. UI → typed public contracts → domain APIs; UI không truy cập database hoặc driver register trực tiếp.
2. Business service → domain interface/contract; không import source private từ Repo khác hay sửa schema copy.
3. `cirnet-contracts` chứa **giao kèo liên ranh giới**, không trở thành kho tập trung toàn bộ logic nghiệp vụ.
4. Domain engine (Simulator, Numerics, Silicon, Robotics) chạy trong worker/sandbox bounded; không thay quyền xác thực, quota, audit hoặc chính sách an toàn.
5. Remote Lab được bảo toàn như domain controller độc lập; Platform chỉ cung cấp danh tính/tổ chức/chính sách chung, **không** quyết định route FPGA.
6. `cirnet-database` điều phối hạ tầng dữ liệu, migration và backup; **chủ sở hữu domain** vẫn giữ thiết kế schema và quyền cập nhật dữ liệu nghiệp vụ.
7. Tất cả giao tiếp cần typed errors, version negotiation, correlation ID, capability discovery và giới hạn tài nguyên.

---

## 3. Kiểm kê Repository và tình trạng nguồn

### 3.1 33 Repository tại ngày rà soát

- **30 Repository thuộc sản phẩm và công cụ** của hệ sinh thái được liệt kê đầy đủ ở §4.
- **`cirnet-rules`:** quản lý baseline Rules, Agent Skills, policies, verification rules và gói phân phối quy tắc; **không** là runtime nghiệp vụ.
- **`about-cirnetlab`:** hồ sơ thương hiệu/tài liệu Organization, có thể làm **nơi lưu tài liệu kiến trúc tổng thể** nếu được phê duyệt; **không** chạy trong sản phẩm.
- **`.github`:** GitHub Organization profile/configuration/automation metadata; **không** là ứng dụng.

Các Repo `cirnet-platform`, `cirnet-toolchains`, `cirnet-devices`, `cirnet-numerics`, `cirnet-integrations`, `cirnet-extensions`, `cirnet-robotics`, `cirnet-iot`, `cirnet-silicon` và `cirnet-ai` **đã được tạo**. Chúng **không** là đề xuất tạo Repo mới. Tại thời điểm kiểm tra, mỗi Repo trong nhóm này chỉ có `README.md` và `LICENSE`; trách nhiệm dưới đây là **định hướng triển khai**, không phải mô tả tính năng đã có.

Nhiều Repo khác như `cirnet-studio`, `cirnet-learn`, `cirnet-analyze`, `cirnet-project`, `cirnet-format-project`, `cirnet-lms`, `cirnet-database` và `cirnet-nhspice` cũng mới ở dạng khung README/License. Sáu Repo Lab-First có mã nguồn và tài liệu sâu hơn, nhưng trạng thái `RELEASED/VERIFIED` phải kiểm tra trên từng artifact cụ thể.

**Local workspace do người dùng khai báo:** `E:\ALL_PROJECTS\LUAN_VAN_TOT_NGHIEP\REPO`. Đây là **root các clone**, không phải một monorepo và không phải đường dẫn đã được bản tài liệu này quét/kiểm chứng trên PC. GitHub inventory và local Git HEAD có thể khác nhau.

---

## 4. Repository Responsibility & Boundary Matrix — 30 Repo sản phẩm/công cụ

Mỗi Repo dưới đây có **một vùng trách nhiệm chính**. Trường *Outputs* là interface/artifact **mục tiêu**; nếu chỉ có README thì chưa có thực thi tương ứng. Các tên được giữ nguyên theo GitHub (bao gồm `cirnet_web` và `cirnet-nhspice`).

### 4.1 Trải nghiệm và ứng dụng — 9 Repo

| Repository | Sở hữu độc quyền / mục tiêu | Outputs và consumers | Không được sở hữu |
|---|---|---|---|
| **[`cirnet_web`](https://github.com/cirnetlab/cirnet_web)** | Website công khai, thông tin sản phẩm, documentation public, demo giáo dục được gắn nhãn | Static public pages, public SEO/content | Không có tài khoản học tập private, không Lab Control API, không lưu đáp án/secret |
| **[`cirnet-interface`](https://github.com/cirnetlab/cirnet-interface)** | CircuitGraph editor, breadboard/Three.js UI, instrument panels và Lab Workbench | Reusable editor component, typed Lab UI consumer | Không sở hữu connectivity từ tọa độ 3D; không tự cấp `SAFE`, không chạy FPGA trực tiếp |
| **[`cirnet-studio`](https://github.com/cirnetlab/cirnet-studio)** | Engineering workspace/shell, tool layout, navigation, project explorer và tích hợp các module | App composition, UX shell, workspace routes | Không viết lại Circuit Editor, IDE engine, Simulator hoặc LMS business logic |
| **[`cirnet-ide`](https://github.com/cirnetlab/cirnet-ide)** | Source editor, LSP/debug UI, terminal, build/upload controls, serial-monitor UI | IDE extension points, editor commands, toolchain UX | Không tự pin compiler SDK, giữ driver USB privilege, hay tự triển khai mỗi board compiler |
| **[`cirnet-blocks`](https://github.com/cirnetlab/cirnet-blocks)** | Visual language, typed blocks, AST/IR generation, deterministic code export và round-trip hạn chế | Block AST/IR, codegen targets, testing fixtures | Không trực tiếp flashing, không chạy arbitrary generated code với quyền backend |
| **[`cirnet-analyze`](https://github.com/cirnetlab/cirnet-analyze)** | Scientific workbench, notebooks, plotting, workflow xử lý dữ liệu, signal visualization | Analysis documents, computation jobs, charts/notebooks | Không sở hữu numeric kernels lõi, không sửa raw capture hoặc lịch sử đo |
| **[`cirnet-learn`](https://github.com/cirnetlab/cirnet-learn)** | Student/Instructor learning UX: course catalog, lesson, assignment, submission, feedback, progress | Education client UI, LMS typed consumers | Không tính điểm có thẩm quyền trong browser, không giữ enrollment/gradebook DB |
| **[`cirnet-desktop`](https://github.com/cirnetlab/cirnet-desktop)** | Desktop native shell, permissions bridge, OS file/USB/serial integration, auto-update và offline UI | Signed desktop installer, local bridge API | Không fork nghiệp vụ Lab/IDE/Studio, không cho renderer gọi native API tùy ý |
| **[`cirnet-app`](https://github.com/cirnetlab/cirnet-app)** | Mobile/Tablet UX, learning/project/observability và offline cache theo quyền | Mobile build, platform-specific UI, push/viewer adapters | Không cam kết full FPGA/compiler/flash trên mọi mobile OS, không giữ safety authority |

### 4.2 Nền tảng, dữ liệu và học vụ — 6 Repo

| Repository | Sở hữu độc quyền / mục tiêu | Outputs và consumers | Không được sở hữu |
|---|---|---|---|
| **[`cirnet-platform`](https://github.com/cirnetlab/cirnet-platform)** | Global identity, tenant/institution, organization membership, workspace context, SSO/OIDC, session, common authorization policy, quotas | Identity/tenant APIs, claims, audit envelope, shared context | Không sở hữu điểm số lớp học, hardware safety, lab scheduler, project format |
| **[`cirnet-contracts`](https://github.com/cirnetlab/cirnet-contracts)** | Cross-repo canonical DTO/schema/protocol, API/RPC/NCAP versions, generated Rust/TypeScript SDKs, conformance fixtures | Versioned contracts and SDK packages | Không đăng ký business API thay domain owner; không thành nơi chứa mọi domain implementation |
| **[`cirnet-project`](https://github.com/cirnetlab/cirnet-project)** | EngineeringProject aggregate, ownership links, revisions/branching metadata, artifact registry/relationships, project permissions và collaboration lifecycle | Project API, immutable revision identities, artifact references/events | Không sở hữu file format/codec, LMS grades, Lab capture bytes hoặc mọi storage backend |
| **[`cirnet-format-project`](https://github.com/cirnetlab/cirnet-format-project)** | Canonical portable project package, manifest/serialization, versioned import/export, integrity, migration/validation của gói | Portable project format spec, parsers/writers, reference fixtures | Không sở hữu project ACL, durable database, grading hoặc framework UI |
| **[`cirnet-database`](https://github.com/cirnetlab/cirnet-database)** | DB topology/provisioning, migration coordination, backup/restore policy, retention tooling, database conventions và cross-domain schema inventory | IaC/config, migration runner, tested backup/restore plans | Không thành single owner mọi bảng; domain owners vẫn quản lý migrations và invariants của mình |
| **[`cirnet-lms`](https://github.com/cirnetlab/cirnet-lms)** | Course/class/enrollment, learning assignments, submissions, assessment rubric, gradebook, competency records, academic audit | LMS service APIs, assessment events, authorized learning records | Không giữ source project/capture raw; không trực tiếp thao tác FPGA; không gán quyền tenant toàn hệ thống |

### 4.3 Phòng thí nghiệm và mô phỏng — 5 Repo

| Repository | Sở hữu độc quyền / mục tiêu | Outputs và consumers | Không được sở hữu |
|---|---|---|---|
| **[`cirnet-remote`](https://github.com/cirnetlab/cirnet-remote)** | **Lab domain control plane**: Experiment Manager, Lab assignment adapters, station scheduling/lease, Lab API/WS, validation/plan orchestration, capture metadata, Lab system release lock | Lab APIs, experiment events/results, pinned Lab release-set | Không thay global identity owner sau migration; không thay Hardware driver; không làm gradebook học vụ |
| **[`cirnet-hardware`](https://github.com/cirnetlab/cirnet-hardware)** | Rust HardwareStation (virtual/physical), local-only RPC, worker supervision, protected hardware operations, safe Stop, transport abstraction | Station RPC, capabilities, driver reports, verified cleanup | Không sở hữu user/business DB, không tự chấm đúng/sai hoặc bypass safety gate |
| **[`cirnet-simulator`](https://github.com/cirnetlab/cirnet-simulator)** | **C++ mặc định lâu dài** cho deterministic digital simulation, bounded DC, worker protocol và **SPICE adapter** | Pinned C++ workers, simulation result/status, waveform/capture | Không tự định nghĩa user rights, điều khiển FPGA thật, hoặc phát hành analog capability không có solver |
| **[`cirnet-fpga`](https://github.com/cirnetlab/cirnet-fpga)** | HDL/RTL, FPGA protocol implementation, timing/trigger/capture, protected routing controller, SDRAM/CDC và hardware testbench | Verified RTL, board-specific bitstreams, timing/HIL evidence | Không sở hữu Web API, IAM, project DB, remote scheduler |
| **[`cirnet-nhspice`](https://github.com/cirnetlab/cirnet-nhspice)** | **Đề xuất có điều kiện:** native SPICE engine dependency packaging/fork patch set, model-compatibility/licensing/provenance nếu thật sự cần phát hành độc lập | Versioned SPICE native artifacts và compatibility tests | Không tạo simulator API/worker thứ hai; adapter gọi solver vẫn thuộc `cirnet-simulator` |

**Quyết định mở:** README `cirnet-nhspice` hiện dùng heading `cirnet-core-spice`, không khớp tên Repo. Chỉ giữ Repo này hoạt động nếu có nhu cầu cụ thể về patch set, phát hành hoặc license boundary; nếu không, `cirnet-simulator` tích hợp upstream engine dưới dependency pin vẫn đủ. **Không tự xóa hoặc đổi tên Repo đã có.**

### 4.4 Developer tooling, toán học và mở rộng — 6 Repo

| Repository | Sở hữu độc quyền / mục tiêu | Outputs và consumers | Không được sở hữu |
|---|---|---|---|
| **[`cirnet-cli`](https://github.com/cirnetlab/cirnet-cli)** | User-facing `cirnet` commands: create/open/project, build/test/doctor/dev commands và workflow orchestration CLI | CLI binary, command contracts, local UX | Không thay domain engine, SDK manager hay bản thân compiler; là front-end cho Toolchains/Projects |
| **[`cirnet-toolchains`](https://github.com/cirnetlab/cirnet-toolchains)** | Toolchain registry, compiler/SDK adapters, reproducible build specification, sandboxed build workers, artifact provenance | Compiler manifests, build plans, build results, SDK/version locks | Không tự xác thực người dùng; không giữ board identity hay hardware flashing driver |
| **[`cirnet-devices`](https://github.com/cirnetlab/cirnet-devices)** | Board/chip catalog, BoardProfile, pin maps, flashing/debug probe adapters, capability and transport compatibility | Versioned board packages, target capabilities, flash/debug adapters | Không sở hữu IoT fleet/telemetry, teaching enrollment hay electrical safety của Lab station |
| **[`cirnet-numerics`](https://github.com/cirnetlab/cirnet-numerics)** | Reusable numerical kernels: vector/matrix, FFT/DSP, ODE, optimization, interpolation và units/numeric precision | Versioned computation libraries, benchmark/conformance suites | Không chứa notebook UI, Lab simulator circuit semantics, quyền người dùng |
| **[`cirnet-integrations`](https://github.com/cirnetlab/cirnet-integrations)** | External system adapters: LTI, learning roster interchange, Git providers, EDA/external systems; secure sync boundaries | Connectors, auth handshakes, mapping/error adapters | Không làm nguồn sự thật cho LMS/Project, không cấp đường tắt vào thiết bị vật lý |
| **[`cirnet-extensions`](https://github.com/cirnetlab/cirnet-extensions)** | Extension SDK, signed/validated manifests, host extension API, compatibility, sandbox/permission model và package lifecycle | Extension ABI/SDK, registry protocol, compatibility tests | Không biến mọi first-party domain thành plugin, không cho extension quyền truy cập secret/hardware mặc định |

### 4.5 Các domain kỹ thuật chuyên sâu — 4 Repo

| Repository | Sở hữu độc quyền / mục tiêu | Outputs và consumers | Không được sở hữu |
|---|---|---|---|
| **[`cirnet-robotics`](https://github.com/cirnetlab/cirnet-robotics)** | Robot/kinematics/control models, robotics scenes, ROS interoperability adapters và experiment specification | Robot models, robotics jobs/traces, device target mappings | Không nhân bản numeric core, IDE, board catalog, generic simulation scheduler |
| **[`cirnet-iot`](https://github.com/cirnetlab/cirnet-iot)** | Device fleet, telemetry ingestion, device twins, protocol adapters (MQTT/HTTP nếu hỗ trợ), policy và remote monitoring | Fleet/device APIs, time-series streams, alerts | Không thay station safety, không sở hữu compiler/board definitions, không cho remote commands không được phép |
| **[`cirnet-silicon`](https://github.com/cirnetlab/cirnet-silicon)** | RTL-to-ASIC education/workflows, synthesis, STA, PDK/EDA adapters, physical design checks và semiconductor project metadata | EDA job plans, reports, verification artifacts, PDK provenance | Không thay HDL/FPGA ownership; không tự tuyên bố tapeout-ready hoặc sao chép proprietary PDK |
| **[`cirnet-ai`](https://github.com/cirnetlab/cirnet-ai)** | AI tutor/coding guidance, retrieval over permitted content, feedback assistance, model gateway, AI evaluation/audit | Advisory outputs, provenance, prompts/policies, safety review reports | Không quyết định electrical SAFE, flash, grade chính thức hoặc thao tác dữ liệu vượt quyền |

### 4.6 Các Repo không thuộc đường thực thi sản phẩm

| Repository | Vai trò | Ranh giới |
|---|---|---|
| **[`cirnet-rules`](https://github.com/cirnetlab/cirnet-rules)** | Nguồn Rules, Skills, Policies, agent instructions và checks dùng xuyên Repo | Không gọi business APIs, không giữ project/user data; tham chiếu kiến trúc chuẩn chứ không sao chép toàn bộ |
| **[`about-cirnetlab`](https://github.com/cirnetlab/about-cirnetlab)** | Organization identity, branding và tài liệu kiến trúc/chính sách cấp Organization | Không tham gia runtime, compile hay điều phối project; phù hợp làm nơi lưu file kiến trúc canonical nếu được duyệt |
| **[`.github`](https://github.com/cirnetlab/.github)** | Organization profile, GitHub templates/config chung | Không chứa domain logic hoặc thư viện runtime |

---

## 5. Các ranh giới dễ chồng lấn: quyết định bắt buộc

| Cặp hoặc nhóm | Owner chính xác | Nguyên tắc phân chia |
|---|---|---|
| **Studio / Interface / IDE** | Studio = product shell; Interface = Circuit Editor/Lab UI; IDE = source tooling | Import APIs/components versioned; không copy editor/codebase giữa Repo |
| **Learn / LMS** | Learn = UX; LMS = học vụ và gradebook | Learn không tự tính/ghi điểm có thẩm quyền |
| **Platform / Remote** | Platform = global Identity, tenant, policy; Remote = Lab experiment, lease, station | Auth đang tồn tại trong Remote phải được chuyển có kiểm chứng khi Platform sẵn sàng; không có hai issuer cùng quyền |
| **Platform / LMS** | Platform = institution membership và global roles; LMS = course/class enrollment và academic rights | `Instructor` trong lớp không tự có admin/device control quyền rộng |
| **Project / Format-Project / Contracts** | Project = lifecycle/ACL; Format = portable encoding; Contracts = public cross-boundary interface | Cùng một entity có owner rõ; chỉ tham chiếu, không tạo ba phiên bản schema nghiệp vụ độc lập |
| **Database / mọi domain** | Database = infra/runner; mỗi domain = bảng/schema/migration nghiệp vụ | Không “centralize ownership” khiến domain thay đổi bảng của nhau trực tiếp |
| **CLI / Toolchains / Devices / IDE** | CLI/IDE = UX; Toolchains = compiler/build; Devices = board/transport/flash | Compile ra artifact trước; flash là hành động riêng cần permission/board match |
| **Analyze / Numerics / Simulator** | Analyze = UX/workflow; Numerics = generic math; Simulator = electrical physical models & C++ worker | C++ circuit simulation không chuyển sang Numerics; plot không làm thay engine |
| **Simulator / NHSPICE** | Simulator = C++ worker + adapter + public simulation contract; NHSPICE = native dependency đóng gói nếu cần | Không hai chủ sở hữu simulation result/worker API |
| **Hardware / Devices / IoT** | Hardware = protected Lab station; Devices = board catalog/flash adapter; IoT = fleet & telemetry | IoT cannot bypass station interlocks; device profile không mặc định chứng nhận an toàn |
| **FPGA / Silicon** | FPGA = RTL/board/HIL; Silicon = ASIC-oriented flow/PDK/physical signoff | Reuse HDL, không sao chép FPGA engine làm ASIC emulator |
| **Extensions / Integrations** | Extensions = plugin/host lifecycle; Integrations = external service connectors | Connector có thể là extension nhưng nguồn thực thi được quản trị theo trust boundary |
| **AI / LMS / Remote** | AI = tư vấn; LMS = assessment authority; Remote/Hardware = safety/run authority | AI suggestion không tự thành điểm, safety verdict hoặc physical command |

**Migration rule:** khi chuyển một responsibility từ Repo A sang Repo B, phải có owner mới, API adapter, dữ liệu migration, backward compatibility, consumer contract tests, rollback và thời điểm tắt đường cũ. Trước khi chuyển xong, Repo A vẫn là source of truth; không âm thầm chia đôi quyền sở hữu.

---

## 6. EngineeringProject là đơn vị dữ liệu xuyên hệ sinh thái

**Mọi trải nghiệm kỹ thuật phải có thể gắn vào một EngineeringProject**, nhưng không ép mọi loại mô hình thành `CircuitGraph`.

```text
EngineeringProject (cirnet-project)
├── identity: projectId, owner, tenantId, permissions, revisions
├── assets: references to immutable content-addressed artifacts
├── documents (versioned, typed)
│   ├── CircuitGraph                    [Lab]
│   ├── SourceTree / IDE Workspace      [IDE]
│   ├── BlockProgram AST/IR             [Blocks]
│   ├── 3D Scene / CAD Geometry         [Studio/Robotics]
│   ├── RTL / Netlist / Constraints      [FPGA/Silicon]
│   ├── Dataset / Notebook              [Analyze]
│   └── BoardProfile / TargetBinding    [Devices]
├── executions: build / simulation / experiment / flash / EDA run IDs
├── evidence: waveforms, captures, build logs, measurements, checksums
└── academic links: LMS assignment/submission/grade REFERENCES only
```

### 6.1 Bốn lớp sự thật

| Loại | Nguồn có thẩm quyền | Không được suy luận |
|---|---|---|
| Project identity/revision | `cirnet-project` | UI cache không là revision đã commit |
| Design content | Artifact/DesignGraph được pin và checksum | 3D layout không quyết định kết nối điện nếu graph không ghi |
| Physical observed state | Hardware measured graph + evidence | Bài mẫu không phải trạng thái dây người học đang cắm |
| Learning outcome | `cirnet-lms` từ rubric, authorized evidence và policy | Simulator PASS không tự là grade PASS; AI comment không tự là grade |

### 6.2 Định danh, phiên bản và lưu trữ

- `tenantId`, `principalId`, `workspaceId`, `projectId`, `revisionId`, `artifactId`, `jobId`, `experimentId`, `stationId`, `assignmentId`, `submissionId` phải có namespace và owner riêng; không tái sử dụng một ID cho loại thực thể khác.
- Metadata mutable nằm trong domain DB; artifacts/reports/captures immutable trong content-addressed blob/object storage; checksum tính trên bytes và quyền truy cập theo owner.
- Các reference liên kết cross-domain phải ghi `schemaVersion`, `producer`, `artifactDigest`, `createdAt`, `ownerScope` và kiểu tài liệu khi thích hợp.
- `cirnet-format-project` định nghĩa **portable package**, manifest, signing/checksum optional theo threat model, preservation of unknown supported metadata, input limits, path traversal protection, migration. Project export là một tác vụ được cấp quyền riêng.
- Không đồng nhất **version của project format** với **version của repository**, SDK, board package, worker protocol, release-set hoặc bitstream.
- Một project có thể chứa nhiều loại artifact nhưng chỉ một *canonical owner* cho từng namespace và policy.

### 6.3 Workflow chuẩn

```text
Create Project → Save Revision → Select Tool/Capability
   → Validate Contracts/Permissions → Build/Simulate/Run
   → Produce Immutable Result + Provenance
   → View/Compare/Annotate → Submit Learning Evidence (optional)
```

Không yêu cầu tất cả bước xuất hiện ở mọi loại project. Một Notebook không phải Lab run; một FPGA bitstream không phải firmware ESP32; một assignment LMS không phải file `.cirnet`.

---

## 7. Cross-repository Contracts và Runtime Integration

### 7.1 Các nhóm contract mục tiêu

| Namespace | Owner thiết kế dữ liệu | Consumer | Quan hệ với `cirnet-contracts` |
|---|---|---|---|
| Identity/Tenant/Permission | Platform | Mọi ứng dụng | Public claims/permission DTO/versioned API trong Contracts |
| Project/Revision/Artifact | Project | Studio/IDE/Analyze/Learn | IDs, event envelope và public DTO được chuẩn hóa |
| Portable Project Package | Format-Project | Project/desktop/CLI | Format schema chuyên biệt; Contracts chỉ quản public transport nếu cần |
| Lab Graph/Endpoint/Result | Contracts + Remote domain owner | Interface/Hardware/Simulator | Canonical Lab contracts, wire/RPC/NCAP và fixtures hiện hành |
| Build/Toolchain/Board | Toolchains + Devices | IDE/Blocks/CLI | BoardProfile/BuildRequest/BuildResult typed interfaces |
| Learning/Competency | LMS | Learn/Studio/Integrations | Enrollment/assignment/submission/grade API versioned |
| Numerical/Notebook | Numerics + Analyze | Analyze/Robotics/Silicon | Stable numeric inputs, units, artifact/result metadata |
| IoT/Robotics/EDA | Domain owner | Studio/IDE/Analyze | Domain-specific schema; shared envelope/IDs nằm trong Contracts |
| Extension Manifest | Extensions | Host application | Versioned host capability/permission and extension API |

**Không tạo vòng phụ thuộc source:** Producer và consumer dùng bản hợp đồng đã đóng gói; không copy “latest” trực tiếp từ `master` của Repo khác. Generated TS/Rust models xuất phát từ nguồn được duyệt, conformance test và fixture kiểm tra hai chiều khi có thay đổi.

### 7.2 Envelope tác vụ và bằng chứng

Tối thiểu, tác vụ có `jobId`, `kind`, `tenantId`, `principalId`, `projectRevisionId`, `inputs[]` (exact digests), `requestedCapability`, `resourceBudget`, `deadline`, `idempotencyKey`, `status`, `resultArtifactIds[]`, `producerVersion`, `correlationId` và **quyền hủy/cleanup**.

Trạng thái chung ví dụ: `CREATED`, `VALIDATING`, `QUEUED`, `RUNNING`, `COMPLETED`, `FAILED`, `CANCELLED`, `UNAVAILABLE`, `QUARANTINED` (chỉ nơi có tài nguyên cần quarantine). **Không áp trạng thái physical safety vào mọi build job.**

Các tác vụ có side effects không được tự replay khi reconnect; `accepted` không đồng nghĩa `completed`; `Stop requested` không đồng nghĩa `safe stop acknowledged`.

### 7.3 Event stream và dữ liệu lớn

- Control API dùng HTTPS + typed SDK; realtime dùng WS/event stream đã xác thực và có sequence/resync; private Hardware RPC chỉ nằm trong trusted edge boundary.
- Waveform, binary capture, CAD assets, firmware, notebook dataset và EDA reports lớn được stream hoặc tải có ủy quyền, không nhét vào một JSON response.
- Event phải mang request/correlation ID, producer version, stream sequence và scope quyền; cần explicit gap detection.
- `NCAP` thuộc Lab capture; các domain khác có format riêng dưới một artifact envelope dùng chung.

---

## 8. CIRNET Lab-First là chuyên ngành được kế thừa nguyên vẹn về an toàn

Đặc tả **`cirnet-architecture-lab-first.md`** tiếp tục là **nguồn chuẩn về điện tử, Lab execution và electrical safety**. Tài liệu hệ sinh thái này chỉ đặt Lab vào bối cảnh lớn hơn, **không thay thế định nghĩa kỹ thuật** của `CircuitGraph`, `LabEndpoint`, `HardwareStation`, `SafetyVerificationResult`, FPGA transport và các điều kiện nghiệm thu.

### 8.1 Virtual Digital E2E

```text
Web Lab Interface (TS/Three.js)
  → Lab Remote (Rust): auth delegation + experiment/lease + plan
  → HardwareStation (Rust): worker supervision
  → Simulator (C++ default, version/hash pinned)
  → HardwareStation → Remote → authenticated Web result/capture
```

Chỉ được công nhận verified khi **real browser → real API → real Rust Hardware process → real C++ worker → Web** sử dụng cùng bộ artifact được pin, kiểm qua contract, waveform, persisted metadata, cancellation, negative cases, resource bounds và đúng target OS. Virtual runtime không được bịa topology scan, raw physical measurement hoặc electrical `SAFE`.

### 8.2 Physical Lab và endpoint resolution

```text
Reference Assignment + Instructor Profile
               + measured DetectedTopologyGraph + StationProfile
                          ↓
Conservative TopologyResolver: reference endpoint → observed student node
                          ↓
Safety Gate + Executability Gate + Authorized Lease
                          ↓
Physical Compiler: protected infrastructure-only routing
                          ↓
Guarded physical execution → measured function → learning evaluation
```

**Bất biến kiến trúc:**

- `reference_node_id` **không phải** `observed_node_id` khi chưa có evidence độc lập; khi thiếu required binding thì `BLOCK_UNEXECUTABLE`.
- `SafetyVerificationResult` là hard gate; `UNSAFE` hoặc `INDETERMINATE` → **BLOCK**. Bài đúng/sai không được dùng thay safety.
- `TopologyVerificationResult` dùng diagnostics/grading và không tự chặn mạch đã an toàn/executable.
- `FunctionalVerificationResult` chỉ từ hành vi đã chạy/đo thực, không phải giá trị giả lập để đạt đáp án.
- **Physical Compiler MUST NOT repair student IC/jumper wiring** bằng FPGA/MUX. Chỉ route nguồn kích, nguồn cấp có bảo vệ, sense/probe/clock/instruments tới observed allowed endpoints.
- Phải có eFuse/current limit, sensing/fast cutoff, safe startup/reset, watchdog, HIL và chứng cứ board cụ thể trước khi bật physical capability.
- Một shared physical routing fabric mặc định có **1 slot**, trừ khi chứng minh được các partition độc lập về điện. Mất xác nhận safe cleanup → station quarantine.

### 8.3 C++ Simulator và SPICE

`cirnet-simulator` giữ **C++ là mặc định lâu dài**. Rust tiếp tục cho `remote`, `hardware`, `contracts` runtime; TypeScript cho UI, Verilog/SystemVerilog cho FPGA. `cirnet-nhspice` **không** thay vai trò chủ của `cirnet-simulator`. Virtual Analog chỉ được công bố khi C++ adapter tích hợp được native SPICE engine thực với model/license provenance và E2E evidence; bounded DC không tương đương full SPICE.

**Python chỉ dùng trong đường kiểm thử/benchmark/parity của CIRNET production platform**, đúng định hướng Lab-First. Việc sau này hỗ trợ chương trình do người học viết bằng Python/MicroPython là **quyết định sản phẩm độc lập**, bắt buộc phân biệt interpreter sandbox/board runtime của người dùng với platform backend; không được âm thầm đưa Python request handlers/solver vào CIRNET production.

---

## 9. IDE, Block Programming, Toolchains và Device Engineering

### 9.1 Pipeline thống nhất

```text
IDE code / BlockProgram / RTL source
    → Syntax + type validation / generated source and maps
    → BuildRequest (compiler + SDK + board target + exact lock)
    → Sandboxed Toolchain Worker → Signed/hashed BuildArtifact
    → Compatibility + permission + physical target check
    → Local/Remote Flash or Debug Adapter (separately authorized)
    → Device observation / telemetry / artifact evidence
```

**Build ≠ Flash ≠ Run on physical hardware.** Các hoạt động này cần scopes khác nhau. Toolchains không có quyền tự flash; Devices phải kiểm tra device identity, target family, available tool, power/firmware compatibility và quyền explicit. Các target có fuse/bootloader/flash không phục hồi được phải có cảnh báo/khóa riêng.

### 9.2 Target abstraction

`BoardProfile`: vendor/family/part, memory, pin functions, voltage domain, bootloader/debug interface, toolchain compatibility, target triples, firmware packaging, flash transport, permission requirements và validation evidence.

Hỗ trợ ESP32, STM32, Arduino-compatible, PIC, Raspberry Pi, Orange Pi, FPGA... **theo adapter đã được nghiệm thu**, không có nghĩa mọi board chạy được cùng một firmware hoặc cùng cách upload. SBC Linux là một target khác MCU; FPGA bitstream không phải firmware MCU.

### 9.3 Block-to-Code

- Block editor sở hữu typed AST/IR, semantic checks và source maps, tránh string concatenation không an toàn.
- Compiler targets được điều phối qua Toolchains; Blocks không fork compiler logic.
- Cho phép export readable source, diagnostics có vị trí block, unit tests và reproducibility metadata.
- Code người học và thư viện bên thứ ba chạy trong sandbox/target riêng, không có quyền DB/service secret hoặc network vô hạn.

### 9.4 Developer CLI

`cirnet-cli` là giao diện người phát triển cho init, doctor, build, test, simulate, package và project operations **khi đã có API tương ứng**. Không cần dồn toàn bộ SDK, drivers, board definitions và engine vào CLI. Các lệnh/syntax chỉ được công bố sau khi có implementation; không nhận một repo có README-only là CLI đã chạy được.

---

## 10. Scientific Computing, Modeling và Engineering Analysis

CIRNET hướng tới công cụ phân tích kỹ thuật có thể tái lập, không phụ thuộc vào một bộ toolbox độc quyền.

- `cirnet-numerics`: numeric kernels, precision, units, deterministic seeds, integration, FFT/signal processing, linear algebra, optimization và benchmark phù hợp; ưu tiên Rust cho service/library integration, FFI C/C++ được phép khi audit/licensing/benchmark xác nhận.
- `cirnet-analyze`: notebook/document UI, variables, plots 2D/3D, experiment comparison, provenance report, dataset linking, computation job management.
- `cirnet-simulator`: thuật toán điện tử, device models, digital event / bounded DC / analog SPICE; không chuyển C++ Simulator sang numerics.
- Notebook phải có execution spec gồm engine/version/dependency lock, input artifact digests, environment, resource bounds và output; không cho phép arbitrary privileged code từ trình duyệt chạy trong process API.
- Quan sát thực có nhãn `MEASURED`, mô hình tính toán có nhãn `COMPUTED`, mô phỏng có nhãn `SIMULATED`; các loại này không được hiển thị như cùng một chứng cứ.
- Cấp quyền truy cập dữ liệu theo project/tenant/role; giữ raw inputs và numerical error/units/uncertainty khi tạo report.

---

## 11. CIRNET Learn & LMS: giáo dục tích hợp kỹ thuật

### 11.1 Data ownership

- `cirnet-platform`: user/institution identity, tenants, global membership và cross-app authorization.
- `cirnet-lms`: course, cohort/class, enrollment, lesson/assessment rules, submission, rubric, gradebook, competency/progress.
- `cirnet-learn`: học liệu và giao diện trải nghiệm cho student/instructor/admin theo quyền, không giữ dữ liệu học vụ có thẩm quyền.
- `cirnet-project`: engineering submissions trỏ tới **revision/artifact immutable** thay vì copy cả project vào bảng điểm.
- `cirnet-remote`: LabExperimentResult/evidence, không tính điểm cuối cùng thay LMS.

### 11.2 Chu trình học tập tích hợp

```text
Instructor → LMS Assignment + Rubric + permitted resources
Student → Learn → EngineeringProject revision
        → Studio/IDE/Blocks/Lab/Analyze (as authorized)
        → submit exact project revision + evidence artifact references
LMS → verifies provenance + policy + rubric
    → grading / instructor review / disclosure → learner feedback
```

Tách **automatic evidence checks**, **teacher assessment** và **AI suggestions**. Có thể auto-score trong phạm vi rubric đã phê duyệt, nhưng phải ghi algorithm version và cơ chế phản hồi/kháng nghị; AI không tự trở thành điểm có thẩm quyền.

### 11.3 Tính tương tác giáo dục

Hỗ trợ tích hợp LMS bên ngoài bằng adapter trong `cirnet-integrations`; cân nhắc LTI 1.3/LTI Advantage, roster interchange và SSO theo nhu cầu thực tế, đối chiếu các tiêu chuẩn hiện hành trước khi cam kết. Một trường đang dùng LMS khác vẫn có thể dùng CIRNET Lab/Studio/IDE như công cụ liên kết, không bắt buộc di chuyển toàn bộ gradebook sang CIRNET.

Privacy-by-design: khả năng người dùng là trẻ vị thành niên, dữ liệu học tập nhạy cảm, consent của người giám hộ khi áp dụng, giới hạn retention/export, tenant segregation, role review và audit thao tác sửa điểm.

---

## 12. Các miền chuyên sâu: Robotics, IoT, Silicon, AI

### 12.1 Robotics

`cirnet-robotics` cung cấp robot models, coordinate frames, joints/kinematics, control loops, motion planning/trajectory, simulated sensors và ROS-compatible adapters khi cần. Dùng Numerics cho tính toán; Devices cho board/actuator target; Toolchains cho build; Project cho model/revision. Chỉ cho robot thật hoạt động khi có envelope an toàn riêng (workspace limits, emergency stop, watchdog, motion constraints). **Electrical Lab safety không tự chứng minh an toàn cơ khí**.

### 12.2 IoT và Edge Devices

`cirnet-iot` quản lý fleet/device twin, provisioning dưới quyền, signed credential lifecycle, telemetry/time-series, commands và alert policies. Phân biệt:

- **Device Catalog** (`devices`) = mô tả loại board và adapters;
- **IoT Device Instance** (`iot`) = thiết bị đã đăng ký, owner, credentials, telemetry;
- **Lab Station** (`hardware`) = tài nguyên thực hành có interlock, reservation và safe Stop.

Các lệnh từ dashboard/AI phải qua actor authorization, policy, bounded side effects, audit và rollback/stop. Không route raw control commands trực tiếp từ public Web vào firmware hoặc hardware transport.

### 12.3 Semiconductor / CIRNET Silicon

`cirnet-silicon` định nghĩa workflows cho HDL, lint, simulation, synthesis, equivalence, static timing, floorplanning, placement/CTS/routing, DRC/LVS và báo cáo dưới tool/PDK phù hợp. `cirnet-fpga` vẫn sở hữu bitstream/board/timing FPGA; `silicon` tập trung ASIC/PDK/flow. Mọi toolchain/PDK có version, license và phạm vi phân phối; không commit proprietary IP/PDK khi chưa có quyền. **RTL simulation ≠ FPGA electrical verification ≠ ASIC sign-off ≠ tapeout authorization**.

### 12.4 AI tutor và engineering copilot

`cirnet-ai` là **advisory layer**: giải thích code/mô hình, gợi ý sửa, search tài liệu có quyền, đề xuất bài tập, hỗ trợ đọc logs và review. Các giới hạn:

- Retrieval chỉ trên dữ liệu người dùng được phép; không đưa secret, answers, projects riêng sang external provider trái chính sách.
- LLM outputs có model/version, nguồn, uncertainty và audit; không nâng output thành kết quả chạy hoặc measurement.
- Không tự flash, powered-run, tắt safety, sửa grade, xuất dữ liệu riêng tư hoặc thay contract mà không có action permission và gate độc lập.
- Code do AI đề xuất phải theo PR/test/review như code của con người; không tạo tài liệu Markdown trùng lặp sau mỗi lần thử sửa lỗi.

---

## 13. Multi-platform Strategy

| Platform | Chức năng ưu tiên | Native boundary | Giới hạn bắt buộc |
|---|---|---|---|
| Web | Studio, Lab, Blocks, IDE web, Analyze, Learn | Browser APIs, backend compute | Không trực tiếp điều khiển USB/FPGA trust path; render 3D client-side |
| Desktop | Full engineering workspace, local file/build/USB/serial/debug | Signed native shell + permissioned IPC | Renderer không có toàn quyền filesystem/process; explicit user consent cho flash |
| Tablet | Learn, block programming, projects, plots, design light | Touch layout, offline cache | Năng lực build/debug/flash tùy OS và adapter thực |
| Mobile | Learn, notifications, telemetry, approvals/view-only khi phù hợp | Push, secure storage, limited device access | Không quảng cáo full EDA/FPGA toolchain nếu chưa có |
| Edge Mini PC | Lab control + HardwareStation + C++ worker + protected devices | Linux, service manager, localhost RPC | Network segmentation, fail-safe hardware, bounded compute, offline recovery |
| Cloud | Multi-tenant APIs, projects/learning/optional compute | Managed compute/storage, isolated workers | Không bắt buộc mọi physical station expose lên Internet |

**Shared design system** và typed SDK có thể tái dùng nhưng không buộc một UI codebase render y nguyên trên mọi màn hình. Desktop native bridge là thành phần đặc quyền; worker compile/simulation thuộc security boundary riêng.

**Offline & sync:** project draft có thể lưu tạm trên client với conflict detection; durable tenant/grade/lease và quyền điều khiển phần cứng luôn theo server authority. Reconnect không tự replay Run, Flash hoặc Grade mutation.

---

## 14. Identity, Permissions, Security và Data Governance

### 14.1 Nhiều lớp quyền

- **Global identity:** user, institution, tenant, org membership (`platform`).
- **Project permissions:** owner/editor/viewer/submitter, artifact export (`project`).
- **Academic permissions:** course, assignment, grading và answer disclosure (`lms`).
- **Lab permissions:** experiment controller, station lease, virtual/physical capability, stop (`remote` + `hardware`).
- **Device permissions:** flash/debug target, IoT command scope (`devices`/`iot`).
- **Extension permissions:** explicitly declared host capabilities (`extensions`).

Không suy luận quyền `project editor` là `firmware flash`, `course instructor` là `physical admin`, hoặc `AI assistant` là `safety verifier`.

### 14.2 Auth and isolation

OIDC/SSO/SAML khi nhu cầu được xác thực; session cookies secure/HttpOnly/SameSite, CSRF/Origin enforcement, MFA với sensitive admins, least privilege. Access tokens phải có audience, scope, expiry và tenant context; không dùng user token nguyên vẹn cho private worker trừ khi threat model chứng minh an toàn. Service-to-service authentication, TLS/mTLS khi cần, signed job capabilities và audit trail cho physical commands.

Ranh giới tenant phải được áp dụng tại **API + data access + artifact retrieval**, không chỉ ẩn UI. Instructor reference answers, private projects, waveforms và assessment data không rò rỉ qua errors, search index, telemetry hoặc LLM prompt context.

### 14.3 Supply chain & untrusted engineering workloads

- Signed/verified release artifacts, SBOM, dependency/license scanning, vulnerability remediation, reproducible locks; verify bytes/digests trước khi thực thi.
- Compiler/EDA/notebook/simulator user input có size/time/CPU/memory/disk/network bounds; sandbox/container/process isolation tùy mức đe dọa.
- Chặn path traversal, zip bombs, malicious archives, arbitrary scripts hoặc post-install hooks trong project packages, extensions và build dependencies.
- Ký firmware/board package khi target hỗ trợ; kiểm tra board identity và rollback recovery theo device family.
- Mandatory kill/stop/cancel, idempotency, quota và circuit breaker cho jobs; physical emergency stop phải có phần cứng độc lập, không phụ thuộc UI.

### 14.4 Dữ liệu, quyền riêng tư và sở hữu trí tuệ

- Data classification: public documentation, internal source, user projects, licensed IP/PDK, instructor answers, student records, telemetry, captures, secrets.
- Rõ data controller/processor và nơi lưu trữ theo môi trường triển khai; data retention, encrypted backups, deletion/export, disaster recovery và audit.
- Mọi export, share, AI retrieval, model training và external connector cần policy/cấp quyền phù hợp; không mặc định dùng nội dung project hoặc dữ liệu học tập để huấn luyện mô hình.
- PDK, firmware SDK, model libraries, EDA adapters, font/assets có license độc lập và attribution; không giả định có quyền phân phối lại chỉ vì mã nguồn đọc được.

---

## 15. Deployment & Operational Topologies

### 15.1 Các kiểu triển khai mục tiêu

| Topology | Phù hợp | Yêu cầu chính |
|---|---|---|
| **Local Desktop / Offline** | Học IDE/Blocks, một số analysis và virtual simulation hỗ trợ local | Sandboxed local engines, cached content, explicit sync |
| **Campus / School On-Prem** | Một hoặc nhiều lớp học, Lab stations, LMS integration | Tenant/role scope, managed edge, backup, capacity, physical access |
| **Cloud Hosted** | Learn/Project/Studio/Analyze và managed jobs | Tenant isolation, autoscale theo bằng chứng, cost quotas, regional data policy |
| **Hybrid Cloud + Lab Edge** | Remote experiment từ bên ngoài và phần cứng trong trường | Secure gateway, no public private RPC, short-lived station permissions, offline safety |

**Không hardcode một topology cho toàn bộ Organization.** Nginx/PostgreSQL/systemd/Linux Mini PC từ Lab-First là cấu hình Lab Edge; chúng không bắt buộc Desktop offline hoặc toàn bộ Cloud phải dùng chung deployment model. Không tạo Kafka/Kubernetes/microservices trước khi cần bởi failure isolation, tải hoặc quy mô nhóm vận hành.

### 15.2 Observability & operations

Metrics theo domain: API latency, project sync conflicts, build queue time, worker crashes, memory/capture volume, lab Stop acknowledgment, safety cutoffs, grading job status, device/IoT health và AI usage budget. Trace/correlation IDs xuyên command boundaries, artifact hashes và release selection; logs cần redaction, tenant scoping, retention và audit. Tạo deployment-specific dashboards khi có measurement thực, không lấy target throughput làm SLO đã đạt.

### 15.3 Reliability

Hạ tầng cần health/readiness, idempotent job handling, transactional outbox/event consistency khi phù hợp, timeout/cancel, scheduled backups và restoration drills. Lab station **quarantine** khi cleanup không chắc chắn; LMS submission/grade cần immutable audit/revision; firmware flash thất bại phải có phương án recovery hợp lệ theo board. Cross-domain eventual consistency được biểu diễn rõ thay vì distributed transaction toàn hệ.

---

## 16. Versioning, Releases và Compatibility

### 16.1 Quy tắc phát hành GitHub cấp Organization

```text
Repository release tag  = <repo_name>-<version>
GitHub Release title    = <repo_name>-<version>
Example                 = cirnet-simulator-0.2.2  (ví dụ đặt tên; không xác nhận đã release)
```

Không tự thêm prefix `v`; không đổi tên tag lịch sử đang được pin. Project format, Contract schema, API, worker protocol, board profile, toolchain adapter, firmware/bitstream, plugin ABI và **system release-set** có versions riêng; không dùng một số phiên bản cho tất cả.

### 16.2 Release-set và khả năng tái lập

Một deployment đã chọn phải có **Composition Manifest** rõ package identity: repository, source SHA, artifact bytes digest, target OS/ABI, build toolchain, public contract/schema/SDK fingerprint, required runtime dependencies, migrations và rollback constraints.

- `cirnet-remote` **tiếp tục là owner của Lab release lock** tại `integration/release-set.json` theo Lab-First; không sửa lịch sử để đưa candidate mới vào mặc định.
- `cirnet-platform` có thể quản **deployed application composition catalog** toàn hệ sinh thái khi thực sự có nhiều app; catalog này **tham chiếu** Lab release lock, không thay quyền phê duyệt electrical/FPGA compatibility của Lab owner.
- `cirnet-toolchains` xuất build provenance; `cirnet-devices` xuất device/board compatibility; `cirnet-project` pin input artifact IDs; `cirnet-lms` pin submission/grade evidence.
- Mỗi run lưu exact software+data+hardware capability, không dùng dynamic `latest`, unpinned SDK hoặc auto-upgrade mid-run.

### 16.3 Dependency update discipline

1. Contract producer xác định API change, fixtures, deprecation/migration window và SDK regeneration.
2. Consumer kiểm thử trên exact selected versions, không sửa generated SDK thủ công.
3. Chỉ cập nhật compatibility manifest khi integration tests có bằng chứng; release candidate không tự thành release.
4. Deploy có migration/backup/rollback; kiểm soát thay đổi station/firmware khi đang có physical lease.
5. Đưa feature vào UI theo **verified capability**, không qua flag giả để demo như đã hoạt động.

---

## 17. Verification & Architecture Conformance

### 17.1 Các cổng kiểm chứng thường trực (không phải Phase)

| Gate | Bằng chứng tối thiểu | Phạm vi |
|---|---|---|
| **ARCH-OWN** | Một domain owner/SoT, không thêm duplicated services/schemas; review dependency graph | Mọi Repo |
| **CONTRACT** | Cross-repo contract tests, SDK generated fixtures, backward-compatibility review | Public API / format / RPC / ABI |
| **SECURITY** | Tenant isolation, authz, secret handling, untrusted input, audit/export controls | Platform/Project/LMS/AI/Tools |
| **REPRO** | Exact artifacts, source/build digests, pinned compiler and input data, deterministic fixtures | Build/Sim/EDA/Scientific jobs |
| **E2E-LAB** | Real browser → Remote → Hardware → C++ worker → result, stop, persistence, no hidden mocks | Virtual Digital Lab |
| **PHYSICAL-SAFETY** | Safe/indeterminate gate, hardware interlock, HIL/measurements, no rewiring, quarantine | Lab physical and FPGA |
| **ACADEMIC** | Authorized submission revision, grade audit, rubric evidence, answer disclosure | Learn/LMS |
| **DEVICE** | Board identity, firmware compatibility, flash permissions, recovery path | IDE/Devices/Toolchains/IoT |
| **NUMERICAL** | Units, reference datasets, tolerance/precision, performance and reproducibility | Numerics/Analyze/Robotics |
| **EDA** | Tool/PDK licenses and pinned outputs; clear limits of RTL/FPGA/ASIC sign-off | Silicon/FPGA |
| **CROSS-PLATFORM** | Windows/Linux/macOS/mobile support only on tested matrix, native permissions, UI state | Relevant clients |
| **RELEASE** | CI, SBOM, license, manifests, install/restore, failover/rollback, reviewed release notes | Exact selected deployment |

Không gọi `PASS` cho gate chưa chạy; dùng `NOT_RUN`, `BLOCKED`, `FAILED` khi đúng. Tool outputs và historical reports phải giữ provenance; không biến fixture giả thành physical evidence.

### 17.2 Definition of Done cho mọi Repo

Mỗi feature có: owner, contract version, affected consumers, least-privilege permission, failure/cancellation path, resource budgets, happy/negative tests, upgrade/migration/rollback, exact evidence, docs EN/VI và changelog khi áp dụng. Chỉ phát hành capability đã kiểm chứng trong môi trường tương ứng.

### 17.3 Test placement

Unit/domain tests đặt gần code owner. Cross-repo conformance fixtures do Contracts và owner domain quản; integration/E2E cần test manifest pin đúng selected artifacts, có thể điều phối bởi Repo có trách nhiệm tích hợp (Lab: Remote; ứng dụng: tương ứng shell/deployment owner). Không gom mọi test thành một Repo duy nhất làm mất owner/traceability.

---

## 18. Quy ước quản lý Repository, tài liệu và AI Agents

### 18.1 Một tài liệu kiến trúc chuẩn, nhiều đặc tả chuyên sâu

**Đề xuất nơi đặt tài liệu:** `about-cirnetlab/architecture/cirnetlab-architecture.md` vì đây là Repo tài liệu cấp Organization, **không phải runtime**. Các Repo sản phẩm và `cirnet-rules` chỉ liên kết tới nguồn chuẩn này. Nếu Organization chọn một docs location khác, vẫn **chỉ giữ một canonical copy** và cập nhật links.

`cirnet-architecture-lab-first.md` là **đặc tả chuyên sâu** của CIRNET Lab; văn bản này không xóa, thay tên, gộp hoặc sao chép toàn bộ Lab-First. Source owner của tài liệu Lab-First cần được phê duyệt và khai báo bằng canonical URL/path; khi đổi từ tài liệu cục bộ sang GitHub phải giữ link migration và lịch sử.

### 18.2 Quy tắc cho AI Agents

```text
Read Organization architecture
  → identify owning Repository + its AGENTS/Rules/Skills
  → inspect canonical schema and selected artifacts
  → select smallest scoped change in source owner
  → update consumers using versioned contract
  → test negative paths and cross-repo boundaries
  → Search Before Create → Update Existing → Consolidate → Verify Docs
  → report evidence/NOT_RUN and request human approval for release
```

`cirnet-rules` quản Agent rules/skills; **không** lưu một bản sao diverge của toàn bộ kiến trúc. New Repo README cần scope, owner, language/runtime, API contracts, data ownership, dependencies, non-goals, acceptance criteria và status thật; tránh README chỉ chứa tên sau khi bắt đầu implementation.

### 18.3 Monorepo vs multi-repo

Giữ **multi-repository** theo domain đang có. Local folder chung `E:\ALL_PROJECTS\LUAN_VAN_TOT_NGHIEP\REPO` chỉ là workspace quản lý các clone, không là nguồn chuẩn cho cross-repo import, không đồng nghĩa có quyền sửa tất cả. Có thể dùng workspace file/automation để chọn đúng commits, nhưng không thêm dependency `../some-repo/src/...` ở production.

**Tách Repo mới chỉ khi** bounded context rõ ràng, version/release độc lập, license/security boundary, reuse đa ứng dụng hoặc đội sở hữu riêng thực sự tồn tại. Hiện 33 Repo đã phủ tương đối rộng các miền đề xuất; **không đề xuất tạo thêm Repo chỉ để chứa features con** như gradebook, waveform viewer, ESP32 compiler, notebook plot hoặc telemetry chart.

---

## 19. Các ví dụ phối hợp xuyên Repository

### 19.1 Một bài thực hành điện tử

`Learn/LMS` giao bài và quyền → `Project` tạo revision → `Studio/Interface` sửa CircuitGraph → `Remote` xác thực experiment → `Hardware` supervises `Simulator C++` → `Remote` lưu result/capture → `Analyze` biểu diễn nếu cần → `LMS` thu immutable evidence + rubric/giảng viên đánh giá. Nếu dùng mạch thật, áp dụng toàn bộ Safety/Observed Endpoint rules của Lab-First; không thay bằng một nút “Run” trong LMS.

### 19.2 Lập trình ESP32/STM32/Arduino

`IDE` hoặc `Blocks` tạo chương trình → `Project` lưu revision → `Toolchains` chọn compiler/SDK manifest chính xác → `Devices` chọn BoardProfile/flash adapter → thực hiện sandboxed build → output firmware có SHA-256 → actor xin flash permission → adapter chọn đúng thiết bị → flash/verify/report → `Project` lưu logs; nếu là assignment, `LMS` chỉ tham chiếu evidence. Board feature chưa triển khai trả `UNAVAILABLE`.

### 19.3 Phân tích tín hiệu

`Hardware/Simulator/IoT` phát capture/telemetry với provenance → `Project` đăng ký artifact → `Analyze` dùng `Numerics` tính FFT/filter/plots → sinh analysis document gắn input digests và units → người dùng so sánh/đánh giá, có thể nộp qua LMS. Không sửa raw measurement để “làm đẹp” kết quả.

### 19.4 FPGA và ASIC learning

`IDE/Project` lưu RTL + constraints → `FPGA` sở hữu testbench/bitstream board workflow; `Silicon` sở hữu ASIC/PDK workflow → `Toolchains` chạy các build/EDA jobs được pin → `Analyze` xem reports/waveforms → `LMS` thu chứng cứ. FPGA bitstream PASS không là ASIC timing/DRC/LVS sign-off.

### 19.5 Robotics và IoT

`Studio/Robotics` chuẩn hóa model → `Numerics` chạy control/kinematics → `Devices/Toolchains` build target code → `IoT` nhận telemetry và quản fleet → `Analyze` biểu đồ → `LMS` đánh giá bài học. Robot/actuator có safety riêng, không nhận “safe” từ phép kiểm điện tử Lab-First nếu chưa có cơ chế đánh giá rủi ro chuyển động.

---

## 20. Architectural Decisions (quyết định mục tiêu)

| ID | Quyết định | Hệ quả không được vi phạm |
|---|---|---|
| **CIRNET-ECO-001** | Ecosystem architecture cấp Org nằm trên Lab-First | Lab-First giữ quyền sở hữu đặc tả electrical safety; Ecosystem không viết lại rules Lab |
| **CIRNET-ECO-002** | Multi-platform, one project, typed contracts | Không nhân đôi business data hoặc engine theo UI platform |
| **CIRNET-ECO-003** | One source of truth per bounded context | Platform/IAM, Project, LMS, Remote, Format, Database có owner tách bạch |
| **CIRNET-ECO-004** | Rust-first backend/Hardware/Contracts; **C++ Simulator default** | Không tự chuyển Simulator sang Rust; Python không tham gia production service path |
| **CIRNET-ECO-005** | Contract-driven, compatibility-first | Cập nhật source owner, generated SDK và consumer tests; không patch copy |
| **CIRNET-ECO-006** | Build / flash / run / grade là các quyền riêng | Không cho nút UI/extension/AI bỏ qua scope hành động có tác dụng phụ |
| **CIRNET-ECO-007** | Identity vs academic vs physical authority tách biệt | LMS Instructor role không cấp hardware SAFE; AI không làm grade/safety authority |
| **CIRNET-ECO-008** | Immutable artifacts + reproducible execution | Mỗi job/run có exact input and output digests, schema versions và provenance |
| **CIRNET-ECO-009** | Verified capabilities, not planned UI promises | Unsupported targets/solvers/instruments trả `UNAVAILABLE`; không bịa telemetry |
| **CIRNET-ECO-010** | Infrastructure-only physical routing | FPGA không được sửa dây kết nối sai của sinh viên |
| **CIRNET-ECO-011** | Modular architecture, deploy by evidence | Không bắt buộc microservices/Kubernetes hoặc chia nhỏ Repo tùy tiện |
| **CIRNET-ECO-012** | Signed/sandboxed extension and build boundaries | Không cho arbitrary plugin/notebook/compiler code chạy trong privileged service |
| **CIRNET-ECO-013** | Multi-tenant education privacy and IP protection | Tenant data, student records, instructor answers, private project/PDK có policy khác nhau |
| **CIRNET-ECO-014** | Names remain stable; artifacts independently versioned | `<repo_name>-<version>`, compatibility lock, không đổi tag lịch sử |
| **CIRNET-ECO-015** | Documentation hygiene and evidence integrity | Một canonical architecture; cập nhật tài liệu hiện có, bảo toàn lịch sử và kết quả thật |

---

## 21. Những vấn đề cần chốt ở cấp Organization (không phải lịch trình)

Các mục này là **quyết định mở**, không được suy ra là hệ thống đã thực hiện:

1. **Architecture storage authority:** phê duyệt nơi đặt một bản canonical `cirnetlab-architecture.md`, Lab-First URL chuẩn và chính sách link/translation EN/VI.
2. **Identity migration:** domain Platform mới có README; Auth hiện có ở Remote. Cần cách migrate account/session/authorization có rollback trước khi Platform trở thành issuer chính.
3. **Project format ownership:** định nghĩa rõ `cirnet-project` vs `cirnet-format-project` vs `cirnet-contracts`, tránh ba nguồn serialization cạnh tranh.
4. **SPICE ownership:** xác nhận có cần `cirnet-nhspice` độc lập không; chuẩn hóa README heading `cirnet-core-spice` với tên Repo và licensing strategy.
5. **Lab release compatibility:** real Virtual Digital E2E với Rust services + C++ Simulator pinned; chứng nhận Physical vẫn cần board HIL/electrical evidence.
6. **Engineering language policy:** Python tiếp tục chỉ phục vụ platform testing/benchmark; nếu hỗ trợ người học lập trình Python/MicroPython, cần ADR riêng cho sandbox/user workload, không đưa vào production platform service.
7. **Shared UX primitives:** xác định reusable UI/package design system giữa Studio/Interface/IDE/Learn/Analyze, không copy source sang từng Repo.
8. **Platform composition & deployment:** quy định tenant IAM, school on-prem/cloud/hybrid, artifact store, data residency, backup và cross-app session lifecycle.
9. **External tool licenses:** toolchains, ROS, numerical libraries, ngspice, EDA/PDK và extension packages cần license/provenance policy trước khi phân phối.
10. **Academic governance:** data retention, FERPA/GDPR/local requirements khi áp dụng, minor consent, rubric/audit, interoperability standards và assessment appeal workflow.
11. **AI boundaries:** model providers, data sharing/retention, human review, cost quotas và không can thiệp physical controls.
12. **Runtime claim vocabulary:** duy trì `TARGET/DESIGNED/IMPLEMENTED/TESTED/VERIFIED/RELEASED/UNAVAILABLE/BLOCKED` có bằng chứng; các README-only Repo không được quảng cáo là supported.

Không có yêu cầu chia dự án thành Phase; mỗi bounded context có thể tiến hóa độc lập **theo compatibility/quality gates**, và chỉ tích hợp vào deployment khi toàn bộ ranh giới liên quan đã được kiểm chứng.

---

## 22. Tóm tắt kiến trúc dài hạn

**CIRNET = Engineering Project + Shared Platform + Domain Applications + Verified Engineering Engines + Learning Evidence.** Sáu Repo Lab-First ban đầu vẫn là lõi **Lab Virtual-to-Physical**; phần mở rộng tạo các miền IDE/Blocks/Toolchains/Devices, Analyze/Numerics, Learn/LMS, Project/Format/Database, Studio/Desktop/Mobile, Robotics/IoT/Silicon/AI. Tổng cộng **30 Repo sản phẩm/công cụ**, một Repo Rules và hai Repo tài liệu/config là các ranh giới đang tồn tại trên GitHub tại ngày rà soát.

**CIRNET không cần thêm Repo để thể hiện tầm nhìn.** Điều cần giữ vững là trách nhiệm mỗi Repo, một source of truth cho mỗi domain, hợp đồng độc lập, artifact được pin, bảo mật và an toàn vật lý theo đúng loại bằng chứng. Một người học có thể chuyển giữa **Learn → Studio/IDE/Blocks → Lab/Analyze → Submit/Assessment** trong cùng một EngineeringProject mà không làm lẫn vai trò của UI, engine, hardware và học vụ.

> **Văn bản này là đặc tả kiến trúc mục tiêu, không phải chứng nhận implementation, production, hiệu năng, an toàn điện hay ASIC sign-off.** Mọi năng lực được quảng bá và phát hành phải được đối chiếu với mã nguồn, bộ artifact cụ thể và kết quả kiểm thử trên môi trường thực.

---

## Phụ lục A — Nguồn và provenance

1. **Nguồn người dùng:** `cirnet-architecture-lab-first(3).md`, bản CIRNET Lab-First được cung cấp trong cuộc trò chuyện, 1.240 dòng. Các bất biến kế thừa đặc biệt nằm ở phần Contract ownership, §6 Virtual Digital E2E, §8–10 Physical/Endpoint/Safety, §11 HardwareStation, §12 FPGA, §18 Release, §19 Verification.
2. **GitHub:** Organization [`cirnetlab`](https://github.com/orgs/cirnetlab/repositories), đã truy xuất danh sách **33 Repo** ngày 2026-11-15 thông qua kết nối GitHub, bao gồm toàn bộ tên ở §4 và §4.6. Các Repository chỉ có README/License đã được kiểm tra riêng; không giả định có implementation.
3. **Ảnh nguồn local:** ảnh Windows Explorer được cung cấp cùng yêu cầu xác nhận một phần tên thư mục dưới workspace, không chứng minh Git commit hay build status trên PC.
4. **Phần tổng hợp mới:** Architecture/Repository Responsibility Matrix, Universal EngineeringProject model, Studio/Platform/LMS/IDE/AI/Robotics/IoT/Silicon boundaries, security/learning integration và quyết định cấp Organization là **đề xuất thiết kế** phát triển từ mục tiêu STEM & Engineering Learning Ecosystem của người dùng.
5. **Không thực hiện:** Không checkout/chỉnh sửa `E:\...\REPO`, không tạo Repo GitHub mới, không sửa source/code, không thay `AGENTS.md`, không merge, tag hay release.

## Phụ lục B — Thuật ngữ ngắn

| Thuật ngữ | Ý nghĩa |
|---|---|
| Bounded context | Phạm vi nghiệp vụ có owner/schema/rules rõ ràng |
| Canonical contract | Định dạng/API liên ranh giới do một nguồn chuẩn quản lý |
| EngineeringProject | Aggregate dự án kỹ thuật, quản lý revisions và references tới artifacts |
| Artifact | Gói dữ liệu/binary/model/report có định danh, digest và metadata |
| Provenance | Thông tin nguồn: input, version, compiler, worker, device và environment |
| Capability | Tính năng thực được backend/driver khai báo, dựa trên implementation/verification |
| Virtual Lab | Engine mô phỏng có giới hạn, không tạo evidence điện thực |
| Physical Lab | Thí nghiệm với mạch/thiết bị thật có safety/interlocks và measured evidence |
| Release-set | Bộ artifact đã được pin, kiểm tương thích để tích hợp/triển khai |
| Source of truth | Một chủ sở hữu hợp lệ cho mỗi domain datum/decision |
| HIL | Hardware-in-the-loop; bằng chứng trên phần cứng mục tiêu |
| PDK | Process Design Kit; assets/rules cho công nghệ sản xuất vi mạch |
