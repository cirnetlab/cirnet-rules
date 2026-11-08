<!-- doc-id: CUT-SIMULATOR -->
<!-- revision: 1 -->
<!-- source-sha256: ce6c4972fe5bfed4989f0d43c15abb2b5f4b2cec6f47ad8017fdd87ef89ea26d -->
<!-- translation-status: synchronized -->

<a id="cutover"></a>

# Đề xuất cutover cirnet-simulator

`CUT-SIMULATOR` · `REL-01` · `PLANNED` · [Snapshot audit](../../specs/cirnet-simulator/audit.md)

## Phụ thuộc và ranh giới

Điều kiện trước [CUT-CONTRACTS](../cirnet-contracts/cutover.md).
Mã nguồn ảnh hưởng `CMakeLists.txt`, `native/worker.cpp`, `pyproject.toml`, `cirnet_sim/backends.py`.

Giữ worker C++ CMake và biên IPC với Hardware. Tách mô hình Python tham chiếu khỏi sản phẩm được chọn, gỡ bộ chọn backend cũ và bí danh namespace khỏi gói sản phẩm.

## Nghiệm thu và khôi phục

Chứng minh hai đầu thống nhất framing và giới hạn, từ chối trước tác dụng phụ mô phỏng, waveform tất định, timeout và thoát worker. Gate của owner `simulator_worker`. Ghi nguồn, revision contract, digest artifact và phiên bản công cụ chính xác. Thay đổi producer cần owner sản phẩm cho phép. Chỉ phục hồi tổ hợp đầy đủ đã duyệt và dữ liệu khớp. Xem [chính sách release](../../policies/governance.md) và [ADR 0001](../../references/adr-0001.md).
