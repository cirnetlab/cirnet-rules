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

## Kiểm chứng rà soát Markdown dư thừa

Ngày 2026-11-15, worktree `rules-common` tại `5d6cc26` mở rộng checker và vòng
đời tài liệu hiện có. Fixture trùng lặp đầu tiên không đạt với checker cũ
do thiếu cảnh báo. Sau khi sửa, cả tám test đều đạt, bao phủ các mức trùng
lặp, rà soát tệp mới trong phạm vi, bản sao có chủ đích, bằng chứng lịch sử
và bảo toàn từng byte của fixture.

| Lệnh | Kết quả quan sát |
| --- | --- |
| `cargo fmt --all --check` | Exit 0 |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Exit 0 |
| `cargo test --offline --locked` | Exit 0. 49 ca đạt, một ca symlink thư mục Windows bị bỏ qua. |
| `cargo run --offline --locked --bin check-docs` | Exit 0. Checker không phát cảnh báo và không tìm thấy cặp ứng viên trên checkout này. |
| `pwsh -NoProfile -File tests/package.Tests.ps1` | Exit 0 cho các ca đóng gói và home giả đã chạy. Fixture reparse Windows là NOT_RUN do bị từ chối quyền tạo. |
| `pwsh -NoProfile -File tests/install.tests.ps1` | Exit 0 cho các fixture cài đặt đã chạy. |
| `git diff --check` | Exit 0 |

Lần đầu chạy Cargo song song gặp khóa tệp thực thi trên Windows và trả 101.
Lượt tuần tự chạy lại cùng mã nguồn. Phép thử `--new-file` dùng hai đường dẫn
hiện có để kiểm tra cách gọi CLI và trả 0. Chính sách tài liệu nhận
`NEW_DOCUMENT_REVIEW` vì chưa có ứng viên, còn mẫu spec nhận
`KEEP_SEPARATE`. Đây là gợi ý rà soát.

Điểm tương đồng đo phần chữ trùng nhau trong phạm vi quét. Người bảo trì
xác định ý nghĩa và chủ sở hữu. Bằng chứng nằm tại
`.superpowers/markdown-hygiene-5d6cc26/`. Lần này kiểm chứng checker và
fixture của bundle local. E01–E08 là NOT_RUN cho thay đổi này và hash
bằng chứng đã lưu vẫn khớp.

## Kiểm chứng đợt hoàn thiện

Ngày 2026-11-15, thay đổi trên `rules-common` dựa trên `66d2c8a` siết mức lỗi
trùng lặp, bổ sung chọn tệp trong CI và mở rộng baseline lên 491 từ, 3.445 byte
UTF-8. Bảy ca hồi quy tập trung tái hiện hành vi cũ. Sau khi sửa, cả 15 ca
hygiene đều đạt. Checker mặc định không phát cảnh báo trên checkout.

| Kiểm tra | Kết quả quan sát trên Windows |
| --- | --- |
| `cargo fmt --all --check`, `git diff --check` | Exit 0 |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Exit 0 |
| `cargo test --offline --locked` | Exit 0. 56 ca đạt và một ca symlink thư mục Windows bị bỏ qua. |
| `cargo run --offline --locked --bin check-docs` | Exit 0. Liên kết, hash EN/VI và bản sao baseline đều đạt. |
| `pwsh -NoProfile -File tests/check-changed-docs.tests.ps1` | Exit 0. Script truyền tệp thêm đã commit, giữ nguyên tên tệp, bỏ qua đổi tên được nhận diện, trả lỗi trùng và báo quét thường khi base không dùng được. |
| `tests/package.Tests.ps1` và `tests/install.tests.ps1` | Exit 0 với các fixture đã chạy. Tạo reparse và junction bị từ chối. |

Bằng chứng nằm tại `.superpowers/markdown-hardening-66d2c8a/`. Hash của 290
tệp đầu vào và bằng chứng được bảo vệ vẫn khớp, cùng mọi branch ref.
Hosted CI, Linux, nạp Agent native và E01–E08 là NOT_RUN cho thay đổi này.
Script Git cần một commit tổ tiên được xác nhận cục bộ. Điểm tương đồng vẫn
cần rà soát mục đích và chủ sở hữu.

<!-- source: en/docs/toolkit-verification.md -->
<!-- source-sha256: 07384198cc8b3972dfc6adeb6d02f19f1044ad0af6b422965d8b428dcd74db0b -->
