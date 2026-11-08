<!-- doc-id: CGACP-DASHBOARD-UX-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: dec990cf0e320a27e26036e474d29966baf3cbd09d670b681664ab1cc09df216 -->
<!-- translation-status: synchronized -->
<a id="acceptance"></a>

# Nghiệm thu thiết kế lại trải nghiệm Dashboard

CGACP v1 giữ Desktop Qt, runtime Rust và lưu trữ SQLite. Khung ứng dụng, cài đặt và lịch sử thông báo phục vụ người vận hành tại máy cục bộ. Runtime tiếp tục quản lý thẩm quyền của liên kết repository, nguồn gốc tự động hóa và phê duyệt độc lập.

- [x] Ghi nhận baseline và đối chiếu triển khai với các triệu chứng được cung cấp.
- [x] Kiểm chứng các luồng khung ứng dụng, repository, hội thoại, tự động hóa và tùy chọn đã triển khai bằng fixture bên dưới.
- [x] Ghi bằng chứng và giới hạn từ UX-01 đến UX-34 bằng cả hai ngôn ngữ.

## Baseline và quyết định thiết kế

Baseline dùng Git revision 63f4b109 trên Windows 11 build 26300, Qt 6.11.2 và tỷ lệ hiệu dụng 125%. Ảnh fixture native tái hiện logo và tên xếp dọc, sidebar cố định, thanh trạng thái dưới, gallery, checkbox thiếu dấu tick và các mảng nền tối ở Giới thiệu. Mã nguồn xác nhận chế độ cột cố định và đường dẫn extended hiển thị nguyên dạng. Đường dẫn extended hợp lệ chưa phải bằng chứng lỗi đăng ký.

Dùng splitter native và một bề mặt Cài đặt. Token palette và primitive checkbox của Qt giữ các trạng thái widget nhất quán. Record tùy chọn SQLite có revision và chỉ lưu trình bày. Lịch sử toast trong bộ nhớ thuộc phiên hiện hành. Runtime giữ thẩm quyền của catalog và đường dẫn thao tác.

## Bằng chứng nghiệm thu

PASS áp dụng cho các kiểm tra được liệt kê. PARTIAL chỉ tình huống có bằng chứng một phần và ghi rõ kiểm tra còn thiếu. Dữ liệu trong ảnh là dữ liệu dùng một lần. Ảnh native dùng nền tảng Qt Windows và widget Dashboard thật. Tên artifact và log phân biệt dữ liệu fixture với tích hợp runtime Rust thật.

| ID | Kết quả | Bằng chứng |
| --- | --- | --- |
| UX-01 | PASS | Geometry và ảnh native xác nhận cú, CGACP và CIRNETLAB cùng hàng. |
| UX-02 | PASS | Kéo chuột đổi độ rộng sidebar và bố trí lại lưới thẻ đang bật. |
| UX-03 | PASS | Thu gọn vẫn giữ các nút biểu tượng. SQLite khôi phục trạng thái và độ rộng khi mở rộng. |
| UX-04 | PASS | Footer chỉ có Local và Cài đặt, phản ánh kết quả bắt tay thật. |
| UX-05 | PASS | Kho mã và Giới thiệu nằm trong Cài đặt. Quay lại trả về hội thoại hoặc form đang dùng. |
| UX-06 | PARTIAL | Dark và Light đổi giao diện, System được giữ sau khi mở lại và lỗi lưu vẫn hiện rõ. Chưa thử đổi chủ đề hệ điều hành khi ứng dụng đang chạy. |
| UX-07 | PASS | Ảnh native trước và sau cho thấy nền liền mạch ở Giới thiệu, thẻ, bảng và hộp xác nhận repository. |
| UX-08 | PASS | Pixel hiển thị dấu tick ở cả hai chủ đề, kể cả khi focus và khi vô hiệu hóa. Click nhãn và Space đổi trạng thái. |
| UX-09 | PASS | Kéo chuột và nhấp đúp fit đổi độ rộng cột đường dẫn. Làm mới giữ độ rộng. |
| UX-10 | PASS | Client Qt mở lại khôi phục độ rộng, thứ tự, sắp xếp, cuộn ngang và binding đã chọn. Đặt lại giữ binding đã lưu trữ. |
| UX-11 | PASS | Tooltip và clipboard offscreen giữ đường dẫn canonical đầy đủ. Handler file URL của Qt ghi nhận thư mục đích đã kiểm tra. |
| UX-12 | PASS | Windows trình bày riêng đường dẫn ổ đĩa thông thường. SQLite và IPC thật giữ root canonical dạng extended và định danh binding. |
| UX-13 | PARTIAL | Đăng ký tệp qua Qt và fixture Rust bao phủ Unicode, dấu cách, linked worktree, detached HEAD, mở lại và liên kết lại. Chưa tự động kiểm thử hộp chọn gốc Windows. |
| UX-14 | PARTIAL | Fixture Rust phân biệt đường dẫn thiếu, thư mục ngoài Git và đường dẫn UNC hoặc thiết bị bị chặn. Chưa tái hiện metadata Git không thể truy cập. |
| UX-15 | PASS | Đổi tên giữ ID. Liên kết lại giữ binding và tăng revision. Test runtime từ chối binding automation đã stale. |
| UX-16 | PASS | Fixture 41 hàng kiểm tra chọn theo ID sau phân trang, sắp xếp, lọc và làm mới. Thao tác Qt thật kiểm chứng đổi tên và lưu trữ theo binding đã lưu. |
| UX-17 | PASS | Lưu trữ bảo toàn tệp và hiển thị hàng đã lưu trữ qua Hiện mục đã lưu trữ. |
| UX-18 | PASS | Ảnh native và kiểm tra geometry xác nhận thanh thông báo cũ không còn chiếm chỗ. |
| UX-19 | PARTIAL | Qt kiểm tra hàng đợi, đóng toast, vị trí, co giãn và giữ focus. Chưa đo riêng thời gian tạm dừng khi rê chuột. |
| UX-20 | PASS | Fixture kiểm tra giới hạn lịch sử, gộp thông báo, chế độ ít thông báo và hiển thị lỗi. Xác nhận chạy vẫn dùng hộp thoại. |
| UX-21 | PASS | Ảnh không còn gallery và fixture không ghi nhận request runtime chỉ phục vụ gallery. |
| UX-22 | PASS | Runtime thật tạo record từ mẫu, mở chỉnh sửa và xác nhận nguồn gốc cùng lịch sử lượt chạy được giữ. |
| UX-23 | PASS | Trạng thái trống và nút thêm automation hiển thị với fixture trống. |
| UX-24 | PASS | Qt kiểm tra hai cột ở viewport rộng và một cột ở viewport hẹp, giữ định danh thẻ. Ảnh native ở các DPI cho thấy nút thao tác vẫn nhìn thấy. |
| UX-25 | PARTIAL | Rust kiểm tra tính lũy đẳng và gate quyền cũ. Qt thật xác nhận lượt chạy một lần và thấy ID. Chưa tiêm lỗi để kiểm tra chạy lại trên giao diện native. |
| UX-26 | PARTIAL | Đã kiểm tra một lịch sử ở Sidebar và hành động tạo hội thoại dùng chung. Runtime kiểm tra lưu bền vững. Chưa thao tác đủ mọi tổ hợp đổi tên, lưu trữ và phân trang qua Qt. |
| UX-27 | PASS | Fixture Qt giữ bản nháp khi đổi hội thoại, vào Cài đặt, đổi chủ đề và kết nối lại. |
| UX-28 | PASS | Hội thoại và automation giữ dữ liệu form và binding khi qua quản lý repository. Fixture phân trang gồm binding ở trang cuối. |
| UX-29 | PARTIAL | Có kiểm tra tự động Space, Escape, điều khiển bấm được, focus và tên truy cập. Chưa thử trình đọc màn hình bên ngoài. |
| UX-30 | PARTIAL | Resize giữ thẻ ổn định và không gửi request catalog, source hay release. Đã đo khởi động với fixture. Chưa benchmark dữ liệu quy mô vận hành. |
| UX-31 | PASS | Database v3 nâng lên v4 sau khi xác minh sao lưu. Fixture migration Windows và Linux giữ ID binding, hội thoại, task, lượt chạy và nguồn gốc. |
| UX-32 | PASS | Cài đặt hiển thị trạng thái bàn giao, thực thi cô lập bị chặn và offline. Lượt chạy một lần thật vẫn theo luồng phê duyệt. |
| UX-33 | PASS | Bản dịch Qt tải được. Ảnh native bao phủ tiếng Việt và kích thước cửa sổ đích ở bốn tỷ lệ hiển thị hiệu dụng. |
| UX-34 | PASS | Bàn giao phân biệt ảnh native, fixture Qt, tích hợp Rust và các kiểm tra chưa thực hiện. Release chưa được công bố. |

## Môi trường kiểm chứng

Build Qt/MSVC trên Windows, CTest offscreen và các bộ test Qt native dùng dữ liệu tạm. Kích thước ảnh đích là 1366×768 và 1920×1080 pixel vật lý, có sai số làm tròn của Qt. Các mức hiệu dụng 100%, 150% và 200% dùng scaling theo tiến trình. Máy chạy ở mức 125%. Chưa thử chuyển giữa màn hình vật lý khác DPI, trình đọc màn hình hệ thống và UI native Linux. So sánh clipboard Windows chạy offscreen vì sandbox có thể chặn COM. Test mở thư mục chặn URL ở Qt để xác minh đích mà không khởi động Explorer.

Bộ test Rust dùng Rust/Cargo 1.99.1 đã ghim và dependency khóa. Container Linux kiểm tra migration. Dữ liệu và checkout các repository sản phẩm nằm ngoài phạm vi kiểm chứng.

Build CMake, CTest, các bộ test Qt native, kiểm tra định dạng Cargo, Clippy, build khóa dependency, test và validator governance trên Windows đều trả mã 0. Rust Windows chạy 68 bài test. CTest chạy một bộ test. Lượt native 125% ghi nhận 11 mục Qt Test đạt, kể cả khởi tạo và dọn dẹp. Các tỷ lệ còn lại chạy tình huống bố cục, trạng thái trống và tiếng Việt. Ứng dụng độc lập cũng mở cửa sổ native và khởi tạo fixture SQLite riêng. Validator kiểm tra 67 cặp tài liệu EN/VI.

Tỷ lệ tương phản đo được giữa chữ chính và nền là 15.03:1 ở Dark và 15.07:1 ở Light. Tương phản của đường viền điều khiển trên surface lần lượt là 3.36:1 và 4.49:1. Các phép đo áp dụng cho những cặp màu nêu trên.
