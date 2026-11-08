<!-- doc-id: CGACP-DESKTOP-ARCHITECTURE -->
<!-- revision: 1 -->
<!-- source-sha256: 08acfe1a5b1bfaebc27cb87e2ac57438641bb2804f26bb5e31a62a4333efea66 -->
<!-- translation-status: synchronized -->
<a id="authority"></a>

# Kiến trúc và thẩm quyền Dashboard

Client Qt Widgets quản lý trình bày và hộp thoại file. Runtime Rust quản lý
kiểm tra, ID repository và binding, intake, scheduler và thao tác ghi SQLite.
Mỗi client khởi động binary cùng thư mục đã xác minh và giao tiếp qua pipe
stdio riêng được kế thừa. Giao thức JSON theo dòng dùng ID request, giới hạn
frame, lỗi có cấu trúc và cursor sự kiện lưu bền vững. Ứng dụng không mở
cổng TCP lắng nghe. Xem [schema IPC](../../en/schemas/dashboard-ipc.schema.json).

Kênh người vận hành ban đầu nằm trong một tài khoản OS. Runtime có thể chạy
riêng để giữ lịch. Client khởi động lại tải hàng và sự kiện đã commit,
tránh phát lại lệnh. Danh tính người vận hành là tài khoản sở hữu tiến
trình và thư mục dữ liệu. Ranh giới này chưa xác thực reviewer độc lập
hoặc chống chương trình đã bị chiếm quyền trong cùng tài khoản. Provider
native và công cụ của Agent có gate riêng.

Metadata repository lấy từ Git sau khi kiểm tra đường dẫn. ID database tách
khỏi tên hiển thị, Git remote và đường dẫn local. Công cụ kiểm tra lại
đường dẫn và revision binding trước khi dispatch task hoặc automation.
Định danh Git common phân biệt worktree cùng repo với repo khác. Profile
release định nghĩa riêng đầu vào producer, owner của gate và miền tương
thích trong dữ liệu governance.

SQLite dùng migration giao dịch, khóa ngoại, WAL và thời gian chờ hữu hạn.
Dịch vụ SQLite nằm trong backend Rust. Chuyển sang PostgreSQL cần hợp đồng
repository cho truy vấn có phạm vi, cập nhật lạc quan, occurrence duy nhất,
fencing claim, outbox và rollback. Adapter và bộ test hợp đồng còn pending.
Xuất offline rồi nhập có kiểm chứng vào database riêng, đối chiếu ID và
digest, sau đó chuyển runtime trong cửa sổ bảo trì. Giữ kho cũ đến khi
xác minh database mới và quy trình phục hồi.

CLI hiện có vẫn quản lý file task và evidence lịch sử. Intake Dashboard
không nâng mức tin cậy của các record đó hoặc cấp quyền ghi. Dashboard ban
đầu chưa tự động ghi source hoặc dùng provider native. Runtime state là
nguồn của dữ liệu Dashboard. Git vẫn là nguồn của plan và spec.

Ảnh tham khảo định hướng bố cục desktop. Triển khai dùng Widgets native
và nhãn của dự án. [Tham chiếu OpenHands](https://github.com/OpenHands/OpenHands) ·
[API tiến trình Qt](https://doc.qt.io/qt-6/qprocess.html) ·
[API sao lưu SQLite](https://www.sqlite.org/backup.html) ·
[Cô lập giao dịch PostgreSQL](https://www.postgresql.org/docs/current/transaction-iso.html) ·
[Codex App Server](https://learn.chatgpt.com/docs/app-server)
