<!-- doc-id: CGACP-LOCAL -->
<!-- revision: 2 -->
<!-- source-sha256: 86ba78e249eddc4f1e71e6824a832f951bb65ddfa4244cbebbf44382190b1092 -->
<!-- translation-status: synchronized -->
<a id="operator-workflow"></a>

# Quy trình vận hành CGACP local

Build một lần bằng `cargo build --locked`. Chạy binary đã kiểm chứng từ
checkout governance hoặc workspace chứa nó. Dò thư mục cùng cấp mặc định
dựa trên entrypoint đã được Git theo dõi và Git root thực. Với worktree
governance khác, dùng `--governance` hoặc đặt `CGACP_GOVERNANCE` cho tiến
trình. Liên kết worktree được lưu local dưới `.cgacp/` đã ignore. Dò
workspace và công việc thông thường chạy offline.

Cập nhật task giải phóng khóa tệp khi thao tác kết thúc, kể cả khi một bản
sao handle vẫn đang mở. Các lượt cập nhật đồng thời vẫn loại trừ nhau và
phải dùng revision hiện tại.

1. Chạy `workspace-discover`. Rà mục repo rồi liên kết một checkout bằng
   `workspace-bind --repo NAME --path WORKTREE --actor OPERATOR`.
2. Viết spec và plan tiếng Anh trong thư mục task tập trung. Đồng bộ bản
   tiếng Việt trong cùng thay đổi. Cung cấp descriptor JSON theo
   [schema task](../../en/schemas/cgacp-task.schema.json) bằng
   `task-create --descriptor descriptor.json`. Lưu đầu vào chỉ dùng cho
   phiên trong thư mục trạng thái local đã ignore.
   Dùng `_workspace` cho task cha liên repo. Mỗi task con có repo, phạm vi
   và nghiệm thu riêng. Task cha chỉ đóng khi các task con bắt buộc đã đóng.
3. Kiểm tra `route --task ID` và `task-context --task ID --session TOKEN`.
   Chuyển trạng thái bằng `task-transition --task ID --revision N --actor ACTOR --to STATE`.
   Khi bắt đầu, công cụ ghi nhận file dirty và định danh worktree. Thay
   đổi có sẵn chồng lấp sẽ chặn bước này. Dùng `task-resume` để xem trạng
   thái và bước tiếp theo. Gửi checkpoint JSON có giới hạn bằng
   `task-checkpoint --checkpoint FILE`. Xung đột revision yêu cầu nạp lại.
   Nguồn thẩm quyền đổi yêu cầu `task-revise` và duyệt lại.
4. Harness thực thi sửa trong phạm vi đã duyệt. Chạy `diff-check` và các
   bước kiểm tra đã cấu hình. `grant-evaluate`, `run-step` và `cancel-step`
   làm việc trên task được quản lý và bước đã đặt tên. Runner hỗ trợ thao tác
   Git chỉ đọc cố định và kiểm tra JSON, UTF-8 tích hợp. Nó gọi executable
   cùng argv trực tiếp. Script repo, build tổng quát, mạng và cô lập runtime
   bên ngoài cần harness được ủy quyền riêng.
5. Sau IMPLEMENTED, dùng `task-verify --task ID --revision N --actor ACTOR`.
   Runner ghi kết quả dựa trên diff và bytes test hiện tại. `review-context`
   cung cấp spec, đầu vào test đã chọn, diff tracked có giới hạn và kết quả.
   Diff nhạy cảm hoặc quá lớn cần review local riêng. `task-review` ghi nhận khai
   báo của reviewer local khác người thực hiện. Nghiệm thu thay đổi ở
   ranh giới nhạy cảm còn bị chặn khi thiếu review đã xác thực.
   `task-accept` cần PASS còn hiệu lực và đúng chủ thể ủy quyền. Kiểm tra
   do người thực hiện khởi chạy được ghi là self-check.
6. Sau nghiệm thu, xem trước với `docs-preview --task ID --proposal FILE`.
   JSON có mảng targets gồm repo, path, locale, base_sha256, content và
   section tùy chọn. Dùng SHA-256 của UTF-8 đã chuẩn hóa LF và tiêu đề
   chính xác khi thay một mục. Cập nhật đề xuất theo bytes hiện tại nếu
   xung đột. Bản xem trước cung cấp patch khi thiếu quyền ghi. Có thể liên
   kết đích còn thiếu và nghiệm thu lại trước khi ghi. Worktree đã nghiệm
   thu không thể bị thay ngầm.
   `docs-apply` xử lý từng đích được cấp quyền và lưu các mục còn
   pending. `docs-check` xác minh bytes sau ghi. `task-close` cần đủ tài
   liệu hoặc lý do docs_not_required đã được chấp thuận. Ghi local, merge
   và publish có trạng thái riêng.
7. Chuẩn bị gói Agent bằng
   `task-bootstrap --task ID --agent codex --session DIRECTORY --dry-run`.
   Chọn claude hoặc antigravity cho adapter còn lại. Dùng phiên ngoài repo
   chỉ chủ sở hữu có quyền ghi. Báo discovery native, thực thi hook và
   thực thi task riêng với việc chuẩn bị gói.

Để kiểm tra release, dùng descriptor RELEASE được ủy quyền tường minh và
`release-check --task ID --actor ACTOR`. Lệnh `check-release` hiện có vẫn
kiểm tra offline khi dùng bố trí workspace cũ. Cả hai đọc train và family
từ policy, kiểm chứng đúng tổ hợp cùng bằng chứng đã duyệt. Profile phần
mềm Windows và các loại gate đã biết xác định phạm vi phiên bản công cụ
này. WORK, VERIFY và ACCEPT chạy độc lập với chế độ release. Snapshot cũ
được giữ ở dạng lịch sử.

JSON trả mã 0 cho PASS, 1 cho FAIL và 2 cho BLOCKED hoặc NOT_RUN. PASS của
runner local xác nhận đúng các phép kiểm tra đã ghi. Kiểm tra đường dẫn
và hash giới hạn thao tác được quản lý. Harness bên ngoài đảm nhiệm cô lập
OS, xác thực người thao tác và interlock phần cứng.
