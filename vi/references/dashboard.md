<!-- doc-id: CGACP-DESKTOP-OPERATOR -->
<!-- revision: 11 -->
<!-- source-sha256: f862d9a7f32de1ee283082722000fe888c722f2779d488389acd6a0c09f4a6bf -->
<!-- translation-status: synchronized -->
<a id="operator"></a>

# Dashboard CGACP

Thế hệ sản phẩm là CGACP v1. Phiên bản ứng dụng lấy từ Cargo.
Tên release ứng viên local là `cirnet-rules-0.1.0`. Rà soát release tách
riêng khỏi thao tác repository, hội thoại và automation hằng ngày.

## Build và chạy

Tại thư mục gốc repo, chạy `run-dashboard.bat` trên Windows hoặc
`sh ./run-dashboard.sh` trên Linux. Mỗi launcher build runtime Rust và
desktop Qt từ source hiện tại, rồi mở dashboard. Có thể truyền tùy chọn
ứng dụng sau lệnh launcher.

Launcher Windows cần Visual Studio 2022 với bộ công cụ build C++. Trên
Linux, CMake cần tìm được trình biên dịch C++ và các gói phát triển Qt 6
Widgets và LinguistTools.

Dùng Rust toolchain đã pin và bộ Qt 6 Widgets cho trình biên dịch C++ local.
Build `cgacp-runtime` bằng Cargo, rồi cấu hình CMake với
`CGACP_RUNTIME_EXECUTABLE` trỏ đến binary vừa build. Build
`cgacp-dashboard` và chạy CTest. Khi chuẩn bị gói Windows, cài vào thư mục
staging riêng và bật `CGACP_DEPLOY_QT`. Mở Dashboard đã cài từ thư mục bin.
Dùng `--data-dir` với thư mục tuyệt đối riêng để kiểm thử dùng một lần.

## Quy trình vận hành

Kéo đường phân cách để đổi độ rộng sidebar. Dùng nút panel hoặc Ctrl+B
để thu gọn hay mở rộng. Kích hoạt Local ở chân sidebar để xem trạng thái
kết nối hiện tại. Mở Cài đặt từ chân sidebar hoặc bằng Ctrl+, rồi dùng biểu
tượng danh mục hoặc ô tìm kiếm bên trái. Mục Giao diện có bản xem trước chế
độ, các màu nhấn định sẵn và màu nhấn tùy chọn. Nhập mã hex cho màu nền và
màu chữ hoặc bấm vòng tròn để mở bảng chọn màu. Xóa mã để dùng màu của chủ đề.
Có thể chọn phông hệ thống, phông đơn cách hoặc một họ phông đã cài đặt.
Mở Nâng cao để chỉnh cỡ chữ giao diện, hội thoại, mã và khoảng đệm dọc bằng
pixel, đồng thời chọn ngôn ngữ và Giảm chuyển động. Tiếng Anh là mặc định.
Tiếng Việt và tiếng Trung giản thể áp dụng khi mở lại Dashboard.
Khôi phục mặc định xuất hiện khi giao diện khác cấu hình mặc định. Nút
Quay lại hoặc Escape đưa về trang trước. Thanh cuộn hiện khi rê chuột lên
thanh hoặc trong chốc lát sau khi lăn. Con lăn trên trường giá trị cuộn vùng
bao quanh. Dùng bàn phím hoặc các nút của trường để đổi giá trị.
Toast hiện ở góc trên phải trong Hội thoại và góc dưới phải ở các trang khác.
Toast đang hiện đổi vị trí theo trang, chừa chỗ cho ô soạn tin. Biểu tượng và
màu thể hiện mức độ thông báo. Lượt tự động hóa đang chờ trong hàng đợi
hiện toast thông tin. Mở Thông báo để xem hoặc xóa mục đã đọc trong phiên.
Cảnh báo và lỗi vẫn có thể hiện toast.

Trong Cài đặt → Kho mã, thêm thư mục hoặc tệp, rồi rà Git root, branch,
worktree và tệp gợi ý trước khi lưu. Chọn cho nhiệm vụ hoặc chọn binding
khi tạo hội thoại hay tự động hóa. Menu thao tác hỗ trợ kiểm tra tình trạng,
sao chép đường dẫn đầy đủ, mở thư mục local đã kiểm tra, đổi tên, liên kết
lại hoặc lưu trữ. Kho mã và Lịch sử chạy tự động hóa tô sáng theo toàn hàng.
Trong Kho mã, có thể bấm để chọn hàng cả ở phần dư sau cột cuối.
Bảng có thể rút gọn đường dẫn dài khi hiển thị. Thao tác
sao chép dùng đường dẫn đã lưu. Liên kết lại khi checkout đổi vị trí làm
revision binding cũ mất hiệu lực và công việc liên quan cần được rà soát.
Lưu trữ giữ các tham chiếu lịch sử và tệp nguồn.

Hội thoại mới mở một cuộc trò chuyện. Ctrl+Enter gửi tin nhắn và tạo đề xuất
task theo phạm vi. Đổi hội thoại qua sidebar hoặc dùng Quay lại từ Cài đặt
để tiếp tục bản nháp. Cuộn tới cuối danh sách để tải hội thoại cũ hơn, hoặc
chọn Tải thêm trong menu ngữ cảnh khi còn trang. Bản nháp chưa gửi nằm trong
ứng dụng đang mở. Profile
handoff hiển thị trạng thái chờ duyệt. Có thể xem tin nhắn đã lưu sau khi
mở lại ứng dụng. Automation hỗ
trợ chạy thủ công và lịch hằng ngày theo múi giờ IANA. Xem trước lịch trước
khi bật và xác nhận lượt chạy một lần khi automation đang tắt. Runtime Rust
quản lý lịch. Để giữ lịch khi đóng Dashboard, chạy
`cgacp-runtime --scheduler` trong terminal riêng với cùng thư mục dữ liệu.
Trạng thái lịch được lưu khi runtime dừng. Các lượt hằng ngày bị lỡ được gộp.
Definition bị tạm dừng hoặc stale phải được kiểm tra trước khi tạo đề xuất mới.

## Lưu trữ và vận hành

Rust sở hữu SQLite trong thư mục dữ liệu ứng dụng local hoặc thư mục do
người vận hành chỉ định. Trước khi nâng kho hiện có lên schema 6, runtime
tạo snapshot đã kiểm tra toàn vẹn. Runtime cũ cần bản sao tương thích đã
rà trong thư mục riêng. Tổng quan Lưu trữ tự tải khi runtime kết nối.
Trang báo số hội thoại và số byte UTF-8 logic của hội thoại đang dùng và
đã lưu trữ. Dùng Làm mới để cập nhật số liệu. Bảo trì cơ sở dữ liệu có
các thao tác riêng để sao lưu, kiểm tra toàn vẹn và thu gọn. Thu gọn yêu cầu
kiểm tra cơ sở dữ liệu mới và xác nhận số trang đã quan sát. Thao tác nền
báo tiến độ và hỗ trợ hủy. Khi diễn tập khôi phục, đóng
runtime, phục hồi bản sao đã rà vào thư mục riêng mới và kiểm tra tại đó
trước khi đổi đường dẫn. Giữ thư mục gốc và các file WAL. Bản sao lưu là
dữ liệu nhạy cảm local.

## Vòng đời hội thoại và Cài đặt

Xóa bản nháp tác động đến văn bản chưa gửi và có hoàn tác trong phiên.
Dùng menu ngữ cảnh bản ghi hội thoại để xóa tin đã gửi, hoặc Thao tác để
xóa hội thoại. Rà ID ổn định, phạm vi local và đối tượng đang chặn trước
khi xác nhận. Hủy hoặc xử lý riêng các nhiệm vụ và lượt chạy đang hoạt
động trước khi thử lại. Xóa gỡ payload SQLite đang dùng và dữ liệu dẫn xuất
local. Dashboard không khôi phục được thao tác này. Trang SQLite trống,
WAL, bản sao lưu và bản sao bên nhà cung cấp có thể còn byte dữ liệu.
Nếu Hội thoại do một lượt chạy tạo ra, bản xem trước chỉ ra Automation gốc.
Xác nhận xóa cũng gỡ định nghĩa Automation và dừng lịch của nó. Lượt chạy
đang hoạt động của định nghĩa đó chặn xóa. Tệp kho mã, các Hội thoại khác
và metadata kiểm toán lượt chạy đã kết thúc vẫn truy cập được.
Dùng Thao tác → Xóa Automation để xóa Automation độc lập hoặc thẻ còn lại
sau khi Hội thoại đã bị xóa từ trước.
Lưu trữ giữ lịch sử. Trong Hội thoại đã lưu trữ, tìm hoặc lọc danh sách,
rồi bỏ lưu trữ hoặc xóa vĩnh viễn ngay trên thẻ tương ứng. Bỏ lưu trữ giữ định danh và không lên lịch
chạy công việc.

Cấu hình hiển thị capability hiệu lực và nguồn thẩm quyền của runtime.
Thực thi native giữ HANDOFF_ONLY khi áp dụng. Bảng màu, phông giao diện,
kích thước pixel và ngôn ngữ được lưu qua API tùy chọn cục bộ.
Khôi phục mặc định áp dụng cho tùy chọn giao diện.

Có thể xem trước và lưu trữ kho file CLI đã dừng bằng `legacy.preview`
và `legacy.apply`. Áp dụng cần digest manifest đã rà và xác nhận tiến
trình ghi cũ đã dừng. Dùng `legacy.inspect` và `legacy.link` để xem hồ
sơ lưu trữ và gắn alias cũ với binding đã rà riêng. Nhập giữ nguyên nguồn
và không kích hoạt lại grant, lease hoặc lượt chạy cũ.

Chạy `scripts/package-dashboard.ps1` trên cây staging Windows đã cài để
ghi SHA-256 artifact và nguồn build. Ứng viên giữ `RELEASE_BLOCKED` khi
còn gate Agent native, cô lập, review xác thực, cài Windows sạch và
điều kiện phân phối.

[Kiến trúc và giới hạn](dashboard-architecture.md) · [Trạng thái tương tác](desktop-interaction.md) · [Ứng viên phát hành](dashboard-release.md) · [Quy trình task](cgacp-local.md)
