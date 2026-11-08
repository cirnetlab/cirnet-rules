<!-- doc-id: AUDIT-CONTRACTS -->
<!-- revision: 1 -->
<!-- source-sha256: 5889b2d69398a881820241e67afa14afafd81b04bc85333a387ee6d8da7eca86 -->
<!-- translation-status: synchronized -->

# Rà soát sản phẩm cirnet-contracts

[English](../../../en/specs/cirnet-contracts/audit.md) · [Bằng chứng liên repo](../../references/product-audit.md)

<a id="snapshot"></a>

## Snapshot và định danh

| Trường | Bằng chứng |
| --- | --- |
| Repository | `https://github.com/cirnetlab/cirnet-contracts.git` |
| Thời điểm và phạm vi | 2026-11-08 UTC, kiểm tra chỉ đọc cục bộ trên Windows |
| Nhánh local và hosted | Local `master` khớp nhánh mặc định hosted `master`. Ref cache `origin/HEAD` trỏ tới `main`. |
| Commit SHA | `4bfd0950f564aaf24abaf6b8b82399c2d54d0e39` |
| Root tree SHA | `5365983a29060c009e07d504bc1c84616e517f2e` |
| Kiểm kê tracked | 334 tệp. SHA-256 của danh sách cây Git thô là `acfcb4331358571052686329f3f0b0c45c86d9f9ab4e633eabefda52d7414980`. Không có gitlink. |
| Cây làm việc | Sạch tại snapshot kiểm tra. |
| Tag local | v0.1.1 |
| Tag lịch sử hosted đã chọn | `v0.1.2` trỏ tới `524cc7a5e2e65f39077feefe70c7fb2ae3a773d6`. |
| Công bố đích | Git hosted không trả ref `v0.2.4` hoặc `v0.2.2`. Tra release công khai GitHub cho `v0.2.4` trả HTTP 404. Bản nháp riêng ngoài phạm vi quan sát. |

<a id="coverage"></a>

## Phạm vi tệp

Đã phân tích JSON trong `contracts/api/openapi.json` và `contracts/manifest.json`, đồng thời xem metadata bố cục capture. Đọc các đoạn về release và giao thức trong `docs/en/api/compatibility.md`, `docs/en/contract-reference.md`, `docs/en/fpga-protocol.md`, `docs/en/verification.md` và bản tiếng Việt tương ứng. Đã xem phụ thuộc và đường sinh trong `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `pyproject.toml`, `native/src/main.rs`, `native/build.rs`, `setup.py`, `scripts/package-native.mjs`, `contracts/codegen/generate_contracts.py` và hai workflow release. Đường dẫn giao thức gồm `contracts/hardware/v2/rpc_v2.proto`, `contracts/public/realtime.schema.json`, `contracts/circuit-schema/` và `contracts/fpga-protocol/`.

Kiểm kê ghi nhận các mô hình/binding sinh tự động, catalog, snapshot tương thích đóng băng, test migration và ví dụ thiết bị. Không đọc từng dòng generated, đối chiếu phân kiểu schema, chạy test, dựng lại wheel/SDK hoặc xác nhận mã bên thứ ba trong lượt này.

<a id="findings"></a>

## Phát hiện và tác động

| ID và trạng thái | Bằng chứng tại commit này | Tác động và xử lý |
| --- | --- | --- |
| AUD-CON-01 · OBSERVED · BLOCKER | Crate `cirnet-contracts-native`, gói Python và SDK npm chuẩn bị 0.2.2. Hệ thống Remote đã phát hành chọn Contracts 0.1.2. `rust-version` là 1.90 và toolchain ghim 1.99.1. | Các miền phiên bản có thể khác nhau hợp lệ. Lựa chọn sản phẩm 0.2.4 cần revision sạch và artifact chính xác. Giữ toolchain và MSRV độc lập. |
| AUD-CON-02 · VERIFIED · HIGH | Đã phân tích bytes OpenAPI đóng băng. OAS 3.1.0, `info.version` 0.1.2, 63 operation ID duy nhất và SHA-256 `dc39c874386eba2b58f8b5c631cbee0d63ded86cf287aea63f54e4a237ff8f08`. | Remote sở hữu ngữ nghĩa thao tác API. Contracts sở hữu handoff bảo toàn, schema và sinh SDK. Sinh SDK từ đúng bản xuất Remote đã rà soát, liên kết digest thô và digest gói. |
| AUD-CON-03 · OBSERVED · BLOCKER | `setup.py` sinh RPC binding Python khi build wheel/sdist. CI Python/Node cũng sinh schema và catalog, tạo gói Python cài được. `native/src/main.rs` có trình sinh Rust và `native/build.rs` biên dịch descriptor chuẩn. | Cô lập generator/wheel cũ làm bằng chứng đối chứng. Chứng minh chuỗi sinh và đóng gói do Rust dẫn dắt, Python giới hạn ở test/benchmark. |
| AUD-CON-04 · OBSERVED · HIGH | Protobuf Hardware chuẩn là `cirnet.hardware.v2`. CircuitGraph 1.0, 1.1 và 1.2, wire NCAP v1/v2, FPGA logic `1.0-scaffold` và bản đồ thanh ghi `1.0-reserved` độc lập với bản gói và OAS. | Thiết lập họ `0.2` bằng biểu diễn, revision và bằng chứng conformance dương/âm đã rà soát. Giữ byte gốc làm bằng chứng lịch sử. Không suy tương thích wire từ thứ tự số. |

<a id="readiness"></a>

## Mức sẵn sàng và bàn giao

**BLOCKED.** Đối chiếu API xuất từ Remote và đầu vào Contracts có thể phát hành trong quy trình duyệt trước. Tạo biên nhận Rust/tài nguyên/SDK chính xác từ revision nguồn sạch. Chứng minh parity trình sinh, số trường Protobuf được giữ chỗ, từ chối đầu vào sai, giới hạn, xác thực/MIME chuẩn và consumer conformance. Sau đó Remote và Hardware xác minh crate Rust đã chọn, còn Interface xác minh digest SDK. Bytes đóng băng và kiểm chứng trước đó là đầu vào đối chứng.
