# CIRNET FPGA — Kiến trúc phần cứng, RTL và thực hành kỹ thuật

> **Tên kiến trúc:** CIRNET FPGA Architecture.  
> **Organization:** [`cirnetlab`](https://github.com/orgs/cirnetlab/repositories) · **Repository chủ sở hữu:** [`cirnet-fpga`](https://github.com/cirnetlab/cirnet-fpga).  
> **Ngôn ngữ tài liệu:** Tiếng Việt · **Tệp:** `cirnetlab-FPGA-architecture-vi.md`.  
> **Kiến trúc tổng thể:** `cirnetlab-architecture-en.md` (**CIRNET — STEM & Engineering Learning Ecosystem**).  
> **Đặc tả điện tử chuyên ngành:** `cirnet-architecture-lab-first.md` (**CIRNET Lab-First**).  
> **Ngày đối chiếu nguồn:** 2026-11-15. **Tình trạng:** kiến trúc đích, KHÔNG phải chứng nhận phần cứng/RTL đã được nghiệm thu.

## Mục đích và phạm vi

Đặc tả này cập nhật kiến trúc FPGA cũ vào hệ sinh thái **CIRNET**: FPGA là một năng lực kỹ thuật có thể học, mô phỏng, lập trình, kiểm thử và đo lường thông qua CIRNET Studio/IDE/Blocks/Lab/Learn/Silicon. Đồng thời, phần **FPGA điều khiển phòng thí nghiệm vật lý** phải tiếp tục tuân thủ mọi bất biến điện và an toàn đã được phát triển trong CIRNET Lab-First. Hai ứng dụng FPGA này dùng chung mô hình artifact/provenance nhưng **không dùng chung quyền truy cập điện thực**.

**Ranh giới của tài liệu:** kiến trúc fabric FPGA, thiết bị ngoại vi gắn trực tiếp với FPGA, protected I/O, routing/scan, instruments, capture/memory/transport, RTL verification và giao điểm với các Repo khác. Không tái định nghĩa các backend nghiệp vụ giáo dục, API công khai, rubric hoặc quyền người dùng.

### Bối cảnh thực tế cần tôn trọng

- `cirnet-fpga` trên `master` hiện có **Lab-First RTL scaffold**, với `fpga/experiment-controller/rtl/top.v`, `fpga/instrument/rtl/top.v`, testbench `fpga/simulation/`, verification tooling, lockfiles và tài liệu EN/VI. README gọi đây là **source scaffold 0.2.2 candidate**; **không** tuyên bố có firmware trên board thật.
- `cirnet-hardware` sở hữu **Rust HardwareStation, private transport, Stop, station lifecycle và worker supervision**; `cirnet-remote` sở hữu quyền truy cập, scheduler/lease, experiment và system release selection.
- `cirnet-contracts` sở hữu canonical schemas/RPC/ICD/SDK. **Các tên kế hoạch phía ngoài không được tự đổi trong RTL**; `FpgaExecutionImage` ở tài liệu này chỉ là **cấu trúc nội bộ FPGA đề xuất**.
- Những mục EP4CE6, EP4CE10, SDR SDRAM 64 MiB, Logic Analyzer, Scope 2CH, Generator 2CH là **target/candidate**, không phải capability đã chứng minh. Gói firmware/bitstream thực phải có đúng board revision, timing, pin, electrical protection và Hardware-in-the-Loop (HIL) evidence.
- Python được phép trong **testing, benchmark và parity verification** của hạ tầng CIRNET; production services sử dụng Rust, simulation engine mặc định C++, web dùng TypeScript, FPGA RTL dùng Verilog/SystemVerilog. **Ngôn ngữ chương trình người học** là phạm vi riêng của IDE/Toolchains, có sandbox/target thích hợp.

### Quy tắc bất biến

> **Safety ≠ Correctness.** FPGA thực thi tín hiệu thật trong giới hạn điện đã được chứng minh; không tự kết luận bài làm đúng/sai, không giả dữ liệu đo và **không sửa dây/IC của sinh viên bằng FPGA routing**. Physical Compiler chỉ định tuyến **hạ tầng nguồn/kích/đo có bảo vệ** tới endpoint thật đã resolve từ quan sát mạch vật lý.

> **Build ≠ Verified hardware.** RTL compile/simulation không phải kết quả đo điện, timing closure hoặc chứng nhận thiết bị đo.

| Phân loại | Ý nghĩa áp dụng |
|---|---|
| `TARGET`/`DESIGNED` | Ý tưởng/thiết kế đã xác lập, chưa khẳng định có mã chạy |
| `IMPLEMENTED` | Có RTL/source tương ứng |
| `TESTED` | Có test xác định và evidence gắn source/artifact |
| `VERIFIED` | Đã kiểm chứng tại target/board/môi trường công bố |
| `RELEASED` | Artifact được pin, phát hành và khớp compatibility manifest |
| `UNAVAILABLE`/`BLOCKED` | Năng lực thiếu hoặc không đủ điều kiện chạy an toàn |

### Phạm vi phần cứng tham khảo

| Thành phần | Quy hoạch (chưa phải thông số công bố) |
|---|---|
| FPGA prototype | EP4CE6E22C8N, protected control/scan/drive/sense; không giả định SDRAM ngoài |
| Board Lab instrument | EP4CE10, package chọn sau pin/clock/resource closure |
| SDRAM target | **64 MiB** (= 67.108.864 byte = 512 Mibit), phải xác minh part/BOM thực |
| Instrument | Logic Analyzer; Oscilloscope 2CH; Function Generator 2CH, cần ADC/DAC/AFE ngoài |
| Shared physical lab | Một active electrical execution context trên một shared fabric |
| Physical analog routing | **Không tự suy ra có** từ việc đã có scope/generator ADC/DAC/AFE |

### Cách đọc các tham chiếu

- **[ECOSYSTEM]**: kiến trúc chung `cirnetlab-architecture-en.md`.
- **[LF]**: `cirnet-architecture-lab-first.md`; chuẩn chuyên sâu về observed topology, safe Run, endpoint resolution và không tự sửa student wiring.
- **[CONTRACTS]**: schema/SDK/Hardware RPC/ICD **chọn theo artifact thực** từ `cirnet-contracts`.
- **[F0]**: bản thảo thiết kế FPGA lịch sử được cung cấp trong yêu cầu; dùng làm cơ sở chi tiết kỹ thuật, **không** là implementation authority hoặc wire ABI hiện tại.
- **[H1]–[H5]**: tài liệu nhà sản xuất tham khảo tại phụ lục; không chứng minh board CIRNET đã đạt các mức đó.

Mọi `MUST`/`MUST NOT` là yêu cầu kiến trúc dự kiến cho implementation. Không được ghi nhận là đã thực hiện nếu không có test/bench evidence.

---

# 1. Các bất biến hardware phải bảo toàn

**Kế thừa:** các bất biến của kiến trúc CIRNET Lab-First [LF] và nguồn FPGA nền tảng [F0].

1. `SAFETY != CORRECTNESS`: topology mismatch không phải input của energy-permit logic.
2. `UNKNOWN_COMPONENT` không tự động là `UNSAFE`; nhưng cũng không tự động là `SAFE`.
3. `SafetyVerificationResult.status = UNSAFE` hoặc `INDETERMINATE` không được tạo quyền energize.
4. `BLOCK_SAFETY`, `BLOCK_UNEXECUTABLE`, `RUN_NORMAL`, `RUN_GUARDED` giữ nguyên ý nghĩa. FPGA có quyền veto một thao tác nguy hiểm/không khả thi; không tự nâng `RUN_GUARDED` thành `RUN_NORMAL`.
5. Mọi mạch được chạy, kể cả `RUN_NORMAL`, đều phải qua guarded power-up và runtime protection.
6. Scan trả raw observations; không trả kết luận loại IC chắc chắn chỉ từ một mẫu điện.
7. Một shared breadboard/fabric chỉ có một active execution context; các instruments trong context đó có thể hoạt động đồng thời nếu admission cho phép.
8. Capture không được âm thầm mất sample, đổi sample rate, thay encoding hoặc cắt ngắn dữ liệu rồi báo hoàn chỉnh.
9. Clock loss, reset, mất cấu hình hoặc host loss không được giữ nguồn/drive vô thời hạn.
10. Safe shutdown không được chờ SDRAM, DMA/readback, command completion hoặc một ACK từ host.
11. Không có master nào ngoài memory subsystem phát arbitrary physical SDRAM address trong normal operation.
12. Capability phải phản ánh build, board profile và validation thực tế; số channel mục tiêu không phải bằng chứng chúng đã tồn tại.

---

# 2. Tổng quan sáu plane và ranh giới ngoài FPGA

Tổ chức sáu plane từ kiến trúc FPGA nền tảng, nhưng chúng là **ranh giới trách nhiệm**, không phải sáu tầng nối tuần tự.

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
       | hard inhibit (không cần FPGA clock hoặc USB)
       +--> digital OE, scan source, generator disconnect, rail cutoff

Electrical interfaces:
  FPGA <-> protected level interface <-> endpoint routing <-> DUT
  DUT  -> protected scope AFE -> ADC -> FPGA
  FPGA -> DAC -> reconstruction/output AFE -> protected disconnect -> DUT
```

**Quyết định tối ưu:** DDS cơ bản không nối vào SDRAM. Chỉ một mở rộng AWG được kiểm chứng mới thêm memory-read client. Safety không đi qua shared register bus để cắt nguồn.

## 2.1 Module ownership

| Module | Sở hữu | Không được sở hữu |
|---|---|---|
| Protocol Endpoint | Framing, sequence, command admission ở mức gói | Energy permit hoặc physical pin bypass |
| Execution Controller | Active image, lifecycle, execution token | Full topology matching |
| Safety Core | Permit gating, fault evidence, stop deadlines | Điểm số, expected truth table |
| Route Controller | Shadow/active route và switch sequencing | Tự sửa circuit của sinh viên |
| Scan Engine | Test-source sequencing và observations | IC identity/classification |
| Drive/Sense | Bounded I/O và measurement front-end | Raw uncontrolled FPGA-pin exposure |
| Time/Trigger | Timebase, sample anchors, event distribution | Giả định mọi clock cùng pha |
| Instruments | Sample production hoặc DAC codes | Physical SDRAM addresses |
| Memory Subsystem | Region bounds, arbiter, SDR timing | Quyết định cho phép cấp nguồn |
| Data Plane | Descriptor readback, packetization | Làm chậm hard inhibit |

---

# 3. Contract conformance: cái gì giữ nguyên, cái gì chưa thể xác nhận

## 3.1 Ma trận traceability tại ranh giới FPGA

| ID | Contract / ràng buộc nguồn | Nghĩa vụ hardware của bản này | Mức đối chiếu |
|---|---|---|---|
| CT-01 | `PhysicalExecutionPlan` [LF] | Nhận primitives từ plan đã compile; dùng ảnh nội bộ, không raw graph | Phù hợp định hướng; cần conformance trên [CONTRACTS] được chọn |
| CT-02 | `InstrumentationExecutionPlan` [F0] | Không tự coi là alias của CT-01; cần mapping đã version hóa | OPEN-GATE |
| CT-03 | `LabEndpoint` [LF] | Resource IDs, drive/sense direction, approved node binding | Bảo toàn yêu cầu; cần test với [CONTRACTS] được chọn |
| CT-04 | `SafetyVerificationResult` [LF] | Host approval không vượt hard limits/interlocks | Bảo toàn yêu cầu; cần test với [CONTRACTS] được chọn |
| CT-05 | `TopologyVerificationResult` [LF] | Không tạo topology-correctness gate trong RTL | Bảo toàn yêu cầu; cần test với [CONTRACTS] được chọn |
| CT-06 | `FunctionalVerificationResult` [LF] | Cung cấp measurements và quality; không tự grading | Bảo toàn yêu cầu; cần test với [CONTRACTS] được chọn |
| CT-07 | `ExecutionDecision` [LF] | Giữ bốn quyết định, guarded restrictions chỉ chặt hơn | Bảo toàn yêu cầu; cần test với [CONTRACTS] được chọn |
| CT-08 | Active Electrical Topology Discovery [LF] | Tách observation, bounded continuity, signature; trả evidence | Làm rõ hardware modes |
| CT-09 | `BreadboardNodeID` [LF] | Dùng NodeMap revision cố định theo PCB | Bảo toàn yêu cầu; cần test với [CONTRACTS] được chọn |
| CT-10 | `HardwareStation` [LF] | Các thao tác cấu hình/arm/run/stop/readback có hiệu lực xác định | Mapping semantics ở §5 |
| CT-11 | `GET_CAPABILITY` [LF] | Feature/limit/health snapshot, không chỉ compile flags | Làm rõ payload; ABI chưa chốt |
| CT-12 | `SET_ROUTE` [LF] | Validate -> shadow -> commit dưới energy inhibit | Làm rõ transaction |
| CT-13 | `ARM_CAPTURE` / `START_CAPTURE` [LF] | Arm/acquisition/start/trigger tách rõ; không tự cấp nguồn | Phải đóng ICD |
| CT-14 | `STOP`, `RESET`, `SAFE_STATE` [LF] | Điện an toàn, invalidate quyền cũ, giữ evidence đúng policy | Làm rõ reset classes |
| CT-15 | Frame sequence/length/CRC [LF] | No side effect trước complete validation | Giữ nguyên envelope semantics |
| CT-16 | `StationCapabilities` [LF] | Cung cấp số resource thật và configuration constraints | Không tự đổi tên field ngoài |
| CT-17 | Scope 2CH [LF] | Hai channel acquisition và alignment được đo | Giữ mục tiêu; chưa đo |
| CT-18 | Generator 2CH [LF] | Hai DDS/control channel độc lập; external DAC/AFE | Giữ mục tiêu; chưa đo |
| CT-19 | Shared SDRAM arbiter [LF] | Một memory controller, bounded client service | Giữ và cụ thể hóa |
| CT-20 | Physical slots = 1 [LF] | Một hardware owner; không nhân bản 30–50 contexts | Giữ nguyên |
| CT-21 | Capture result [LF] | Sample rate/count/timestamp/encoding và quality có thể ánh xạ | Schema chưa xác nhận |
| CT-22 | EP4CE6 -> EP4CE10 [LF] | Portable core, target wrappers, protocol semantics ổn định | Giữ nguyên |

## 3.2 Không được đồng nhất ba mức “khớp”

- **Architectural semantics:** có thể đánh giá từ hai Markdown đã cung cấp.
- **Schema / wire ABI compatibility:** cần source Contracts/ICD thực; chưa thể xác nhận.
- **Hardware implementation conformance:** cần RTL simulation, fitter/timing và bench evidence; chưa thể xác nhận.

Không dùng một checklist tài liệu thay cho hai mức sau. Không tuyên bố tương thích ABI hoặc đã đạt hardware closure nếu chưa kiểm chứng bằng artifact và test tương ứng.

---

# 4. Execution image: cấu hình nguyên khối, bất biến sau phê duyệt

## 4.1 Nội dung cần có ở ranh giới hardware

`FpgaExecutionImage` là cấu trúc nội bộ đề xuất. Không yêu cầu FPGA parse JSON/Protobuf hoặc chứa toàn bộ application plan.

| Nhóm | Nội dung logic tối thiểu |
|---|---|
| Identity | image revision, plan reference, link/boot session, execution epoch |
| Binding | hardware-profile revision, NodeMap revision, scan-evidence generation |
| Policy | `RUN_NORMAL` / `RUN_GUARDED`, bounded duration, approved safety envelope |
| Routing | resource IDs, endpoint-to-node bindings, exclusive resource masks |
| Digital drive | allowed channels, direction/mode, initial values, update limits |
| Power/scan | allowed rails/test sources, validated profile IDs, staged timeouts |
| Clock/generator | frequencies/tuning words, phases, waveform IDs, output limits |
| Trigger/capture | arm dependencies, source/consumer masks, pre/post depth, formats |
| Memory | descriptor requests, capacities, workload profile ID, permitted concurrency |
| Integrity | complete-image length, configuration checksum, sealed flag |

Không đặt IC identity, expected truth table hoặc topology-match status vào phép tính `energy_permit`.

## 4.2 Ba mức giới hạn

```text
Effective hardware limits
  = intersection(
      physical protection limits,
      validated board/build envelope,
      approved execution restrictions
    )
```

Runtime request có thể giảm quyền, không được mở rộng giới hạn đã có trong board/build. Hardware-profile ID chỉ chọn một cấu hình đã được validation; nó không phải cơ chế để host tự ghi lại absolute electrical limits tùy ý.

## 4.3 LOAD -> SEAL -> APPLY -> ARM

1. Load vào `SHADOW_IMAGE`; active outputs không đổi.
2. Kiểm tra toàn bộ length, range, resource masks, profile identity, descriptor requests và cross-field constraints.
3. Seal một image revision cụ thể. Sau seal, mọi sửa route/power/direction/capture layout hoặc tham số ngoài tập runtime updates đã phê duyệt làm mất hiệu lực seal và approval tương ứng. Cập nhật nằm trong envelope và atomic-update mode đã được image cho phép tuân theo §4.4, không tự mở rộng quyền.
4. Plan-level approval phải ràng buộc đúng sealed image. Safety kiểm tra trước compile không đủ để phê duyệt route/drive phát sinh sau compile.
5. Apply infrastructure trong trạng thái không cấp năng lượng DUT; route commit thành công không tự enable outputs.
6. Kiểm tra lại live health, evidence validity, resource reservation trước power-up; kiểm tra memory admission trước khi capture thực sự bắt đầu.

Configuration checksum giúp phát hiện nhầm ảnh/ảnh bị sửa; **không phải chữ ký xác thực và không thay thế protection**.

## 4.4 Cho phép cập nhật lúc chạy ở phạm vi hẹp

`WRITE_OUTPUT` chỉ đổi value trên mask đã cấp quyền. Không đổi direction, route ownership, electrical domain hoặc output envelope. Cập nhật nhiều bit cần atomic latch/timestamp boundary nếu image yêu cầu đồng bộ.

Đổi route, power domain, AFE range hoặc memory layout trong lúc chạy bị từ chối ở V1. Đổi các tham số generator chỉ được hỗ trợ nếu có atomic update mode đã công bố; nếu chưa có thì phải stop và prepare lại.

---

# 5. Semantic command contract — không tự đặt một ABI ngoài mới

**Kế thừa:** các operation thuộc hợp đồng Lab-First [LF] (phải đối chiếu `cirnet-contracts` được chọn). **Làm rõ:** completion, side effects và internal decomposition.

| Operation nguồn | Nghĩa vụ hardware | Quy tắc completion |
|---|---|---|
| `GET_CAPABILITY` | Snapshot build/profile/resources/health/version | Không đổi execution state |
| `SET_CLOCK` | Stage cấu hình clock trong allowed envelope | Không tự phát clock |
| `SET_ROUTE` | Stage + validate + apply transaction trong trạng thái được phép | Applied chỉ sau toàn bộ sequencer và required verification thành công |
| `WRITE_OUTPUT` | Latch value/mask hợp lệ vào drive engine | Không vượt channel ownership hoặc permit |
| `CONFIGURE_TRIGGER` | Stage trigger config; latch atomically trước arm | Không tự trigger/run |
| `ARM_CAPTURE` | Reserve buffer, arm local engines, xác nhận acquisition readiness | Không tự bật DUT power hoặc experiment source |
| `START_CAPTURE` | Cho phép acquisition/force-start theo mode đã cấu hình | Phải ghi rõ ICD: không đồng nhất ngầm với `run()` |
| `READ_CAPTURE` | Đọc descriptor hoặc bounded range của dữ liệu đã sealed | Không cấp quyền ghi arbitrary SDRAM |
| `STOP` | Dừng experiment và đưa phần điện về safe theo bounded shutdown | Không đợi host tải hết capture |
| `SAFE_STATE` | Loại bỏ mọi energy permit; scan stimulus cũng OFF | ACK nhận lệnh khác với hoàn tất trạng thái điện |
| `RESET` | Reset class được công bố; mọi quyền chạy cũ bị invalidated | Không auto-resume, không tự clear fault đang tồn tại |
| `configure_generator()` | Cấu hình 2CH từ station abstraction | Binary primitive tương ứng chưa có định nghĩa trong [LF] |
| `run()` | Start deterministic execution sau all-required-ready barrier | Không tự bỏ qua scan/safety/power/admission |

Các tên như `LOAD_ROUTE_SHADOW`, `COMMIT_ROUTE`, `CONFIG_GENERATOR`, `READ_STATUS`, `PREPARE`, `ARM`, `START` trong bản thảo FPGA lịch sử có thể dùng làm **micro-operations nội bộ**. Chỉ expose thành wire commands khi ICD xác lập opcode, payload, version và compatibility.

### Các trường semantics cần đóng trong ICD

Mỗi command cần command identity, sequence, context token nếu có, bounded payload, acceptance states, side effects, timeout, completion status và error class. Một operation bất đồng bộ có thể trả `ACCEPTED` rồi `APPLIED`/`FAILED`; không được trả success cuối cùng ngay khi chỉ mới enqueue.

`SAFE_STATE`/emergency inhibit là idempotent. Các command làm phát xung, single-step hoặc start phải có duplicate suppression; retry cùng sequence không được phát thêm một lần.

---

# 6. Execution lifecycle và instrument lifecycle tách nhau

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

`SCAN_ACTIVE` không phải `SAFE_IDLE`: continuity scan có giới hạn vẫn đưa năng lượng vào DUT. `SCAN_READY` nếu cần status chỉ nghĩa là observations đã sẵn sàng, không nghĩa là topology đúng hay safety đã PASS.

Không bắt một execution phải đi qua `CAPTURING` như trạng thái độc quyền sau `RUNNING`. Capture có thể hoạt động trước, trong và sau start để thu pre-trigger hoặc transient power-up.

## 6.2 Capture FSM, riêng cho từng instrument

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

`ARMED/PRETRIGGER` được phép ghi dữ liệu trước trigger nếu mode yêu cầu. Vì vậy bandwidth reservation phải có hiệu lực từ thời điểm bắt đầu pre-trigger acquisition, không đợi experiment `START`.

## 6.3 Barrier đồng bộ

Start chỉ hợp lệ khi route, power, required capture engines, generator configuration và trigger consumers đều xác nhận ready. Mọi ACK qua CDC phải mang đúng generation của lần arm đó. Timeout của một consumer làm abort lần arm, không khởi chạy “một phần” một cách im lặng.

Một `run()` không sử dụng capture vẫn có thể tồn tại nếu profile cho phép; internal execution arm không buộc host giả tạo một `ARM_CAPTURE`.

## 6.4 Mapping state ra ngoài

State FPGA là chi tiết implementation. Khi đưa status qua hardware boundary phải có mapping rõ tới lifecycle đã có trong Contracts; không thay enum phía ngoài bằng enum RTL một cách ngầm định.

---

# 7. Safety island và cây energy permits

## 7.1 Hai lớp phản ứng độc lập

```text
Fault/current/voltage/comparator/supervisor
         |
         +--> EXTERNAL LATCHED INHIBIT --> physical enables OFF
         |
         +--> FPGA synchronized status --> fault record / lifecycle cleanup
```

External safety island phải xử lý được FPGA clock stopped, FPGA unconfigured, brownout và stuck control. FPGA còn hoạt động thì ghi nhận và điều phối cleanup; không phải điều kiện để đường inhibit phần điện có hiệu lực.

Watchdog trong FPGA dùng cùng clock đã ngừng không thể tự đếm tới timeout. Vì vậy cần external watchdog/supervisor có timebase độc lập hoặc cơ chế phần cứng tương đương đã chứng minh cắt được khi FPGA clock mất. Đây là **yêu cầu bổ sung**, không phải khẳng định bo hiện có đã trang bị.

## 7.2 Tách từng loại permit, tránh phụ thuộc vòng

Không dùng một `global_drive_enable` duy nhất cho cả scan, rail power và experiment output.

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

`rail_enable_permit` không phụ thuộc `power_stage_ready` của chính rail chưa bật. `scan_source_permit` không phụ thuộc approval của execution plan chỉ có thể tạo sau scan. Mọi permit thực tế còn phải qua external inhibit; công thức trên là semantics, không phải RTL combinational đa-clock để nối trực tiếp.

## 7.3 Channel-level enforcement

Mỗi source channel có requested enable, ownership, route-ready, allowed mode và envelope. Hiệu lực thực là giao của các điều kiện đó với permit của source class tương ứng. Một bit enable ghi ở register không thể vượt logic này.

Board profile phải có hard upper bounds cho current/voltage/duration/frequency phù hợp phần cứng. Không nhận giá trị tùy ý từ host rồi giả định nó đã an toàn.

## 7.4 An toàn của unknown component

Chỉ được chạy guarded khi **supply, các source/sense paths, endpoint binding và runtime protection đều đủ để chứng minh electrical envelope cần thiết**. Không cần biết tên IC để đánh giá mọi điều kiện đó; nhưng không được dùng “có eFuse” làm bằng chứng an toàn cho mọi loại IC.

Unknown VCC/GND/polarity, nguồn ngoài chưa cách ly, injection-current path chưa kiểm soát hoặc pin-domain không được chứng minh phải giữ safety indeterminate. Đây là hạn chế điện thực, không phải topology-correctness gate. [LF]

---

# 8. Safety staging và freshness của evidence

## 8.1 Trình tự hardware đúng

```text
SAFE electrical state
  -> infrastructure self-check + passive voltage observation
  -> bounded continuity scan theo scan permit riêng
  -> raw observations + validity/coverage/generation
  -> [boundary: candidate route/drive/power configuration]
  -> freeze candidate execution image
  -> plan-level safety approval gắn đúng image revision
  -> hardware limits/resource checks
  -> de-energized route apply + required route verification
  -> guarded power-up + runtime qualification
  -> readiness barrier -> execution
```

Mọi sửa đổi phát sinh sau approval phải được kiểm tra lại hoặc bị từ chối. Safety staging ở đây không thay `ExecutionDecision` contract; nó xác định các điều kiện phần cứng không thể bỏ qua trước energize.

## 8.2 Passive pre-check không chứng minh mọi short

Quan sát 0 V ở hai node khi không có stimulus không đủ kết luận chúng nối tắt hay đang tách rời. Pre-check chỉ báo những điều thực sự đo được: external voltage, interlock status, rail feedback, source disabled, front-end readiness.

Kiểm tra continuity/impedance cần test excitation đã giới hạn, dù điện áp hoặc dòng rất nhỏ. Không gọi tất cả các phép đo đó là “không đưa năng lượng”.

## 8.3 Evidence binding

Một scan result phải gắn ít nhất với NodeMap/profile revision, scan generation, power condition, coverage mask, completion/abort status và timestamp validity. Image dùng evidence của revision khác bị từ chối.

Wiring có thể thay đổi sau scan. Không được hứa rằng token phần mềm phát hiện được mọi thao tác cắm/rút dây. V1 cần một trong các điều kiện vận hành phần cứng được validation: interlocked access, board-change detection phù hợp, hoặc procedure buộc re-scan trước mỗi lần energize cùng runtime protection cho thay đổi ngoài khả năng phát hiện.

Time-to-live của evidence chỉ là giới hạn độ cũ, **không phải cảm biến thay đổi topology**. Khi có sự kiện thay đổi board/interlock hoặc nghi ngờ evidence stale, inhibit và scan lại.

---

# 9. Fault, STOP, SAFE_STATE và recovery

## 9.1 Two-stage shutdown, không phụ thuộc dữ liệu

**Stage 1 — energy inhibit:** disable digital/clock drive, scan stimulus, generator output stage; yêu cầu rail cutoff bằng đường phần cứng. Latch nguyên nhân fault. Không đợi request bus hoặc FIFO drain.

**Stage 2 — isolation và cleanup:** sau khi nguồn đã ở mức cho phép switching, route controller đưa hạ tầng về default-open; capture ngừng nhận mẫu, seal partial khi có thể; giữ fault/status evidence.

Memory hỏng hoặc clock acquisition mất không được cản Stage 1 hay route isolation. Không xếp “đóng descriptor xong” làm tiền đề mở route.

## 9.2 Điện áp dư và nguồn ngoài

`power_enable = 0` không chứng minh DUT rail đã về 0 V. Cần discharge policy, feedback hoặc một upper-bound discharge interval có validation. Khi còn nguồn ngoài không kiểm soát được, trạng thái là inhibited/fault, không báo `SAFE_COMPLETE` giả.

Đường emergency isolation phải phù hợp việc ngắt trong điều kiện fault thực tế. Khi năng lượng không giảm trong deadline, giữ fault latched và hard inhibit; không tiếp tục make route mới. Protective earth/reference ground không bị tùy tiện đưa qua sequence “mở tất cả switch”.

## 9.3 STOP bình thường

STOP bắt đầu bounded quiescence: ngừng clock/generator ở boundary hợp lệ nếu đạt deadline, disable normal drive, remove rail permit, chờ electrical-safe criterion rồi isolate routing. Capture drain chạy độc lập trong thời hạn được cấp.

Nếu hết quiescence deadline hoặc protection fault xuất hiện, chuyển ngay sang hard inhibit. Fault có thể cắt ngắn một pulse; đó là hành vi an toàn cần ghi nhận, không cố giữ waveform đẹp trong khi có nguy hiểm.

## 9.4 Reset classes

| Loại | Hiệu lực điện | Hiệu lực dữ liệu/context |
|---|---|---|
| Execution abort / `SAFE_STATE` | Mọi experiment/scan source OFF; route cleanup | Invalidate run token; giữ sealed captures nếu memory khỏe |
| Controller soft reset | Inhibit trước, reset FSM/command state | Không tái dùng execution epoch; policy descriptor được công bố |
| Full FPGA configuration/reset | External hardware giữ enables OFF | Close link, invalidate descriptors không được bảo đảm giữ, reinitialize memory |
| Board power loss | Fail-safe bằng phần cứng ngoài FPGA | Không giả định SDRAM retention hay timestamp continuity |

`RESET` không tự clear physical fault latch đang active. Recovery yêu cầu fault input hết, live health ổn định, evidence/plan mới khi cần và explicit recovery. Không auto-restart sau power recovery, PLL relock hoặc reconnect.

## 9.5 Fault evidence

Giữ first-fault code, subsequent-fault mask, execution identity, route/image revision, requested/effective enables và timestamp-quality flag trong register/on-chip memory nhỏ. Nếu timebase không còn hợp lệ, báo `timestamp_valid = false`; không bịa một thời điểm chính xác.

---

# 10. Watchdog và execution identity

## 10.1 Watchdog nhiều mục đích

- **Host-liveness watchdog:** chỉ refresh bởi heartbeat hợp lệ của active link/context; không bởi bất kỳ packet nào.
- **Execution duration limiter:** thời gian energize tối đa không thể gia hạn vô hạn chỉ nhờ heartbeat.
- **Scan dwell limiter:** giới hạn thời gian từng stimulus và toàn scan, độc lập việc host đọc observations.
- **External FPGA-liveness watchdog:** phát hiện clock/control ngừng theo fault model đã validation; trực tiếp inhibit phần điện. Tín hiệu service phải phản ánh điều kiện/progress cần giám sát, không là bộ chia tự chạy vẫn refresh trong khi control đã treo. Watchdog không chứng minh phát hiện mọi lỗi logic tùy ý.

Replay heartbeat/sequence cũ không được làm mới timeout. Mất host khi đang scan cũng phải tắt stimulus; scan không phải ngoại lệ của watchdog.

## 10.2 Identity nhiều tầng

Mọi side-effect command cần kiểm tra tuple tương đương:

```text
(link_boot_session, execution_epoch, image_revision, command_sequence)
```

`execution_epoch` tăng/đổi khi prepare execution mới, STOP kết thúc execution, explicit abort, fault hoặc plan invalidation. Counter không wrap rồi tái dùng trong một phiên; gần wrap thì inhibit và tạo phiên mới.

Counter trong FPGA bị reset về cùng giá trị sau hard reset **không đủ** chống stale command. Full reset phải đóng transport session, loại dữ liệu cũ trong bridge/FIFOs/parser và chỉ nhận side-effect commands sau fresh-session binding.

## 10.3 Boot freshness là closure gate bắt buộc

Board integration phải chọn và chứng minh một cơ chế:

- Boot generation từ supervisor/nguồn lưu giữ độc lập, không lặp lại trong phạm vi replay cần chặn; hoặc
- Verified reset-and-drain của toàn transport boundary, kết hợp session identity mới và handshake không chấp nhận byte/transaction cũ trước binding.

Không giả định EP4CE6/EP4CE10 tự có persistent monotonic counter hoặc entropy source sẵn. Nếu bridge không hỗ trợ purge/binding đủ mạnh, không công bố reset-replay closure; giữ outputs inhibited cho đến khi link được thiết lập sạch.

Đây là chống thao tác trễ/lặp ngoài ý muốn, không phải thiết kế một trust/authentication system trong FPGA.

## 10.4 At-most-once side effects

Baseline chỉ cho một outstanding mutating transaction, cộng một đường inhibit ưu tiên. Cache response gần nhất hoặc replay window hữu hạn theo ICD. Duplicate đã thực hiện trả lại status, không chạy lại pulse/START/COMMIT.

`STOP` theo execution phải scoped để packet dừng cũ không dừng execution mới. Global `SAFE_STATE` hợp lệ vẫn là lệnh giảm năng lượng idempotent, nhưng không được bỏ qua framing/integrity validation để tìm một chuỗi byte giống opcode trong bulk payload.

---

# 11. Routing fabric: chỉ kết nối hạ tầng tới mạch thật

## 11.1 Phân loại physical paths

```text
R_SCAN   : bounded stimulus / high-impedance scan sense
R_DIGITAL: protected source, clock, LA/digital sense endpoints
R_SCOPE  : approved measurement path tới scope AFE
R_GEN    : approved generator output path
R_POWER  : current-limited supply distribution, nếu board hỗ trợ
```

Các path có thể dùng một phần cùng switch hardware, nhưng phải có resource exclusivity rõ. Power không tự đi qua digital signal MUX. Scope/generator analog path không tự đi qua switch số chưa được characterization.

Không bắt buộc lắp full N×N crosspoint giữa mọi breadboard node. **Baseline ưu tiên endpoint-to-node routing** và banked scan selection, giữ wiring nội bộ của sinh viên là wiring thật. Đây là lựa chọn hardware đề xuất để giảm switch count/pins và tránh sửa bài làm.

Nếu authority hiện hành có một chế độ circuit-assembly khác cho phép fabric tổng hợp net, chế độ đó phải được xác lập riêng; không tự suy diễn hoặc bật trong Lab-First student-wiring mode này.

## 11.2 Resource table

Mỗi route resource có ID, electrical class, allowed directions, exclusive group, supported endpoints/nodes, on/off bounds, voltage/current envelope, power-off behavior và verification method. FPGA cần lookup/mask checks nhỏ; không chạy routing solver tổng quát.

Một sense bus thụ động không được đóng nhiều switch vào các net khác nhau rồi vô tình nối tắt chúng. “Sense fan-out” chỉ hợp lệ khi có electrically isolated buffered inputs hoặc cùng net đã được xác nhận. Một source fan-out cũng chỉ hợp lệ nếu image phê duyệt các kết nối điện mà nó tạo ra.

## 11.3 Node mapping

`BreadboardNodeID` định danh electrical group trên PCB, không định danh từng lỗ nhựa độc lập. NodeMap phải mô tả rails có bị chia đoạn, các đường cố định và endpoint reserved. Không hard-code giả định mọi breadboard “830 lỗ” có topology giống nhau. [LF]

---

# 12. Route transaction: atomic về quyền nhìn thấy, không giả định vật lý tức thời

## 12.1 Một trình tự duy nhất cho V1

```text
Validate complete shadow route + ownership
  -> assert ALL affected-source inhibits
  -> remove DUT rail permit theo policy V1
  -> confirm electrical condition permits switching
  -> OPEN old switch paths
  -> wait validated break interval
  -> program new switch configuration
  -> wait settle interval
  -> readback/electrical verification nếu yêu cầu
  -> publish active route revision
  -> keep normal outputs OFF
```

V1 không live-reroute. Tắt FPGA output chưa đủ khi IC thật đang được cấp nguồn và tự drive output. Vì vậy route-changing transaction phải tính cả DUT rails, generator và mọi nguồn ngoài.

## 12.2 Ý nghĩa “atomic commit”

Shadow table được xuất bản thành một active revision duy nhất sau completion. Switch ngoài FPGA có thể cập nhật tuần tự; tính an toàn đạt bằng isolation và sequencing, không phải lời hứa tất cả tiếp điểm đổi cùng một clock.

Checksum/configuration readback chỉ chứng minh register của switch đã nhận dữ liệu; nó không tự chứng minh tiếp điểm điện không stuck-on. Required verification phụ thuộc fault model của route class. Không có readback thì capability phải nói rõ và dùng kiểm chứng khác khi safety cần.

## 12.3 Failure handling

Timeout, incomplete table, illegal simultaneous closures hoặc verification failure không tạo active route mới. Inhibit được giữ và route revision đánh dấu invalid. Không tự khôi phục drive cũ sau commit thất bại.

Simple invalid/stale request đã bị reject trước side effect không cần phá active route hợp lệ. Đây là thay đổi có chủ đích so với tài liệu FPGA lịch sử [F0]; xem fault taxonomy ở §27. Lỗi làm trạng thái vật lý không còn đáng tin thì vẫn hard inhibit.

---

# 13. Scan architecture: ba chế độ điện rõ ràng

## 13.1 Mode contract

| Mode nội bộ đề xuất | Có test energy? | Điều kiện | Kết quả |
|---|---|---|---|
| `PASSIVE_OBSERVE` | Không chủ động inject | Các input paths có rating/protection phù hợp | Điện áp/logic hiện có, external-source evidence |
| `LIMITED_ENERGY_CONTINUITY` | Có, giới hạn V/I/dwell/charge | Scan envelope đã validation, normal sources OFF | Continuity/impedance observations, không auto-identity |
| `ACTIVE_SIGNATURE_SCAN` | Có power/vector có kiểm soát | Pin/power mapping đủ căn cứ + scan-specific approval | Raw response của supported test profile |

Tên “Active Electrical Topology Discovery” của [LF] vẫn là tên phương pháp tổng thể. `PASSIVE_SCAN` của tài liệu FPGA lịch sử mô tả cả stimulus; do đó không dùng tên này để suy ra không có năng lượng. Legacy mapping sang mode mới chỉ thực hiện khi ICD xác nhận semantics.

`ACTIVE_SIGNATURE_SCAN` không bắt buộc thành công trước mọi Run. Unknown component có thể không đủ điều kiện signature scan nhưng vẫn chạy guarded khi các điều kiện điện và endpoint route được chứng minh độc lập.

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

Không giữ stimulus active khi Observation FIFO đầy hoặc host ngừng đọc. Sequencer phải có hành vi pause an toàn: stimulus OFF trước khi chờ; nếu không thể pause tại boundary hợp lệ thì abort và đánh dấu incomplete.

## 13.3 Observation record

Các trường logic cần có: scan ID/generation, step index, stimulus node/mode/profile, sense-node range/bitmap, raw ADC/logic result, measurement range, power condition, sample timing, saturation/fault/quality flags và completion coverage.

Không gửi một ma trận Boolean “connected/not-connected” như bằng chứng tuyệt đối nếu front-end chỉ đo điện áp qua linh kiện bán dẫn. IC protection structures, leakage và đường song song có thể làm observation không tương đương một jumper kim loại; kết luận phải dựa vào resolution phía ngoài và điều kiện test được ghi rõ. Quy định power-off/injection của switch phải được kiểm tra theo từng part. [H2]

## 13.4 Tối ưu scan theo số node, không theo số lỗ

Gọi `N` là số electrical nodes và `P` là số sense nodes có thể đo thực sự song song. Full stimulus-by-node scan có khoảng `N × ceil(N/P)` lượt sense-bank, trước các tối ưu hợp lệ. Đây là mô hình chi phí, không phải cam kết scan mọi IC.

```text
T_scan ≈ sum_over_steps(
    t_select + t_stimulus_settle + t_measure + t_release
)
```

Dùng banked front-end, scan masks, calibration baseline và FIFO streaming để giảm pin/memory. Không cần giữ full N×N matrix trong M9K. Bỏ qua các step chỉ khi test profile cho phép, và luôn báo coverage thực.

---

# 14. Voltage-domain boundary, nguồn và bảo vệ

## 14.1 Mọi đường tới DUT đều phải được xét

Không chỉ digital output. Scan source, high-impedance sense, LA input, scope input, generator output, trigger input và supply paths đều là ranh giới điện.

FPGA GPIO không đồng nghĩa input chịu 5 V hay arbitrary DUT voltage. Các bảng thiết bị Cyclone IV và operating limits của part thực phải là cơ sở; dùng external protection/translation phù hợp, không dựa vào một điện trở nối tiếp đơn lẻ như giải pháp phổ quát. [H1, H5]

## 14.2 Disabled không nhất thiết isolated

Switch/buffer unpowered có thể bị back-power qua đường tín hiệu; powered-off protection là đặc tính cần kiểm tra, không được suy từ chân OE đã OFF. Các đặc tính fail-safe logic và signal-path power-off isolation cũng khác nhau. TI mô tả trực tiếp nguy cơ back-power qua ESD diode nếu thiếu bảo vệ. [H2]

Phải review cả chiều DUT -> FPGA và FPGA/instrument -> DUT khi một rail mất nguồn. Thử nghiệm cả nguồn ngoài tại endpoint, power sequencing và brownout.

## 14.3 Electrical envelope cần mô tả

| Nhóm | Tham số phải xác lập từ part/bench |
|---|---|
| Power | Voltage range, current-limit tolerance, cutoff response, capacitance/energy, discharge, reverse-current behavior |
| Digital source | VOH/VOL under load, source/sink limits, short/contention duration, OE off behavior |
| Digital sense | Thresholds, hysteresis nếu có, input range, leakage, off-state behavior |
| Routing | Ron, capacitance, off leakage/isolation, charge injection, switch transient, power-off limits |
| Scan | Max voltage/current, dwell, injected charge/energy, measurement uncertainty |
| Scope | Input rating, loading, coupling/range, overrange behavior, common reference |
| Generator | Amplitude/offset/load envelope, source impedance, current limit, DC output at fault |

Mức bảo vệ không được mô tả chỉ bằng giá trị nominal. Phải dùng corner/tolerance và fault response phù hợp đường yếu nhất. Current-limited rail không thay thế giới hạn injection từng pin hoặc kiểm tra polarity.

---

# 15. Protected digital drive và digital sense

## 15.1 Drive engine

Mỗi channel gồm value register, allowed-direction/mode bits, enable-request, ownership và effective-enable gate. Value nên được nạp khi OE đang OFF để tránh transient do enable trước data.

Push-pull là mode rõ ràng. Open-drain chỉ được công bố nếu output stage thực hỗ trợ; khi đó logic “1” nghĩa release và pull-up phải nằm trong electrical envelope. Không giả lập open-drain bằng cơ chế làm pin FPGA tiếp xúc điện áp ngoài rating.

Scan, clock, generator và digital stimulus không cùng sở hữu một source endpoint. Nếu cần chuyển source class thì stop/inhibit và apply image mới.

## 15.2 Hai đường sense khác nhau

```text
Protected digital inputs
     +-> control/status synchronizer -> snapshot / status FSM
     +-> measurement capture frontend -> LA / trigger / event path
```

Không đặt debounce/glitch filtering của status vào raw measurement path một cách mặc định. Mọi filter phục vụ measurement phải cấu hình được và ghi trong descriptor.

Synchronizer của một bit status không phải giải pháp đảm bảo tính nguyên tử cho bus nhiều bit. Một snapshot của asynchronous DUT bus có thể phản ánh skew/transition thực; không quảng bá như coherent bus transaction trừ khi có handshake/clock synchronous mode thích hợp.

## 15.3 Điều kiện capture hợp lệ

Capability phải công bố capture clock/source, input setup/hold hoặc sampling uncertainty thích hợp, detectable pulse-width envelope và channel skew đã đo. Không dùng tần số fabric danh nghĩa làm “logic bandwidth” của toàn đường đo.

---

# 16. Experiment clock và deterministic digital timing

Nguồn experiment clock dùng output register được clock bởi timebase có constraint; divider/phase accumulator/pulse scheduler tạo enables hoặc output transitions. Không tạo nhiều unconstrained fabric clocks để clock các FSM bên trong. [F0]

Cần hỗ trợ tập feature thực được validation: programmable period, duty quantization, finite cycle count, single-step, initial level, deterministic start và stop policy.

`SET_CLOCK` chỉ cấu hình. Clock chỉ phát khi image cho phép, source ownership đúng và execution đã qua barrier.

Normal stop có thể hoàn tất pulse ở boundary phù hợp trong deadline. Emergency inhibit ưu tiên cắt năng lượng ngay, chấp nhận truncated pulse và đặt quality/event flag. Không bắt external safety path chờ một cạnh clock có thể không bao giờ đến.

Tần số, duty accuracy và jitter từ divider/NCO phải được khai báo theo timebase/profile. Từ “deterministic” nghĩa hành vi theo luật và clock xác định; không nghĩa zero jitter hoặc bất kỳ tần số thực nào cũng tạo chính xác tuyệt đối.

---

# 17. Clock, reset và CDC architecture

## 17.1 Clock plan theo resource thật

Các domain khái niệm:

| Domain | Công việc | Chiến lược |
|---|---|---|
| `control_clk` | Protocol/control/safety housekeeping/scan control | Clock ổn định; clock enable cho xử lý chậm |
| `capture_clk` | LA và/hoặc scope ingress | Theo clocking ADC/measurement đã chọn |
| `dac_clk` | DDS/output sample update | Có thể dùng chung nguồn với capture khi hợp lệ |
| `sdram_clk` | SDRAM controller/arbiter | Theo timing memory và PCB |
| `transport_clk` | FIFO bridge | Theo clock external bridge |
| External supervisor clock | FPGA-liveness/cutoff | Không phụ thuộc FPGA PLL |

Không mặc định mỗi domain tiêu thụ một PLL riêng. EP4CE6/EP4CE10 có hai general-purpose PLL theo [H1]. Dùng chung source/PLL outputs khi timing cho phép; chỉ tách domain khi ngoại vi thực sự yêu cầu.

## 17.2 CDC primitives

- Single-bit level ổn định: synchronizer đã timing review.
- Pulse/event: toggle/request-ack hoặc event FIFO, có điều kiện không mất event.
- Multi-bit configuration: stable mailbox + handshake, latch atomic ở domain nhận.
- Sample stream: asynchronous FIFO với local full/empty.
- Counter/timestamp: snapshot handshake hoặc Gray/event mapping đã constraint, không synchronize từng bit binary độc lập.

Gray coding không loại bỏ nhu cầu timing/skew constraints. Vendor documentation nêu các multi-bit Gray CDC paths phải giữ thuộc tính chuyển một bit tại sampling boundary. [H3]

## 17.3 Reset domain crossing

Asynchronous assertion khi protection cần; synchronous deassertion theo từng clock domain. Domain chỉ ready khi clock hợp lệ, reset đã release an toàn và local state/FIFO initialized.

Một phía async FIFO reset riêng có thể làm pointer/occupancy không nhất quán. Cần coordinated reset protocol hoặc flush/generation invalidation ở cả hai phía trước khi dùng lại. Không phát `ready` chỉ dựa vào PLL `locked` chưa được qualification.

## 17.4 Clock loss

Loss of required clock trong khi energized -> external/FPGA inhibit theo detection path đã validation; capture partial/invalid tùy phần dữ liệu còn tin cậy. SDRAM clock loss làm data retention không được đảm bảo nếu refresh dừng; không báo capture cũ valid chỉ vì các counter control vẫn còn.

---

# 18. Timebase, trigger và tính trung thực của thời gian đo

## 18.1 Global epoch không phải một bus counter nối khắp chip

Giữ một global timebase 64-bit cho event correlation. Raw captures dùng local sample counter và các anchor mapping sang timebase; không đính 64-bit timestamp cho từng ADC sample thông thường.

```text
SampleDomainAnchor:
  timebase_generation
  source_clock_id
  source_sample_counter
  reference_timestamp
  sample_rate_numerator / denominator
  alignment_uncertainty
```

Config/timestamp snapshot qua CDC phải atomic. Với clock không phase-related, global event tới các consumers có latency và uncertainty khác nhau. Descriptor ghi điều này; không suy “shared trigger” thành “mọi channel lấy mẫu cùng một thời điểm tuyệt đối”.

## 18.2 Trigger fabric

Baseline ưu tiên một trigger context cho mỗi execution, source mask nhỏ và broadcast tới các consumers được chọn. Không triển khai general-purpose trigger network phức tạp nếu chưa có nhu cầu.

Sources có thể là digital edge/pattern, ADC threshold, external trigger, software force, experiment start hoặc generator sync. Chỉ công bố sources thực đã tích hợp.

Trigger được phát hiện ở source domain, giữ sample index/timestamp anchor và phân phối bằng handshake/FIFO hoặc qualified pulse. Các consumer xác nhận arm generation trước start. Nếu cần alignment chặt hơn độ trễ CDC, phải dùng common-clock synchronous path hoặc calibration/protocol chuyên biệt đã kiểm chứng.

## 18.3 Pre-trigger và no-trigger

Depth trước/sau trigger là tham số explicit. Hết thời gian chờ trigger trả `NO_TRIGGER`/timeout ở capture status đề xuất, không tự đánh đồng với DUT sai chức năng. Các mã này phải map qua schema kết quả hiện hành, không tự thêm enum application.

---

# 19. Logic Analyzer

## 19.1 Datapath V1

```text
Protected digital capture inputs
 -> capture registers / sample counter
 -> trigger qualifier
 -> raw sample packer
 -> ingress FIFO
 -> descriptor-bounded ring/linear writer
 -> shared SDRAM
```

Raw sampling là baseline dễ chứng minh. `EDGE_EVENT` và `RLE_COMPRESSED` là optional build modes; không bắt buộc cả ba trong phiên bản đầu.

## 19.2 Packing

Với `N` channels, packed width thực phải ghi trong descriptor. Nếu dùng 16-bit hoặc 32-bit aligned sample words, bandwidth tính theo số byte lưu thực, không theo số bit có tín hiệu.

Thứ tự channel/bit, byte order, sample period và mapping từ endpoint tới channel là metadata bất biến trong một capture.

## 19.3 RLE phải có worst-case bound

RLE có thể giảm dữ liệu khi giá trị lặp; nhưng tín hiệu thay đổi mỗi sample có thể làm encoding bằng hoặc lớn hơn raw, tùy record format. Admission phải dùng worst-case record rate đã định nghĩa.

V1 khuyến nghị lưu raw và nén khi readback nếu cần, hoặc dùng format block có explicit raw fallback. Không đổi encoding giữa capture mà không có block tag. Pre/post-trigger counts tính trên logical samples, không tính số RLE records.

## 19.4 Overflow

FIFO full khi source vẫn sample được -> latch overflow, ghi sample index/valid-prefix information, ngừng hoặc seal partial theo policy. Không backpressure một DUT/ADC vốn không thể bị dừng rồi vẫn tuyên bố đúng sample rate.

Nếu capture là required của execution, V1 dừng execution an toàn khi mất capture. Overflow là lỗi chất lượng/thực thi, không mặc định là electrical `UNSAFE`; classification ở §27.

---

# 20. Oscilloscope 2CH

## 20.1 Ranh giới phần cứng

```text
CH1 input -> protection/attenuation/coupling/AFE1 -> ADC CH1 --+
                                                            +-> FPGA
CH2 input -> protection/attenuation/coupling/AFE2 -> ADC CH2 --+
```

Dùng ADC 2CH hoặc hai ADC với clocking/alignment phù hợp. Multiplexer luân phiên một ADC không được quảng bá là hai channel simultaneous nếu không có sample-and-hold và timing chứng minh tương đương.

ADC interface, resolution, sample rate và analog bandwidth **chưa được hai tài liệu nguồn xác định**. Không gắn con số đo/sampling rate cố định vào architectural capability trước khi chọn AFE/ADC và closure.

## 20.2 Acquisition core

Ingress validation/alignment -> sample counter -> trigger condition -> sample packer -> FIFO -> memory writer. ADC status/overrange bits nếu có được giữ hoặc tổng hợp vào metadata.

Hai channel có calibration/range metadata riêng nhưng cùng capture timing model rõ ràng. Range/coupling thay đổi làm dừng acquisition hoặc bắt đầu capture revision mới; không trộn dữ liệu trước/sau thay range trong một waveform không có marker.

## 20.3 Decimation đúng semantics

Lấy mỗi mẫu thứ `D` mà không anti-alias filtering không được trình bày như waveform bandwidth-limited đã decimate đúng. V1 có thể giữ raw, xuất một window; hoặc dùng min/max envelope cho display với encoding riêng. Averaging/filtered decimation chỉ công bố sau khi filter và metadata được xác lập.

Không đưa phép xử lý nặng vào FPGA chỉ để giảm dung lượng nếu nó phá resource/timing budget. Raw capture/window readback là baseline.

## 20.4 Measurement validity

Descriptor cần actual sample rate, trigger index, channel alignment, ADC code format, range/gain/offset profile IDs, clipping/overrange, valid sample count và timestamp quality. ADC numerical resolution không đồng nghĩa effective resolution hoặc accuracy của instrument; những thông số sau cần đo.

---

# 21. Function Generator 2CH

## 21.1 Kiến trúc ưu tiên

Hai phase accumulators độc lập, với waveform lookup/arithmetic nhỏ, DAC code formatting và output-enable riêng. Có sync operation chung để reset phase/latch update theo clock đã xác định.

```text
DDS1 accumulator -> waveform core -> DAC1 codes -> external AFE1 -> isolate -> OUT1
DDS2 accumulator -> waveform core -> DAC2 codes -> external AFE2 -> isolate -> OUT2
```

Sine LUT có thể chia sẻ dual-port ROM hoặc time-multiplex nếu chứng minh đủ throughput cho cả hai channels. Không hy sinh independent-channel semantics để tiết kiệm M9K.

## 21.2 Bounded parameter update

Frequency, phase, waveform, amplitude/offset settings phải stage và commit tại boundary xác định. Amplitude/offset được thực hiện bằng DAC code scaling, external analog stage hoặc tổ hợp tùy hardware; không mặc định FPGA đã có analog gain stage.

Cần enforce đồng thời output swing, DC offset, load/current, waveform frequency và route envelope. Đường fault phải disconnect/mute external output stage; chỉ đặt DAC code về zero không luôn có nghĩa output 0 V hoặc Hi-Z.

## 21.3 DDS và memory

Baseline DDS dùng local LUT/registers, **không chiếm bandwidth SDRAM**. AWG arbitrary samples từ SDRAM là feature riêng, mặc định OFF. Chỉ thêm khi có read FIFO, rate admission, preload, underrun policy và output protection đã validation.

Nếu AWG underrun, output đi về trạng thái được electrical profile xác nhận; không mặc định giữ last sample, vì last sample có thể là mức DC không an toàn.

---

# 22. Capture descriptors và ownership xuyên lifecycle

## 22.1 Descriptor ngoài và trong

| Public/boundary descriptor | Internal-only fields |
|---|---|
| descriptor ID + generation; capture reference | Physical base address |
| instrument/channel map; encoding | Allocator cursor/region ownership |
| allocated capacity; valid bytes/sample count | SDRAM bank/row/column |
| pre/post-trigger, trigger position | Outstanding write counters |
| sample rate/clock/timebase anchors | DMA/write-ring pointers |
| termination reason; quality flags | Memory-controller scheduling state |
| sealed/completion state | Hardware-specific stride/alignment details không cần expose |

Host không yêu cầu base address. Capacity và read offset là logical quantities được bounds-check; không tạo cổng raw-memory access trong normal operation.

## 22.2 Allocation tối ưu cho một execution slot

Dùng một capture arena với số descriptor hữu hạn, page/burst-aligned, **allocation khi chưa acquisition**. Baseline là contiguous/bump allocation theo execution, không compact hoặc general-purpose heap khi chạy.

Scope/LA có vùng riêng trong arena theo nhu cầu mỗi image; không hard-partition toàn bộ 64 MiB mãi mãi. Không cần một memory manager đa-user/general virtual memory.

Nếu captures cũ chưa release chiếm vùng cần cho execution mới, trả resource unavailable hoặc chờ release; không ghi đè ngầm. V1 có thể yêu cầu drain/release trước reuse, thay vì xây concurrent multi-experiment capture store trong FPGA.

## 22.3 Completion và readback

`SEALED_COMPLETE` chỉ xuất hiện sau khi sample producer kết thúc, tất cả FIFO/outstanding writes đã commit và metadata cuối cùng được publish atomic. `valid_bytes` không vượt `allocated_bytes`.

Baseline host chỉ đọc sealed data. Readback của vùng đang ghi chỉ là optional snapshot/segmented feature có protocol riêng. `STOP`/`SAFE_STATE` không tự xóa sealed capture; nếu không, workflow run -> stop -> read_capture sẽ mất dữ liệu.

## 22.4 Generation chống reuse

Descriptor handle gồm ID và allocation generation. Khi vùng tái sử dụng, generation đổi; queued readback của handle cũ bị reject. Run token hết hiệu lực không có nghĩa read-only descriptor handle còn được giữ cũng hết hiệu lực; hai vòng đời được tách rõ.

Full FPGA/memory reset invalidates data khi không có retention guarantee. Không cam kết capture persistent qua mất nguồn.

## 22.5 Ring pre-trigger

Ghi oldest-valid offset, write position tại trigger, số pre-trigger hợp lệ và số post-trigger thực. Readback phải tuyến tính hóa theo thời gian từ oldest-valid, không xuất raw physical memory order rồi yêu cầu bên nhận đoán wrap.

---

# 23. SDR SDRAM subsystem

## 23.1 Capacity và organization

[F0] ghi “64 MB” nhưng chưa chọn part. Tài liệu này đề xuất dùng đơn vị chính xác **64 MiB** cho memory-address contract, phù hợp tổng dung lượng 512 Mibit. Nếu part/BOM cuối khác, công bố `capacity_bytes` thực; không quảng bá target như dung lượng sẵn có.

Data bus x16 là candidate, không bất biến. Row/column/bank geometry, chip count, CAS, refresh và frequency phải lấy từ part/revision thực. Không suy chúng chỉ từ dung lượng.

## 23.2 Controller responsibilities

Power-up timing, initialization, precharge, mode set, active/read/write, burst termination, bank/row tracking, refresh deadline và bus turnaround. Mọi timing counter được derive từ datasheet và clock bằng phép làm tròn bảo thủ.

SDR interface cần constraints cả output lẫn input, clock phase/board delay, IO standards và voltage banks. Mô phỏng logic controller không thay timing closure của bus ngoài.

## 23.3 Một địa chỉ hợp lệ đi qua ba kiểm tra

```text
instrument logical offset
  -> region bounds / owner / generation check
  -> physical address translation
  -> SDR geometry + timing controller
```

Memory controller không được nhận request vượt region dù instrument có lỗi. Counter phép cộng offset/length dùng đủ bit để không wrap-around trước khi kiểm tra bounds.

## 23.4 Health và corruption

Memory BIST trước enable deep-capture capability; stress refresh/arbitration theo profile. BIST không tự cung cấp ECC cho mọi runtime sample. Nếu không có ECC, capability phải nói rõ; packet CRC bảo vệ transfer không chứng minh SDRAM contents luôn không lỗi.

Fault/control essentials nằm trong registers/M9K, không chỉ trong SDRAM. Khi SDRAM fail, deep capture unavailable; các chức năng không phụ thuộc SDRAM chỉ tiếp tục nếu safety và build profile cho phép độc lập.

---

# 24. Memory bandwidth, latency và admission

## 24.1 Không đánh đồng capacity với throughput

```text
B_peak = data_bus_bits / 8 × f_memory

R_scope = active_scope_channels × stored_bytes_per_sample × f_scope
R_la    = stored_bytes_per_la_sample × f_la
R_awg   = stored_bytes_per_awg_sample × f_dac_stream  [chỉ khi AWG tồn tại]
```

Dùng packed bytes thực. Scope resolution 12-bit được lưu 16-bit thì tính 2 byte/channel/sample. Có RLE thì tính worst-case encoding, không average compression ratio.

## 24.2 Chọn đúng một cách tính overhead

**Mô hình A:** `B_usable = B_peak × eta`, với `eta` đã bao gồm refresh, precharge/activate, turnaround và các overhead được model.

**Mô hình B:** `B_usable` là sustained payload bandwidth đã đo ở workload/corners đại diện, trong khi refresh và controller overhead thực đang hoạt động.

Không vừa trừ refresh khỏi measured payload bandwidth vừa cộng lại refresh như một payload client lần nữa. Metadata/control overhead chưa có trong phép đo phải được tính riêng và ghi rõ.

```text
sum(R_realtime_payload) + R_extra_payload
    <= beta × B_usable
```

`beta < 1` là margin dự án, không phải thông số datasheet. Admission chỉ PASS khi đồng thời đạt **bandwidth, maximum service latency, buffer capacity, routing/clock/trigger exclusivity và electrical limits**.

## 24.3 Ví dụ tính toán — hoàn toàn giả định, không phải capability

Giả sử x16 SDR ở 100 MHz: peak là 200 MB/s. Giả sử `eta = 0,65` và `beta = 0,80`, payload được admission là 104 MB/s.

- Scope 2CH, mỗi channel 25 MS/s, lưu 2 byte/sample: 100 MB/s.
- LA 16CH, 20 MS/s, lưu 2 byte/sample: 40 MB/s.
- Tổng 140 MB/s: **không admission** theo ví dụ này, dù RAM có 64 MiB.

Giảm tới Scope 2CH × 20 MS/s và LA16 × 10 MS/s cho 100 MB/s chỉ vượt qua phép cộng bandwidth; còn phải chứng minh FIFO/service latency/timing/AFE. Không có giá trị nào trong ví dụ được công bố thành thông số bo.

## 24.4 FIFO sizing theo worst service gap

Với client `i`:

```text
D_i_bytes >= R_i_max × L_i_max + burst_i_bytes + margin_i_bytes
```

`L_i_max` gồm refresh, maximum non-preemptible burst, turnaround, arbitration và CDC visibility delay. Chỉ so tổng MB/s là chưa đủ để bảo đảm không overflow.

ADC/LA input liên tục không thể được cứu bằng host backpressure. Nếu service gap không có upper bound thì FIFO hữu hạn không thể tạo lossless guarantee.

## 24.5 Bounded arbiter ưu tiên cho V1

Một SDR controller với burst-limited, weighted round-robin/time-slot scheduling theo **workload profiles hữu hạn**. Refresh có deadline; realtime clients có reserved service, high-water intervention có giới hạn để không starve client khác.

Host readback là best-effort, baseline có thể tạm ngừng khi acquisition active. Sau capture, readback lấy bandwidth còn lại. Không cần một general deadline scheduler phức tạp hoặc online optimization trong FPGA.

Record maximum observed service gap/FIFO high-water/overflow để validation. Finite bench test là evidence; analytical bounds/RTL invariants vẫn cần cho upper-bound claims.

## 24.6 Capacity admission

```text
sum(allocated_capture_bytes)
  + optional_AWG_reserved_bytes
  + reserved_memory_bytes
  <= usable_capacity_bytes
```

Depth, pre/post window và format phải fit trước acquisition. Đọc memory chậm không làm thay sample rate; nó ảnh hưởng thời gian giải phóng buffer/availability của execution kế tiếp.

---

# 25. Transport endpoint và control latency

## 25.1 Kiến trúc không phụ thuộc bridge

Giữ lựa chọn USB-UART/USB-SPI cho prototype và USB3 FIFO cho high-throughput target. FPGA triển khai adapter phía FIFO/UART/SPI; USB PHY/controller của bridge nằm ngoài FPGA. [LF]

Control queue nhỏ và bulk queue riêng. Bulk chunks bounded; serializer/arbitration chèn control giữa chunks. Backpressure readback không khóa RX command parser hoặc kill path.

## 25.2 Giới hạn thời gian phải có phạm vi

```text
T_local_control <=
    maximum_nonpreemptible_chunk_time
  + bounded_queue_delay
  + parse_validate_time
  + dispatch_time
```

Bound này chỉ có ý nghĩa với giả định link/bridge vẫn phục vụ và control packet đã đến boundary được định nghĩa. Không hứa hard upper bound từ host application qua USB/OS khi host có thể ngừng scheduling hoặc link ngắt.

Bảo đảm an toàn khi host/link không tiến triển dựa vào **local watchdog và external inhibit**, không dựa vào `STOP` chắc chắn đến kịp. Performance measurement của transport không thay thế safety cutoff budget.

## 25.3 Bridge width và pin budget

FTDI mô tả FT600/FT601 có FIFO bus 16/32 bit; đó là ví dụ cho trade-off bus width, không phải BOM đã chốt. Bridge wider không tự nâng sustained SDRAM bandwidth và có thể làm E144 không đủ pins. [H4]

Chọn bridge cùng ADC/DAC/SDRAM trước freeze package. Không bổ sung USB3 vào cuối schematic mà không tính sẵn pins, clock và power domains.

---

# 26. Protocol integrity và error-free side effects

## 26.1 Envelope

Giữ các ý nghĩa SOF/version/command/sequence/length/payload/CRC/EOF trong [LF]. Cách encode bytes, escaping, opcode, CRC polynomial và endianness là phần ICD cần freeze/version; tài liệu này không tự đặt các giá trị đó.

Một gói chỉ có thể tạo side effect sau khi length/version/integrity/sequence/context/state/permissions đều hợp lệ. Không vừa nhận payload route vừa thay switch ngoài FPGA.

## 26.2 Version compatibility

Phân biệt protocol version, feature set, image schema revision, HardwareProfile revision và firmware/build revision. Major semantics không tương thích -> reject safe; optional feature thiếu -> unsupported.

Nếu bổ sung token hoặc ảnh cấu hình làm thay wire layout, phải dùng negotiated extension hoặc protocol version mới đã xác lập. Không tuyên bố binary backward compatibility chỉ vì giữ cùng tên command.

## 26.3 Parser bảo thủ

Bounded frame size, bounded staging buffer, partial-frame timeout và resynchronization rule. Invalid payload không được làm overflow RAM ghi cấu hình. CRC sai không refresh watchdog.

Không scan byte pattern `SAFE_STATE` trong raw capture stream để làm emergency shortcut; dữ liệu measurement có thể chứa bất kỳ byte nào. Emergency electrical cutoff độc lập protocol.

## 26.4 Bus nội bộ

Một register/command fabric đơn giản với request-valid/ready, response-valid/status, fixed arbitration và bounded slave response. Slave timeout không làm treo execution shutdown. Không cần soft CPU, full AXI infrastructure hoặc DMA subsystem đa-master tổng quát cho control nếu small register bus đã đủ.

---

# 27. Fault taxonomy: phân biệt từ chối command với mất an toàn điện

Đây là phần thay đổi chủ đích so với quy tắc bản trước “invalid command/protocol error/stale plan đều đưa route về open”. Một command bị từ chối trước side effect không được làm mất hiệu lực cấu hình đang chạy đã phê duyệt, trừ khi chính lỗi đó làm safety state không còn đáng tin.

| Lớp | Ví dụ | Hành vi hardware đề xuất |
|---|---|---|
| Benign request rejection | Read offset sai, unsupported optional feature, stale command không có side effect | NACK, không đổi active outputs/routes, không refresh watchdog |
| Transport corruption | CRC/length lỗi trước commit | Drop/NACK/resync; giữ safety watchdog; persistent loss -> timeout shutdown |
| Configuration/admission rejection | Resource conflict, image mismatch, rate vượt validated profile | Không apply/arm; rollback reservations; không energize plan mới |
| Electrical critical fault | Overcurrent, overvoltage, unsafe source contention, interlock fault | Immediate external inhibit + latched fault + isolation |
| Trusted-state loss | Control-state corruption, inconsistent active image, required clock mất | Inhibit, invalidate context, explicit recovery |
| Measurement failure | Capture overflow, missing samples, ADC alignment lost | Invalid/partial capture; stop execution nếu capture required; không gắn electrical UNSAFE giả |
| Required subsystem unavailable | SDRAM chưa ready khi image cần capture | Block thực thi configuration; không giả capability |

Status hardware chỉ cung cấp nguyên nhân chi tiết và mức severity. Việc ánh xạ ra `BLOCK_SAFETY`/`BLOCK_UNEXECUTABLE` và enum error phía ngoài phải theo Contracts thực, không tự thay enum đó bằng tên nội bộ.

**Bất biến thay thế:** invalid command không thể tạo enable mới, mở rộng quyền hoặc làm thay đổi configuration đang được bảo vệ. Quy tắc này khác với “mọi lỗi gói đều phải cắt một thí nghiệm đang an toàn”.

---

# 28. Capability: hardware truth, health và availability

## 28.1 Không đổi schema ngoài một cách ngầm định

[LF] dùng các field như `logicAnalyzerChannels`, `oscilloscopeChannels`, `generatorChannels`, `physicalExecutionSlots`. Bản bản cũ minh họa nested/snake-case fields. Kiến trúc này coi đó là representation nội bộ; mapping ra field ngoài phải explicit và được schema test.

FPGA không tự quảng bá component-resolution algorithms ngoài RTL. Nó báo scan primitives/test-profile support; `AUTO_DISCOVERY`, `INSTRUCTOR_PROFILE`, `HYBRID` vẫn là contract ngoài mà FPGA cung cấp evidence để phục vụ.

## 28.2 Ba lớp trạng thái

```text
IMPLEMENTED: có module/peripheral integration trong build
VALIDATED  : profile cụ thể đã qua tests/timing/electrical validation
AVAILABLE  : tại thời điểm hiện tại health/resources cho phép dùng
```

Một instrument đã validated 2CH nhưng đang bị lỗi không cần xóa vĩnh viễn channel count để giả thiết bị khác; nên trả capability ổn định và health/availability riêng khi schema cho phép. Nếu schema chỉ có một capability snapshot, phải định nghĩa rõ nó là usable-now hay installed-capacity.

## 28.3 Hardware manifest logic tối thiểu

Build ID, device/package, protocol/features; board/profile/NodeMap revision; digital source/sense counts; instrument channel counts; supported sample/clock modes; SDRAM actual capacity và validated workload profiles; routing/trigger/memory exclusions; electrical envelope IDs; timestamp metadata; watchdog/cutoff bounds đã validation.

`maxSampleRate` đơn lẻ không mô tả đúng mọi tổ hợp Scope/LA/Generator. Expose validated configuration profiles hoặc per-instrument limits + concurrency constraints qua contract được version hóa.

## 28.4 Calibration và identity

Hardware profile/AFE calibration data cần có revision/integrity và trạng thái valid. Thiếu calibration có thể cho raw-code measurement nếu mode đó được hỗ trợ và ghi rõ; không hiển thị calibrated-voltage accuracy giả. Calibration không phải permission để vượt absolute hardware limits.

---

# 29. Resource và package strategy EP4CE6 -> EP4CE10

## 29.1 Giới hạn silicon tham chiếu

Theo Cyclone IV Device Handbook, các bảng 1-1, 1-3 [H1]:

| Resource | EP4CE6 | EP4CE10 |
|---|---:|---:|
| Logic elements | 6.272 | 10.320 |
| Embedded memory | 270 Kbit | 414 Kbit |
| Embedded 18×18 multipliers | 15 | 23 |
| General-purpose PLLs | 2 | 2 |
| User I/O ở package E144 | 91 | 91 |
| User I/O ở U256/F256 theo bảng package | 179 | 179 |

Đây là device resources, không phải số pins/RAM/LE còn trống trên một board. Không dùng “144 pins” làm “144 user I/O”; dedicated/multipurpose/configuration usage và bank constraints còn phải xét.

## 29.2 Pin pressure có thể là giới hạn trước LE

**Ví dụ giả định để chứng minh trade-off, không phải schematic:**

- SDRAM x16 với 13 address, 2 bank, 2 DQM, 4 command/control, CKE và CLK: 39 tín hiệu.
- Hai ADC parallel 12-bit: ít nhất 24 data pins.
- Hai DAC parallel 12-bit: ít nhất 24 data pins.
- USB3 FIFO x16: ít nhất 16 data pins.

Chỉ tổng trên đã là **103 tín hiệu**, chưa tính clock/control ngoại vi, scan, LA, safety và digital source. Không fit vào 91 user I/O E144 của ví dụ. Đây không phải bằng chứng mọi cấu hình EP4CE10 E144 đều bất khả thi; serial/shared interfaces có thể giảm pin nhưng đổi throughput/complexity.

## 29.3 Thứ tự quyết định phần cứng

1. Chốt required simultaneous instrument configurations và electrical envelope.
2. Chọn candidate ADC/DAC/SDRAM/bridge interfaces tương thích.
3. Lập pin/bank/clock và memory-service budget của **cấu hình cuối**, kể cả khi RTL instruments triển khai sau.
4. Thử E144; chỉ dùng serialized/banked control cho tín hiệu tốc độ thấp phù hợp.
5. Nếu E144 không fit, ưu tiên package EP4CE10 U256/F256 nếu đáp ứng ràng buộc còn lại; nếu compute/memory timing cũng không đủ thì đánh giá device khác.

Không giảm số channel đã cam kết hoặc bỏ safety pins một cách im lặng chỉ để giữ package. Mọi thay đổi target cần biểu hiện đúng ở capability và decision record.

## 29.4 Ngân sách thiết kế, không phải synthesis result

Mục tiêu dự án có thể giữ LE khoảng 70–75% như đề xuất trước đây, nhưng đạt tỷ lệ này không đảm bảo timing. M9K phải budget theo **số block bị chiếm sau width/depth packing**, không chỉ chia tổng số byte FIFO cho tổng RAM.

Ưu tiên tài nguyên theo thứ tự: safety/control -> I/O/scan/routing -> ingress FIFO/trigger -> memory controller -> required instruments -> optional compression/AWG/debug. Khi thiếu tài nguyên, giảm tính năng optional trước, không giảm độ đầy đủ của safety gating.

Không có bảng utilization measured cho thiết kế hiện tại; mọi con số fit phải đến từ Fitter của đúng target và revision.

---

# 30. Những tối ưu kiến trúc được chọn và các phương án không chọn

| Quyết định chọn cho V1 | Lý do | Đánh đổi được chấp nhận |
|---|---|---|
| Một active image/context | Một shared electrical fabric | Không multi-execution trong cùng fabric |
| Sáu plane, small command bus | Rõ isolation và ownership | Không general-purpose SoC trong FPGA |
| External hard inhibit | Không phụ thuộc clock/USB | Cần safety hardware ngoài fabric |
| No live-reroute | Tránh powered DUT/source contention khi đổi route | Đổi topology cần stop/prepare lại |
| Scan sequencer + raw evidence | Giữ RTL nhỏ, tránh suy đoán identity | Resolution ở ngoài FPGA |
| Banked endpoint routing | Giảm pin/switch count, không sửa wiring | Không full N×N arbitrary net assembly |
| Scope pair stream + LA stream | Hạn chế số realtime memory clients | Channels trong scope pair chung capture timing model |
| DDS local LUT, AWG OFF | Không tăng SDRAM read pressure không cần thiết | Arbitrary waveform là mở rộng riêng |
| Capture arena allocation lúc idle | Bounds-check đơn giản, không fragmentation khi chạy | Reuse sau release, không general heap |
| Raw capture baseline | Dễ chứng minh capacity/quality | Compression không được dùng để hứa rate cao |
| Workload profiles + bounded arbiter | Admission/latency có thể kiểm chứng trên FPGA nhỏ | Không tùy ý mọi tổ hợp rate |
| Burst readback sau sealed | Dữ liệu immutable, tránh coherency phức tạp | Không promise raw continuous streaming |
| Common portable RTL + wrappers | Giảm rủi ro migration | Pin/timing vẫn review riêng từng board |

Phương án chỉ “thêm nhiều module vào bản trước” không giải quyết ambiguity về permit, epoch, partial commit và readback lifecycle. Phương án đổi FPGA ngay cũng không giải quyết các ambiguity đó. Vì vậy ưu tiên làm rõ transaction/safety/data semantics trước khi tăng chip size.

---

# 31. Portable RTL organization

Đây là cách **tổ chức module mục tiêu**; mã nguồn `cirnet-fpga` hiện có cấu trúc khác và không được tự ý di chuyển hoặc đổi public RTL interface chỉ để khớp sơ đồ tham khảo. Xem §37.1 để đối chiếu với các đường dẫn thực tế.

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

Vendor FIFO/PLL primitives được bọc lại thay vì rải IP instance board-specific khắp portable core. Các tên/công cụ simulation cụ thể chưa được xem là “repository đang sử dụng” khi chưa có source để đối chiếu.

---

# 32. Verification: acceptance properties của kiến trúc

## 32.1 Safety invariants

| ID | Thuộc tính phải kiểm chứng |
|---|---|
| V-01 | Unconfigured/reset/brownout/clock stopped -> hazardous enables OFF bằng external hardware |
| V-02 | Fault asserted -> physical inhibit trong bound đã đo, không phụ thuộc protocol/memory |
| V-03 | Không có scan stimulus trong `SAFE_IDLE` |
| V-04 | Scan dwell timeout hoạt động cả khi host hoặc scan-data FIFO bị tắc |
| V-05 | Approval của image A không energize image B; sửa cấu trúc/envelope sau seal làm mất approval |
| V-06 | Không route commit khi affected source/DUT energy chưa ở điều kiện switching hợp lệ |
| V-07 | Route commit thất bại không restore outputs tự động |
| V-08 | Unknown/mismatch alone không xuất hiện trên fault/permit cone of logic |
| V-09 | Guarded startup áp dụng cho cả RUN modes |
| V-10 | Fault recovery/PLL relock/reconnect không auto-resume |
| V-11 | Stale execution/descriptor generation không tác động execution hoặc vùng memory mới |
| V-12 | Packet CRC/length/version lỗi không tạo side effect hoặc refresh heartbeat |

## 32.2 Capture/memory properties

| ID | Thuộc tính phải kiểm chứng |
|---|---|
| V-13 | Mỗi sample nhận hợp lệ có đúng thứ tự/encoding, hoặc có explicit loss flag |
| V-14 | Pre-trigger ring wrap được tuyến tính hóa đúng và trigger index hợp lệ |
| V-15 | Completion chỉ sau last write drain; valid bytes không vượt allocation |
| V-16 | STOP -> SAFE vẫn cho đọc sealed capture còn được giữ |
| V-17 | Descriptor release/reuse không cho queued stale read truy cập dữ liệu mới |
| V-18 | Worst-case RLE hoặc raw fallback không vượt reserved service/capacity |
| V-19 | Arbiter đạt max service gap cho mọi realtime client được admission |
| V-20 | Refresh/turnaround không bị các burst khác starve |
| V-21 | Async FIFO reset ở mọi phase không để stale sample/new generation trộn nhau |
| V-22 | Scope/DDS/LA chạy đồng thời theo từng supported workload profile, không theo maximum độc lập |

## 32.3 Tests cần có

Module simulation cho FSM, image sealing, route sequencer, scan dwell, source gates, triggers, packers, ring descriptors, allocator và arbiter. Integration fault injection ở mọi transition, đặc biệt giữa seal/apply/power-up/start và khi memory FIFO đầy.

Randomized clocks/reset ratios cho CDC; duplicate/reordered/truncated command tests; bit-flip/length-bound tests; SDR memory-model timing checks; fitter và full timing analysis trên từng target.

Bench tests: short/overvoltage trong envelope test được kiểm soát, power-off backfeed, stuck switch/enable fault model, clock removal, bridge reset, cable disconnect, capture overrun, no-trigger, and reset-replay. Test electrical faults phải dùng fixture giới hạn năng lượng thích hợp; không lấy sinh viên/IC tùy ý làm tải thử bảo vệ.

---

# 33. Closure gates và thông số phải được chứng minh

| Gate | Nội dung bắt buộc | Trạng thái từ tài liệu hiện có |
|---|---|---|
| G-01 Authority/ICD | CIRNET Lab-First + Contracts, plan-name mapping, commands/enums, ABI version | OPEN |
| G-02 Electrical envelope | Rail/I/O/scan/scope/gen limits, injection/backfeed, cutoff/discharge | OPEN |
| G-03 Pin/package | Final peripherals + pins/banks/safety/debug, Fitter feasibility | OPEN |
| G-04 Clock/CDC/RDC | Clock sources, PLL budget, constraints, reset/coherence/uncertainty | OPEN |
| G-05 RTL/resource | Utilization, FSM/transaction/data invariants | OPEN |
| G-06 SDR timing/service | Exact memory part, timing, refresh, service-gap and payload budget | OPEN |
| G-07 Measurement | LA capture envelope, Scope alignment/calibration, generator output quality | OPEN |
| G-08 Transport/replay | Local control bound, watchdog, bridge purge/session freshness | OPEN |
| G-09 Integrated bench | Simultaneous workloads và fault injection toàn hệ | OPEN |

**OPEN không đồng nghĩa thiết kế không thể thực hiện.** Nó nghĩa tài liệu nguồn chưa cung cấp bằng chứng closure; không được nâng trạng thái chỉ vì architecture đã được viết lại.

## 33.1 Timing budgets cần điền bằng số đo/model

`T_fault_detect`, `T_external_cutoff`, `T_FPGA_inhibit`, `T_rail_discharge_max`, `T_route_break_min`, `T_route_commit_max`, `T_scan_dwell_max`, `T_stop_quiesce_max`, `T_host_watchdog`, `T_external_watchdog`, `L_memory_client_max`, `T_local_control_max`.

Không có numeric default an toàn cho mọi linh kiện. Mỗi bound phải kèm điều kiện tải, voltage/temp corners, clock/profile và phương pháp xác nhận. Không dùng một timeout chung cho mọi state.

## 33.2 Debug/configuration cũng phải bị interlock

JTAG/configuration/bench override không được trở thành cổng bypass protection vô tình. Chế độ test có thể điều khiển pins ngoài expected RTL behavior; fixture/board phải giữ external inhibit khi configuration hoặc debug thao tác chưa được validation. Debug không thay đổi student execution envelope.

---
---

# 34. FPGA trong CIRNET — STEM & Engineering Learning Ecosystem

## 34.1 Hai vai trò FPGA phải cách ly

| Vai trò | Mục đích | Owner chính | Quyền và yêu cầu |
|---|---|---|---|
| **Lab Control FPGA** | Scan, protected routing, clock/trigger, measurements, instruments và safety interlocks | `cirnet-fpga` (RTL) + `cirnet-hardware` (host/driver) | Chỉ bitstream đã duyệt, nạp bởi operator được ủy quyền, fail-safe trên board cụ thể |
| **Learner FPGA Target** | Sinh viên viết HDL, build, simulate, flash/debug trên board học tập | `cirnet-ide`, `cirnet-toolchains`, `cirnet-devices`; `cirnet-silicon` phụ trách EDA learning | Artifacts không tin cậy mặc định; sandbox, board isolation, upload permission riêng |

**Không cho phép** learner-generated bitstream ghi đè Lab Control FPGA đang nối protected fabric của sinh viên. Nếu sử dụng cùng chủng FPGA thì vẫn phải là **khác thiết bị hoặc khác trust/isolation boundary** với interlock được chứng minh. Phải tách namespace artifact, toolchain profile, execution policy và release channel.

### 34.2 Luồng học FPGA trong hệ sinh thái

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

- **RTL simulation PASS** không tương đương FPGA board chạy đúng, và FPGA board chạy đúng không tương đương **physical electrical safety**.
- CIRNET Blocks có thể cung cấp HDL-oriented blocks **nếu** có typed IR, semantics và source-map; không dùng chuỗi sinh RTL để bỏ qua syntax/timing validation.
- `cirnet-ai` được phép gợi ý/giải thích, không được tự cấp quyền nạp bitstream, kích nguồn, xóa bằng chứng hoặc xác nhận grades.

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

Backend không được gửi raw route bypass từ browser. Truyền `PhysicalExecutionPlan` hoặc `InstrumentationExecutionPlan` theo **schema đã được Contracts xác nhận**; Hardware compiler tạo primitive/hardware image; FPGA không cần parse full JSON, không sở hữu student grading. Kết quả `SafetyVerificationResult`, `TopologyVerificationResult`, `FunctionalVerificationResult` phải phân biệt như Lab-First.

## 34.4 Instrumentation là năng lực dùng chung, không là license bypass

Các ứng dụng (Lab, Analyze, Silicon, Robotics, IoT) có thể **đọc** waveform và measurement artifact qua quyền dữ liệu. Để điều khiển nguồn, trigger, generator hoặc DUT, phải đi qua authorized station lease và safety path. Không cho module Robotics/IoT/AI gọi FPGA register trực tiếp qua public API.

---

# 35. Ownership Matrix — trách nhiệm xuyên Repository

| Repository | FPGA-related ownership | Không được thực hiện |
|---|---|---|
| `cirnet-fpga` | RTL, FPGA module logic, SDC/QSF target, testbench, source/bitstream provenance và board-specific timing evidence | Không Auth, Scheduler, Projects, Grading, REST, simulator engine |
| `cirnet-hardware` | Rust HardwareStation, protected transport/driver, station safe-state, worker supervision, applied-image acknowledgment | Không tự đổi canonical schema hoặc thay hard electrical protection |
| `cirnet-remote` | Experiment admission, auth delegation, durable lease/quarantine, provenance lock, capture/API | Không ghi register FPGA/MUX từ public handler |
| `cirnet-contracts` | Canonical types, FPGA protocol mapping, RPC, hardware capability, capture envelope | Không tự triển khai RTL hoặc kiểm chứng electrical limits |
| `cirnet-interface` | Browser circuit/lab editor, capabilities UI, authorized waveform display | Không kết luận mạch SAFE hoặc tạo route thật |
| `cirnet-studio` | Engineering workspace tổng hợp các editor và result views | Không fork `cirnet-interface` hoặc tự triển khai FPGA driver |
| `cirnet-ide` | HDL editor, LSP, build/debug UI | Không tạo compiler hoặc programmable bridge bypass |
| `cirnet-toolchains` | Pinned HDL simulator, synthesis, fitter, timing analyzer, reproducible builds | Không tự quyết định điện an toàn hoặc nạp board không có authorization |
| `cirnet-devices` | BoardProfile, part/package/pins/clock/flash/debug adapter và validation metadata | Không quản lý shared Lab lease hoặc grades |
| `cirnet-silicon` | HDL/RTL/ASIC learning workflow, logical synthesis/STA/PDK adapters theo quyền | Không thay Lab Control FPGA production firmware |
| `cirnet-project` / `cirnet-format-project` | Project revision, artifact links, portable file manifest/format migrations | Không chứa secrets của station hoặc firmware ngoài quyền sở hữu |
| `cirnet-analyze` / `cirnet-numerics` | Phân tích measured waveform, FFT/statistics/uncertainty và reproducible reports | Không tự ghi raw hardware routing commands |
| `cirnet-lms` / `cirnet-learn` | Assignment/gradebook và UX; rubric và learning evidence | Không quyết định `SAFE`, không cấp quyền hardware từ điểm số |
| `cirnet-platform` / `cirnet-database` | Identity, org/tenant/policy và persistence infrastructure | Không quản lý pin FPGA hay đè domain schema của Hardware |
| `cirnet-extensions` / `cirnet-integrations` | Signed/validated plugin/adapters và interoperability boundary | Không phát hành extension có quyền disable safety không kiểm soát |
| `cirnet-rules` | AI Agent baseline/Skills/Policies, documentation discipline | Không là execution service hoặc tự publish firmware |

`cirnet-app`, `cirnet-desktop`, `cirnet-cli`, `cirnet-robotics`, `cirnet-iot`, `cirnet-ai` chỉ tiêu thụ các giao diện công khai được phân quyền. Quyền USB/Serial trên Desktop hoặc CLI không tự tạo quyền điều khiển Lab Control FPGA. `cirnet_web` chỉ được cung cấp thông tin public/được duyệt; `.github`, `about-cirnetlab` là tài liệu/cấu hình ngoài runtime.

---

# 36. Quy chuẩn Project, BoardProfile, Image và Artifact

## 36.1 Mô hình artifact tách nguồn và chứng cứ thực

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

Dữ liệu do `cirnet-project` sở hữu về lifecycle; file packaging/import/export thuộc `cirnet-format-project`; schema giao tiếp qua `cirnet-contracts`; runtime build thuộc Toolchains; target board thuộc Devices. Không copy tất cả thành nhiều “truth” trong mỗi UI.

## 36.2 Bắt buộc trong manifest build/bitstream

- Source commit hoặc content digest; project revision; generator/toolchain version và source/license của IP cores.
- Device family, exact part/package/speed grade, board revision, pin/IO-bank electrical standards và memory/clock constraints.
- Binary SHA-256 từ artifact **bytes thực**, release label, signing/trust policy (khi có) và selected compatibility set.
- Validated limits/capabilities của board; không suy từ device datasheet ra thông số PCB.
- Khi cài trên Lab Control FPGA: approved firmware chain, safe-state prerequisites, physical operator permission và rollback/secure recovery plan.

## 36.3 Version domains

Tách biệt version của: Repository tag, RTL source, bitstream, ICD/wire ABI, Contracts schema, BoardProfile, toolchain, pinout/PCB, FPGA IP core, capture encoding, selected CIRNET system release. Định dạng tag/release chung **`<repo_name>-<version>`**; ví dụ `cirnet-fpga-0.2.2` chỉ minh họa **cấu trúc tên**, không khẳng định tag đã phát hành. Không sửa lịch sử đã pin.

---

# 37. Quy tắc phát triển RTL và xác minh thực tế

## 37.1 Dựa vào cấu trúc repository thực tế

Tại lần đối chiếu, `cirnet-fpga` có `fpga/experiment-controller/rtl/top.v`, `fpga/instrument/rtl/top.v`, testbenches trong `fpga/simulation/`, `fpga/common`, `fpga/constraints`, `fpga/quartus`, source/verification scripts và `integration/` lockfiles. Cấu trúc portable RTL ở §31 là **định hướng module hóa**, không phải lý do tự động di chuyển các file hoặc thay tên interfaces đã được nghiệm thu. Hãy mở rộng module trong vị trí phù hợp và giữ các test/paths cũ nếu còn consumer.

## 37.2 Test và conformance bắt buộc

| Lớp | Evidence bắt buộc trước công bố |
|---|---|
| Source hygiene | Lint, format, paths/docs, license/IP provenance, no secret, correct locks |
| RTL unit/integration | V-01…V-22 ở §32, fault injection, reset/CDC, no-rewire invariant |
| FPGA protocol | Artifact-pinned [CONTRACTS] ICD, framing/replay/sequence, timeout, Stop và safe downgrade |
| Synthesis/fitter | Exact target part/package, utilization, actual pin assignment, IO banks, DRC |
| Timing/CDC/RDC | Target-specific SDC, setup/hold, async paths, reset crossing, all relevant corners |
| Electrical safety | Schematic/BOM/revision + independent cutoff measured, power-off/backfeed, fault-latched recovery |
| Instrument bench | ADC/DAC/AFE calibration, actual sample rate, skew/jitter, trigger, output envelope và quality flags |
| Full Lab E2E | Real Rust Remote/Hardware, exact firmware/board/Contracts, authenticated Web, measured evidence |
| Learning target | RTL simulation and board programming isolated from trusted Lab Controller, exact assignment artifact |

Không dùng Python test scripts trong production FPGA control path. Python cho offline tests/benchmarks được phép, khớp `cirnet-fpga` hiện có `pytest.ini`, `tests/`, `requirements-verification.lock`.

## 37.3 Ba loại Stop hoàn toàn khác nhau

- **UI Stop requested:** một thao tác ở client, chưa chứng minh gì về phần điện.
- **Hardware Stop/SAFE_STATE acknowledged:** hardware đã xác nhận trạng thái điện/tài nguyên theo policy.
- **External hard cutoff:** đường an toàn vật lý độc lập FPGA clock/OS/USB. Phải chứng minh bằng bench trong fault model đã duyệt.

## 37.4 Các điều cấm quan trọng

- Không nạp firmware/bitstream người học lên chip điều khiển protection của Lab đang phục vụ người khác.
- Không cấp phép unsafe route, nguồn ngoài, clock/FPGA reset hoặc debug/JTAG để bypass external safety island.
- Không tạo HDL hoặc FPGA routing để “sửa” sơ đồ vật lý sai của sinh viên.
- Không tuyên bố scope/LA/generator, SDRAM, HIL hoặc digital physical support đã hoạt động nếu thiếu evidence.
- Không cho extension/AI/LMS/CLI yêu cầu side effects ngoài lease, board profile và protected RPC.
- Không chuyển nội dung vi mạch bán dẫn/EDA vào RTL điều khiển Lab khi đó là workflow học tập độc lập.

---

# 38. Architectural Decision Records

| ADR | Quyết định chính thức đề xuất |
|---|---|
| FPGA-ADR-001 | Safety-gated, not correctness-gated; giữ bốn `ExecutionDecision` |
| FPGA-ADR-002 | `cirnetlab-architecture-en.md` bao phủ toàn Organization; CIRNET Lab-First làm chuẩn Lab safety; FPGA chỉ sở hữu hardware RTL |
| FPGA-ADR-003 | Canonical Contracts/ICD được chọn theo artifact; `FpgaExecutionImage` là format nội bộ |
| FPGA-ADR-004 | Approval gắn sealed image; thay đổi cấu hình critical đòi reapproval |
| FPGA-ADR-005 | External hard inhibit/watchdog độc lập clock và transport |
| FPGA-ADR-006 | Scan/source/power permits độc lập, không phụ thuộc vòng |
| FPGA-ADR-007 | Passive observation, limited-energy continuity, active-signature scan có semantics rõ |
| FPGA-ADR-008 | Route change khi an toàn về điện, không live-reroute trong baseline |
| FPGA-ADR-009 | Một shared fabric, một execution context; instrument capture chạy trực giao |
| FPGA-ADR-010 | Boot/session/execution freshness khác descriptor readback lifetime |
| FPGA-ADR-011 | Lệnh benign bị reject không phá một execution đang an toàn; mất trust state phải inhibit |
| FPGA-ADR-012 | DDS local LUT; AWG SDRAM tùy chọn có admission và underrun policy riêng |
| FPGA-ADR-013 | Raw capture có quality flags rõ, RLE không được dùng thay worst-case bound |
| FPGA-ADR-014 | Bounded capture arena + SDR arbiter; không general heap/OS trong FPGA |
| FPGA-ADR-015 | Host/USB control latency không thay physical cutoff deadline |
| FPGA-ADR-016 | EP4CE10 package chỉ chốt sau pins/clock/SDRAM/ADC/DAC/USB budget closure |
| FPGA-ADR-017 | Waveform provenance bao gồm rate/skew/uncertainty/partial/loss/calibration |
| FPGA-ADR-018 | Capability dựa trên verified board evidence; plan/RTL scaffold không đủ |
| FPGA-ADR-019 | Lab Control FPGA và Learner FPGA Target phải cách ly về trust, permission và physical I/O |
| FPGA-ADR-020 | C++ là mặc định cho circuit Simulator; Rust cho services/Hardware; RTL cho FPGA; Python là verification lane của platform |
| FPGA-ADR-021 | Toolchains/Devices/Silicon/Project/LMS consume versioned artifacts; không nhân bản ownership |

---

# 39. Vấn đề mở, hiệu lực và provenance

## 39.1 Những việc không thể xác nhận chỉ từ Markdown

1. Final FPGA part/package/board/pinout, target clock/timing closure và source-specific synthesis results.
2. Mạch eFuse/supervisor/AFE/ADC/DAC, electrical protection threshold, fault-cutoff/discharge times và điện áp nguồn ngoài.
3. Giao thức physical byte-level, stale-session defense, HDL/firmware hashes và board profile tương thích.
4. SDRAM part/geometry, BIST, refresh/service bound và simultaneous LA/Scope/Generator workload.
5. Phép đo thực về scope channel synchronization, generator output quality và sample-rate/uncertainty.
6. Full HIL và Physical Lab authorization/lease/recovery ở đúng target board.
7. Quy tắc nội bộ cho extension, student bitstream và chip debug chưa có test cách ly.

Giữ `OPEN/NOT_RUN/UNAVAILABLE` cho tới khi có đúng loại bằng chứng. Không tự sửa selected release set, không tạo Git tags/releases và không coi việc cập nhật tài liệu là code/test đã triển khai.

## 39.2 Nguồn tài liệu

- **Tài liệu gốc do người dùng cung cấp:** bản dự thảo FPGA phần cứng lịch sử [F0], chứa sáu planes, 22 boundary checks, 22 verification properties, 9 closure gates, khối tính SDRAM/pin và hardware safety. Nội dung kỹ thuật hợp lý được bảo toàn, còn tên tổ chức cũ và các giả định nguồn lỗi thời đã được loại bỏ.
- **Kiến trúc mới:** `cirnetlab-architecture-en.md` (Ecosystem), `cirnet-architecture-lab-first.md` (Lab-First).
- **Mã nguồn tham khảo (2026-11-15):** `cirnetlab/cirnet-fpga` branch `master`, cùng README `cirnet-hardware`, `cirnet-contracts`, `cirnet-simulator`; đây là source observation, không thay HIL.
- **Nguồn phần cứng [H1]–[H5]:** bảng và liên kết bên dưới là references phục vụ lựa chọn/đối chiếu part, không xác nhận BOM hoặc board đã đạt.

| ID | Tài liệu kỹ thuật tham khảo | Đối tượng sử dụng |
|---|---|---|
| H1 | [Cyclone IV Device Handbook](https://cdrdv2-public.intel.com/655151/cyiv-5v1.pdf) | Part resources/pin/clock/PLL/constraints, đọc đúng part và package |
| H2 | [TI TMUX1575](https://www.ti.com/product/TMUX1575) | Ví dụ powered-off signal-path protection và hạn chế back-power; không là BOM mặc định |
| H3 | [Intel timing constraint guidance](https://www.intel.com/content/www/us/en/docs/programmable/683241/24-3/user-configurable-timing-constraint.html) | CDC/Gray constraint principles; dùng tài liệu đúng vendor/target |
| H4 | [FTDI FT601](https://ftdichip.com/products/ft601q-b/) | Ví dụ 16/32-bit high-throughput FIFO; không thay measurement throughput |
| H5 | [Intel programmable device temperature support](https://www.intel.com/content/www/us/en/products/docs/programmable/temperature.html) | Part operating envelope, không suy 5-V tolerance cho raw FPGA GPIO |

**Kết luận:** FPGA của CIRNET được tổ chức như một năng lực **kỹ thuật giáo dục có thể tái sử dụng** và một **Lab Control Fabric được bảo vệ**; các đường học HDL/FPGA/ASIC phải tái sử dụng nguồn/toolchain/board/evidence, nhưng không được thừa hưởng quyền cấp điện hoặc phá interlock. Phần hardware cũ được giữ ở mức nguyên lý, bổ sung ownership theo 33 Repo và mọi compatibility/electrical claim đều yêu cầu kiểm chứng thực tế.
