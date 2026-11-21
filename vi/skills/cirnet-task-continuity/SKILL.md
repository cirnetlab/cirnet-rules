---
name: cirnet-task-continuity
description: Dùng khi khôi phục công việc CIRNET bị gián đoạn, tiếp tục từ checkpoint hoặc bàn giao việc đang làm cho phiên hay tài khoản khác trong cùng họ agent.
---

# Khôi phục hoặc bàn giao công việc CIRNET

Dùng `scripts/task-continuity.ps1` từ checkout `cirnet-rules` đã được duyệt.
Công cụ tập trung thao tác trên checkpoint mà repo sở hữu lưu cục bộ.
Dùng vị trí checkpoint đã được duyệt và được Git bỏ qua.

1. **Tìm checkpoint đã công bố.** Xác nhận worktree sở hữu, họ agent và phạm
   vi đọc. Dùng `bootstrap -ConfirmScope -Compact` với repo và thư mục gốc
   chứa checkpoint đã được duyệt. Nếu có nhiều kết quả phù hợp, lấy Task ID
   từ người vận hành. Khi đã biết Task ID, dùng ID đó để khôi phục snapshot
   đã công bố. Nếu con trỏ hoặc snapshot không hợp lệ, làm theo quy trình
   khôi phục trong tài liệu.
2. **Khôi phục nhiệm vụ.** Đọc yêu cầu gốc, phạm vi từng được duyệt, các quyết
   định, việc còn lại và bước tiếp theo. So sánh worktree, nhánh, HEAD và thay
   đổi tệp liên quan giữa bản ghi và trạng thái hiện tại. Mở lại các Plan và
   Spec đã ghim của repo sở hữu rồi xác minh revision hoặc hash. Cùng chủ sở
   hữu xử lý sai lệch, xung đột và phần thiếu bằng chứng trước khi dựa vào
   checkpoint.
3. **Xác nhận quyền và bằng chứng.** Xác lập quyền hiện hành một cách độc
   lập trong phiên đang dùng. Xem quyền và kết quả đã lưu là bản ghi lịch sử
   cho đến khi đối chiếu với phạm vi và đầu vào hiện tại. Trước khi sửa thông
   thường, lặp lại lệnh khôi phục phù hợp với `-ConfirmContinuation`. Với
   `bootstrap`, ghim checkpoint đã khôi phục bằng `-ExpectedCheckpointId`
   và yêu cầu `CanContinue` trước khi sửa. Đối chiếu từng tiêu chí với bằng
   chứng hiện hành trước khi kết luận PASS hoặc đóng task. Phép kiểm bị gián
   đoạn hoặc chưa chạy giữ trạng thái `NOT_RUN`.
4. **Lưu và bàn giao.** Cập nhật tiến độ bằng `save` trước khi chuyển giao.
   Chỉ chia sẻ Task ID và vị trí checkpoint cho người nhận có quyền. Nếu
   được cấp quyền riêng để chuyển gói giữa các agent cùng họ, soát các trường
   được phép để phát hiện dữ liệu nhạy cảm, rồi dùng
   `export -ConfirmScope -ConfirmExport`. Gửi SHA-256 dự kiến qua kênh tin cậy.
   Phiên nhận xác minh gói và kiểm tra lại Git, tham chiếu, bằng chứng và quyền
   hiện hành trước khi thao tác.

Giữ các Plan và Spec trong repo sở hữu và tham chiếu bằng đường dẫn cùng
revision hoặc hash. Đưa mọi đề xuất sửa tài liệu qua quy trình phê duyệt
của chủ sở hữu. Dùng công cụ và mô hình lưu trữ hiện có.

Dùng revision áp dụng trong checkout đã được duyệt.
Lệnh Task Continuity (`vi/docs/distribution.md#task-continuity`) ·
Đặc tả Task Continuity (`vi/specs/cirnet-rules/task-continuity/spec.md`)

<!-- source: en/skills/cirnet-task-continuity/SKILL.md -->
<!-- source-sha256: a993de332fa35d075a86698b79b99a9e0906da1913fddfe6b47658ea23b8d7a4 -->
