# CIRNET Lab-First — Kiến trúc hệ thống Lab

> **Phạm vi:** Lab điện tử ảo và vật lý trong sáu repository Lab-First.  
> **Tài liệu liên quan:** [Kiến trúc Ecosystem](cirnetlab-architecture-vi.md) · [Kiến trúc FPGA](../cirnetlab-FPGA-architecture-vi.md) · [English](cirnet-architecture-lab-first-en.md)  
> **Tên kiến trúc chuẩn:** **CIRNET Lab-First** (không gắn phiên bản vào tên).  
> **Định hướng kỹ thuật:** **Rust core services + C++ Simulator · Contract-driven · Virtual-to-Physical · Safety-gated · Lab-first**.  
> **Tài liệu nguồn:** `cirnet-Remote_Architecture_Lab-First.md`; được tái cấu trúc và chuẩn hóa cho sáu Repository, bổ sung các ranh giới đã xác định trong mã nguồn hiện có.  
> **Ngày rà soát cơ sở:** 2026-11-15. Các quan sát triển khai bên dưới mô tả thời điểm đó. Nghiệm thu production hoặc an toàn điện cần bằng chứng riêng cho cấu hình đã chọn.

## 0. Tóm tắt điều hành

**CIRNET** là nền tảng phòng thí nghiệm điện tử từ xa, sử dụng **một quy trình thiết kế** để thực hiện **mô phỏng ảo** và **thí nghiệm trên mạch thật**. Sinh viên dựng mạch trên Web; kết nối điện và dữ liệu được quản lý bằng hợp đồng chung; Mini PC điều phối; FPGA xử lý các thao tác xác định thời gian và các thiết bị đo/điều khiển.

Triết lý bất biến:

> **Computer checks SAFETY → a demonstrably safe, executable circuit may run → physics produces observations → software evaluates and teaches.**

**SAFETY ≠ CORRECTNESS.** Mạch sai đáp án không mặc nhiên bị chặn. Ngược lại, phần mềm **không được suy diễn** rằng mạch chưa rõ danh tính là an toàn. Chỉ cho phép cấp năng lượng khi có bằng chứng đáp ứng giới hạn điện, bảo vệ phần cứng hoạt động và các endpoint cần dùng thực sự route được.

| Nội dung | Kiến trúc mục tiêu | Quan sát tại thời điểm rà soát |
|---|---|---|
| Tổ chức mã nguồn | Sáu Repository độc lập theo trách nhiệm, Contracts làm chuẩn giao tiếp | Đã hình thành ranh giới chính |
| Công nghệ xử lý mặc định | **Rust** cho Backend, Contracts runtime, Hardware Service; **C++** cho Simulator | Rust Remote/Contracts/Hardware đã có candidate; Simulator có C++ worker được pin |
| Python | **Chỉ testing, benchmark, parity và công cụ xác minh ngoại tuyến** | Luồng Python lịch sử vẫn tồn tại trong bản release được pin; phải chuyển đổi theo cổng nghiệm thu |
| Giao diện | Vue 3 + TypeScript + Pinia + Three.js | Đã có nền tảng Web |
| Virtual Digital | E2E: Web → Rust Remote → Rust Hardware → Simulator → Web | Cần nghiệm thu trên một bộ artifact được pin cùng nhau |
| Virtual Analog | C++ adapter tới SPICE-compatible engine; mixed-signal-ready | Chưa xác nhận solver analog hoạt động; không quảng cáo là `SUPPORTED` |
| Physical Digital | Safety-gated, quét topology, resolve endpoint, route hạ tầng, quan sát mạch thật | Hardware/FPGA vật lý chưa được HIL và kiểm định an toàn điện đầy đủ |
| Physical Analog | Nâng cấp tương lai thông qua capability/adapter/hardware | Ngoài phạm vi Physical Lab giai đoạn đầu |
| Instrument | Logic Analyzer, Scope 2CH, Generator 2CH; ADC/DAC/AFE/SDRAM ngoài FPGA | Giao diện FPGA hiện là scaffold, chưa phải thiết bị đo đã nghiệm thu |
| Tải người dùng | 30–50 máy khách kết nối. Mỗi hạ tầng điện dùng chung đã xác minh có một slot thực thi. | Mục tiêu cần kiểm thử tải trên phần cứng đích |

**Quy tắc báo cáo:** `TARGET`, `DESIGNED`, `IMPLEMENTED`, `TESTED`, `VERIFIED`, `RELEASED`, `UNAVAILABLE`, `BLOCKED` phải được dùng đúng nghĩa. Mã chạy trong test không đồng nghĩa bản phát hành hoặc năng lực điện thực.

---

## 1. Mục tiêu, nguyên tắc và giới hạn

### 1.1 Mục tiêu

1. Cung cấp Web-CAD/Workbench thống nhất cho **design → simulation → physical experiment → observe → assessment**.
2. Đặt **CircuitGraph** là nguồn sự thật của **thiết kế logic**; cảnh 3D chỉ là phép chiếu của thiết kế.
3. Giữ các hợp đồng dữ liệu độc lập với phiên bản FPGA, backend, browser và loại simulator.
4. Hoàn thiện **Virtual Digital end-to-end trước**, sau đó mở rộng Virtual Analog, Physical Digital và instrument thực.
5. Cho sinh viên lắp mạch thật ở vị trí khác giảng viên: tự dò mạng điện, tìm endpoint theo dữ liệu quan sát, **không nối lại dây sai bằng phần mềm**.
6. Tách biệt **an toàn điện**, **đối chiếu topology** và **đánh giá chức năng**.
7. Chạy backend/hardware/simulation trên Mini PC; trình duyệt tự render 3D; FPGA phụ trách các thao tác deterministic và timing-critical.
8. Hỗ trợ nhiều phiên làm việc với quyền sở hữu, lease, recovery và release xác định.
9. Thay thế **Python trong đường xử lý production** bằng Rust cho Backend/Contracts/Hardware và C++ cho Simulator, có kiểm chứng tương thích và rollback; vẫn dùng Python cho testing/benchmark khi cần.

### 1.2 Không thuộc phạm vi kiến trúc giai đoạn đầu

- Không xây Kubernetes, Kafka, RabbitMQ, Redis Cluster, full microservices hoặc server-side 3D rendering nếu chưa có bằng chứng cần thiết.
- Không chạy các mạch analog tùy ý trên physical FPGA khi chưa có analog routing/protection độc lập.
- Không coi vị trí vật lý breadboard của giảng viên là đáp án chấm điểm.
- Không tuyên bố nhận dạng mọi IC bằng scan điện.
- Không chuyển C++ Simulator sang Rust; **C++ là ngôn ngữ mặc định** cho Simulator, với worker được pin và kiểm chứng.
- Không cho phép thay đổi UI/Contracts để che giấu tính năng chưa triển khai (ví dụ quảng cáo oscilloscope khi không có AFE/ADC).

### 1.3 Nguyên tắc kiến trúc bắt buộc

- **Safety-gated, not correctness-gated.** Không dùng topology matcher như Run Gate.
- **Observed physical truth.** Dây và linh kiện sinh viên đang cắm là trạng thái vật lý duy nhất được thực thi.
- **One source of truth per domain.** Không để frontend, Hardware và Simulator tự định nghĩa các bản schema riêng.
- **Contract-first, compatibility-first.** Thay đổi public schema/ABI phải có owner, migration và fixtures xuyên Repo.
- **Rust core services + C++ Simulator, Python verification-only.** Không đưa Python mới vào API, station, solver worker hoặc logic điều khiển production.
- **Explicit capabilities.** Tính năng chưa chạy được phải trả `UNAVAILABLE`/giá trị capability bằng 0, không tạo mẫu đo giả.
- **Smallest verified change.** Tập trung thay đổi có thể nghiệm thu; không chuyển sang microservices hoặc framework mới không có nhu cầu.
- **Search Before Create → Update Existing → Consolidate When Appropriate → Verify Documentation.** Giữ tài liệu ngắn, không tạo báo cáo trùng cho cùng một vấn đề.

---

## 2. Organization CIRNETLAB và quyền sở hữu mã nguồn

| Repository | Domain sở hữu | Công nghệ runtime mục tiêu | Điều không được sở hữu |
|---|---|---|---|
| [`cirnet-remote`](https://github.com/cirnetlab/cirnet-remote) | Lab API/WS, phân quyền, cấu hình kỹ thuật bài Lab, quản lý thí nghiệm, lịch/lease, lưu capture, triển khai và **khóa phát hành Lab**. Sở hữu định danh và dự án Lab cục bộ trong phạm vi độc lập. | **Rust** (`axum`, `tokio`, `tonic`, PostgreSQL adapter) | Thao tác thanh ghi MUX/FPGA trực tiếp, lõi mô phỏng, bản ghi dùng chung đã chuyển giao |
| [`cirnet-interface`](https://github.com/cirnetlab/cirnet-interface) | Vue Workbench, CircuitGraph editor, Pinia UI state, Three.js, instrument UI, typed SDK consumer | **TypeScript** trên browser | Không quyết định electrical safety, không tạo hardware routing và không làm mô phỏng có thẩm quyền server |
| [`cirnet-contracts`](https://github.com/cirnetlab/cirnet-contracts) | Canonical CircuitGraph/Assignment/Physical Lab/schema, catalog, RPC, NCAP, wire protocol, codegen, SDK | **Rust** runtime + generator; TS bindings được sinh | Không đăng ký API business routes thay Remote, không quản lý station hoặc user |
| [`cirnet-hardware`](https://github.com/cirnetlab/cirnet-hardware) | `HardwareStation`, local-only RPC, station lifecycle, safe stop, FPGA transport/driver, physical protection integration, supervised simulator worker | **Rust** (`tonic`, `tokio`) | Không cấp quyền người dùng, không giữ durable lease/DB nghiệp vụ |
| [`cirnet-simulator`](https://github.com/cirnetlab/cirnet-simulator) | Deterministic simulation engine, digital event model, bounded DC, SPICE adapter boundary, worker protocol | **C++ là ngôn ngữ mặc định**; worker được pin, giám sát và kiểm chứng | Không truy cập browser, quyền user, FPGA vật lý hoặc DB ứng dụng |
| [`cirnet-fpga`](https://github.com/cirnetlab/cirnet-fpga) | RTL, timing, trigger, protected control, FPGA protocol decoder, digital routing controller, instrument/SDRAM interface, testbench | **HDL/RTL** (Verilog/SystemVerilog), không bị thay bằng Rust | Không chứa auth, REST, database hoặc trình biên dịch graph business |

**Cấu hình triển khai.** Lab độc lập dùng định danh, dự án và bài Lab cục bộ của Remote. Với phạm vi đã chuyển giao, Platform cung cấp định danh và ngữ cảnh tenant, Project sở hữu phiên bản dự án và ACL, còn LMS sở hữu bài tập học vụ, bài nộp và điểm. Remote tiếp tục sở hữu phân quyền Lab, lease, cấu hình kỹ thuật và bằng chứng thí nghiệm. Áp dụng quy tắc chuyển giao theo phạm vi và quay lui tại [Ecosystem §5.1](cirnetlab-architecture-vi.md#51-lab-độc-lập-và-quyền-sở-hữu-khi-tích-hợp).

**Quản trị giao diện.** Bên sở hữu phát hành nguồn chuẩn thành artifact có phiên bản. Consumer dùng đúng phiên bản đã chọn. Remote chọn chính xác phiên bản và mã băm của bộ phát hành Lab.

**Lưu ý công nghệ:** Rust là ngôn ngữ mặc định cho Backend/Contracts/Hardware, **C++ là ngôn ngữ mặc định cho Simulator**; không thay TypeScript trên browser hay HDL trong FPGA. Worker C++ phải được pin và kiểm chứng; Python chỉ phục vụ lanes kiểm thử, benchmark và đối chiếu lịch sử, không nằm trong đường request production mới.

### 2.1 Phân biệt baseline đã phát hành và source candidate

Tại thời điểm rà soát, hệ sinh thái có **system release lock lịch sử** và các **source candidate** có nhịp tiến hóa riêng (ví dụ Rust Remote, Rust Contracts, Rust Hardware, C++ Simulator). Chúng **không** tự động cấu thành một hệ thống đã được phát hành chỉ vì cùng tồn tại trên `master`.

Mỗi tài liệu phát hành phải phân biệt:

```text
Published, pinned release set
        !=
Current master branches / candidate builds
        !=
Target architecture / future capability
```

Các tag lịch sử vẫn phải được giữ để rollback. Quy ước tag/release mới của CIRNETLAB: `<repo_name>-<version>`; áp dụng **về sau**, không đổi tên tag/asset đã phát hành và đang được pin.

---

## 3. Kiến trúc logic tổng thể

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
│ │ Safety/Topology/Functional results · WS/Binary Gateway         │ │
│ └───────────────────────────┬───────────────────────────────────┘ │
│                             │ localhost gRPC / Protobuf           │
│ ┌───────── cirnet-hardware · RUST SERVICE PROCESS ──────────────┐ │
│ │ Virtual/Physical HardwareStation · admission/stop             │ │
│ │ Driver · capability · protection interface · transport         │ │
│ └─────────────────┬─────────────────────┬───────────────────────┘ │
│                   │                     │                         │
│         Virtual execution         Physical execution               │
│                   ▼                     ▼                         │
│     cirnet-simulator worker      FPGA host transport               │
│     C++ engine (default)         (Serial/USB-SPI/USB3)             │
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

**Đường dữ liệu và đường điều khiển độc lập:** UI không quyết định cổng cấp điện; public API không điều khiển chân FPGA; Hardware Service không tự chấm đúng/sai; FPGA không nắm user/project.

### 3.1 Dịch vụ và tiến trình

- **Nginx:** static frontend, HTTPS, REST/WS reverse proxy.
- **Rust Remote:** modular monolith, một scheduler có thẩm quyền cho station chia sẻ; không tách microservices trong giai đoạn đầu.
- **Rust Hardware Service:** tiến trình riêng, cách ly crash/timeout từ USB/FPGA/simulator worker.
- **Simulator Worker:** tiến trình được giám sát; CPU-bound không làm nghẽn Tokio/HTTP hoặc Stop.
- **PostgreSQL:** lưu phiên Lab, lease, thí nghiệm, metadata capture và bản ghi định danh Lab cục bộ trong cấu hình độc lập. Lưu waveform lớn bên ngoài cơ sở dữ liệu.
- **Python:** không được khởi động từ API, Hardware Service hoặc Simulator worker production mới; chỉ chạy trong test/benchmark tools riêng.

---

## 4. Quy chuẩn công nghệ Rust services + C++ Simulator

### 4.1 Runtime

| Boundary | Lựa chọn chuẩn | Ghi chú |
|---|---|---|
| HTTP/WS | Rust + `axum` + `tokio` | Ưu tiên stack hiện hữu của Rust Remote, không mở framework cạnh tranh |
| Local Hardware RPC | Rust + `tonic` / Protobuf | Service chỉ bind loopback hoặc socket riêng được xác thực/phân quyền |
| Database | Rust + `sqlx` / PostgreSQL | SQLite chỉ cho development/tests hoặc đường migration được kiểm soát |
| Serialization/validation | Rust `serde` + nguồn schema từ Contracts | Không viết hai logic validator không tương thích |
| Experiment domain | Typed Rust state machine và immutable plans | Các side effects được kiểm soát theo lifecycle |
| Virtual Digital | **C++** discrete-event simulator/worker mặc định | Worker C++ được pin SHA-256, giám sát và kiểm chứng qua Hardware Service |
| Analog | C++ adapter tới native SPICE-compatible solver | Chưa có solver đáng tin cậy thì trả `UNAVAILABLE` |
| FPGA runtime | RTL được synthesize trên FPGA | Rust chỉ quản lý host driver/transport, không thay HDL |
| Testing & benchmark | Rust tests, TypeScript tests, Python pytest/benchmark | Python được phép ở offline verification lane, không ở runtime production |

Các tên crate/phụ thuộc trong bảng là định hướng phù hợp với mã nguồn đã có, **không** là cam kết nâng cấp version hoặc yêu cầu viết lại toàn bộ.

### 4.2 Quy tắc chuyển đổi khỏi Python

1. Không thêm Python service, Python request handler, Python simulator worker hoặc Python dependency mới vào runtime production.
2. Chỉ thay thế một đường xử lý khi có **contract parity, functional parity, error parity, performance evidence, rollback path**.
3. Giữ Python lịch sử trong lane riêng để đối chiếu bản release đã pin; không xóa các artifact/release/fixture phục vụ kiểm chứng.
4. Không đổi wire contract hoặc dữ liệu persisted chỉ vì chuyển implementation sang Rust.
5. Duy trì **C++ Simulator** làm backend mặc định dưới **hash-pinned worker** có provenance, timeout, safe Stop và kiểm chứng tương thích; không yêu cầu Rust replacement.
6. Không chạy `ngspice` hoặc SPICE từ Python production; C++ Simulator tích hợp solver native qua adapter có version và chịu trách nhiệm về cancellation, limits, validation; Rust Hardware giám sát tiến trình worker.
7. Mỗi lần chuyển đổi ghi rõ `CURRENT`, `TARGET`, `VERIFIED`, `NOT_RUN` và rollback condition; không đánh dấu hoàn tất chỉ vì unit test PASS.

### 4.3 Quy tắc hiệu năng và an toàn bộ nhớ

- Async I/O cho HTTP/RPC/WS; CPU-bound worker tách riêng bằng process/giới hạn hàng đợi.
- Không giữ mutex qua `await`; không dùng global mutable state cho mỗi user/experiment.
- Hạn chế queue/capture theo cấu hình; áp backpressure, timeout, cancellation.
- Xử lý số lượng lớn waveform bằng bounded binary stream, không serialize toàn bộ thành JSON.
- Không tối ưu hóa bằng `unsafe` nếu chưa có benchmark, invariant và test chứng minh.
- Mọi buffer/worker/connection có owner, cancellation token, cleanup và hành vi retry xác định.

---

## 5. Nguồn sự thật và hệ hợp đồng CIRNET

### 5.1 Ba lớp thiết kế và thực thi

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

- **DesignGraph:** thiết kế logic có thể chỉnh sửa, biểu diễn bằng hợp đồng chuẩn `CircuitGraph`. Nó mô tả các linh kiện, cổng, net, thuộc tính và vị trí. Vòng đời dự án và lịch sử chỉnh sửa có chủ sở hữu và hợp đồng riêng.
- **ValidatedGraph:** tham chiếu hợp lệ, signal widths/domains, net rules, component model availability; **không** coi sai đáp án là lỗi kỹ thuật.
- **ExecutionPlan:** mapping backend, bounded event schedule hoặc resource allocation; có owner, phiên bản contract, fingerprint và expiry theo scope.
- **Three.js Scene:** chỉ chiếu DesignGraph, không được tạo connectivity từ giao nhau 3D hoặc tọa độ gần nhau.

**Hai nguồn sự thật khác nhau và phải được giữ độc lập:**

```text
Logical design truth     = DesignGraph
Observed physical truth  = DetectedTopologyGraph + measured evidence
```

Đường dây sinh viên cắm sai thuộc **observed truth**; hệ thống có thể cảnh báo và chấm sai nhưng **không** được biến nó thành mạch đáp án ở bước Physical Compiler.

### 5.2 Contract owner và phiên bản độc lập

`cirnet-contracts` sở hữu dạng chuẩn và kiểm thử tương thích của:

- `CircuitGraph` và các version 1.0 / 1.1 / 1.2 đã có; legacy documents phải còn đọc được.
- `SignalDomain`: `DIGITAL`, `ANALOG`, `POWER`, `GROUND`, `CLOCK`, `MIXED`; namespace versioned, `UNKNOWN` không đồng nghĩa DIGITAL.
- `AssignmentDefinition`, `LabEndpoint`, instructor/reference profiles, assessment policy.
- `DetectedTopologyGraph`, `ObservedPinBinding`, `TopologyResolutionResult`, `ResolvedEndpointBinding`.
- `SafetyVerificationResult`, `TopologyVerificationResult`, `FunctionalVerificationResult`, `ExecutionDecision`.
- `StationCapabilities`, `HardwareStation` RPC boundary, error/status codes.
- Lược đồ dùng chung `SimulationRequest/Result/Error` và `VirtualExecutionPlan`.
- Hợp đồng điều khiển FPGA ở mức logic và ánh xạ tương thích, được kiểm chứng cùng Hardware với giao diện board do FPGA sở hữu.
- `ExperimentResult`, `CaptureMetadata`, NCAP v1/v2 và streaming envelope.

Simulator sở hữu định dạng khung worker riêng và fixture, với Hardware là consumer. FPGA sở hữu giao diện board và ABI khung vật lý. Hardware sở hữu driver và tích hợp transport.

**Không gộp phiên bản:** version của Repo, toolkit, CircuitGraph, Assignment, Hardware RPC, FPGA frame, firmware, SDK, NCAP và selected system release là những miền khác nhau. Chúng phải được ghi riêng trong compatibility matrix.

### 5.3 Quy tắc chỉnh sửa Contracts

1. Một mô hình canonical, các DTO/validator TypeScript và Rust được sinh/kiểm tra từ cùng nguồn.
2. Không sửa generated files ở consumer để vá một lỗi source; sửa tại owner rồi regenerate.
3. Thay đổi additive có thể vẫn phá strict consumer; phải thử old-reader/new-writer và new-reader/old-writer khi phù hợp.
4. Unknown optional metadata giữ nguyên khi save/load nếu contract cho phép; không âm thầm làm mất document.
5. Broken contract phải tạo version mới có migration/compatibility window và fixtures, không sửa lén bản đã frozen.
6. Public API operations do **Remote** đăng ký, còn Contracts sở hữu handoff/schema/SDK; private worker frames không bị nhầm thành public API.
7. Tất cả Schema IDs, package digest và resource paths phải có nguồn gốc/manifest, không fetch network động khi thực thi release đã pin.

---

## 6. Virtual Digital — luồng end-to-end bắt buộc

Đây là **ưu tiên nghiệm thu đầu tiên** của kiến trúc Rust services + C++ Simulator. Mục tiêu không phải chỉ một simulator đơn lẻ PASS, mà là **trình duyệt thực sự nhận được kết quả từ một chuỗi process và artifact tương thích**.

### 6.1 Luồng chính

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

**Luồng Stop/cancel riêng:** `Browser Stop → Remote permission + lease → Hardware supervisor → Simulator cancellation → safe acknowledgement → Remote release/quarantine → UI state`. `Stop acknowledged` **không** được đồng nghĩa một nút UI đã được nhấn; phải là cleanup thực tế được xác nhận.

### 6.2 Quy trình dữ liệu tối thiểu

1. **Editor:** lưu DesignGraph hợp lệ, preserve graph identity và revision; scene không ảnh hưởng net connectivity.
2. **Remote:** kiểm tra request quyền user, ownership, CSRF/Origin, versioned DTO, station capability, idempotency và quota.
3. **Contracts:** kiểm tra wire/schema, source/SDK artifact đã pin, backend identity; phân loại domain và support level.
4. **Plan:** compiler tạo VirtualExecutionPlan bất biến có model IDs, net resolution, timing parameters, bounded event budget, instrument selection.
5. **Hardware:** tạo một virtual station execution, quản lý worker, deadline, cancellation, expected protocol/digest.
6. **Simulator:** đọc request đã xác nhận, chạy event scheduler, trả samples/traces + explicit status; không tạo samples nếu unsupported/error.
7. **Hardware → Remote:** Hardware xác minh khung worker, trạng thái và capture rồi trả phản hồi gRPC. Remote lưu metadata thí nghiệm và capture, đồng thời lưu dữ liệu capture.
8. **Remote → Web:** trả Run acknowledgement riêng với kết quả cuối; WS event mang stream identity/sequence và có quy trình resync; binary capture được cấp theo quyền.
9. **UI:** hiển thị state `QUEUED/RUNNING/COMPLETE/FAILED/UNAVAILABLE`; loading/pending không tự suy ra thành PASS.

### 6.3 Bài thí nghiệm reference tối thiểu

```text
SW1 ──┐
      AND (74HC08 logical model) ── LED1
SW2 ──┘

Run vectors: 00, 01, 10, 11
Expected settled AND: 0, 0, 0, 1
```

End-to-end test phải ghi nhận **cả request/response trên mỗi process boundary**, mapping event timestamps, 0/1/X, NCAP/raw bytes, persisted metadata và trạng thái UI. Sau đó tăng độ bao phủ cho NOT, OR, 7-segment, flip-flop, counter, clock, reset, power-aware HC08/LED trong scope model thực sự hỗ trợ.

### 6.4 Bộ artifact duy nhất và điều kiện pin

Remote sở hữu manifest chọn **bộ phát hành hệ thống Lab**. Xác định đường dẫn theo revision và cấu hình Remote đã chọn. Ví dụ, khóa tham chiếu Python lịch sử nằm ở `migration/python-reference/integration/release-set.json` trong cấu trúc được mô tả bởi tài liệu hệ thống Remote. Phiên kiểm thử candidate dùng manifest riêng với artifact được khóa theo mã băm và giữ nguyên các khóa phát hành lịch sử.

Một bộ E2E được chọn phải có ít nhất:

| Artifact | Trường phải khóa |
|---|---|
| `cirnet-interface` | Git revision, package/build hash, selected Contracts SDK digest |
| `cirnet-remote` | Rust binary/package SHA-256, source revision, API/WS contract identity |
| `cirnet-contracts` | Rust crate/source revision, schema fingerprint, SDK artifact digest, RPC descriptor hash |
| `cirnet-hardware` | Rust service digest, selected RPC version, station profile/capability set |
| `cirnet-simulator` | C++ worker executable SHA-256, source revision, private worker protocol version |
| `cirnet-fpga` | Source/RTL revision khi kiểm tra FPGA; Virtual Digital không được đòi board thực |
| System | release-set identity, test profile, OS, toolchain, selected backend and data fixture |

Không dùng `latest`, branch HEAD động, unpinned executable hoặc SDK copy thủ công khi nghiệm thu. Kiểm SHA-256 artifact trên **bytes thực tế**. Digest đảm bảo integrity, không thay thế trust/signature hoặc permission review.

**Bất biến:** source candidate từ sáu Repo không đồng nghĩa selected release; bộ artifact phải được giải quyết trước khi khởi chạy E2E, không đổi mid-run.

### 6.5 Acceptance gates: Virtual Digital E2E

| Gate | Kiểm chứng | Điều kiện PASS |
|---|---|---|
| VD-01 | Contract + SDK compatibility | Browser/Rust DTO/RPC descriptor giống selected versions; canonical fixtures đạt |
| VD-02 | Real process chain | Web → Remote → Hardware → Worker thật → Remote → Web; không mock một boundary được tuyên bố là real |
| VD-03 | Functional parity | Truth tables, timing/event order, 0/1/X, powered/unpowered states theo model được công bố |
| VD-04 | Persistence/capture | Metadata và NCAP bytes đọc lại đúng owner, sample count/timestamp hợp lệ |
| VD-05 | Failure isolation | Bad CRC/frame, wrong digest, crash, timeout, malformed graph trả explicit error; không bịa samples |
| VD-06 | Cancellation/Stop | Timeout, disconnect, Stop dừng đúng worker; owner/lease được release hoặc quarantine đáng tin cậy |
| VD-07 | Concurrency/resource bounds | Bounded worker pool, admission, backpressure, queue overflow, latency và memory được đo |
| VD-08 | Real browser | UI hiển thị đúng Run acknowledgement, tiến trình và kết quả; reconnect không replay command |
| VD-09 | Release provenance | Manifest pin source/artifact/compiler/backend; logs giữ run ID và exact SHA |
| VD-10 | Cross-platform target | Windows integration và Linux Mini PC readiness được phân biệt; production chỉ PASS khi test trên môi trường mục tiêu |

Chỉ báo cáo **Virtual Digital E2E verified** khi cả mười cổng đều đạt cho bộ artifact và môi trường đã nêu. Ghi rõ cổng thất bại, chưa chạy hoặc bị chặn. Kết luận E2E cần bằng chứng từ toàn bộ chuỗi Browser–API–Hardware–Worker.

---

## 7. C++ Simulator và hướng mở rộng Analog

### 7.1 Digital engine mục tiêu

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

Thuật toán: **discrete-event**, không fixed timestep toàn hệ thống.

```text
Input change at t=0
   → enqueue affected device
   → calculate output according to validated model
   → schedule propagation at t + Δ
   → update dependent net and probes
   → stop at time/event/sample limit or cancellation
```

- Deterministic ordering cho các event cùng thời điểm, với model input và seed được lưu để tái lập.
- Phân biệt ideal gate và power-aware device; không dùng một bộ model để tuyên bố tính đúng của mọi IC vật lý.
- Model không hỗ trợ phải trả `UNEXECUTABLE`/`UNAVAILABLE` rõ ràng.
- Giới hạn events, samples, duration, queue depth, CPU/memory và max capture độc lập nhau.
- Thời gian mô phỏng và thời gian đồng hồ thực là hai miền khác nhau; không gọi event timestamp là measured FPGA clock.
- `CNP1/CNP2` là private C++ worker contracts được pin; mọi thay đổi protocol phải có version handshake, compatibility tests và migration rõ ràng.

### 7.2 Duy trì và hoàn thiện C++ Simulator

C++ worker là triển khai Simulator mặc định lâu dài, không có yêu cầu chuyển sang Rust. Thực hiện:

1. Khóa C++ worker theo SHA-256, source revision, protocol version và bộ fixture đã nghiệm thu.
2. Mở rộng C++ worker theo contract hiện có; khi cần breaking change, dùng protocol version mới và adapter tương thích.
3. Chạy regression/parity fixtures cho event order, bounded DC, error/status, stop/cancel, NCAP và resource limits.
4. So sánh C++ worker giữa các candidate/release được pin bằng test ngoại tuyến; **không** chạy nhiều worker cho một thí nghiệm vật lý.
5. Đo latency, memory, throughput và stability dưới cùng profile; benchmark Python chỉ nằm ở verification lane.
6. Giữ C++ làm selected backend trong manifest được phê duyệt; giữ artifact C++ cũ để rollback khi nâng cấp phiên bản.

### 7.3 Virtual Analog và Mixed-Signal

```text
ValidatedGraph
  ├── DIGITAL ── C++ discrete-event engine ──┐
  ├── ANALOG  ── C++ SPICE adapter ───────────┼─→ Unified Result Model
  └── MIXED   ── Explicit A/D & D/A bridges ─┘
```

- `SignalDomain` hỗ trợ từ bây giờ ở schema và UI, **không** tự bật solver capability.
- SPICE-compatible engine (ví dụ `ngspice`) là một **dependency native có version/license/digest riêng**, được C++ Simulator tích hợp qua adapter/API phù hợp và worker cách ly.
- C++ adapter sở hữu netlist generation, model provenance, parameter ranges, resource/time limits, cleanup, parse waveform và error mapping.
- Mixed-signal cần xác định timebase, threshold, interpolation, convergence, synchronization và anti-oscillation policy; không làm cầu nối giả.
- Chỉ đặt `virtualAnalog=true` sau khi có real solver, end-to-end fixtures, exact artifacts và release acceptance.
- Bounded DC educational model hiện có **không đồng nghĩa full analog SPICE**.
- Physical Analog Routing không được tự xuất hiện khi Virtual Analog được hỗ trợ.

---

## 8. Physical Lab: Design vs Observed Electrical Topology

### 8.1 Các lớp dữ liệu

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

`ReferenceTopologyGraph` là sơ đồ tham chiếu của giảng viên; `DetectedTopologyGraph` là quan sát tại breadboard. Các ID và tọa độ tương ứng **không** được giả định giống nhau. Giáo viên đặt IC tại E10/F10, sinh viên đặt tại E35/F35 vẫn hợp lệ về topology nếu điện tương đương.

### 8.2 Breadboard Node Map và quét chủ động

Breadboard 830 có PCB riêng nên mỗi nhóm lỗ dẫn điện cần ánh xạ cố định:

```text
A1,B1,C1,D1,E1 → BreadboardNodeID N001
BreadboardNodeID → Scan Matrix / Sense Front-End → RoutingResource
```

Quy trình scan:

1. Giữ lease hợp lệ và xác nhận `SAFE_STATE` khi nguồn mạch và nguồn kích thí nghiệm đã ngắt, các tuyến ở trạng thái mở mặc định.
2. Xác nhận hệ thống liên động và trạm sẵn sàng, đồng thời xác minh năng lượng dư hoặc từ bên ngoài nằm trong giới hạn an toàn. Giữ nguồn quét tắt nếu chưa xác minh được các điều kiện này.
3. Chỉ quét những điểm được phép với **quyền cấp nguồn quét riêng** có giới hạn điện áp, dòng và thời lượng đã xác minh. Giữ nguồn mạch và nguồn kích thí nghiệm tắt. Ngắt nguồn quét khi quá hạn, có lệnh Stop hoặc có lỗi.
4. Đo với front-end trở kháng cao, cân nhắc leakage, charge và cross-coupling.
5. Lập observed nodes/continuity, pin contacts, signatures, confidence và provenance của phép đo.
6. Dùng instructor profile hoặc signature library **chỉ khi có đủ điều kiện điện an toàn** để gợi ý IC orientation/identity.
7. Ngắt nguồn quét và xác nhận trạng thái an toàn trước khi áp dụng tuyến. Tạo `DetectedTopologyGraph` lưu các nhánh sai, thiếu, chưa nhận dạng hoặc mơ hồ cùng trạm/hồ sơ, thế hệ thực thi, phạm vi và hiệu lực của lần quét.
8. Đưa kết quả sang Safety Verification, Topology Resolution và topology assessment riêng biệt.

**Không tuyên bố:** đo continuity là xác định chắc chắn VCC/GND hoặc pin function của mọi IC; nhiều linh kiện có hành vi phi tuyến, diode bảo vệ, pin ẩn, ngưỡng nguồn và trạng thái phụ thuộc kích thích. Nếu không thể chứng minh safety → `INDETERMINATE` và **BLOCK**.

### 8.3 Component resolution modes

| Mode | Ý nghĩa | Sử dụng |
|---|---|---|
| `AUTO_DISCOVERY` | Suy luận location/identity từ scan, signatures trong phạm vi hỗ trợ | Tùy capability, không tuyệt đối hóa confidence |
| `INSTRUCTOR_PROFILE` | Giảng viên cung cấp package, VCC/GND, pin direction/function, expected IC | Có ích khi giới hạn IC hợp lệ cho bài thực hành |
| `HYBRID` | Instructor cung cấp dự kiến pin semantics, scanner tìm vị trí thực, signature hỗ trợ | **Mặc định khuyến nghị** khi triển khai Physical Digital đầu tiên |

Không resolve được linh kiện → `OPAQUE_PHYSICAL_COMPONENT`, `identity = UNKNOWN`, với những quan sát thực đang có. Không tự chép `ExpectedComponentSet` vào `observed identity`.

### 8.4 Resolve LabEndpoint trên **mạch sinh viên thực tế**

`LabEndpoint` mô tả điểm hệ thống được phép kích/đo:

```text
INPUT_SOURCE | OUTPUT_SENSE | CLOCK | VCC | GND |
GENERATOR | SCOPE | LOGIC_ANALYZER
```

Mỗi endpoint có thể dùng:

- `fixed_node_id`: node vật lý cố định và đã xác minh của **station thực tế**.
- `reference_node_id`: node từ bài mẫu/giảng viên, **phải resolve** tới observed node của sinh viên trước khi route.
- `design_port`: mapping logical trong Virtual; không được trực tiếp dùng làm node thật.
- Legacy `node_id`, nếu còn được hỗ trợ, chỉ mang nghĩa fixed placement đã định nghĩa và có migration note.

**Quy trình resolution bắt buộc:**

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

Invariant an toàn:

1. `reference_node_id` và `observed_node_id` thuộc hai không gian định danh. Ánh xạ cần bằng chứng độc lập dù chuỗi ID trùng nhau.
2. Không dùng tọa độ/nhãn net giống nhau như bằng chứng liên tục điện.
3. Mỗi `ObservedPinBinding` phải trỏ tới observed node tồn tại; pin/connection IDs không được trùng; malformed evidence → `DRIVER_ERROR` và fallback an toàn.
4. Không tự gán pin role hoặc identity chỉ vì bài mẫu yêu cầu IC đó.
5. Nếu **required endpoint** chưa resolve hoặc không route được → `BLOCK_UNEXECUTABLE`.
6. Nếu ambiguity chỉ ảnh hưởng **grading** nhưng fixed endpoints vẫn xác định và safety vẫn chắc chắn → có thể tiếp tục guarded theo policy; không chặn chỉ vì `AMBIGUOUS_TOPOLOGY`.
7. `confidence=0` hoặc evidence thiếu không được âm thầm nâng lên thành 100% hay tạo threshold electrical safety giả.
8. Các observed terminals cùng net phải được xét jointly khi phân bổ resource; không collapse tham lam làm mất route khả thi hoặc che conflict.
9. Topology matching phải bounded (giới hạn đồ thị, mapping, thời gian), trả `AMBIGUOUS` khi cạn budget; không để untrusted input gây tìm kiếm vô hạn.

---

## 9. Physical Compiler chỉ định tuyến **hạ tầng**, không sửa mạch

### 9.1 Điều cấm ở mức architecture

> **Physical Compiler MUST NOT repair, recreate, or replace student IC/jumper wiring using FPGA routing.**

Điều này áp dụng ngay cả khi hệ thống biết sơ đồ đúng của giảng viên. Việc nối lại các net logical trên FPGA để “cứu” mạch sinh viên sẽ khiến physical experiment đo **mạch do phần mềm dựng lại**, không còn đo mạch sinh viên đang cắm.

### 9.2 Đầu vào Physical Compiler

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

Output chỉ được phân bổ:

- Đường **VCC/GND có bảo vệ**, input stimuli, clock, generator, output sense.
- Scope/Logic Analyzer channels, trigger, capture, timing, test vectors.
- MUX/crosspoint/resources cho **kết nối hạ tầng** tới các observed endpoints được phép.
- Runtime interlocks, power policy, protection budget, release path.

Đánh giá an toàn và khả năng thực thi trên đúng kế hoạch bất biến trước khi áp dụng tuyến hoặc yêu cầu cấp nguồn thí nghiệm (§10.4). **Không được** tạo tuyến nối ngõ ra AND tới LED nếu sinh viên chưa nối như vậy. FPGA chỉ đo các điểm có thật, và `FunctionalVerificationResult` có thể báo FAIL/NO_RESPONSE/PARTIAL theo hành vi quan sát được.

### 9.3 Ví dụ cần kiểm thử

```text
Reference:  SW1 → AND1.A, SW2 → AND1.B, AND1.Y → LED1
Observed:   SW1 → AND1.A, SW2 → AND1.B, AND1.Y --X-- LED1

Không được:
  Route AND1.Y → LED1 bằng FPGA để sửa thiếu dây.

Được phép khi an toàn và execute được:
  Route allowed input stimuli vào observed SW endpoints.
  Route sense probe vào observed AND1.Y / LED nodes.
  Run guarded và ghi actual signals.
  Return functional FAIL / NO_RESPONSE, kèm topology diagnostics.
```

Trường hợp khác cần thử: IC sai nhưng safe, thiếu IC, IC opaque, dây short nguy hiểm, endpoint không tìm thấy, hai output tranh chấp, topology mơ hồ chỉ ở nhánh không liên quan, resource exhausted, stale scan/plan, và driver disconnect giữa guarded power-up.

### 9.4 Routing Solver

- Bài toán constrained graph routing cho **infrastructure endpoints**, không cho logical student nets.
- Dữ liệu vào: observed endpoint bindings, protected capabilities, resource graph, voltage/domain/driver constraints, reservation, instrument allocation.
- Thuật toán có thể dùng BFS/Dijkstra/backtracking và conflict checking, miễn deterministic/bounded.
- Phải tránh output contention, incompatible voltage domains, feedback short, bypass protection, route reuse giữa leases.
- Kế hoạch mang `hardware_profile_id`, `scan_epoch`, `lease_id` và `plan_hash`. Driver báo `applied` riêng, kèm bằng chứng khớp kế hoạch và thế hệ thực thi.

---

## 10. Safety, Topology và Functional Verification

### 10.1 Ba lớp kết quả độc lập

| Result | Nội dung | Quyền chặn Run |
|---|---|---|
| `SafetyVerificationResult` | `SAFE / UNSAFE / INDETERMINATE`, short, contention, polarity, current, voltage, protection, stale evidence | **Có**: UNSAFE/INDETERMINATE chặn |
| `TopologyVerificationResult` | `MATCH / MISMATCH / UNKNOWN / AMBIGUOUS`, missing/extra nets/components, orientation, pin map, reference confidence | **Không**: diagnostic và grading, không tự cấp hay tước quyền Run |
| `FunctionalVerificationResult` | `PASS / FAIL / PARTIAL / NO_RESPONSE / NOT_EVALUATED`, observed vs expected truth table/timing/waveform | Chỉ xuất hiện **sau** chạy hoặc là NOT_EVALUATED; không làm pre-Run gate |

### 10.2 Execution Decision policy

```text
BLOCK_SAFETY
BLOCK_UNEXECUTABLE
RUN_NORMAL
RUN_GUARDED
```

| Điều kiện được chứng minh | Decision |
|---|---|
| VCC–GND short, overcurrent, dangerous output contention, protection failure | `BLOCK_SAFETY` |
| `SafetyVerificationResult = INDETERMINATE` | `BLOCK_SAFETY` (fail closed) |
| Missing/unroutable required endpoint, unavailable station/driver, resource conflict | `BLOCK_UNEXECUTABLE` |
| Safe + wrong IC/wiring, nhưng mọi endpoint cần thiết đã route được | `RUN_NORMAL` hoặc `RUN_GUARDED` tùy profile |
| Safe + opaque component + proven executable/protection | `RUN_GUARDED` |
| Safe + topology mismatch chỉ ở nội dung chấm điểm | **Không tự chặn** |
| Digital station nhận analog physical request chưa hỗ trợ | `BLOCK_UNEXECUTABLE` |

**Lưu ý:** `SAFE + WRONG → RUN` là viết tắt của **SAFE + EXECUTABLE + AUTHORIZED + protection verified → RUN**. Không được bỏ qua yêu cầu thực thi, quyền truy cập, trạng thái lease hoặc interlock.

Cả hai chế độ Run đều cần bảo vệ đã xác minh và cấp nguồn có bảo vệ. Trạm thực thi các quyền cấp nguồn quét, nguồn mạch và nguồn kích thí nghiệm riêng, theo điều kiện bật tương ứng.

### 10.3 Electrical Safety Envelope

**Bảo vệ phần cứng thực** trước khi có quyền `physicalDigital=true`:

```text
Supply → eFuse/current limit → current sense → fast cutoff → breadboard
FPGA drive → protected buffer/level interface → current limiting → route → node
```

Các yêu cầu tối thiểu:

- Current-limited supply, sensor calibrated, overcurrent trip và independent fast cutoff.
- Default power OFF, route OPEN, drive high-impedance, no unintended startup pulse.
- Guarded staged power-up với active monitoring và fault latching.
- FPGA heartbeat, watchdog, per-output enable, contention detection, safe reset/stop.
- Hardware profile chứa voltage/current ranges, power domains, max drive, pin/route constraints dựa trên schematic và đo đạc.
- Interlock không phụ thuộc hoàn toàn vào một software boolean từ browser/server.
- Driver không xác minh được electrical safety phải trả `UNAVAILABLE`/`INDETERMINATE`, không trả SAFE giả.

**Thiếu schematic/datasheet/AFE/eFuse/measurement → `TBD_BY_HARDWARE_DESIGN`**, capability physical vẫn bằng false/0. RTL simulation không phải chứng nhận nguồn điện hay thời gian phản ứng fault trên board.

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

Sơ đồ mô tả trình tự điều khiển. Dừng an toàn tiến hành độc lập với việc thu dữ liệu và đánh giá chức năng. Áp dụng và xác minh tuyến khi nguồn mạch và nguồn kích thí nghiệm đã ngắt. Trước khi cấp nguồn, Hardware kiểm tra hiệu lực lần quét, kế hoạch đã duyệt và tuyến đã áp dụng theo lease, trạm/hồ sơ và thế hệ thực thi hiện tại. Thay đổi dây hoặc cấu hình làm mất hiệu lực bằng chứng thì phải dừng an toàn và chuẩn bị lại. Lỗi hoặc mất quyền cấp năng lượng khiến trạm vô hiệu hóa các nguồn liên quan và đưa hệ thống về trạng thái an toàn. Giữ trạm cách ly nếu chưa xác nhận trạng thái đó.

---

## 11. HardwareStation và Capability Discovery

### 11.1 Abstraction chung

```text
HardwareStation
  ├── VirtualHardwareStation
  └── PhysicalHardwareStation
```

Application chỉ gọi service contract chung (tên cụ thể theo canonical Contracts):

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

Không ép VirtualStation tạo scan hoặc electrical safety evidence giả. Cho phép `NotApplicable`/`NOT_RUN` có kiểu dữ liệu rõ ràng.

### 11.2 StationCapabilities là dữ liệu có thẩm quyền từ server/driver

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

Frontend **không** hard-code số channel/slot và không bật nút điều khiển nếu capability bằng 0. `virtualAnalog` không tự true chỉ vì schema đã có field ANALOG. `physicalDigital` không true khi mới có RTL scaffold.

Capability công khai **không** chứa địa chỉ MUX, pin FPGA riêng tư, register, USB endpoint hoặc routing matrix chi tiết.

---

## 12. FPGA, Transport và Instrument Architecture

### 12.1 Phân chia Mini PC và FPGA

**Mini PC/Rust:** API, auth, projects, simulator orchestration, compiler, resolver, safety policy, scheduler, waveform conversion, persistence, telemetry và user-level status.

**FPGA/RTL:** timing-deterministic control, clock, trigger, timestamp, digital IO/route switching, scan coordination, acquisition, DDS and SDRAM/CDC. FPGA không xử lý user/auth/DB/WebSocket.

### 12.2 Development hardware milestones

| Giai đoạn | Hardware | Scope |
|---|---|---|
| Prototype | **EP4CE6** | Safe-state control, protocol skeleton, GPIO/clock, elementary protected digital routing, small captures; **không** tự nhận có SDRAM |
| Instrument platform mục tiêu | **EP4CE10 + external SDRAM** | Logic Analyzer, Scope 2CH, Generator 2CH, SDRAM arbiter, trigger, CDC; cần board/datasheet/pins thực |
| Physical Analog tương lai | Analog switch/relay/crosspoint + dedicated AFE/protection | Không được coi FPGA digital V1 đã có analog routing |

Instrument analog dùng phần cứng ngoài:

```text
Oscilloscope CH1/CH2: Probe → AFE → external ADC → FPGA capture → SDRAM
Generator CH1/CH2:  FPGA DDS → external DAC → filter/output/protection
Logic Analyzer:     Digital inputs → FPGA trigger/sampler → SDRAM
```

Đủ interface trong RTL **không** đồng nghĩa đã có ADC, DAC, sample rate hay channel enabled. Chỉ publish capability sau đo/hiệu chuẩn thực.

### 12.3 SDRAM/CDC/trigger

- Scope capture, Logic Analyzer và Host Readback đi qua **SDRAM Arbiter duy nhất**; không cho ba master điều khiển DRAM độc lập.
- Clock domains tách: System, Acquisition, DDS, SDRAM, USB/host; qua CDC synchronizer/FIFO hoặc handshake đã kiểm chứng.
- Reset release, PLL lock, timing constraints, setup/hold và resource budget cần Quartus/timing report trên target part.
- Overflow/backpressure phải trả lỗi/partial theo contract, không đánh dấu complete với waveform bị truncation im lặng.
- Không ghi max sample rate/channels mà chưa có schematic và đo đạc.

### 12.4 Transport và protocol

```text
FpgaTransport
  ├── SerialTransport      (prototype, when built)
  ├── UsbSpiTransport      (prototype, when built)
  ├── Usb3FifoTransport    (high-throughput target)
  └── EthernetTransport   (future option)
```

Framing concept:

```text
SOF | ProtocolVersion | Command | Sequence | Length | Payload | CRC | EOF
```

Command family: `GET_CAPABILITY`, `SET_CLOCK`, `SET_ROUTE`, `WRITE_OUTPUT`, `CONFIGURE_TRIGGER`, `ARM_CAPTURE`, `START_CAPTURE`, `READ_CAPTURE`, `STOP`, `RESET`, `SAFE_STATE`.

**Bắt buộc:** sequence/generation, timeout, response/status, CRC/integrity, command authorization, version negotiation, replay rejection, fail-safe disconnect. Physical bytes/bit order/CRC polynomial **chỉ được chốt** khi có firmware/board implementation được kiểm chứng; scaffold không thể chứng minh ABI thực.

---

## 13. Browser Workbench và hiệu năng đồ họa

### 13.1 Module frontend

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
| Hardware safety, lease, applied plan | **Server/Hardware authoritative** |

Không cho Pinia thay thế simulator/DB/FPGA. Không cho Three.js tự tạo net do wire visuals giao nhau. `DesignGraph` chỉ thay đổi khi người dùng commit hành động editor; placement ghost không phải linh kiện đã tồn tại.

### 13.2 Asset và UX performance

- GLB → Meshopt → KTX2/Basis → browser cache; asset ownership/provenance được giữ.
- InstancedMesh cho breadboard holes, chân lặp lại; lazy loading/LOD với model chưa cần.
- Không render lại khi graph/camera không đổi; pause khi canvas hidden/zero-size và xử lý WebGL context loss.
- Snapshot Undo/Redo có giới hạn size/count ở giai đoạn đầu; một drag hoàn chỉnh là một entry, không snapshot từng frame.
- Virtual và Physical dùng cùng các panel/waveform bindings; khác nhau qua capabilities, status và measured/provenance labels.

---

## 14. Auth, Authorization, Assignment và quyền dữ liệu

### 14.1 Auth và điều khiển phiên

- Role: `Student`, `Instructor`, `Administrator`; RBAC + ownership là hai kiểm tra riêng.
- Trong phạm vi độc lập, Remote xác thực người dùng Lab cục bộ. Trong phạm vi tích hợp đã chuyển giao, Remote xác minh định danh Platform và áp dụng quyền Lab. Băm mật khẩu nếu có thông tin đăng nhập cục bộ, dùng cookie Secure và HttpOnly cho phiên trình duyệt, kiểm tra Origin/CSRF cho thao tác ghi. Production dùng HTTPS và PostgreSQL.
- Một controller tab/client identity gắn với session/lease để điều khiển Run/Stop; viewer chỉ được xem dữ liệu đã được phép.
- API phải phân biệt request accepted, command running, measurement complete, safe stop acknowledged.
- Reconnect/sync có thể quan sát trạng thái cũ nhưng **không** tự phát lại `Run`/`Apply`/`Create`.
- Quyền chỉnh sửa source code hoặc quyền xem project **không đồng nghĩa quyền export dữ liệu** của người khác. Trước khi thực hiện export mơ hồ, xác định actor, owner/scope, allowed fields và destination.

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

Sinh viên chỉ nhìn thấy đề bài và phản hồi được công bố; các sơ đồ chuẩn, mappings và đáp án bí mật không được lộ qua REST, WebSocket, capture, SDK hoặc error details. Chính sách công bố chỉ đổi **thông tin được phép hiển thị**, không đổi thuật toán an toàn hoặc grading.

### 14.3 Learning evaluation

```text
Safety:   SAFE
Topology: MISMATCH
Function: FAIL
```

Đây là **trạng thái hợp lệ** khi mạch đã được phép chạy an toàn nhưng sinh viên đấu sai. Hệ thống phản hồi tín hiệu thật và gợi ý học tập, không giả mạo output để “đạt đáp án”.

---

## 15. Scheduler và tải 30–50 users

Mục tiêu năng lực:

```text
30–50 concurrent connected users
      !=
30–50 simultaneous physical executions
```

- Virtual sessions có worker pool có giới hạn, workload đo theo CPU/RAM/capture, không chạy CPU-bound engine trực tiếp trong API event loop.
- Breadboard hoặc hạ tầng điện dùng chung đã xác minh có **mặc định một slot vật lý**. Hồ sơ chưa đạt công bố 0 slot. Những người khác có quyền có thể chỉnh sửa, xếp hàng hoặc đọc kết quả trong khi hạ tầng chỉ có một bên điều khiển.
- Admission: FIFO + per-user rate limit + max runtime + lease TTL; có ưu tiên giảng viên nếu policy chấp nhận, nhưng phải công khai và bounded.
- Phải serialize cho cả burst đồng thời; không dựa vào giả định sinh viên ít khi bấm Run trùng thời điểm.
- Disconnect, lease expiry, Stop, crash → attempt safe cleanup; thất bại → quarantine. Tuyệt đối không giao tài nguyên cho user kế tiếp khi chưa xác nhận safe release.
- `physicalExecutionSlots > 1` chỉ khi từng partition được **cách ly điện**, có safety/routing/resource profile riêng và HIL xác nhận; không thay bằng cấu hình số trên JSON.
- Mọi queue, worker và waveform stream đều có backpressure; benchmark mục tiêu 30–50 users phải đo throughput, tail latency và failure recovery.

---

## 16. Capture, waveform và lưu trữ

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

Tùy loại Virtual/Physical, các phần `?` có thể là `NOT_APPLICABLE`/`NOT_EVALUATED`, không suy luận thành SAFE hoặc PASS.

### 16.2 Lưu và truyền waveform

- Capture lớn lưu file/object storage, PostgreSQL chỉ lưu ownership, path/ID, sample metadata, checksum và retention.
- Dùng `WebSocket Binary` hoặc authorized streaming/download cho waveform lớn; không đẩy raw ADC/NCAP thành JSON khổng lồ.
- Header tối thiểu: `capture_id`, `instrument`, `channel`, `sample_rate`, `sample_count`, `timestamp`, `encoding`, `protocol_version`.
- Payload tùy contract: `Int16`, `Uint16`, `Float32`, digital events/RLE; không tự decode theo loại khác nếu version mismatch.
- Có quota, storage retention, access control, signed/authorized reads, resume/resync và cleanup.
- NCAP v1 digital và v2 sampled phải được phân biệt; codec support không đồng nghĩa producer hiện đã phát ra sampled analog capture.

---

## 17. Edge deployment

### 17.1 Mini PC mục tiêu

| Thành phần | Mục tiêu |
|---|---|
| CPU | Intel Xeon W-1370 |
| RAM | 16 GB (tăng sau load test nếu cần) |
| SSD | 256 GB ban đầu; 512 GB–1 TB khi captures và assets tăng |
| OS production | Linux |
| Proxy | Nginx |
| Application | Rust Remote + Rust Hardware, supervised simulator worker |
| DB | PostgreSQL; SQLite cho test/prototype |
| Service manager | systemd |
| Python | Chỉ công cụ test/benchmark ngoài runtime production |

### 17.2 Services

```text
nginx.service
postgresql.service
cirnet-remote.service
cirnet-hardware.service
[supervised cirnet-simulator worker processes]
```

Dùng systemd hardening/permissions, local-only gRPC, health/readiness, startup order và fail-safe service restart. Không expose private Hardware RPC lên WAN. Browser xử lý Three.js trên thiết bị người dùng, không gửi render 3D về Mini PC.

Docker Compose phù hợp development/integration test; không bắt buộc production. Cấu hình endpoint/credentials theo environment/service policy, không commit secret. Database migration/backup và capture consistency là phần của release/rollback.

### 17.3 Observability

Mỗi request/experiment/worker command gắn `correlation_id`, `principal_id` (phần cho phép), `station_id`, `lease_id`, `operation_id`, `plan_hash`, release-set SHA và endpoint/safety generation. Log không để lộ password, token, đáp án giảng viên hoặc waveform riêng tư. Metrics: worker queue, run latency, capture bytes, cutoff/fault events, timeout, quarantine, WS lag, database health và resource utilization.

---

## 18. Release engineering, provenance và tích hợp sáu Repo

### 18.1 Release naming

Quy tắc chung `cirnetlab` cho **các bản phát hành mới**:

```text
Git tag            = <repo_name>-<version>
GitHub Release     = <repo_name>-<version>
Example            = cirnet-remote-0.2.4
```

Ghi rõ `0.2.4` ở trên là **ví dụ định dạng**, không tuyên bố đã phát hành phiên bản đó. Các tag lịch sử khác convention vẫn được bảo toàn.

### 18.2 System release set

Remote sở hữu danh sách artifact được chọn; mỗi candidate/release set cần có:

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

Trường `?` không hiện diện hoặc `UNAVAILABLE` với simulator-only release; không bịa firmware hash khi chỉ có scaffold source.

### 18.3 Quy trình phát hành đa Repo

1. Owner Contracts phát hành schemas/SDK/RPC/fixtures đã kiểm tra.
2. Simulator, Hardware và Interface build theo đúng phiên bản đã khóa, gồm SDK của Interface. Xác minh mã băm của bản build, giao thức worker, khả năng hủy và tính tương đương của lỗi.
3. Remote tạo manifest candidate và khóa tất cả artifact trong phạm vi kiểm thử đã nêu.
4. Interface và Remote chạy kiểm thử trình duyệt thật trên cùng bộ candidate.
5. FPGA nếu có physical release phải có bitstream/board/timing/HIL/safety evidence; không gắn nhãn hardware-supported cho scaffold.
6. Hoàn tất ma trận nghiệm thu cho candidate đã khóa, phân biệt phạm vi kiểm thử Windows và Linux theo từng kết luận. Remote chọn bộ phát hành sau khi các cổng áp dụng đều đạt.
7. Đóng gói artifacts, manifest, migration/rollback notes; phê duyệt riêng trước tag/publish.
8. Rollback chỉ với bộ artifact/DB/capture backup tương thích, dừng admission, xác nhận safe cleanup/quarantine, không tự replay Start.

Không “chọn đúng branch” thay cho chọn đúng artifact. Không nâng cấp SDK/worker độc lập mà không kiểm tra consumer pins và release lock.

---

## 19. Verification strategy và Definition of Done

### 19.1 Các lớp test

| Layer | Owner | Nội dung |
|---|---|---|
| Unit/domain | Mỗi Repo | Typed errors, state machines, bounded resource behavior, validators |
| Contract/conformance | Contracts + consumers | Schema, generated SDK, gRPC descriptors, NCAP, ABI/version compatibility |
| Worker integration | Hardware + Simulator | Real worker process, hash pin, CNP1/CNP2 or successor, Stop, timeout, corrupt frame |
| API/service integration | Remote + Hardware | HTTP/gRPC/DB/WS, ownership/lease, operation identity, capture |
| Browser E2E | Interface + Remote | Real workbench editing, authorization, run, results/reconnect, waveform |
| Physical control simulation | FPGA | RTL tests for reset, watchdog, fault, permit loss, routing default-open |
| Hardware-in-the-loop | Hardware + FPGA | Board-level electrical safety, timing, fault injection, actual transport/ADC/DAC |
| Load & soak | Remote + all | 30–50 connected clients, workers/queues, Stop latency, storage/capture stability |
| Release acceptance | Remote integration owner | Exact selected artifact set, Linux target, migration, rollback and provenance |

### 19.2 Không được đánh đồng loại bằng chứng

```text
RTL simulation PASS   != physical electrical safety PASS
Contract check PASS   != real end-to-end service PASS
Windows candidate PASS != Linux production PASS
Digital/DC model PASS != full analog SPICE PASS
Source HEAD available != published release artifact available
```

Python benchmark scripts chỉ được chạy trong môi trường kiểm thử. Không được mở đường production gọi Python để “lấy dữ liệu test”, không mock FPGA/capture rồi công bố đã kiểm chứng vật lý.

### 19.3 Definition of Done cho một tính năng

- Contract owner và compatibility boundary rõ ràng.
- Code change thuộc đúng repo, không sửa repo khác vô tình.
- Happy/negative tests theo đúng boundary và selected versions.
- Failure/cancel/Stop và security/authorization được kiểm tra.
- Evidence có exact source/artifact/release-set identity, logs/exit code và môi trường.
- Cập nhật tài liệu chuẩn EN/VI và changelog phù hợp, không tạo Markdown dư thừa.
- Trạng thái công khai chỉ là `VERIFIED` tại môi trường đã thật sự thử; phần khác `NOT_RUN`.

---

## 20. Lộ trình phát triển: ưu tiên tối ưu, có cổng nghiệm thu

Các giai đoạn dưới đây **không** yêu cầu tạo đồng thời tất cả abstraction. Bắt đầu với vertical slice có khả năng đo được, sau đó mở rộng theo dependency.

| Phase | Mục tiêu | Owner chính | Exit gate |
|---|---|---|---|
| **A — Hợp đồng nền cho Virtual Digital** | Ổn định graph, bài Lab, kế hoạch ảo, năng lực, RPC, kết quả và NCAP cần cho B, với SDK Rust/TS tương đương trong phạm vi đó. | Contracts + Remote + Interface | Fixture sinh tự động, tương thích cũ/mới và kiểm thử đúng phiên bản đã chọn đều đạt |
| **B — Virtual Digital E2E (Rust services + C++ Simulator)** | Web → Rust Remote → Rust Hardware → Worker → Web với digital gates/LED/clock/NCAP | Tất cả trừ physical FPGA | Gates VD-01…VD-10 trên artifact set pin; không mock boundary đã công bố |
| **C — Mở rộng và tối ưu C++ Simulator** | Mở rộng C++ worker đã dùng ở B, giữ giao thức, tính tương đương và khóa artifact. | Simulator + Hardware | Bằng chứng về chức năng, lỗi, Stop, hiệu năng và quay lui |
| **D — Virtual Instruments** | Logic Analyzer, generator/scope **ảo** với capability chính xác | Simulator + Interface + Contracts | Typed samples, bounded capture, UI and E2E tests |
| **E — Virtual Analog** | C++ SPICE adapter + native solver, analog waveform; mixed bridge khi đủ điều kiện | Simulator + Contracts | Real analog solver fixtures and licensing/provenance; không quảng cáo quá scope |
| **F — Nghiệm thu Edge nhiều người dùng** | Mở rộng cơ chế nhận yêu cầu, lập lịch và kiểm thử Linux ở B lên cấu hình Edge và tải mục tiêu. | Remote + Hardware | Tải 30–50 máy khách kết nối, phục hồi và dừng an toàn ở chế độ ảo |
| **G — FPGA Emulator & EP4CE6** | FPGA protocol emulator, RTL control, small digital resources | FPGA + Hardware | CRC/timeout/Stop/RTL parity; physical power vẫn unavailable nếu chưa có board evidence |
| **H — Physical Safety & Discovery** | Breadboard PCB/Scan/Protected Drive, HYBRID resolver, physical endpoint bindings | FPGA + Hardware + Contracts + Remote | Board HIL chứng minh SAFE/INDETERMINATE, fault cutoff và observed-physical routing |
| **I — Physical Digital Lab** | Protected routing, guarded execution, real IC capture, three verifications | Hardware + FPGA + Remote + Interface | Safe-wrong runs allowed, unsafe blocked, no auto-rewiring; HIL + teaching tests |
| **J — EP4CE10 Instruments & Scale** | SDRAM, Logic Analyzer, Generator 2CH, Scope 2CH, CDC/AFE calibration, multi-user queue | FPGA + Hardware + Remote | Real sample/trigger/timing/electrical evidence, resource and load gates |
| **K — Future Physical Analog** | Analog routing/protection/physical compiler adapters | Hardware + FPGA + Contracts | Chỉ sau analog electrical safety design và measured HIL |

**Mốc nghiệm thu đầu tiên.** Ổn định các hợp đồng cần cho B và hoàn thành luồng Virtual Digital qua C++ worker thật, với bằng chứng ở §6.5 gồm giới hạn tài nguyên và hệ điều hành đích theo từng kết luận. C, D, E và F mở rộng kết quả này theo các cổng riêng. Nhánh vật lý đi từ chuẩn bị giao thức và board (G), qua bảo vệ và nhận diện (H), tới Physical Digital (I) và mở rộng thiết bị đo (J). Virtual Analog (E) có thể tiến hành độc lập với nhánh vật lý. Physical Run cần phần cứng bảo vệ, hợp đồng endpoint và hồ sơ trạm đã xác minh.

### 20.1 Acceptance scenario dạy học quan trọng nhất

Mạch sinh viên khác vị trí giảng viên, cố tình đấu sai một dây, nhưng protection chứng minh an toàn và các endpoint vẫn execute được:

1. Scan nhận đúng observed graph, **không** copy reference wiring.
2. Topology: `MISMATCH`; Safety: `SAFE`.
3. `ExecutionDecision`: `RUN_GUARDED` hoặc `RUN_NORMAL` theo profile.
4. Hardware routing chỉ kết nối FPGA stimuli/sense vào observed endpoints.
5. FPGA đo đúng hành vi vật lý của mạch sai; Functional: `FAIL`, `PARTIAL` hoặc `NO_RESPONSE` tùy đo đạc.
6. UI hiển thị rõ ba lớp Safety/Topology/Function và chỉ thông tin giảng viên cho phép tiết lộ.
7. Stop/timeout → SAFE_STATE; resource release chỉ sau xác nhận.

Đây là acceptance scenario **cốt lõi** cho Lab-First. Nếu hệ thống tự nối lại net đúng bài bằng MUX thì scenario này **FAIL** dù UI hiện PASS.

---

## 21. Rủi ro kiến trúc và biện pháp giảm thiểu

| Rủi ro | Tác động | Biện pháp |
|---|---|---|
| Quét continuity không nhận dạng được IC/pin đủ tin cậy | Sai mapping, nguy cơ cấp điện | HYBRID, evidence confidence, profile limits, `INDETERMINATE` fail closed |
| Reference endpoint nhầm với observed node | Nối sai, thí nghiệm giả | Versioned bindings + tests reference≠student + required-binding gate |
| Compiler tự sửa dây sinh viên | Sai bản chất học tập | Infrastructure-only plan; negative fixtures proving no auto-rewiring |
| Software logic thông báo SAFE khi không có interlock | Nguy hiểm điện | Independent hardware envelope, HIL measured cutoffs, capability unavailable until verified |
| Chỉ test unit hoặc Windows nhưng nhận production PASS | Regression khi triển khai Linux | E2E exact artifacts + Linux target/load acceptance |
| Nâng cấp C++ Simulator làm sai timings/errors | Sai kết quả dạy học | Regression/parity, pinned worker, staged upgrade, explicit rollback |
| Nói Virtual Analog supported khi SPICE chưa hoạt động | Sai kỳ vọng người dùng | `virtualAnalog=false`, `ANALOG_UNAVAILABLE`, real solver test |
| Mismatch SDK/Contracts/worker | Crash hoặc dữ liệu sai | Artifact/digest pin, handshake, conformance matrix, fail startup |
| Multi-user physical contention | Route conflict, nguy hiểm | Lease, one slot, FIFO, interlocks, quarantine on uncertain cleanup |
| Lộ đáp án và waveform của người khác | Sai quyền truy cập | RBAC + ownership + disclosure projections + export authorization |
| FPGA resource/timing overclaim | Instrument không chạy đúng | Part-specific synthesis/timing, SDRAM arbiter, ADC/DAC/AFE bench measurements |
| Tài liệu nhiều bản trùng mâu thuẫn | AI Agent đọc sai chuẩn | Canonical architecture, links, update-existing, source evidence, review duplicate |

---

## 22. Architectural Decision Records (ADR)

### ADR-CIRNET-001 — Lab-First: Safety is hard gate; correctness is not

**Decision:** Chặn khi unsafe/indeterminate hoặc không executable; wrong-but-safe được phép chạy theo protection policy; topology chỉ phục vụ diagnostics/grading; function chỉ đánh giá từ measured behavior sau Run.

**Consequence:** Phải có hardware interlock và `SafetyVerificationResult` riêng, không gộp correctness check vào admission.

### ADR-CIRNET-002 — Rust core services và C++ Simulator

**Decision:** Rust là mặc định cho Backend, Hardware và Contracts runtime; **C++ là mặc định cho Simulator**. TypeScript dành cho Web, HDL cho FPGA, Python chỉ testing/benchmark. C++ worker được pin, giám sát và duy trì như implementation chính thức.

**Consequence:** Nâng cấp C++ Simulator không được đổi public behavior hoặc xóa historical release; cần đo parity/rollback.

### ADR-CIRNET-003 — CircuitGraph vs DetectedTopologyGraph

**Decision:** User DesignGraph là truth của ý định logic, physical observed graph là truth của mạch đang cắm. Cả hai không được biến đổi qua lại mà không có bằng chứng measurement/resolution.

**Consequence:** Frontend không xác nhận kết nối vật lý chỉ bằng tọa độ; Physical Compiler không tự sửa dây.

### ADR-CIRNET-004 — HYBRID resolution mặc định cho Physical Digital đầu tiên

**Decision:** Instructor profile cung cấp pin semantics; scan tìm location thực; signature hỗ trợ khi đã được chứng minh an toàn.

**Consequence:** Cho phép sinh viên đặt IC tự do mà không tuyên bố nhận dạng được mọi IC.

### ADR-CIRNET-005 — Endpoint binding requires observed physical evidence

**Decision:** ID tham chiếu và ID quan sát thuộc hai không gian định danh. Ánh xạ endpoint bắt buộc tới node quan sát thực tế trước khi định tuyến. Bằng chứng an toàn và khả năng thực thi vẫn có thể đầy đủ khi điểm chưa rõ chỉ ảnh hưởng việc chấm bài.

**Consequence:** Compiler dùng `TopologyResolutionResult`, không dùng reference graph để routing shortcut.

### ADR-CIRNET-006 — Infrastructure-only physical routing

**Decision:** FPGA routing chỉ nối **controlled/stimulus/sense/power infrastructure** tới endpoint đã resolve, không sửa nội bộ mạch student.

**Consequence:** Functional result phản ánh đúng sai của dây thật; software-corrected topology bị xem là architecture violation.

### ADR-CIRNET-007 — Contract-driven, pinned cross-repository integration

**Decision:** Contracts sở hữu nguồn chuẩn; Remote sở hữu release set đã pin; mọi E2E/release chọn exact artifacts và test qua real boundaries.

**Consequence:** HEAD của Repo không phải selected system; không sửa generated consumer bindings tại chỗ.

### ADR-CIRNET-008 — Capability flags express measured support

**Decision:** Digital physical, analog virtual, Scope/Generator, FPGA channels và execution slots chỉ bật nếu có implementation + verification phù hợp.

**Consequence:** Interface điều chỉnh theo capabilities, không hiển thị mẫu đo giả hoặc mặc định physical safe.

### ADR-CIRNET-009 — Bounded multi-user physical execution

**Decision:** Nhiều user kết nối/queue; shared fabric physical V1 chỉ một slot, mở rộng khi có partition cách ly điện đã chứng minh.

**Consequence:** Scheduler, lease và quarantine là bắt buộc; không yêu cầu broker sớm.

### ADR-CIRNET-010 — Versionless architecture name, versioned artifacts

**Decision:** Kiến trúc Lab có tên **CIRNET Lab-First**, không kèm phiên bản. Mỗi repository, lược đồ, RPC, giao thức, hồ sơ phần cứng và bộ phát hành có phiên bản độc lập.

**Consequence:** Các tài liệu con dẫn về kiến trúc chuẩn; không tạo nhiều “kiến trúc chính thức” khác nhau theo từng Repo.

---

## 23. Tài liệu chuẩn và quy trình cập nhật xuyên Repo

Cặp Anh–Việt này là [tài liệu chuẩn](../../vi/README.md#kiến-trúc) về thực thi Lab và an toàn điện. [Kiến trúc Ecosystem](cirnetlab-architecture-vi.md) quy định ranh giới tổ chức và tích hợp giữa các miền. Tài liệu của từng chủ sở hữu liên kết tới nguồn có thẩm quyền và ghi revision hợp đồng hoặc artifact đang áp dụng.

- Kiến trúc Lab: tài liệu này và [bản tiếng Anh](cirnet-architecture-lab-first-en.md).
- Implementation/API/system release authority: `cirnet-remote`.
- Canonical schema/SDK/RPC/NCAP: `cirnet-contracts`.
- Browser UI: `cirnet-interface`.
- Runtime station/transport: `cirnet-hardware`.
- Worker/models: `cirnet-simulator`.
- Board/RTL/timing/bench: `cirnet-fpga`.

Khi triển khai thay đổi liên Repo:

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

Ưu tiên cập nhật tài liệu cũ. Không tạo đồng thời `architecture-final.md`, `architecture-v2.md`, `architecture-latest.md` trong cùng một scope; lưu bản cũ vào lịch sử khi thực sự cần, bảo toàn provenance. Các Repo có thể giữ những release/architecture histories đã frozen; việc chuẩn hóa tên không được sửa lịch sử mất tính truy vết.

---

## 24. Điều kiện phê duyệt kiến trúc và vấn đề còn mở

Tài liệu này chuẩn hóa **quyết định thiết kế**, không tự động chứng minh đã hoàn thiện sản phẩm. Các mục sau phải giữ `OPEN / TBD / NOT_RUN` cho đến khi có bằng chứng:

1. **C++ Simulator mặc định:** còn cần nghiệm thu E2E trên bộ artifact được pin, gồm Digital/DC, cancellation và benchmark của release set.
2. **Virtual Analog:** SPICE solver native chưa được công nhận trong selected runtime; model licensing, adaptation và E2E còn mở.
3. **Physical station:** cần board schematic, voltage/current profile, eFuse/current sense/fast cutoff, safe drive và actual hardware validation.
4. **Automatic Topology Discovery:** cần PCB node-map, scan front-end và phép đo thực; discovery của mọi IC là không thực tế.
5. **FPGA transport/bitstream:** cần part/pin assignment, protocol physical ABI, clock/reset constraints, synthesis/timing và HIL.
6. **EP4CE10 instruments:** cần ADC/DAC/AFE/SDRAM selected BOM, CDC, analog accuracy, calibration và measurement.
7. **Linux Mini PC acceptance:** cần tải 30–50 connected users và physical lease/Stop latency trong môi trường mục tiêu.
8. **Exact cross-repo release:** cần một manifest mới pin chính xác các Rust/C++/TS artifacts và tích hợp thật; release lock lịch sử không được tự sửa.

### Điều kiện đóng kiến trúc triển khai

- Architectural decision được phản ánh trong Contracts và mỗi owner Repo.
- Một **Virtual Digital E2E vertical slice** có real browser và real selected processes PASS, artifact pin và rollback đã thử.
- Mọi hardware capability có bằng chứng đúng loại; không có **Safety/Topology/Functional** mixing.
- Rust là mặc định ở Backend/Contracts/Hardware đã nghiệm thu; C++ Simulator là mặc định và phải được pin, kiểm chứng đúng release set.
- Python không còn trong production execution path mới, ngoại trừ lane reference lịch sử chạy cách ly để kiểm chứng.
- Bộ docs EN/VI và release notes không có thông tin mâu thuẫn về khả năng hoặc provenance.

---

## 25. Tóm tắt hành động ưu tiên

Hoàn thành luồng Virtual Digital dựa trên hợp đồng với C++ worker đã khóa theo mã băm trước. Xem §20 về các mốc riêng cho Simulator, Analog, tải và phần cứng. Chỉ bật từng năng lực trong phạm vi đã xác minh.

> **Kết luận:** **CIRNET Lab-First** là kiến trúc xuyên sáu Repository, hướng tới Rust cho các dịch vụ lõi và **C++ mặc định cho Simulator**, hợp đồng thống nhất, virtual-to-physical và thực nghiệm an toàn. Virtual Digital E2E là mốc kiểm chứng đầu tiên. Khi chuyển sang mạch thật, hệ thống chỉ định tuyến **hạ tầng đo/điều khiển** tới các endpoint đã xác nhận trên **mạch sinh viên thật**; **không được sửa lại dây của sinh viên bằng routing FPGA**. Hành vi thực đo mới là căn cứ đánh giá học tập.

---

## Phụ lục A — Tra cứu các quyết định gốc được kế thừa

| Chủ đề trong tài liệu nguồn | Vị trí chuẩn hóa trong tài liệu này |
|---|---|
| Web-first, Lab-first, Safety vs Correctness | §0, §1, §10, ADR-001 |
| DesignGraph / ValidatedGraph / ExecutionPlan | §5 |
| Domain separation, CircuitGraph vs Three.js | §5, §13 |
| Physical placement, active electrical topology scanning | §8 |
| AssignmentDefinition, LabEndpoint, Instructor Profile | §8, §14 |
| Safety/Topology/Functional Verification và Run policy | §10 |
| Physical compiler, routing solver | §9 |
| HardwareStation, capabilities, Hardware Service | §11, §3 |
| FPGA protocol, EP4CE6, EP4CE10, SDRAM, Scope, Generator | §12 |
| Digital/Analog simulation, worker pool | §6, §7 |
| API/WS/capture/DB/auth | §3, §14, §16, §17 |
| 30–50 users / lease / scheduler | §15 |
| Roadmap và bài thí nghiệm trọng điểm | §20 |
| Architectural decisions / ADR | §22 |

## Phụ lục B — Ranh giới xác minh và tài liệu tham chiếu

- **Nguồn nội dung chính:** `cirnet-Remote_Architecture_Lab-First.md` do người dùng cung cấp (bản đề xuất Lab-First trước khi chuẩn hóa toàn Organization).
- **Nguồn kiểm chứng trạng thái Repository:** branch `master` của sáu Repository `cirnetlab` được đối chiếu vào 2026-11-15, đặc biệt `cirnet-remote/docs/en/architecture/lab-first-v3-1.md`, `system.md`, `native-migration-status.md`; các README/architecture của Interface, Contracts, Hardware, Simulator, FPGA.
- **Sự khác biệt có chủ đích so với đề xuất cũ:** bỏ tên sản phẩm cũ; bỏ phiên bản trong tên kiến trúc; chuyển Backend/Contracts/Hardware từ Python sang Rust và giữ C++ làm ngôn ngữ mặc định của Simulator; Virtual Analog chưa VERIFIED; bổ sung observed endpoint resolution và cấm physical compiler sửa student wiring; yêu cầu artifact-pinned real Virtual Digital E2E.
- **Không thay đổi tự động:** Source Repo, CI, bitstream, releases, schemas, SDK artifacts hay deployed Mini PC. Tài liệu xác lập hướng phát triển và tiêu chí nghiệm thu; triển khai đòi hỏi PR/test/release riêng ở từng owner.
