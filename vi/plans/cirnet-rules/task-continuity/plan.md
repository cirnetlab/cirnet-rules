# Kế hoạch triển khai Task Continuity

Triển khai [đặc tả Task Continuity](../../../specs/cirnet-rules/task-continuity/spec.md)
trong checkout common-rules. Điểm vào PowerShell phục vụ Git worktree được
chọn rõ ràng. Chủ repo chọn vị trí checkpoint được Git bỏ qua hoặc kho cục bộ
nằm ngoài checkout. Nội dung và quyền sở hữu của Plans/Specs hiện có thuộc
về các repo tương ứng.

- [x] Thêm bộ kiểm thử tích hợp PowerShell ở `tests/continuity/`, dùng các repo Git tạm và gọi script công khai. Không quan sát được lượt chạy ban đầu khi chưa có điểm vào trong lần build này.
- [x] Triển khai ghi nhận Git chỉ đọc và tham chiếu tài liệu được ghim ở `scripts/continuity/`. Kiểm tra thay đổi staged, unstaged, untracked, HEAD, nhánh, worktree và index. Kiểm tra tham chiếu được cấp quyền, gồm các checkout liên quan.
- [x] Triển khai snapshot hoàn chỉnh và công bố con trỏ hiện hành bằng thao tác nguyên tử. Kiểm tra snapshot hỏng, điều kiện công bố không đạt, truy cập đồng thời và tiếp tục giữa agent cùng loại. Đối chiếu Git index và Plans/Specs hiện có của repo đích bằng hash.
- [x] Thêm trình chạy các mốc được kiểm soát và lệnh vòng đời trong `scripts/task-continuity.ps1`. Kiểm tra mã thoát thực tế, lần chạy bị gián đoạn, thay đổi đầu vào và đóng công việc khi bằng chứng còn hiệu lực. Xác minh bản export có phiên bản chỉ gồm metadata được phép và từ chối gói bị sửa.
- [x] Ghi cách dùng từ checkout trong [hướng dẫn phân phối](../../../docs/distribution.md#task-continuity), đồng bộ tài liệu tiếng Anh và tiếng Việt, và thêm bộ test vào ma trận CI Windows/Ubuntu.
- [ ] Ghi nhận lượt chạy thành công trên Ubuntu và bằng chứng khôi phục, bàn giao thực tế giữa các tài khoản khác nhau trong mỗi loại agent được hỗ trợ trước khi xác nhận nghiệm thu toàn bộ đặc tả.

## Kiểm chứng cục bộ

Kết quả quan sát tại thư mục gốc `cirnet-rules` trên Windows:

- PASS: `cargo build --offline --locked`, `cargo fmt --all -- --check` và
  `cargo clippy --offline --locked --all-targets -- -D warnings`.
- PASS: `cargo test --offline --locked` (61 test đạt, bỏ qua một test tạo
  symlink thư mục trên Windows).
- PASS: `pwsh -NoProfile -File ./tests/continuity/task-continuity.tests.ps1`.
  Bộ test dùng repo Git tạm và tiến trình con thực tế trên máy cục bộ.
- PASS: `cargo run --offline --locked --bin check-docs --` với `--new-file`
  cho cả hai bản ngôn ngữ của Plan và Spec mới,
  `./tests/validation/check-changed-docs.tests.ps1` và `git diff --check`.
  Checker tài liệu báo cảnh báo tham khảo cho các biến thể kiến trúc hiện có.
- BLOCKED: Lượt chạy Ubuntu. Môi trường cục bộ không truy cập được WSL và
  Docker Linux engine.
- NOT_RUN: Hết hạn mức và bàn giao thực tế giữa các tài khoản khác nhau.
  Bộ test kiểm tra loại agent Codex và Claude Code bằng fixture.

## Xử lý lỗi GitHub Actions

[Job Ubuntu của c2a7671](https://github.com/cirnetlab/cirnet-rules/actions/runs/44495650591/job/135462022825)
in thông báo PASS của bộ test rồi thoát với mã 1. Runner trả lại
`$LASTEXITCODE`, có giá trị 1 do lệnh `verify` cuối cùng được thiết kế để
bị từ chối. Bộ test nay kiểm tra mã từ chối đó và trả 0 sau khi mọi assertion
và bước dọn fixture hoàn tất.

Cách gọi PowerShell của GitHub Actions tái hiện mã thoát 1 trên Windows trước
khi sửa và trả 0 sau khi sửa. Khi chủ động tạo lỗi lúc chuẩn bị fixture, mã
thoát vẫn là 1. Lượt CI trên GitHub cho bản sửa vẫn đang chờ.

<!-- source: en/plans/cirnet-rules/task-continuity/plan.md -->
<!-- source-sha256: 1ca50f15e332322f4c3b26cfb2b737627bcff35d51465d5c9c975e1821c997f2 -->
