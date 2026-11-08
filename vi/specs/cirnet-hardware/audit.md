<!-- doc-id: AUDIT-HARDWARE -->
<!-- revision: 1 -->
<!-- source-sha256: 9b7d97635d1a2aaa36ba6bb3d10e1e0be91109ff3bb985c3a21df47f7724f803 -->
<!-- translation-status: synchronized -->

# Rà soát sản phẩm cirnet-hardware

[English](../../../en/specs/cirnet-hardware/audit.md) · [Bằng chứng liên repo](../../references/product-audit.md)

<a id="snapshot"></a>

## Snapshot và định danh

| Trường | Bằng chứng |
| --- | --- |
| Repository | `https://github.com/cirnetlab/cirnet-hardware.git` |
| Thời điểm và phạm vi | 2026-11-08 UTC, kiểm tra chỉ đọc cục bộ trên Windows |
| Nhánh local và hosted | Local `master` khớp nhánh mặc định hosted `master`. Ref cache `origin/HEAD` trỏ tới `main`. |
| Commit SHA | `2139cccc8b36dc403c2f7182dfb670a97513858a` |
| Root tree SHA | `e59bab4b1750b19e203ca36e0aa9a0a761e47268` |
| Kiểm kê tracked | 91 tệp. SHA-256 của danh sách cây Git thô là `b0d52180ef62b38738db4a3e43e97105d679a03824b885c147f6accfbbf8a80c`. Không có gitlink. |
| Cây làm việc | Sạch tại snapshot kiểm tra. |
| Tag local | v0.1.1, v0.1.2 |
| Tag lịch sử hosted đã chọn | `v0.1.2` trỏ tới `9a8935b6a596abe2d21ce90416ba3b914e9f36be`. |
| Công bố đích | Git hosted không trả ref `v0.2.4` hoặc `v0.2.2`. Tra release công khai GitHub cho `v0.2.4` trả HTTP 404. Bản nháp riêng ngoài phạm vi quan sát. |

<a id="coverage"></a>

## Phạm vi tệp

Đã xem metadata phụ thuộc trong `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `pyproject.toml` và `integration/`. Đọc các phần entry/readiness, trạng thái trạm, quản lý worker và khung trong `native/src/main.rs`, `native/src/station.rs`, `native/src/simulator.rs`, `native/src/simulator/protocol.rs`, `native/preview.proto` và `native/tests/simulator_parity.rs`. Đối chiếu các bước đóng gói/build của workflow, `scripts/check_installed.py` và tài liệu Anh/Việt về ranh giới trạm và verification. Phần Python tracked `hardware_service/` và fixture vật lý hiện có được kiểm kê. Không vận hành trạm, controller, lease, worker, firmware hoặc giao vận vật lý.

<a id="findings"></a>

## Phát hiện và tác động

| ID và trạng thái | Bằng chứng tại commit này | Tác động và xử lý |
| --- | --- | --- |
| AUD-HW-01 · OBSERVED · BLOCKER | Crate/executable Rust là 0.2.2. Phụ thuộc build và runtime chọn Contracts `=0.1.2` tại `e640f80b1199779659d3db7817fee16d13cc24c1`. `pyproject.toml` dùng bản đối chứng cô lập `0.2.2.dev0`, ghim Contracts và Simulator 0.1.2 đã phát hành. | Cho đợt 0.2.4, chọn đúng byte artifact từ nguồn sạch, crate/schema đã rà soát và worker tương thích. Chỉ giữ luồng đối chứng làm bằng chứng ngoài sản phẩm. |
| AUD-HW-02 · OBSERVED · HIGH | RPC cục bộ chuẩn là `cirnet.hardware.v2.HardwareService`, định danh RPC 2.0. `cirnet.hardware.internal.v1.SimulatorPreview` riêng có trường Protobuf 1 cho phản hồi Describe unsigned và bytes JSON ở request/response. Worker dùng CNP1/CNP2. Digest sản phẩm được kiểm riêng. | Rà soát biểu diễn RPC và IPC chính xác trong họ 0.2. Giữ cú pháp protobuf, không tái sử dụng trường đã bỏ và không coi ABI/tag wire là SemVer. |
| AUD-HW-03 · OBSERVED · HIGH | Native chấp nhận backend mô phỏng C++. Nhánh readiness TCP chuẩn bị Preview trước. Nguồn quản lý cấp quyền, generation, hủy và dọn dẹp, có thể từ chối xác nhận safe state. Hướng dẫn verification tách hỗ trợ vật lý khỏi bằng chứng trạm ảo. | Xác minh từ chối trước cấp điện, khởi động có bảo vệ, watchdog, Stop/ack và cách ly. Giữ tách biệt correctness của mạch, an toàn và bằng chứng điện. |
| AUD-HW-04 · VERIFIED · HIGH | `git diff --shortstat v0.1.2 HEAD -- hardware_service pyproject.toml setup.py MANIFEST.in scripts/check_installed.py` trả năm đường dẫn thay đổi, gồm `rpc.py` và mã trạm đóng gói. ZIP `METADATA` của wheel được chọn ghi `cirnet-hardware-service` 0.1.2. `.github/workflows/hardware-service.yml` tại HEAD chỉ có quyền đọc và build wheel `0.2.2.dev0` chưa phát hành để kiểm chứng. | Đường build HEAD đã đọc tách nguồn thay đổi khỏi định danh 0.1.2 đã phát hành. Không thấy bước công bố wheel trong workflow đã rà. Giữ nguyên định danh lịch sử và yêu cầu định danh producer mới được duyệt nếu sau này cho phép artifact khác. |

<a id="readiness"></a>

## Mức sẵn sàng và bàn giao

**BLOCKED.** Phụ thuộc Contracts và Simulator. Gắn chính xác cặp Rust/C++ bằng biên nhận đã rà soát, kiểm cả hai khung IPC và phiên bản không hỗ trợ, rồi xác minh tích hợp với Remote. Cần bằng chứng Stop dưới tải, quyền sở hữu/cách ly đúng đích và HIL trước khi công bố khả năng vật lý. Nguồn và báo cáo parity trước đó chỉ xác định thiết kế và phạm vi ứng viên.
