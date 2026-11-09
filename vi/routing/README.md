<!-- doc-id: OPENRULES-ROUTING -->
<!-- revision: 1 -->
<!-- source-sha256: 6a180357b286931d07ae6946be6ed28132f22b5502a6ef4b744354f493df6b2d -->
<!-- translation-status: synchronized -->
<a id="routing"></a>

# Workspace local và định tuyến

`workspace-discover` đọc metadata trong giới hạn của workspace và đề xuất
ứng viên. Dùng `workspace-bind` với ID repo đã đăng ký, đường dẫn tương đối
và nhãn người thao tác. Lệnh kiểm tra Git root, thư mục quản trị, thư mục
dùng chung và commit mốc. Thay liên kết cần cung cấp định danh trước đó.
`workspace-health` kiểm tra một liên kết. URL remote là tùy chọn trong luồng
này. Khi thêm repo, rà profile, ownership, phân loại và quan hệ có kiểu
trong registry, sau đó liên kết với worktree local thực tế.

Dùng đường dẫn file chính xác cho phạm vi task. Thẻ module và quan hệ trực
tiếp giải thích tác động có thể có. Phạm vi đọc tường minh giới hạn nguồn
được mở. Quyền ghi được kiểm tra riêng. CLI gắn nhãn tài liệu sản phẩm và
task là dữ liệu. Nguồn thẩm quyền nằm trong các mục chỉ dẫn governance.
Task UI nhỏ chỉ tra repo liên quan. WORK tách khỏi validator release.

`task-context` cấp entrypoint tiếng Anh, mini-spec, mini-plan, skill và
nguồn sản phẩm đã chọn. Token phiên ổn định yêu cầu delta. Đổi token khi
bắt đầu phiên mới hoặc sau compaction. File dirty, thiếu và untracked trong
phạm vi được kiểm tra theo nội dung. Cache lưu digest, có hạn lưu và có
thể dọn bằng `context-prune`. Context báo số byte nguồn, file, repo, cache
hit và thời gian.
