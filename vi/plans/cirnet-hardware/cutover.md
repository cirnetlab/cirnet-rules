<!-- doc-id: CUT-HARDWARE -->
<!-- revision: 1 -->
<!-- source-sha256: c0ef2f33655d75ba664ab630433584b46021f73ea7f347026123215656a08e7c -->
<!-- translation-status: synchronized -->

<a id="cutover"></a>

# Đề xuất cutover cirnet-hardware

`CUT-HARDWARE` · `REL-01` · `PLANNED` · [Snapshot audit](../../specs/cirnet-hardware/audit.md)

## Phụ thuộc và ranh giới

Điều kiện trước [CUT-CONTRACTS](../cirnet-contracts/cutover.md), [CUT-SIMULATOR](../cirnet-simulator/cutover.md), [CUT-FPGA](../cirnet-fpga/cutover.md).
Mã nguồn ảnh hưởng `Cargo.toml`, `native/src/main.rs`, `native/src/station.rs`, `native/src/simulator/protocol.rs`, `pyproject.toml`.

Tích hợp crate Contracts và worker C++ chính xác. Giữ RPC cục bộ và vòng đời trạm có kiểm soát. Tách dịch vụ và wheel Python khỏi dependency sản phẩm đích. Từ chối peer cũ trước cấp điện.

## Nghiệm thu và khôi phục

Xác minh tương tác RPC/Preview và worker, khởi động fail-closed, quarantine và Stop chính thức dưới tải trên bộ đã chọn. Gate của owner `hardware_safe_stop`. Ghi nguồn, revision contract, digest artifact và phiên bản công cụ chính xác. Thay đổi producer cần owner sản phẩm cho phép. Chỉ phục hồi tổ hợp đầy đủ đã duyệt và dữ liệu khớp. Xem [chính sách release](../../policies/governance.md) và [ADR 0001](../../references/adr-0001.md).
