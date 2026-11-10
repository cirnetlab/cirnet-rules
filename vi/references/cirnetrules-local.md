<!-- doc-id: CIRNETRULES-LOCAL -->
<!-- revision: 3 -->
<!-- source-sha256: 41f65aafbd434a78149823823cae553b5c3a2edc1f304e5014b1638713467154 -->
<!-- translation-status: synchronized -->
<a id="operator-workflow"></a>

# Quy trình vận hành CIRNETRULES local

Build một lần bằng `cargo build --locked`. Chạy binary đã kiểm chứng từ
checkout governance hoặc workspace chứa nó. Dò thư mục cùng cấp mặc định
dựa trên entrypoint đã được Git theo dõi và Git root thực. Với worktree
governance khác, dùng `--governance` hoặc đặt `CIRNETRULES_GOVERNANCE` cho tiến
trình. Liên kết worktree được lưu local dưới `.cirnetrules/` đã ignore. Dò
workspace và công việc thông thường chạy offline.

Cập nhật task giải phóng khóa tệp khi thao tác kết thúc, kể cả khi một bản
sao handle vẫn đang mở. Các lượt cập nhật đồng thời vẫn loại trừ nhau và
phải dùng revision hiện tại.

1. Chạy `workspace-discover`. Rà mục repo rồi liên kết một checkout bằng
   `workspace-bind --repo NAME --path WORKTREE --actor OPERATOR`.
2. Viết spec và plan tiếng Anh trong thư mục task tập trung. Đồng bộ bản
   tiếng Việt trong cùng thay đổi. Cung cấp descriptor JSON theo
   [schema task](../../en/schemas/cirnetrules-task.schema.json) bằng
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

<a id="recover-maintenance-task"></a>

## Khôi phục task bảo trì

Dừng các tiến trình cũ có quyền ghi và giữ kho tệp gốc. Dùng
[cổng lưu trữ lịch sử](dashboard.md#legacy-archive) để lưu bằng chứng task
và lease đã rà soát. Với từng task cần khôi phục, chuẩn bị descriptor mới
dùng ID gốc, spec và plan tập trung hiện hành, chủ thể ủy quyền hiện hành,
danh sách hành động rỗng và bỏ thời hạn. Rà soát riêng bằng chứng gốc.

1. Dùng `task-create --descriptor descriptor.json --legacy-review preview`.
   Đối chiếu đường dẫn nguồn, SHA-256, trạng thái và revision với descriptor
   đề xuất cùng spec, plan và governance hiện hành mà nó tham chiếu.
2. Chạy lại cùng lệnh với `review_sha256` trả về thay cho `preview`. Nếu
   đầu vào thay đổi, rà soát bản xem trước mới. ID có trong nhiều kho cũ
   hoặc trùng với bản ghi hiện hành khác sẽ chặn khôi phục. Khôi phục các
   phụ thuộc trước task tham chiếu chúng.
3. Dùng `task-show --task ID` để xác nhận `PROPOSED` và biên nhận nguồn.
   Dùng `workspace-bind` để xác minh binding Git hiện hành và dựng lại
   context từ nguồn hiện tại. Dùng `task-revise` để cấp quyền mới, rồi qua
   các bước kiểm tra sẵn sàng và thực thi thông thường.

Task gốc, phê duyệt, checkpoint, lease và bằng chứng vẫn nằm trong kho cũ
và nằm ngoài phạm vi quyền hiện hành. Có thể lặp lại quy trình với từng
task khác đã rà soát. Việc tạo task thông thường vẫn bị chặn khi checkout
còn kho cũ.

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
