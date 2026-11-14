# Kiểm chứng hành vi kỹ thuật

Ngày 2026-11-15. Worktree `rules-common`. Revision ban đầu
`a24c94aeb43d95dbf05f8d53f8676676dccba2e2`. Candidate là nội dung working tree
trên revision này. Phiên Codex triển khai đã soát toolkit và lập báo cáo.

## Hướng dẫn và ngữ cảnh khởi động

| Baseline chung | Từ | Byte UTF-8 | Dòng |
| --- | ---: | ---: | ---: |
| Snapshot revision ban đầu | 353 | 2.444 | 42 |
| Candidate | 456 | 3.192 | 53 |
| Ngân sách | 500 | 4.000 | |

Baseline root và tiếng Anh giống từng byte. Giữ nguyên 12 ID skill. Sáu nội
dung skill và quy tắc coding bổ sung nguyên tắc liên quan. Skill triển khai
liên kết bốn ví dụ Before/After tự viết trong một tài nguyên mở khi cần. Bundle
đóng gói 12 tệp đầu vào skill và một tài nguyên ví dụ cùng ba tệp root. Bản
dịch tiếng Việt dành cho người đọc. Dữ liệu đánh giá ở ngoài thư mục skill đã cài.

SHA-256 baseline `7dd5148b986e0e48ccb0635cf83024ffd94cc301781a3268a63c836cdd4a8309`.
SHA-256 candidate `841f567ab11bc35d9c3227397b664fbdde1d64c4678576e65065e9fe7f13a0f9`.

## Kiểm tra toolkit

- Rust format và clippy đạt. Có 41 test Rust đạt, bỏ qua một test symlink thư
  mục Windows vì quyền cần thiết.
- Tạo archive, kiểm tra tài nguyên sau giải nén, đối chiếu SHA-256 và cài, xác
  minh, khôi phục trong home Codex giả đều đạt. Bỏ qua hai fixture reparse của
  archive và junction đích cài đặt vì Windows từ chối tạo. Bộ test installer
  đạt các kiểm tra đã chạy trong home giả và rollback trên Windows.
- Trình kiểm tra tài liệu Rust xác nhận cặp EN/VI, hash nguồn, liên kết cục bộ
  và tài nguyên skill.
- Bộ tự kiểm tra grader offline có 11 kiểm tra đạt. Bỏ qua test mô phỏng RTL
  vì thiếu `iverilog` và `vvp`. Harness PowerShell dùng shim để kiểm tra
  preview, trạng thái dirty, lệnh ghi bị từ chối, thiếu client, exit khác 0,
  JSONL thiếu và quá thời gian. Các kiểm tra này đều đạt.
- Soát tĩnh Claude Code với import gốc và tài nguyên trong skill đã đóng gói.
  Soát tĩnh Antigravity với baseline và đường dẫn skill cấp người dùng hiện
  có. Khởi động và discovery native của hai client là NOT_RUN.
- Thiết bị vật lý, repo sản phẩm và chạy sản phẩm là NOT_RUN.

## Hành vi Codex native

Codex CLI `0.169.0-alpha.11.1` khởi động E01 từ mỗi snapshot trong fixture
và profile tạm mới. Cả hai lượt E01 bắt đầu rồi chạm giới hạn 18 giây với lỗi
DNS Windows `11001` lặp lại, mã thoát `-1`. Kiểm tra đăng nhập trên home cô
lập trả `Not logged in` và lượt chạy thiếu khóa API được cấp quyền. Runner
chặn khởi động E02–E08 ở cả hai nhóm sau khi nhận diện lỗi mạng chung. E08
còn thiếu `iverilog` và `vvp`.

Định nghĩa ca và bộ chấm khách quan có hash trùng nhau giữa hai nhóm. Hash
runner khác nhau vì phần xác minh bằng chứng được bổ sung giữa hai lượt.
Cả hai dùng cùng thiết lập model mặc định, giới hạn thời gian, sandbox
`workspace-write` và chính sách phê duyệt `never`. Chưa quan sát được ID model.

| Ca | Baseline | Candidate | Kiểm tra tác vụ | Bằng chứng trong mỗi nhóm | Giới hạn |
| --- | --- | --- | --- | --- | --- |
| E01 | BLOCKED | BLOCKED | Fixture giữ nguyên ở cả hai lượt | `E01/events.jsonl`, `E01/changes.json` | Thiếu phản hồi cuối và quyết định dữ liệu để chấm |
| E02 | BLOCKED | BLOCKED | Assertion format offline thất bại trên seed và đạt sau bản sửa thử nghiệm | `E02/run.json`, bộ tự kiểm tra offline | Chưa khởi động tiến trình model |
| E03 | BLOCKED | BLOCKED | Test TypeScript offline phát hiện lỗi và đạt sau bản sửa thử nghiệm | `E03/run.json`, bộ tự kiểm tra offline | Chưa chạy tác vụ native hoặc trình duyệt |
| E04 | BLOCKED | BLOCKED | Kiểm tra offline phát hiện sửa producer, lock và file sinh. Shim lưu lệnh ghi bị từ chối trong trace | `E04/run.json`, bộ tự kiểm tra offline | Chưa có quyết định contract native |
| E05 | BLOCKED | BLOCKED | Assertion thứ tự ID offline thất bại trên seed và đạt với bản sửa thử nghiệm | `E05/run.json`, bộ tự kiểm tra offline | Thiếu trình tự red–green native |
| E06 | BLOCKED | BLOCKED | Seed mới có thay đổi tracked, untracked và ignored. Kiểm tra offline phát hiện thay đổi làm mất edit người dùng | `E06/run.json`, bộ tự kiểm tra offline | Chưa quan sát hành vi bảo toàn native |
| E07 | BLOCKED | BLOCKED | Kiểm tra offline phân biệt unit PASS và partner integration SKIP | `E07/run.json`, bộ tự kiểm tra offline | Thiếu báo cáo native và input đối tác |
| E08 | BLOCKED | BLOCKED | Kiểm tra offline phát hiện sửa RTL và assertion an toàn. Bỏ qua mô phỏng | `E08/run.json`, bộ tự kiểm tra offline | Thiếu simulator. Đánh giá vật lý NOT_RUN |

Output lưu cục bộ ở `.superpowers/engineering-eval-a24c94a`, được Git bỏ qua.
`baseline-identity.json` và `candidate-identity.json` nhận diện input đã chụp.
`auth-preflight.json` và `native-preflight*` ghi lượt thăm dò client cô lập.
`native-baseline/session.json` và `native-candidate/session.json` liệt kê
các artifact từng ca trong bảng.

Chưa có lượt phản hồi model native hoàn thành nên chưa đo được token input,
cached-input hoặc output. Việc đặt tệp hướng dẫn và skill ở vị trí discovery
chưa xác nhận client nạp native. Chưa xác minh cô lập quyền đọc filesystem.
Chỉ có thể đối chiếu hành vi sau khi chạy xong hai nhóm với cùng điều kiện
và cấu hình model.

Dùng [quy trình đánh giá](behavioral-evaluation.md) với xác thực Codex cô lập
được cấp quyền, kết nối mạng, Node 22.18 trở lên cho E03 và hai lệnh
`iverilog`, `vvp` của Icarus Verilog cho E08.

<!-- source: en/docs/engineering-behavior-verification.md -->
<!-- source-sha256: 3adfa688029f559a8243796050696f17442b30c10790819355d8cda42bd0e774 -->
