# Kế hoạch bằng chứng độ tin cậy CIRNET

Kế hoạch triển khai [đặc tả bằng chứng độ tin cậy](../../../specs/cirnet-rules/reliability-evidence/spec.md)
trong checkout `cirnet-rules` trên `master`. Khi tiếp nhận đã xác minh mốc
`15175dd1faaf8b7f56f0eb93124417dc200a892e` và cây làm việc sạch.
Sau đó người vận hành cấp quyền triển khai trực tiếp trong checkout này.

## Giai đoạn 0 rà soát năng lực

Đường dẫn dưới đây tính từ gốc checkout của chủ sở hữu trừ khi có ghi chú.
Trạng thái mô tả baseline đã xem. Các test được nêu xác định phạm vi kiểm
thử. Các quan sát ghi ở dưới là bằng chứng thực thi của lần rà soát.

| Năng lực | Triển khai và chủ sở hữu tại baseline | Hành vi bao phủ và bằng chứng | Khoảng trống và bước xử lý giới hạn |
| --- | --- | --- | --- |
| Khám phá môi trường | Hướng dẫn Rules tại `en/docs/context.md`. Manifest và hướng dẫn kiểm chứng của chủ sở hữu. Đã đọc `package.json` của Interface | `GUIDANCE_ONLY` khi chọn phép kiểm của chủ sở hữu. Tên trong manifest là ứng viên và quyền được xác minh riêng | Bổ sung xem trước có giới hạn cho nguồn npm, Cargo, CMake và RTL giả lập. Chỉ đọc tệp được chỉ định và trả metadata đã chọn. Giữ các lệnh ở trạng thái chưa chạy |
| Build và chạy | `scripts/task-continuity.ps1 run` và `scripts/continuity/Verification.psm1` của Rules | `IMPLEMENTED_VERIFIED` cho ranh giới gọi trực tiếp lệnh giả lập. Wrapper theo dõi pin, timeout, dọn dẹp và mã thoát tiến trình quan sát được | Mã thoát ngoài còn thiếu bằng chứng bước con và độ bao phủ assertion. Giữ việc thực thi và thu bằng chứng của chủ sở hữu theo quyền cấp riêng |
| Bằng chứng test | `scripts/benchmark/role.ps1`, `scripts/benchmark/report.mjs` và `tests/benchmark/rtl.tests.ps1` của Rules. Chủ sở hữu xác định assertion | `PARTIAL`. Bộ rà kiểm mã thoát lệnh và marker bước con RTL. Phép thử wrapper trực tiếp bên dưới cho thấy thiếu bằng chứng cho các bước thông thường | Dùng quan sát chủ sở hữu đã ghim để xác định bằng chứng bắt buộc và tùy chọn. Giữ nguyên các ca, runner và báo cáo benchmark |
| Phân tích log | `scripts/benchmark/role.ps1`, `scripts/benchmark/inspect-source.mjs` và `tests/benchmark/privacy.tests.ps1` của Rules | `IMPLEMENTED_NOT_VERIFIED` với đọc có giới hạn và phát hiện canary trong benchmark. Tìm kiếm mã xác định phạm vi cấu hình | Giữ đầu ra thô ở chủ sở hữu. Dùng lại bản tóm tắt có giới hạn. Kết quả canary phát hiện lộ dữ liệu và cần giới hạn phòng ngừa riêng |
| Tái hiện | `en/skills/cirnet-debugging/SKILL.md`, `tests/benchmark/checks.test.mjs` của Rules và contract fixture được chọn | `PARTIAL`. Có fixture và đã tái hiện riêng ba giới hạn wrapper bên dưới | Chuyển quy tắc tuyên bố còn thiếu thành fixture âm tính có tính xác định. Giữ assertion baseline |
| Tự sửa | `en/skills/cirnet-debugging/SKILL.md` và `en/skills/cirnet-regression-testing/SKILL.md` của Rules | `GUIDANCE_ONLY` cho giới hạn sửa theo cụm lỗi. Thiếu phép kiểm xác định đối với lịch sử thử lại | Đánh giá chính sách thử lại dựa trên lịch sử quan sát. Giữ quyền và thao tác ở chủ sở hữu, bảo toàn kỳ vọng và quyền đã cấp |
| UI | `en/skills/cirnet-engineering-workspace-ui/SKILL.md` của Rules. `package.json` của Interface và kiểm chứng chủ sở hữu tạo thành luồng sản phẩm | `GUIDANCE_ONLY` tại Rules. Đã tìm thấy đầu mục build, test, boundary và E2E của Interface. Lệnh sản phẩm chưa chạy | Giữ thẩm quyền tính năng và phân biệt rà tĩnh, trình duyệt mock và bằng chứng sản phẩm đã cài |
| Hiệu năng | `scripts/benchmark/role.ps1` của Rules ghi thời lượng Candidate và mức dùng đầu vào tùy chọn có cấp quyền. Chủ sở hữu sản phẩm kiểm soát tải so sánh | `IMPLEMENTED_NOT_VERIFIED` cho đo lường native. Baseline sản phẩm có thể so sánh là `NOT_MEASURED` | Đo khi có luồng chủ sở hữu đã ghim. Fixture này chưa cho kết quả cải thiện hiệu năng hoặc hiệu quả |
| Kiến trúc và phạm vi | `en/policies/architecture.md`, `en/references/architecture-routing.md`, `hooks/check_docs.rs` và `scripts/shared/WorkPrompt.psm1` của Rules | `IMPLEMENTED_NOT_VERIFIED` khi tiếp nhận đối với toàn vẹn tài liệu và phạm vi bàn giao đã ghim. Đối chiếu phát hiện thay đổi ròng | Dùng preflight và đường dẫn bằng chứng chủ sở hữu đã duyệt. Môi trường thực thi áp dụng truy cập và cô lập |
| Hoàn tất | `scripts/continuity/Verification.psm1`, `scripts/benchmark/report.mjs` và `en/rules/testing.md` của Rules | `PARTIAL`. Phép thử trực tiếp quan sát tiến trình lỗi, đầu vào cũ và công cụ thiếu. Candidate vẫn có bước rà soát độc lập | Đánh giá tuyên bố chính xác và xác định mọi bước bắt buộc còn thiếu. Giữ mô hình nghiệm thu hiện có và yêu cầu bằng chứng mới đúng phạm vi |

### Khoảng trống đã quan sát

Quan sát tiến trình trực tiếp của Task Continuity trả `PASS` và mã thoát
ngoài `0` trong ba ca giả lập. Một ca gọi tiến trình con lồng nhau thoát
`7`, một ca khai báo bước bắt buộc bị bỏ qua vì lý do giả lập, và một ca
thiếu bằng chứng mã thoát con. Các ca cho thấy khác biệt giữa mã thoát
tiến trình và độ bao phủ tuyên bố. Các phép thử trực tiếp độc lập ghi
nhận `FAIL` khi mã thoát là `17`, `NOT_RUN` khi hết thời gian và thiếu mã
thoát, `BLOCKED` khi công cụ không khả dụng, và `NOT_RUN` khi dấu vân tay cũ.

### Quyết định tái sử dụng

Lệnh phía chủ sở hữu và CI hiện có chọn và chạy các phép kiểm được duyệt.
Task Continuity ghi thực thi trực tiếp và độ mới. Kiểm tra bàn giao và rà
Candidate đã phụ trách quyền và đánh giá độc lập. Phần còn thiếu là phép
kiểm dữ liệu hẹp cho độ bao phủ bước, xem trước rõ và chấp nhận lượt sửa.
Chi phí gồm bộ đọc có giới hạn, phân loại nhỏ và fixture tập trung. Bộ
thực thi hoặc điều phối dùng chung sẽ lặp trách nhiệm vòng đời và quyền
hiện có, đồng thời tăng phần phải kiểm thử và bảo trì. Mỗi phép kiểm mới
trả kết quả theo phạm vi.

## Triển khai và kiểm chứng

Các lệnh dưới đây chạy từ gốc `cirnet-rules`.

- [x] **Giai đoạn 0.** Xem thẩm quyền hiện tại và tái hiện ba quan sát
  wrapper. Gắn khoảng trống với phạm vi đã quan sát.
- [x] **Giai đoạn 1.** Bổ sung `scripts/verify-claim.ps1` dùng bộ đọc giới
  hạn, hàm băm và Git hiện có. Yêu cầu hồ sơ chủ sở hữu đã ghim, đầu vào
  hiện tại và bằng chứng bước thực tế. Giữ engine và schema Continuity.
  `./tests/verification/verify-claim.tests.ps1` trả mã thoát `1` trước khi
  có điểm vào và `0` sau triển khai. Fixture bao phủ mã thoát con lồng,
  bỏ qua bước bắt buộc và tùy chọn, bước thiếu điều kiện hoặc hết thời gian,
  pin cũ, mức assertion tối thiểu và hồ sơ quan sát sai định dạng.
- [x] **Giai đoạn 2.** Thêm `scripts/preview-owner-checks.mjs`. Test npm,
  Cargo, CMake và RTL giả lập đạt, giữ nguyên tệp fixture và chưa chạy
  lệnh được chỉ định. Fixture liên kết thư mục bị bỏ qua do quyền trên
  Windows. Xem trước chỉ đọc manifest Cargo thật chọn đúng checker và giữ
  thực thi ở `NOT_RUN`.
- [x] **Giai đoạn 3.** Thêm module tư vấn thử lại. Sáu test đạt, gồm hai
  lần quan sát tiến trình con thoát `7` rồi từ chối lần chạy lại không đổi
  tiếp theo, hai lần sửa đã khởi chạy, và từ chối khi thiếu phạm vi, an
  toàn hoặc tiến triển. Module trả quyết định và không chạy lệnh.
- [x] **Giai đoạn 4.** Nối test mới vào ma trận Windows/Ubuntu hiện có.
  Chạy các lane liên quan cục bộ và xác minh tài liệu song ngữ, gói phát
  triển và phạm vi. Bảng ghi kết quả quan sát trên Windows.

| Lệnh hoặc lane | Kết quả cục bộ quan sát được |
| --- | --- |
| `./tests/verification/verify-claim.tests.ps1` | Thoát 0. Kiểm tra wrapper 0 với tiến trình con thất bại hoặc bỏ qua bước bắt buộc, thiếu quan sát, thiếu assertion, pin cũ, thiếu điều kiện hoặc PASS hợp lệ. |
| `node --test tests/verification/preview-owner-checks.test.mjs tests/verification/retry-policy.test.mjs tests/benchmark/source.test.mjs tests/benchmark/rtl.test.mjs` | Thoát 0. 19 đạt và một fixture liên kết thư mục bị bỏ qua. |
| `cargo fmt --all -- --check`, `cargo test --offline --locked`, `cargo clippy --offline --locked --all-targets -- -D warnings` | Mỗi lệnh thoát 0. 61 test Rust đạt, một ca symlink Windows bị bỏ qua. |
| `cargo run --offline --locked --bin check-docs` với bốn đường dẫn Plan/Spec mới qua `--new-file` | Thoát 0. Đã kiểm tra cặp EN/VI, hash, liên kết và cả 17 định nghĩa Skill. |
| `./tests/continuity/task-continuity.tests.ps1`, `./tests/continuity/bootstrap.tests.ps1`, `./tests/validation/work-prompt.tests.ps1` | Mỗi lệnh thoát 0. Windows bỏ qua fixture symlink ở gốc checkpoint. |
| `./tests/distribution/package.Tests.ps1`, `./tests/distribution/install.tests.ps1` | Thoát 0 cho các ca đã chạy. Windows bỏ qua fixture liên kết và reparse. |
| `cargo build --offline --locked --bin check-docs`, `scripts/package.ps1 -AllowDirty -Apply`, `scripts/package.ps1 -Verify` | Build và xác minh gói cục bộ đạt. ZIP phát triển có 17 định nghĩa Skill. |
| `git diff --check HEAD` và rà đường dẫn được bảo vệ | Thoát 0. Engine, benchmark, mã đóng gói và lịch sử hiện có khớp HEAD. |

Git HEAD giữ ở `15175dd1faaf8b7f56f0eb93124417dc200a892e`. Rà soát phạm
vi bao phủ đường dẫn mã nguồn đã và chưa theo dõi. Đầu ra chạy và
checkpoint cục bộ nằm tại các vị trí chủ sở hữu cho Git bỏ qua. CI hosted,
Linux, lựa chọn native, E2E sản phẩm và nghiệm thu vật lý giữ `NOT_RUN`.
Chỉ số độ tin cậy, context và hiệu năng có thể so sánh giữ `NOT_MEASURED`.
Một pilot được chủ sở hữu duyệt sau này có thể thử một tác vụ native có
giới hạn, với người độc lập rà các phép kiểm đã chọn và bằng chứng hoàn thành.

<!-- source: en/plans/cirnet-rules/reliability-evidence/plan.md -->
<!-- source-sha256: 1e9b31d690147da5b500659c18e10e30e2e43b0fcac75dc129d0b3e1117a195e -->
