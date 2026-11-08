<!-- doc-id: CUT-CONTRACTS -->
<!-- revision: 1 -->
<!-- source-sha256: eca6de4389f5ed9bad459dfe6ca3680717ac9e77a5f976eeafefe19407cdbaf9 -->
<!-- translation-status: synchronized -->

<a id="cutover"></a>

# Đề xuất cutover cirnet-contracts

`CUT-CONTRACTS` · `REL-01` · `PLANNED` · [Snapshot audit](../../specs/cirnet-contracts/audit.md)

## Phụ thuộc và ranh giới

Phụ thuộc sản phẩm đầu tiên. Thống nhất thứ tự OpenAPI Remote và sinh SDK trước khi đóng băng artifact producer.
Mã nguồn ảnh hưởng `contracts/manifest.json`, `contracts/api/openapi.json`, `native/src/main.rs`, `setup.py`, `contracts/codegen/generate_contracts.py`.

Chốt biểu diễn family và provenance SDK từ bản xuất Remote đã duyệt. Gỡ phụ thuộc sinh/đóng gói Python trong production, giữ phần đối chứng ở test.

## Nghiệm thu và khôi phục

Sinh lại schema/binding bằng quy trình Rust, giữ bytes đóng băng, chứng minh operation ID duy nhất và ca wire thuận/nghịch đa ngôn ngữ. Gate của owner `contract_conformance`. Ghi nguồn, revision contract, digest artifact và phiên bản công cụ chính xác. Thay đổi producer cần owner sản phẩm cho phép. Chỉ phục hồi tổ hợp đầy đủ đã duyệt và dữ liệu khớp. Xem [chính sách release](../../policies/governance.md) và [ADR 0001](../../references/adr-0001.md).
