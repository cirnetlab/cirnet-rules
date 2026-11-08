<!-- doc-id: CUT-REMOTE -->
<!-- revision: 1 -->
<!-- source-sha256: e5d03e4e5de4275de58d736c5e8b8db9f7d5397df8bbf24b503b587e2e783342 -->
<!-- translation-status: synchronized -->

<a id="cutover"></a>

# Đề xuất cutover cirnet-remote

`CUT-REMOTE` · `REL-01` · `PLANNED` · [Snapshot audit](../../specs/cirnet-remote/audit.md)

## Phụ thuộc và ranh giới

Điều kiện trước [CUT-CONTRACTS](../cirnet-contracts/cutover.md), [CUT-HARDWARE](../cirnet-hardware/cutover.md), [CUT-INTERFACE](../cirnet-interface/cutover.md), [CUT-SIMULATOR](../cirnet-simulator/cutover.md), [CUT-FPGA](../cirnet-fpga/cutover.md).
Mã nguồn ảnh hưởng `Cargo.toml`, `integration/release-set.json`, `native/src/openapi.rs`, `native/src/websocket.rs`, `services/api/pyproject.toml`, `scripts/install_release.py`.

Sở hữu operation công khai, lập lịch, lưu trữ và một release-set. Thay Python production và logic release sản phẩm theo chính sách công cụ Rust. Dùng migration offline hoặc dữ liệu mới tường minh rồi chọn artifact sạch chính xác.

## Nghiệm thu và khôi phục

Diễn tập provenance chính xác, trình duyệt native, Windows sạch, rollback và từ chối legacy. Dừng an toàn nếu thiếu bộ rollback đã duyệt. Gate của owner `clean_windows`. Ghi nguồn, revision contract, digest artifact và phiên bản công cụ chính xác. Thay đổi producer cần owner sản phẩm cho phép. Chỉ phục hồi tổ hợp đầy đủ đã duyệt và dữ liệu khớp. Xem [chính sách release](../../policies/governance.md) và [ADR 0001](../../references/adr-0001.md).
