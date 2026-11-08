<!-- doc-id: CGACP-DESKTOP-OPERATOR -->
<!-- revision: 1 -->
<!-- source-sha256: 1d203c18d4bc8f4c8a034650f24bc7fded24db46dd681c732735b4d0f9c95207 -->
<!-- translation-status: synchronized -->
<a id="operator"></a>

# Dashboard CGACP

Thế hệ sản phẩm là CGACP v1. Phiên bản ứng dụng lấy từ Cargo.
Tên release ứng viên local là `cirnet-rules-0.1.0`. Rà soát release tách
riêng khỏi thao tác repository, hội thoại và automation hằng ngày.

## Build và chạy

Dùng Rust toolchain đã pin và bộ Qt 6 Widgets cho trình biên dịch C++ local.
Build `cgacp-runtime` bằng Cargo, rồi cấu hình CMake với
`CGACP_RUNTIME_EXECUTABLE` trỏ đến binary vừa build. Build
`cgacp-dashboard` và chạy CTest. Khi chuẩn bị gói Windows, cài vào thư mục
staging riêng và bật `CGACP_DEPLOY_QT`. Mở Dashboard đã cài từ thư mục bin.
Dùng `--data-dir` với thư mục tuyệt đối riêng để kiểm thử dùng một lần.

## Quy trình vận hành

Mở Repositories, chọn thư mục hoặc file, rồi rà Git root, branch, worktree
và đường dẫn file gợi ý. Xác nhận để lưu đăng ký. Chọn binding đã đăng ký
khi tạo New Chat hoặc Add Automation. Dùng Relink khi checkout đã lưu đổi
vị trí và Archive để dừng việc mới mà vẫn giữ lịch sử và file trên đĩa.
Relink làm revision binding cũ mất hiệu lực. Công việc liên quan cần được
xem xét trước khi tiếp tục.

New Chat lưu tin nhắn và đề xuất task theo phạm vi. Profile handoff hiển thị
trạng thái chờ duyệt. Xem lại hội thoại sau khi mở ứng dụng. Automation hỗ
trợ chạy thủ công và lịch hằng ngày theo múi giờ IANA. Xem trước lịch trước
khi bật và xác nhận lượt chạy một lần khi automation đang tắt. Runtime Rust
quản lý lịch. Để giữ lịch khi đóng Dashboard, chạy
`cgacp-runtime --scheduler` trong terminal riêng với cùng thư mục dữ liệu.
Trạng thái lịch được lưu khi runtime dừng. Các lượt hằng ngày bị lỡ được gộp.
Definition bị tạm dừng hoặc stale phải được kiểm tra trước khi tạo đề xuất mới.

## Lưu trữ và vận hành

Rust sở hữu SQLite trong thư mục dữ liệu ứng dụng local hoặc thư mục do
người vận hành chỉ định. Dùng `storage.backup` qua runtime để tạo snapshot
online và `storage.verify` để kiểm tra. Khi diễn tập khôi phục, đóng
runtime, phục hồi bản sao đã rà vào thư mục riêng mới và kiểm tra tại đó
trước khi đổi đường dẫn. Giữ thư mục gốc và các file WAL. Bản sao lưu là
dữ liệu nhạy cảm local.

Có thể xem trước và lưu trữ kho file CLI đã dừng bằng `legacy.preview`
và `legacy.apply`. Áp dụng cần digest manifest đã rà và xác nhận tiến
trình ghi cũ đã dừng. Dùng `legacy.inspect` và `legacy.link` để xem hồ
sơ lưu trữ và gắn alias cũ với binding đã rà riêng. Nhập giữ nguyên nguồn
và không kích hoạt lại grant, lease hoặc lượt chạy cũ.

Chạy `scripts/package-dashboard.ps1` trên cây staging Windows đã cài để
ghi SHA-256 artifact và nguồn build. Ứng viên giữ `RELEASE_BLOCKED` khi
còn gate Agent native, cô lập, review xác thực, cài Windows sạch và
điều kiện phân phối.

[Kiến trúc và giới hạn](dashboard-architecture.md) · [Ứng viên phát hành](dashboard-release.md) · [Quy trình task](cgacp-local.md)
