<!-- doc-id: CUT-INTERFACE -->
<!-- revision: 1 -->
<!-- source-sha256: 8fe1369224ff94aceb7095e273c3a42af73a58f0a7c342a76a702139379b7970 -->
<!-- translation-status: synchronized -->

<a id="cutover"></a>

# Đề xuất cutover cirnet-interface

`CUT-INTERFACE` · `REL-01` · `PLANNED` · [Snapshot audit](../../specs/cirnet-interface/audit.md)

## Phụ thuộc và ranh giới

Điều kiện trước [CUT-CONTRACTS](../cirnet-contracts/cutover.md).
Mã nguồn ảnh hưởng `package.json`, `package-lock.json`, `integration/compatibility.json`, `scripts/bootstrap-contracts.mjs`, `src/services/api/client.ts`.

Giữ Vue/TypeScript/Vite. Build từ SDK Contracts và bản xuất Remote native đã duyệt. Gỡ tương thích response/route cũ hiện hành sau khi xử lý dữ liệu được duyệt.

## Nghiệm thu và khôi phục

Xác minh operation ID ổn định, phạm vi và luồng trình duyệt, từ chối peer cũ và đối soát lỗi không phát lại lệnh ghi. Gate của owner `browser_native`. Ghi nguồn, revision contract, digest artifact và phiên bản công cụ chính xác. Thay đổi producer cần owner sản phẩm cho phép. Chỉ phục hồi tổ hợp đầy đủ đã duyệt và dữ liệu khớp. Xem [chính sách release](../../policies/governance.md) và [ADR 0001](../../references/adr-0001.md).
