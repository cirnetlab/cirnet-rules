# Kiểm chứng nâng cấp toolkit

Ngày 2026-11-14. Baseline `35e677b1264958839a7e0cb883c4477eb24da6df`.
Branch worktree: `rules-common`. Checker baseline đạt. Cargo có 27 test
đạt và bỏ qua một test symlink thư mục trên Windows.

## Phần context do toolkit đóng góp

Đo byte file UTF-8, từ tách theo khoảng trắng và số dòng vật lý. Metadata
skill chỉ gồm giá trị name và description đã parse. Adapter Claude import
baseline một lần. File nguồn adapter có 11 byte.

| Văn bản toolkit | Trước | Candidate |
| --- | --- | --- |
| Baseline common độc lập | 5.811 byte / 833 từ / 91 dòng | 2.444 byte / 353 từ / 42 dòng |
| Giá trị metadata của ba skill mặc định | 353 byte | 302 byte |
| Giá trị metadata của toàn bộ skill | 353 byte / 3 skill | 1.390 byte / 12 skill |
| Danh mục dự phòng tùy chọn | Không có file riêng | 984 byte / 77 từ |

Candidate dùng một baseline Anh, metadata tùy chọn và thân skill đã chọn.
Đã đọc thử lựa chọn typo, API và FPGA trong [nạp context](context.md).
Context client, kết quả tokenizer và chọn trigger tự động cần quan sát native.
Bảng đo phần văn bản toolkit.

## Kiểm tra local

Môi trường local là Windows với PowerShell 7.6.9 và Rust 1.99.1.

| Kiểm tra | Kết quả |
| --- | --- |
| `cargo fmt --all --check` và `cargo clippy --offline --locked --all-targets -- -D warnings` | PASS |
| `cargo test --offline --locked` | 41 đạt, bỏ qua 1 ca symlink Windows |
| `cargo run --offline --locked --bin check-docs` | PASS, 45 cặp EN/VI và 12 skill, không có cảnh báo |
| `pwsh -NoProfile -File tests/package.Tests.ps1` | PASS các fixture đã chạy, không tạo được junction Windows |
| `pwsh -NoProfile -File tests/install.tests.ps1` | PASS các fixture đã chạy, không tạo được junction Windows. Rollback sau khi ghi một phần phục hồi byte và timestamp |
| `git diff --check` và parse script PowerShell | PASS |

Smoke bổ sung tạo ZIP development, giải nén vào path Unicode, xác minh 15
file payload trong allowlist và chạy preview, apply, verify, restore trên
home giả riêng của Codex, Claude và Antigravity. Đã thử tùy chọn Codex home
rõ ràng. Dữ liệu test ở output worktree được ignore. Review và test còn bao phủ
trạng thái receipt sai, chồng lấn bundle, link reference Markdown trùng và
input baseline nhị phân.

Tạo release sạch là NOT_RUN trong worktree candidate còn dirty. Test âm
từ chối thiếu hoặc sai revision và nguồn dirty. CI hosted, Linux, discovery
client thật, chạy sản phẩm hoặc phần cứng đều chưa chạy.
Candidate chưa có tag hoặc bản phát hành.

Xem [thiết lập](setup.md) cho ca smoke client thủ công và
[phân phối](distribution.md) cho lệnh bundle.

<!-- source: en/docs/toolkit-verification.md -->
<!-- source-sha256: bac59e329e35f2f67e58695b34094c42a40d33174793fb36ec94d5ba47b2640e -->
