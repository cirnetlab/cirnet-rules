# Bản đồ nguồn của sáu repository

Đã kiểm tra local ngày **2026-11-14**. Đây là dữ liệu tham khảo, không phải thẩm
quyền quyết định của sản phẩm hay bước khởi tạo bắt buộc trên toàn tổ chức. Chỉ đọc
repo sở hữu và các dependency trực tiếp liên quan đến task; kiểm tra lại chỉ dẫn
local và các lựa chọn hiện hành.

Cả sáu checkout đều chỉ đọc. Mỗi checkout ở nhánh `master`, với
`git status --short` sạch khi kiểm tra. Không chạy build, test, cài đặt hay thao tác
runtime của sản phẩm. **SOURCE_REVIEW** bên dưới nghĩa là quan sát mã/tài liệu;
tái hiện runtime là **NOT_RUN**. Artifact bên ngoài bị thiếu là **NOT_VERIFIED**.
Các số phiên bản ở đây chỉ mô tả ảnh chụp này, không phải chính sách common.

## Danh tính local

Các repository là thư mục con trực tiếp của `<repo-root>` do người dùng chọn.
Các đường dẫn bên dưới tương đối với root của từng sản phẩm. Link ghim revision
Git đã kiểm tra, không khẳng định một nhánh đang thay đổi hoặc release lịch sử là
bản hiện hành.

| Repository | Nhánh local | HEAD đầy đủ | Working tree ban đầu |
| --- | --- | --- | --- |
| cirnet-remote | master | `1a38c1331a07bdc3b5644c70eebb40590ae53452` | clean |
| cirnet-interface | master | `f1ff70b9f22554c0680b92679e9e44a5544247ff` | clean |
| cirnet-contracts | master | `4bfd0950f564aaf24abaf6b8b82399c2d54d0e39` | clean |
| cirnet-hardware | master | `2139cccc8b36dc403c2f7182dfb670a97513858a` | clean |
| cirnet-simulator | master | `c05be23bef71de44a9027319af945d3e635d0882` | clean |
| cirnet-fpga | master | `bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0` | clean |

## Hệ thống được chọn so với source hiện tại

[Lựa chọn release](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/integration/release-set.json)
của Remote vẫn chọn **system 0.1.4 / Interface 0.1.4**, cùng **API, Contracts,
Simulator, Hardware và FPGA scaffold 0.1.2**. Các phiên bản source/candidate native
hiện tại bên dưới không thay thế lựa chọn đó. Nhãn phiên bản hay kết quả PASS đã
ghi nhận của candidate đều không chứng minh việc triển khai, nghiệm thu release
đầy đủ hoặc mức độ sẵn sàng vật lý.

## cirnet-remote

**Sở hữu:** ngữ nghĩa ứng dụng HTTP/realtime, xác thực, lưu bền project/capture,
lập lịch, tích hợp/triển khai và **lựa chọn system release**. Sử dụng Contracts
canonical và Hardware RPC local; không sở hữu schema của chúng.

**Quan sát:** release-set chọn system và Interface **0.1.4**, cùng API,
Contracts, Simulator, Hardware và FPGA scaffold **0.1.2**. Runtime được chọn là
Python FastAPI cùng Hardware gRPC local riêng biệt và Interface trên trình duyệt.
Source Rust hiện tại là **0.2.4**, với native Contracts vẫn ghim ở **0.1.2**;
phiên bản source không kích hoạt hay thay thế release đã chọn đó. Tài liệu chuyển
đổi native ghi nhận kết quả candidate trong phạm vi xác định, gồm các bundle
Windows khác nhau với các gate còn lại khác nhau. Nghiệm thu đúng release, vận hành
có tải Linux/Xeon và HIL vật lý được ghi riêng là chưa chạy. Không có byte của
bundle/manifest native bên ngoài để kiểm tra tại đây.

**Bằng chứng source:** các route HTTP dùng Contracts chung; export OpenAPI kiểm
tra danh mục operation; quyền scheduler được lấy trước recovery và giữ suốt quá
trình drain. Test có các trường hợp chặn trạng thái unsafe/indeterminate, sai khác
topology nhưng an toàn, cleanup khi mất quyền điều khiển và quarantine. Sự tồn tại
của test không phải một kết quả PASS mới.

**Tài liệu đã đọc:** [README.md](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/README.md) · [AGENTS.md](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/AGENTS.md) · [docs/en/architecture/system.md](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/docs/en/architecture/system.md) · [docs/en/architecture/lab-first-v3-1.md](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/docs/en/architecture/lab-first-v3-1.md) · [docs/en/architecture/native-migration-status.md](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/docs/en/architecture/native-migration-status.md) · [docs/en/contributing/verification.md](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/docs/en/contributing/verification.md) · [docs/en/operations/native-candidate.md](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/docs/en/operations/native-candidate.md).

**Manifest:** [Cargo.toml](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/Cargo.toml) · [integration/release-set.json](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/integration/release-set.json) · [integration/contracts-version.json](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/integration/contracts-version.json).

**Source đã đọc:** [native/src/http_api.rs](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/native/src/http_api.rs) · [native/src/openapi.rs](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/native/src/openapi.rs) · [native/src/hardware.rs](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/native/src/hardware.rs) · [native/src/db/authority.rs](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/native/src/db/authority.rs) · [services/api/app/main.py](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/services/api/app/main.py).

**Source test đã đọc, chưa chạy:** [native/tests/scheduler_authority.rs](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/native/tests/scheduler_authority.rs) · [services/api/tests/test_lab_lifecycle.py](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/services/api/tests/test_lab_lifecycle.py).

## cirnet-interface

**Sở hữu:** workbench trình duyệt Vue 3/TypeScript, lệnh/lịch sử tài liệu, chỉnh
sửa graph, phép chiếu scene Three.js và test trình duyệt. **Graph logic là nguồn
sự thật**; hình học dây, điểm giao và trạng thái camera không tạo kết nối điện.
Contracts sở hữu schema canonical, DTO/validator sinh tự động và danh mục thiết bị.

**Quan sát:** source package là **0.2.4**, với trạng thái tương thích
`UNRELEASED_NATIVE_HANDOFF` nhắm đến Remote 0.2.4 và Contracts 0.2.2.
Lựa chọn dependency/artifact thực tế vẫn khóa Contracts SDK **0.1.2**.
Đây không phải Interface 0.1.4 được chọn trong system release của Remote.
Profile virtual mặc định không cung cấp thao tác phần cứng Scope/Generator;
desktop/mobile là đề xuất, không phải tuyên bố năng lực đã triển khai.

**Bằng chứng source:** catalog và validator import `@cirnet/contracts`;
lệnh graph kiểm tra thay đổi dự kiến trước khi áp dụng; SceneManager chiếu ID
graph và cổng logic. Test bao phủ danh tính graph ổn định, lịch sử, bảo toàn trạng
thái khi chỉnh sửa không hợp lệ và chẩn đoán tại ranh giới mã sinh tự động.

**Tài liệu đã đọc:** [README.md](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/README.md) · [AGENTS.md](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/AGENTS.md) · [docs/en/architecture/workbench.md](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/docs/en/architecture/workbench.md) · [docs/en/contributing/verification.md](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/docs/en/contributing/verification.md).

**Manifest:** [package.json](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/package.json) · [integration/compatibility.json](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/integration/compatibility.json) · [integration/contracts-artifacts.json](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/integration/contracts-artifacts.json).

**Source đã đọc:** [src/data/editorCatalog.ts](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/src/data/editorCatalog.ts) · [src/contracts/structural.ts](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/src/contracts/structural.ts) · [src/domain/document/editorDocument.ts](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/src/domain/document/editorDocument.ts) · [src/domain/document/graphCommands.ts](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/src/domain/document/graphCommands.ts) · [src/three/SceneManager.ts](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/src/three/SceneManager.ts).

**Source test đã đọc, chưa chạy:** [tests/document-core.test.ts](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/tests/document-core.test.ts) · [tests/generated-boundary.test.ts](https://github.com/cirnetlab/cirnet-interface/blob/f1ff70b9f22554c0680b92679e9e44a5544247ff/tests/generated-boundary.test.ts).

## cirnet-contracts

**Sở hữu:** schema canonical, định dạng truyền, danh mục thiết bị, binding sinh
tự động, SDK công khai, tài nguyên validation và fixture tương thích. Consumer
không được định nghĩa lại schema canonical hoặc sửa output SDK sinh tự động thành
nguồn sự thật. Remote sở hữu ngữ nghĩa operation HTTP và lựa chọn hệ thống.

**Quan sát:** phiên bản source của package Rust và SDK công khai là **0.2.2**;
Contracts mà Remote chọn vẫn là **0.1.2**. Các định danh contract có phiên bản độc
lập, gồm Hardware RPC **2.0** và interface FPGA scaffold/reserved. Phạm vi hiển thị
trong manifest tách contract trình duyệt công khai khỏi plan thực thi/điều khiển
nội bộ. Hỗ trợ schema/codec không chứng minh có producer runtime khả dụng.

**Bằng chứng source:** protobuf canonical giữ lỗi unavailable/unsafe rõ ràng
và tách năng lực virtual/physical. Validation phân giải tài nguyên offline;
bundle công khai từ chối tham chiếu private hoặc không có trong manifest. Fixture
kiểm tra độ bao phủ registry, tính tương đương descriptor, giá trị optional và
quyết định safety/topology tách biệt. Cây source không chứng minh SDK mới đã được
publish hoặc được chọn.

**Tài liệu đã đọc:** [README.md](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/README.md) · [AGENTS.md](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/AGENTS.md) · [docs/en/contract-reference.md](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/docs/en/contract-reference.md) · [docs/en/verification.md](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/docs/en/verification.md) · [docs/en/native.md](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/docs/en/native.md).

**Manifest:** [contracts/manifest.json](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/contracts/manifest.json) · [Cargo.toml](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/Cargo.toml) · [packages/public-sdk/package.json](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/packages/public-sdk/package.json).

**Source đã đọc:** [contracts/hardware/v2/rpc_v2.proto](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/contracts/hardware/v2/rpc_v2.proto) · [native/src/lib.rs](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/native/src/lib.rs) · [native/src/validation/mod.rs](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/native/src/validation/mod.rs) · [native/src/validation/bundle.rs](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/native/src/validation/bundle.rs).

**Source test đã đọc, chưa chạy:** [native/tests/hardware_v2_parity.rs](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/native/tests/hardware_v2_parity.rs) · [tests/contracts/test_contract_registry.py](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/tests/contracts/test_contract_registry.py) · [tests/contracts/test_lab_first.py](https://github.com/cirnetlab/cirnet-contracts/blob/4bfd0950f564aaf24abaf6b8b82399c2d54d0e39/tests/contracts/test_lab_first.py).

## cirnet-hardware

**Sở hữu:** triển khai Hardware RPC 2.0 local, vòng đời station, start/stop và
cleanup có bảo vệ, giám sát worker và transport FPGA riêng. Contracts sở hữu
contract RPC. Remote sở hữu admission, lease lưu bền, quarantine, lưu bền capture
và quyết định safe-release ở cấp hệ thống.

**Quan sát:** candidate Rust **0.2.2** là **chỉ virtual** và vẫn dùng native
Contracts 0.1.2. Hệ thống được chọn dùng Python Hardware lịch sử **0.1.2**.
Lựa chọn simulator lịch sử là `legacy`; candidate native giám sát worker C++ ghim
hash. Đây là các luồng riêng biệt. Transport/thu nhận vật lý là **UNAVAILABLE**,
không được chứng minh bằng test virtual.

**Bằng chứng source:** dịch vụ loopback/local-socket cung cấp RPC canonical.
Xử lý an toàn dùng generation để ngăn thao tác cũ và chờ cleanup được xác nhận;
cleanup chưa xác nhận giữ nguyên fault/quyền sở hữu. Yêu cầu physical trả về
`OFFLINE`, `HARDWARE_UNAVAILABLE`, `hardware_applied=false`. Bản Python tham chiếu
bảo vệ bằng safety/topology mới nhất, chuẩn bị, routing, cấp nguồn và capture;
các lớp transport Serial/USB-SPI là placeholder chưa khả dụng. Test bao phủ xác
nhận cleanup, Start bị bỏ dở, thiết bị không khả dụng và quarantine khi thiếu xác
nhận safe.

**Tài liệu đã đọc:** [README.md](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/README.md) · [AGENTS.md](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/AGENTS.md) · [docs/en/architecture/station-boundaries.md](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/docs/en/architecture/station-boundaries.md) · [docs/en/contributing/verification.md](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/docs/en/contributing/verification.md).

**Manifest:** [Cargo.toml](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/Cargo.toml) · [integration/simulator-version.json](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/integration/simulator-version.json).

**Source đã đọc:** [native/src/main.rs](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/native/src/main.rs) · [native/src/station.rs](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/native/src/station.rs) · [hardware_service/station/physical.py](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/hardware_service/station/physical.py) · [hardware_service/fpga_transport.py](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/hardware_service/fpga_transport.py).

**Source test đã đọc, chưa chạy:** [native/src/station/tests.rs](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/native/src/station/tests.rs) · [native/tests/test_rpc_lifecycle.py](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/native/tests/test_rpc_lifecycle.py) · [tests/test_transport.py](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/tests/test_transport.py) · [tests/test_physical_station.py](https://github.com/cirnetlab/cirnet-hardware/blob/2139cccc8b36dc403c2f7182dfb670a97513858a/tests/test_physical_station.py).

## cirnet-simulator

**Sở hữu:** mô phỏng số/DC có giới hạn mang tính xác định và framing worker
CNP1/CNP2 riêng. Hardware sở hữu giám sát tiến trình và vòng đời station; Contracts
sở hữu định dạng circuit/capture canonical; Remote sở hữu điều phối REST.

**Quan sát:** source worker C++ native là **0.2.2**, với bố cục frame CNP1 và
CNP2 tách biệt với phiên bản worker/schema/system. Python **0.2.2.dev0** là luồng
tham chiếu/verification hiện tại; Python Simulator lịch sử được chọn là **0.1.2**.
`ngspice` là **UNAVAILABLE**, có zero kênh, không có sample và không fallback.
Hỗ trợ DC có giới hạn không có nghĩa backend SPICE đã được triển khai.

**Bằng chứng source:** worker giới hạn frame/event/sample và từ chối input sai
định dạng mà không tạo capture một phần. Test có chuyển trạng thái, giá trị logic
không xác định, DC có giới hạn CNP2, frame sai định dạng, giới hạn tách biệt, hủy và
tái sử dụng giao thức. Descriptor backend không khả dụng được kiểm tra bắt buộc
trong source test Python.

**Tài liệu đã đọc:** [README.md](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/README.md) · [AGENTS.md](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/AGENTS.md) · [docs/en/architecture/backends.md](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/docs/en/architecture/backends.md) · [docs/en/contributing/verification.md](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/docs/en/contributing/verification.md) · [native/README.md](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/native/README.md).

**Manifest:** [CMakeLists.txt](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/CMakeLists.txt) · [pyproject.toml](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/pyproject.toml) · [integration/contracts-version.json](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/integration/contracts-version.json).

**Source đã đọc:** [native/worker.cpp](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/native/worker.cpp) · [native/worker_limits.hpp](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/native/worker_limits.hpp) · [cirnet_sim/backends.py](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/cirnet_sim/backends.py).

**Source test đã đọc, chưa chạy:** [tests/test_cpp_worker.py](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/tests/test_cpp_worker.py) · [tests/test_backends.py](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/tests/test_backends.py) · [tests/native_cpp_backend_cases.py](https://github.com/cirnetlab/cirnet-simulator/blob/c05be23bef71de44a9027319af945d3e635d0882/tests/native_cpp_backend_cases.py).

## cirnet-fpga

**Sở hữu:** scaffold RTL Verilog, mô phỏng/testbench và permit điều khiển/an
toàn logic. Hardware sở hữu transport/driver station; Remote sở hữu điều phối hệ
thống.

**Quan sát:** phiên bản source **0.2.2**, so với scaffold được chọn **0.1.2**.
Đóng gói khai báo source-only, logical `1.0-scaffold`, register map
`1.0-reserved` và `physicalAvailable: false`. Firmware vật lý, transport, nạp
board, thu nhận, timing closure và mức độ sẵn sàng điện là **UNAVAILABLE hoặc chưa
được xác minh**, không được scaffold này chứng nhận.

**Bằng chứng source:** instrument availability, capture-valid và output enable
được nối cứng ở trạng thái tắt. safe_state của controller tắt routing/drive được
mô hình hóa. Scan readiness không chứng minh topology đúng. Decoder vật lý, đo
lường, ma trận chuyển mạch/mạch bảo vệ, chân board chính xác và giới hạn đã đo là
các input còn thiếu. Verification đòi hỏi hành vi mô phỏng thực tế và cả hai marker
PASS do owner quy định; chỉ compile/elaboration là chưa đủ. Provenance lưu lại là
hồ sơ chuyển đổi, không phải Git HEAD hay trạng thái worktree hiện tại.

**Discovery:** không có `AGENTS.md` ở root. Chỉ dẫn nằm trong
`fpga/AGENTS.md`; hãy đọc rõ ràng file đó khi làm FPGA, thay vì giả định client mở
ở repo root tự động nạp file lồng bên dưới.

**Tài liệu đã đọc:** [README.md](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/README.md) · [fpga/AGENTS.md](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/fpga/AGENTS.md) · [docs/en/scaffold.md](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/docs/en/scaffold.md) · [docs/en/verification.md](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/docs/en/verification.md) · [fpga/experiment-controller/README.md](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/fpga/experiment-controller/README.md) · [fpga/constraints/README.md](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/fpga/constraints/README.md) · [fpga/quartus/README.md](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/fpga/quartus/README.md).

**Manifest:** [VERSION](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/VERSION) · [integration/toolchain-lock.json](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/integration/toolchain-lock.json) · [integration/source-provenance.json](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/integration/source-provenance.json).

**Source đã đọc:** [fpga/experiment-controller/rtl/top.v](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/fpga/experiment-controller/rtl/top.v) · [fpga/instrument/rtl/top.v](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/fpga/instrument/rtl/top.v) · [scripts/build-source.mjs](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/scripts/build-source.mjs) · [scripts/check_rtl.py](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/scripts/check_rtl.py).

**Source test đã đọc, chưa chạy:** [fpga/simulation/tb_permit_matrix.v](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/fpga/simulation/tb_permit_matrix.v) · [tests/build-source.test.mjs](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/tests/build-source.test.mjs) · [tests/fpga/test_fpga_scaffold.py](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/tests/fpga/test_fpga_scaffold.py) · [tests/fpga/test_lab_first_control.py](https://github.com/cirnetlab/cirnet-fpga/blob/bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0/tests/fpga/test_lab_first_control.py).

## Ranh giới an toàn của miền sản phẩm

Trong Remote/Hardware/FPGA, **SAFETY != CORRECTNESS**: chỉ sai khác topology
không phải lý do chặn một thí nghiệm an toàn điện và có thể thực thi. Trạng thái
điện không xác định hoặc không an toàn không cho phép cấp nguồn. Không được thay
wiring quan sát được bằng wiring kỳ vọng để làm kiểm tra đạt. Đây là các phân biệt
của miền sản phẩm; đọc spec hiện tại của owner thay vì áp chúng lên mọi repository.

Toolchain, bộ cấp phát output, thư mục plan/spec, gate release, phiên bản schema,
số kênh và board mục tiêu riêng của từng repository vẫn thuộc local.
Bản đồ này không thay đổi các quy tắc đó hay cấp quyền ghi hoặc thao tác vật lý.

<!-- source: en/references/repository-map.md -->
<!-- source-sha256: 120a7bd7553864d03e2e66f60f885269e3fad883f55b1a91529d0fdd4bb17609 -->
