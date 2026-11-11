<!-- doc-id: CIRNETRULES-DESKTOP-ARCHITECTURE -->
<!-- revision: 9 -->
<!-- source-sha256: 1ec642f675b368df6a84a06cd5e9d21994f779b792b88825037b6beb6637cec7 -->
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
tỉ lệ cũ khi thiếu giá trị pixel. Bảng kiểu áp dụng họ phông đã chọn cho
widget, menu và tài liệu văn bản đang mở. Khi khởi động, client đọc ngôn ngữ
đã lưu qua kết nối runtime đã xác minh. Khi đổi ngôn ngữ, client thay bản
dịch và cập nhật văn bản trên các widget hiện có, giữ trạng thái ứng dụng
và runtime. Repository vòng đời có kiểu dữ liệu và unit
of work SQLite commit cùng nhau phạm vi xóa, biên nhận idempotency và
metadata lượt chạy cần giữ. Intake và tự động hóa cũ còn phụ thuộc SQLite.
Chuyển sang PostgreSQL cần hợp đồng
repository cho truy vấn có phạm vi, cập nhật lạc quan, occurrence duy nhất,
fencing claim, outbox và rollback. Adapter và bộ test hợp đồng còn pending.
Xuất offline rồi nhập có kiểm chứng vào database riêng, đối chiếu ID và
digest, sau đó chuyển runtime trong cửa sổ bảo trì. Giữ kho cũ đến khi
xác minh database mới và quy trình phục hồi.

Qt xác minh executable cùng thư mục trên worker có thể hủy. Luồng GUI
sở hữu transport tiến trình và xử lý tối đa bốn frame hoặc bốn mili giây
giữa các callback. Client đọc hết phản hồi hoàn chỉnh trước khi kết thúc
yêu cầu chưa có phản hồi lúc tiến trình thoát. Một timer theo dõi tối đa
32 yêu cầu đã gửi. Lượt đọc chi tiết và lượt đọc sự kiện mỗi loại giữ một
ý định mới nhất và tối đa hai suất gửi. Callback lỗi thời được giải phóng ngay.
Runtime công bố `read_cancellation` để dùng lệnh tùy chọn `request.cancel`.
View kiểm tra lượt mở và revision trước khi áp dụng kết quả.
`event_watermark` là sự kiện cuối trong cùng snapshot SQLite với trang mới nhất.

Dispatcher phân yêu cầu vào writer FIFO, lượt đọc SQLite tương tác hoặc
worker đọc chậm cho Git và hệ thống tệp. Kết nối worker mở sau bootstrap và
dùng chung thẩm quyền cấp preview của runtime. Scheduler và worker lưu trữ
hiện có giữ kết nối riêng. Đường phản hồi chuyển từng frame cho stdout và đóng
pipe bị nghẽn khi quá hạn. Lượt đọc đang chạy có thể bị ngắt trên kết nối riêng.
Thao tác ghi giữ các kiểm tra giao dịch và biên nhận, kể cả khi gửi phản hồi lỗi.
Khóa SQLite có thể làm chậm thao tác ghi trong ngân sách chờ.

Handshake khởi động thao tác chỉ đọc `storage.chats`. Trạng thái lưu trữ, sao lưu,
kiểm tra và thu gọn dùng worker bảo trì hiện có. Truy vấn transcript trả tối đa
bốn tin trong giới hạn byte của trang. Client chèn và cắt theo ranh giới tin nhắn,
đồng thời giới hạn cửa sổ hiển thị. Có thể xem toàn bộ nội dung đã lưu từ menu
tin nhắn. Xem [kiểm chứng hiệu năng](../specs/cirnet-rules/desktop-performance/verification.md).

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

## Soạn thảo governance và snapshot

Qt dùng chung transport stdio riêng có phiên bản với trang repository,
hội thoại và lưu trữ. Bộ dispatch governance kiểm tra request có kiểu.
Adapter SQLite quản lý revision lạc quan, lịch sử công bố bất biến và audit,
quan hệ tài nguyên, gán binding, manifest ghim cho hội thoại và lượt chạy.
Migration 007 bổ sung các bản ghi này. Nháp và tệp tương đối của tài nguyên
được lưu cùng nhau trong database soạn thảo cục bộ. Công bố và khôi phục
chuyển con trỏ trong giao dịch. Chưa có giao dịch công bố qua hệ thống tệp.

Xem trước context và ghim hội thoại dùng chung resolver cho hồ sơ đã chọn
hoặc mặc định và các binding ID cụ thể. Resolver duyệt đồ thị có giới hạn,
chọn revision tiếng Anh đang hoạt động và hoãn tải nội dung kỹ năng hoặc
tham chiếu. Biên nhận ghim metadata revision và digest. Biên nhận chưa chứng
minh engine ngoài đã nạp nội dung. Thực thi chưa hỗ trợ và duyệt có xác thực
vẫn giữ gate. Probe adapter dùng worker có giới hạn và chỉ kiểm tra endpoint
loopback cố định.
