<!-- doc-id: VERIFY-LOCAL -->
<!-- revision: 1 -->
<!-- source-sha256: 2e16ed3d24550b5197c8bcaa648adac481bffcbc0ee5ca2f3ad71bdaa17992f1 -->
<!-- translation-status: synchronized -->

<a id="evidence"></a>

# Bằng chứng kiểm chứng

Commit gốc checkout là `a21d372c667ecdb1963dac4dbabc30df816c2e87`. Governance
đang sửa local, nên commit này nhận dạng gốc, chưa phải bản phát hành bất biến
của triển khai. Dùng Windows x64, Rust và Cargo 1.99.1, Git 2.52.0. Kiểm tra
sản phẩm chỉ đọc.

## Các lượt chạy local ngày 2026-11-08 UTC

Chạy từ thư mục gốc governance. Alias toolchain local là `stable`, đã xác minh
Rust và Cargo đều là 1.99.1. Repo và CI pin bản phân phối theo số phiên bản.
Trên máy local, biến `RUSTUP_TOOLCHAIN` trong tiến trình chọn alias đã cài.

| Lệnh hoặc kiểm chứng | Mã thoát và quan sát |
| --- | --- |
| `cargo +stable fmt --all --check` | 0. Định dạng khớp. |
| `cargo +stable clippy --locked --offline --all-targets -- -D warnings` | 0. Qua lint nghiêm ngặt. |
| `cargo +stable build --locked --offline` | 0. Build được CLI. |
| `cargo +stable test --locked --offline` | 0. Qua 16 unit test và 3 test tích hợp CLI. |
| `validate` và `check-translations` | 0. Registry hợp lệ, 49 cặp, liên kết nội bộ, anchor, phiên bản và hash nguồn khớp. |
| `context --workspace .. --repo cirnet-hardware --task audit --risk hardware` | 0. Context tiếng Anh đúng phạm vi, có quy trình rà soát an toàn phần cứng. |
| `check-drift --workspace ..` | 0. Sáu snapshot sạch, hồ sơ audit và định danh Git sản phẩm khớp. |
| `check-versions --workspace ..` | 2, `BLOCKED`. Mã nguồn quan sát và release được chọn còn cần cutover. |
| `check-release --workspace ..` | 2, `BLOCKED`. Xem [kết quả JSON](../../en/references/evidence/check-release-blocked.json), SHA-256 `0d4cd3f52cbfe410e0e7a16588fc26dc1ae749dbfcc912a27fed5a58934d9a4a`. |
| `check-release` với workspace rỗng | 2, `BLOCKED`. |
| `doctor --workspace ..` | 0 với alias 1.99.1 đã cài. 2 khi shell local thiếu toolchain dưới tên số đã pin. Dò công cụ không mở phiên Agent. |
| `bootstrap` trong phiên cô lập | 21 lượt CLI đạt, bảy lượt mỗi Agent. Dry-run, cài, cài lặp, rollback, gỡ lặp, khôi phục và rollback cuối. Còn ba bản sao lưu, không có gói đang hoạt động. |

Xem [audit sản phẩm](product-audit.md) cho snapshot, lệnh và giới hạn, và
[ma trận adapter](../adapters/README.md) để phân biệt kiểm thử di động với
discovery trong harness. CI chạy trên GitHub, kiểm thử board, build sản phẩm,
phê duyệt ngữ nghĩa và kích hoạt release là các gate độc lập.
