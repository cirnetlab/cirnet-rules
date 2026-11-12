<!-- doc-id: CNR-GUIDED-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: 360955502d9dcca39ae2db8cc3fa03a7849688805f215f1bb1d453e52123ec20 -->
<!-- translation-status: synchronized -->
<a id="scope"></a>

# Phạm vi kiểm tra và thiết lập Agent trong CNR

Người vận hành có thể kiểm tra từng thành phần hoặc toàn bộ phạm vi từ CLI và TUI. Báo cáo ghi điều kiện tiên quyết, phát hiện, bằng chứng và bước tiếp theo. Phép kiểm độc lập tiếp tục khi authority không khả dụng. Kiểm tra giữ nguyên trạng thái governance và repository.

Một trình hướng dẫn phục vụ Codex, Claude Code và Antigravity. Người vận hành chọn repository, worktree đã xác nhận và Agent. Trình hướng dẫn hiển thị trước từng thao tác ghi và giữ digest đã review trong bộ nhớ. Apply dùng grant của task hiện hành và bảo toàn nội dung người dùng. Khối chỉ dẫn do CNR quản lý có ownership tường minh và có thể audit khi rollback.

Mẫu khởi chạy MCP, cấu hình đã đăng ký, kiểm tra giao thức và hành vi client native là các loại bằng chứng riêng. Setup phải giữ cấu hình ngoài phạm vi và hướng dẫn nhập thủ công nếu thiếu cơ chế ghi phù hợp. Native discovery cần bằng chứng riêng cho từng Agent.

Nghiệm thu dùng fixture Git và cấu hình Agent cách ly. Task này chỉ bao gồm checkout governance hiện tại. Sửa checkout sản phẩm, cấu hình người dùng, xuất bản và kích hoạt release cần quyền riêng.
