# CIRNET Rules — Adaptive Stable checkpoint

**Trạng thái** `BLOCKED_WITH_CHECKPOINT`. Giữ Adaptive Candidate. Chưa có
quota native mới cho đợt này. Đã dùng 0 lượt Agent, nên chưa có telemetry
native mới và các verdict theo snapshot trước vẫn là bằng chứng lịch sử.

## Source và bản sửa

HEAD khi bắt đầu là `0cdae068bffc31ff803de4b873502593bfaf4d1f`. Working
tree và index sạch. SHA-256 của `git diff --binary HEAD` cho bản sửa source
đã kiểm tra là `6505c8d2832b80ffc3511c8a2627096885e025ac02799dedc68ac322534a4efd`.
Báo cáo checkpoint này chưa có trong Git diff đó.

Nguyên nhân lỗ hổng bằng chứng A/B là Work luôn nhận bộ Skill lịch sử
từ snapshot `common`. Preview cũ có thể xác nhận template Author hiện
tại, còn Work vẫn dùng Refactoring Skill đóng băng.
[Checkpoint trước](adaptive-stable-acceptance-f4db620.md)

| File | Bản sửa |
| --- | --- |
| [A/B runner](https://github.com/cirnetlab/cirnet-rules/blob/1266f8c635b5d4f5d13413921b8ce8efe0736119/scripts/eval-codex-ab.ps1) | Thêm profile `-CurrentWorkRules`. Candidate Author và cả hai Work dùng snapshot hiện tại gồm 54 file Rules và tài liệu. Baseline Author vẫn dùng A gốc. Continuation từ chối đổi profile và dùng lại bytes đã đóng băng. |
| [Reporter](https://github.com/cirnetlab/cirnet-rules/blob/1266f8c635b5d4f5d13413921b8ce8efe0736119/scripts/eval-codex-report.mjs) | Đối chiếu revision, baseline và hash các tham chiếu cùng Skills đã stage. Nếu Work Skill bị thay thế, reporter từ chối nhận kết quả dưới profile Current. |
| [Regression](https://github.com/cirnetlab/cirnet-rules/blob/1266f8c635b5d4f5d13413921b8ce8efe0736119/tests/behavioral-ab.tests.ps1) | Kiểm tra staging thực tế ở hai Work, template Candidate Author, continuation, và từ chối hash bị thay thế trên bản sao fixture test. |
| [Hướng dẫn EN](https://github.com/cirnetlab/cirnet-rules/blob/1266f8c635b5d4f5d13413921b8ce8efe0736119/en/docs/behavioral-evaluation.md#compare-authoring-with-current-work-skills) và [VI](https://github.com/cirnetlab/cirnet-rules/blob/1266f8c635b5d4f5d13413921b8ce8efe0736119/vi/docs/behavioral-evaluation.md#so-sánh-biên-soạn-với-work-skills-hiện-tại) | Mô tả profile, phạm vi suy luận và lệnh preview/pilot. Bản VI đã đồng bộ source SHA. |

Phép kiểm tập trung trả `1` trước sửa vì runner chưa nhận profile mới
và trả `0` sau sửa khi snapshot chứa đúng Work Skill hiện tại. Profile
đo khác biệt giữa Author với Work được giữ cố định. Nó cung cấp bằng
chứng hành vi cho Work Skills hiện tại. Muốn đo riêng tác động của thay
đổi Work Skills cần một thiết kế đối chứng riêng.

## Bằng chứng offline

| Phép kiểm | Trạng thái |
| --- | --- |
| Candidate smoke, A/B shim và regression Python | `PASS` trên Windows và Ubuntu cục bộ. Các ca FAIL/BLOCKED mô phỏng được giữ nguyên trong assertions. |
| Privacy, Handoff Gate, EN/VI source SHA và 13 Skills | `PASS`. Các ca cần symlink trên Windows vẫn `NOT_RUN` vì quyền tạo link. |
| Cargo fmt/build/test/Clippy, check-docs và git diff --check | `PASS`. |
| E08 fixture offline | `PASS` 2/2 với Icarus portable. Đã tái hiện lỗi khởi động và kiểm tra bốn điều kiện an toàn trong fixture sau sửa. Native E08 và physical verification vẫn `NOT_RUN`. |
| E08 toolchain trong Windows MXC | `PASS` với thư mục ToolPath trên PATH. Ở probe ban đầu thiếu PATH, compiler trả `0xC0000135`. Probe có PATH trả compile `0`, simulation `0` và đúng marker. |
| Bằng chứng benchmark lịch sử | `PASS`. Kiểm kê 5.525 file khớp SHA-256 `f7b994c23345afc8bd483af4f932722ce50e3644bec85630e85af75837959f40`. |

Icarus portable nằm dưới OS temp tại
`cirnet-icarus-20261115-535fe18/ucrt64/bin`.
`iverilog.exe` có SHA-256
`25e1ba02e72f58d9d484bc5457d69a6b179d22d86bdf5ba5d6f9f0d4fe620bbf`.
`vvp.exe` có SHA-256
`f0775603e1f1bcf1b2dea16dff963a4b7d71b7b33d5bbe23d2b01ff5b2304e09`.

## Snapshot để tiếp tục

| Preview | Lượt dự kiến mỗi phiên | SHA-256 manifest |
| --- | ---: | --- |
| Candidate E01/E06 | 4 | `f7b07a907ab86074fbed4b128b4bf63c22411f072e1f8cbd88077978258a1035` |
| Candidate E05 | 2 | `03f219dbda134d8a9540f07e65bcf8be14680950dfcb6dad62b3edd22dae2c0d` |
| Candidate P01 | 2 | `481e4c8c496c9df692391be6884f8bd4527b62ce3f389d428029ba40b09a84cd` |
| A/B Current pilot | 16 | `c95e8c8a53c1a96b40ff16db0bee03d5cd20ac95116fde0dcea9b13140e69c11` |
| A/B Current đủ lịch | 96 | `99a9b70febb2aff23767e88cbbd109f2c2fed0e33889a7629fb5e7c121feb6f9` |

Pilot và lịch đầy đủ có cùng digest snapshot
`1c52463f5957461164ce936220193015a95df5df5b8340559e86837cbe149727`.
Đã đối chiếu 54 hash của source và từng bản sao. Candidate previews
xác nhận evaluator khớp với bằng chứng native cũ nhưng Rules hoặc runner
khác SHA-256. Native trên source hiện tại vẫn `NOT_RUN`.

## Các gate còn mở và quyền cần cấp

- G0 có bằng chứng offline về 13 Skills và EN/VI. G1–G6 chưa đạt nghiệm
  thu release trên snapshot hiện tại. G7 là `NOT_ASSESSED`.
- E06 byte-exact, E01 Privacy, E05/P01 behavior và handoff trên snapshot
  cuối cần native và rà độc lập. Hai phiên E01/E06 cùng snapshot cần 8
  lượt. E05 và P01 cần 4 lượt bổ sung. Xin quota đợt đầu cho tổng cộng
  12 lượt, dừng sau FAIL để xử lý nguyên nhân.
- Sau khi Candidate đạt, cần quyền riêng cho pilot 16 lượt và sau pilot
  đạt mới cấp quyền lịch đầy đủ 96 lượt. Dùng Icarus ToolPath đã xác minh
  khi thực hiện E08. Rà lựa chọn LIGHT, STANDARD, DEEP từ trace thực tế
  và giữ đúng hành vi báo thiếu partner input ở E07.
- Chỉ xét latency, token và tool calls trên nhóm có cùng cấu hình và
  verdict đạt. Đợt này không có mẫu native mới.

Sau khi owner cấp quota và phương thức xác thực, chạy bằng cấu hình
`gpt-6.2-sol / xhigh`, 300 giây mỗi vai trò và session tạm mới. Đối chiếu
SHA của đầu vào trong hai lượt E01/E06 trước khi kết luận độ lặp lại.
[Lệnh Candidate-only](https://github.com/cirnetlab/cirnet-rules/blob/1266f8c635b5d4f5d13413921b8ce8efe0736119/en/docs/model-compatibility.md#run-the-candidate-only-smoke) ·
[Lệnh A/B Current pilot](https://github.com/cirnetlab/cirnet-rules/blob/1266f8c635b5d4f5d13413921b8ce8efe0736119/en/docs/behavioral-evaluation.md#compare-authoring-with-current-work-skills)
