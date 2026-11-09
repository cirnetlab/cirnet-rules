<!-- doc-id: CGACP-DESKTOP-ARCHITECTURE -->
<!-- revision: 6 -->
<!-- source-sha256: 7db95bed241a4c4a05a37e0a6773f5d2b293d985c81ebb806c0d2c76670d3e3b -->
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
Schema 5 thêm biên nhận vòng đời, định danh tăng dần và xem trước phạm vi xóa.
Nâng cấp tạo snapshot của kho hiện có trong `backups/` riêng và dừng nếu
chưa xác minh được snapshot. Schema 6 ghi dấu xóa Automation để định nghĩa
đã xóa không thể chạy theo lịch hoặc xuất hiện lại khi phát lại yêu cầu tạo.
Khi xóa cả Hội thoại, runtime tìm lượt chạy gốc theo ID đã lưu, kiểm tra
revision và lượt chạy đang hoạt động, rồi xóa định nghĩa trong cùng giao dịch.
Client Qt gọi bất đồng bộ
`preferences.get` và `preferences.set` và gộp các thay đổi bố cục liên tiếp
trước khi ghi. Revision dự kiến ngăn cửa sổ thứ hai âm thầm ghi đè lần cập
nhật mới hơn. Tùy chọn chỉ chứa dữ liệu bố cục và giao diện, tách khỏi quyền
task và đường dẫn repository. Runtime kiểm tra màu RGB, kích thước pixel,
họ phông và ngôn ngữ tùy chọn trong bản ghi hiện có. Client chỉ quy đổi
tỉ lệ cũ khi thiếu giá trị pixel, rồi dùng cùng một phông ứng dụng cho
widget và menu. Khi khởi động, client đọc ngôn ngữ đã lưu qua kết nối runtime
đã xác minh. Client dùng tiếng Anh nếu hết thời gian chờ mà chưa lấy được
ngôn ngữ. Repository vòng đời có kiểu dữ liệu và unit
of work SQLite commit cùng nhau phạm vi xóa, biên nhận idempotency và
metadata lượt chạy cần giữ. Intake và tự động hóa cũ còn phụ thuộc SQLite.
Chuyển sang PostgreSQL cần hợp đồng
repository cho truy vấn có phạm vi, cập nhật lạc quan, occurrence duy nhất,
fencing claim, outbox và rollback. Adapter và bộ test hợp đồng còn pending.
Xuất offline rồi nhập có kiểm chứng vào database riêng, đối chiếu ID và
digest, sau đó chuyển runtime trong cửa sổ bảo trì. Giữ kho cũ đến khi
xác minh database mới và quy trình phục hồi.

Qt xác minh executable cùng thư mục trên worker có thể hủy, rồi áp dụng
frame stdio trên GUI theo lát tối đa tám frame hoặc năm mili giây giữa các
ranh giới frame. Client xử lý các phản hồi hoàn chỉnh trước khi kết thúc
yêu cầu chưa có phản hồi lúc tiến trình thoát. Một timer theo dõi tối đa
32 yêu cầu đang chờ. Mỗi trang
transcript có tối đa bốn tin. View áp dụng trang mới theo ordinal và giới
hạn dữ liệu hiển thị của transcript và sidebar.

Runtime dùng kênh vào và ra có giới hạn. Dispatcher sở hữu kết nối SQLite
tương tác và scheduler dùng kết nối riêng. Worker lưu trữ có một luồng
chạy, ba vị trí thao tác đang hoạt động và lịch sử có giới hạn. Khi kết nối,
Dashboard khởi tạo tác vụ chỉ đọc `storage.chats` để tính dung lượng nội dung
hội thoại trên kết nối SQLite riêng. Dashboard theo dõi trạng thái tác vụ độc lập
với điều hướng. Cùng worker thực hiện sao lưu, kiểm tra, quét cơ sở dữ liệu
và thu gọn. Các thao tác báo trạng thái và hỗ trợ hủy cộng tác. Khóa SQLite
vẫn có thể làm chậm thao tác ghi trong ngân sách chờ. Git local và nhập
dữ liệu cũ còn chạy đồng bộ trong dispatcher. Worker không sở hữu widget.

CLI hiện có vẫn quản lý file task và evidence lịch sử. Intake Dashboard
không nâng mức tin cậy của các record đó hoặc cấp quyền ghi. Dashboard ban
đầu chưa tự động ghi source hoặc dùng provider native. Runtime state là
nguồn của dữ liệu Dashboard. Git vẫn là nguồn của plan và spec.

Sidebar quản lý lịch sử hội thoại dùng chung. Các bộ chọn repository dùng
cùng catalog phân trang với Cài đặt. Đường dẫn hiển thị tách khỏi đường dẫn
canonical đã lưu và định danh binding. Thẻ tự động hóa hiển thị định nghĩa
đã lưu và trạng thái chạy. Runtime giữ thông tin nguồn gốc mẫu hiện có.
Thông báo giữ lịch sử có giới hạn trong phiên.

Ảnh tham khảo định hướng bố cục desktop. Triển khai dùng Widgets native
và nhãn của dự án. [Tham chiếu OpenHands](https://github.com/OpenHands/OpenHands) ·
[API tiến trình Qt](https://doc.qt.io/qt-6/qprocess.html) ·
[API sao lưu SQLite](https://www.sqlite.org/backup.html) ·
[Cô lập giao dịch PostgreSQL](https://www.postgresql.org/docs/current/transaction-iso.html) ·
[Codex App Server](https://learn.chatgpt.com/docs/app-server)
