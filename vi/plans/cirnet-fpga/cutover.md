<!-- doc-id: CUT-FPGA -->
<!-- revision: 1 -->
<!-- source-sha256: ec986cfe1a788e8b6a72bce15af774e09cb110f1e2084461a7a76f38de4221e4 -->
<!-- translation-status: synchronized -->

<a id="cutover"></a>

# Đề xuất cutover cirnet-fpga

`CUT-FPGA` · `REL-01` · `PLANNED` · [Snapshot audit](../../specs/cirnet-fpga/audit.md)

## Phụ thuộc và ranh giới

Điều kiện trước [CUT-CONTRACTS](../cirnet-contracts/cutover.md).
Mã nguồn ảnh hưởng `VERSION`, `scripts/build-source.mjs`, `fpga/experiment-controller/rtl/top.v`, `fpga/instrument/rtl/top.v`.

Giữ HDL và đóng gói mã nguồn. Thống nhất envelope logic và metadata thanh ghi với Contracts và Hardware. Giữ đường board vật lý là chưa khả dụng nếu thiếu bằng chứng riêng.

## Nghiệm thu và khôi phục

Chứng minh fixture logic đã duyệt và công cụ RTL khóa. Từ chối metadata đích không hỗ trợ và ghi rằng nguồn/mô phỏng chưa cung cấp capability vật lý. Gate của owner `fpga_scaffold`. Ghi nguồn, revision contract, digest artifact và phiên bản công cụ chính xác. Thay đổi producer cần owner sản phẩm cho phép. Chỉ phục hồi tổ hợp đầy đủ đã duyệt và dữ liệu khớp. Xem [chính sách release](../../policies/governance.md) và [ADR 0001](../../references/adr-0001.md).
