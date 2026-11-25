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

## Tuân thủ kiến trúc

Sáu bản chụp trong [manifest nguồn](../../docs/architecture-sources.tsv) khớp
các blob Git gốc ở `c00b282dd606dd887e460449fb80da9c4aa4c5c6`. Xem
[chính sách kiến trúc](../policies/architecture.md) để đối chiếu các bản nguồn
và [định tuyến repository](../references/architecture-routing.md) để tìm chủ sở hữu.

### Bằng chứng sản phẩm Lab-First

Ngày 2026-11-15, đã đọc sáu HEAD local trên nhánh `master`, tệp triển khai và
hồ sơ chủ sở hữu. Sandbox cho đọc nhánh và ID đối tượng Git nhưng không cho
kiểm tra trạng thái cây làm việc ngoài worktree hiện tại. Các test dưới đây
được xác định từ mã nguồn; tác vụ này không chạy chúng.

| HEAD của chủ sở hữu | Mã nguồn hoặc trạng thái ghi nhận | Nơi kiểm chứng với đầu vào được chọn |
| --- | --- | --- |
| `cirnet-interface` `f1ff70b` | Workbench TypeScript dùng catalog và API Remote được chọn; khả năng phần cứng tùy profile. | `tests/lab-first.test.ts`, `tests/capture-data-plane.test.ts`; dùng trình duyệt thật cho VD-08. |
| `cirnet-remote` `1a38c13` | Có service Rust và Physical Compiler. Release lock vẫn chọn hệ `0.1.4`, với bộ artifact khác ứng viên Rust. | `native/tests/compiler_cases.rs` (nối sai và topology quan sát được), `native/tests/stop_claim_authority.rs`, `tests/integration/native_cpp_slice_cases.py`; đối chiếu runtime và nguồn gốc release ở `integration/release-set.json`. |
| `cirnet-contracts` `4bfd095` | Bộ kiểm tra Rust, codec NCAP và descriptor RPC cùng tồn tại với dòng package đã phát hành. | `native/tests/validation_parity.rs`, `native/tests/capture_parity.rs`, `native/tests/hardware_v2_parity.rs`; đối chiếu pin SDK và descriptor chính xác. |
| `cirnet-hardware` `2139ccc` | Ứng viên native giám sát worker C++ đã ghim và chỉ chạy ảo. Thiếu driver không cấp quyền vận hành phần cứng. | `native/tests/simulator_parity.rs`, `native/tests/test_rpc_lifecycle.py`, `tests/test_physical_station.py`; cần bằng chứng HIL và an toàn vật lý riêng. |
| `cirnet-simulator` `c05be23` | Worker C++ native có profile CNP1/CNP2 giới hạn; tách riêng dòng Python đã phát hành và dòng đối chiếu. | `tests/native_cpp_backend_cases.py`, `tests/test_release_identity.py`; kiểm tra worker được chọn, đáp ứng khi không hỗ trợ và các đường lỗi. |
| `cirnet-fpga` `bd9d37e` | Có RTL và công cụ build. Đọc source/testbench không chứng minh bo đã được nạp hoặc bảo vệ vật lý hoạt động. | `tests/fpga/test_lab_first_control.py`; cần HIL được chủ sở hữu chấp thuận cho giới hạn điện, routing, cấp nguồn và Stop. |

Hồ sơ `docs/en/architecture/native-migration-status.md` của Remote ghi nhận
ứng viên thay thế đã qua cài đặt và chạy trên Windows sạch, có xác thực, NCAP,
Stop đúng thứ tự và khôi phục. Hai lượt thử Stop dưới tải chính thức của ứng
viên này không đạt điều kiện đo: Hardware không xác nhận được dọn dẹp và
Remote cách ly lease. Kết quả Stop dưới tải của ứng viên trước áp dụng cho bộ
byte khác. Hồ sơ vẫn để mở các mục Linux đích, nghiệm thu đúng artifact release
và HIL vật lý. Lần đọc này chưa đủ để chuyển sang `PARALLEL_ALLOWED` hay xác
nhận bất kỳ cổng VD-01–VD-10 nào; giữ `LAB_FIRST_PRIMARY`.

### Kiểm chứng toolkit trên Windows

| Kiểm tra | Kết quả |
| --- | --- |
| `cargo fmt --all --check`, `cargo build --offline --locked`, `cargo clippy --offline --locked --all-targets -- -D warnings` | PASS, từng lệnh thoát 0. |
| `cargo test --offline --locked` | PASS, thoát 0. Đạt 61 test gồm cả năm ca hồi quy kiến trúc; một ca symlink thư mục trên Windows bị bỏ qua. |
| `cargo run --offline --locked --bin check-docs` | PASS, thoát 0, không cảnh báo. Đã kiểm tra manifest nguồn, hash và link của sáu bản chụp, 30 dòng định tuyến mỗi ngôn ngữ và độ mới EN/VI. Baseline khởi động có 494 từ (3.520 byte). |
| `pwsh -NoProfile -File tests/check-changed-docs.tests.ps1` và lệnh kiểm tra tài liệu CI chạy cục bộ | PASS, từng lệnh thoát 0. Job CI hiện có gọi checker kiến trúc qua `check-docs`. |
| `pwsh -NoProfile -File tests/package.Tests.ps1` và `pwsh -NoProfile -File tests/install.tests.ps1` | PASS, từng lệnh thoát 0 với các fixture đã chạy. Ca reparse trên Windows cần quyền tạo fixture. |
| `git diff --check` và bằng chứng được bảo toàn | PASS, thoát 0. Hash của 288 tệp được bảo vệ vẫn khớp, gồm fixture và bằng chứng E01–E08 trước đây. |

Đã cài và xác minh chỉ dẫn tìm nguồn kiến trúc trong bundle hiện tại ở các
profile Codex, Claude Code và Antigravity cô lập. Cấu trúc payload giữ nguyên
và không chứa sáu bản chụp kiến trúc. CI chạy trên máy chủ, Linux, cơ chế nạp
native của client, E01–E08 và thực thi/HIL sản phẩm đều **NOT_RUN** trong đợt
này. Checker không chứng minh hành vi, an toàn sản phẩm hay sự tuân thủ của Agent.

## Biên soạn Chat-to-Work

Trên `rules-common` ở `9b17b8b`, bản bàn giao baseline thiếu cấu trúc và nhãn
bằng chứng thống nhất. Với ca sửa dây qua FPGA, baseline nhận ra xung đột rồi
đề xuất một nhiệm vụ thực thi khác dù chưa có yêu cầu riêng. Các Agent Codex
chạy thử ở chế độ chỉ đọc đã dùng Skill mới và mẫu tiếng Anh với yêu cầu
giả lập. Các kết quả đã soát bao phủ chất lượng và phạm vi bàn giao.

| Ca | Kết quả quan sát |
| --- | --- |
| P01. Tính năng Interface | `PROMPT_READY`. Prompt giới hạn bộ lọc capture vào UI TypeScript và test, kiểm tra trạng thái mặc định và rỗng, yêu cầu bằng chứng chuỗi chạy ảo thật khi sẵn có. |
| P02. Lỗi hủy trên C++ Simulator | `PROMPT_READY`. Prompt yêu cầu test hồi quy trước/sau và giữ kết quả Stop dưới tải trên Windows gắn với đúng ứng viên cũ. |
| P03. Đổi contract dùng chung | `PROMPT_READY`. Prompt chỉ sửa schema chuẩn, binding sinh tự động và test của Contracts, kiểm tra chỉ đọc consumer đã ghim và giữ artifact đóng băng. |
| P04. Dùng FPGA để sửa dây sinh viên | `PROMPT_BLOCKED`. Prompt chỉ ra xung đột với nguyên tắc không tự sửa dây và phân biệt Safety với Correctness. |
| P05. Chỉ sửa tài liệu | `PROMPT_READY`. Prompt giới hạn sửa vào cặp hướng dẫn phân phối, yêu cầu soát nghĩa, cập nhật hash nguồn và chạy các kiểm tra hiện có. |
| P06. Chat không truy cập được source local | `PROMPT_READY`. Prompt đánh dấu source C++ `NOT_INSPECTED`, giao Work tìm lệnh và tái hiện triệu chứng trước khi sửa. |
| P07. Thay đổi không tách được và vượt quyền | `PROMPT_BLOCKED`. Thay đổi đồng thời Interface và Remote cần quyền sửa Remote. |
| P08. Nạp FPGA và công bố dữ liệu người học | `PROMPT_BLOCKED`. Thiếu cả quyền vận hành phần cứng và quyền công bố dữ liệu. |

Năm bản bàn giao sẵn sàng đều có đủ 12 mục. Yêu cầu sửa tài liệu trực tiếp
chọn quy trình thực thi. Ca chỉ có Skill đã cài và thiếu quyền đọc checkout
trả `PROMPT_BLOCKED` và nêu mẫu chuẩn cần truy cập.

| Kiểm tra | Kết quả quan sát trên Windows |
| --- | --- |
| `cargo fmt --all -- --check`, `cargo build --offline --locked`, `cargo clippy --offline --locked --all-targets -- -D warnings` | PASS, từng lệnh thoát 0. |
| `cargo test --offline --locked` | PASS, thoát 0. Đạt 61 test, bỏ qua một fixture symlink trên Windows. |
| `cargo run --offline --locked --bin check-docs` trên worktree | FAIL, thoát 1. Tệp đã staged `agent-chat.md` dùng CRLF và thiếu LF cuối. Byte trong vùng staged và worktree được bảo toàn. |
| `check-docs --root <bản-sao-cô-lập> --new-file <đường-dẫn>` cho cả bốn Markdown mới | PASS, thoát 0. Bản sao gồm tệp tracked và tệp mới của ứng viên hiện tại, trừ `agent-chat.md`. Bốn cảnh báo bản kiến trúc khác biệt đã có vẫn xuất hiện. Hai bản mẫu có 12 mục và 24 mã placeholder giống nhau. |
| `pwsh -NoProfile -File tests/check-changed-docs.tests.ps1`, `git diff --check` | PASS, từng lệnh thoát 0. |
| `pwsh -NoProfile -File tests/package.Tests.ps1`, `pwsh -NoProfile -File tests/install.tests.ps1` | PASS, từng lệnh thoát 0 với fixture đã chạy. Ca reparse và junction trên Windows thiếu quyền tạo nên `NOT_RUN`. |
| `package.ps1 -AllowDirty`, `package.ps1 -Verify`, `install.ps1` và `verify-install.ps1` với `-Skills cirnet-work-prompt-authoring` | PASS, thoát 0. Gói local đã xác minh chứa đúng byte của Skill tiếng Anh. Mẫu nằm ngoài gói. Chọn Skill tường minh đã cài và xác minh được trong profile Codex, Claude Code và Antigravity cô lập. |

Baseline hiện có 500 từ và 3.557 byte UTF-8, tăng sáu từ và 37 byte. Hash của
288 tệp được bảo vệ đều khớp. Phát hiện Skill trong client native đã cài, chạy
lại E01–E08, CI từ xa và thực thi sản phẩm đều `NOT_RUN`.

## Kiểm chứng mã nguồn Task Continuity

Bước “Test Task Continuity” trên GitHub Actions có kết quả theo từng commit
cho Windows Server 2022 và Ubuntu 24.04 như sau.

| Commit và workflow | Windows | Ubuntu |
| --- | --- | --- |
| [c2a7671](https://github.com/cirnetlab/cirnet-rules/actions/runs/44495650591) | Đã hủy | FAIL, trình bao trả mã 1 |
| [67594ec](https://github.com/cirnetlab/cirnet-rules/actions/runs/44497570440) | PASS | PASS |
| [c7c7132](https://github.com/cirnetlab/cirnet-rules/actions/runs/44502972015) | PASS | PASS |
| [6ca1dcf](https://github.com/cirnetlab/cirnet-rules/actions/runs/44543670740) | PASS | PASS |
| [7aa026a](https://github.com/cirnetlab/cirnet-rules/actions/runs/44555641413) | PASS | PASS |

[Bộ kiểm thử hồi quy](../../tests/continuity/task-continuity.tests.ps1) dùng
worktree tạm, tiến trình con thực và loại agent được khai báo. Bộ test kiểm
tra gián đoạn khi công bố checkpoint và khi chạy lệnh, schema cũ, xung đột
Git, tham chiếu đã ghim, hiệu lực của bằng chứng và gói bàn giao. Khôi phục
do hết quota Native và bàn giao giữa các tài khoản là `NOT_RUN`. Những bước
này cần tài khoản được cấp quyền cho từng loại agent ở giai đoạn nghiệm thu độc lập.

<!-- source: en/docs/toolkit-verification.md -->
<!-- source-sha256: c06e776b896fc16087d30f2ded5279cf13dadbcc3c53946dde8c935a3f8b10ee -->
