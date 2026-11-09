<!-- doc-id: CIRNETRULES-GOVERNANCE-WORKSPACE -->
<!-- revision: 1 -->
<!-- source-sha256: 00eabfac828261a9c5eecd4219726a61f0f924e45228b9a09fbf4f9e619ea3cb -->
<!-- translation-status: synchronized -->
<a id="workspace"></a>

# Không gian quản trị governance

Mở **Cài đặt → Cấu hình**. Thư viện dùng một danh mục phân trang cho kỹ năng, quy tắc, hồ sơ, tham chiếu, chính sách, lệnh và hook. Lọc theo loại hoặc tìm theo tên. Chọn một hàng để mở bản nháp hiện tại hoặc revision đã công bố. Trang Agent và Runtime giữ thông tin chẩn đoán và các giới hạn thực thi.

## Nháp, duyệt và công bố

1. Chọn **Tạo tài nguyên**, nhập tên hiển thị và slug ổn định, rồi chọn loại tài nguyên. Với kỹ năng, nhập Markdown chuẩn có `name` và `description` trong frontmatter. Dùng **Nhập tệp** để xem trước nguồn UTF-8 và tạo nháp cục bộ có lưu tên tệp làm nguồn gốc.
2. Sửa **EN · Bản chuẩn** và **VI · Bản dịch**. Dùng **Tệp → Thêm tệp** cho văn bản có đường dẫn tương đối trong tài nguyên, hoặc **Đính kèm văn bản** để chọn một tệp cục bộ. Trường ID phụ thuộc lưu các liên kết ổn định. Dùng Ctrl+F để tìm và thay thế trong editor hiện tại, Ctrl+S để lưu nháp.
3. Lưu hoặc chờ tự lưu rồi chọn **Kiểm tra**. Xem chẩn đoán và trạng thái dịch. Khi bản tiếng Anh thay đổi, rà soát bản tiếng Việt tương ứng. Xem trước Markdown tắt HTML và tải tài nguyên.
4. Dùng **Xem khác biệt** để kiểm tra nội dung, rồi chọn **Duyệt** và **Áp dụng**. Áp dụng yêu cầu digest của bản nháp đã duyệt và revision hiện tại. Có thể soạn nháp lệnh, hook và chính sách. Công bố cần xác thực người duyệt hoặc cần ranh giới thực thi vẫn bị chặn khi thiếu điều kiện.
5. Dùng **Lịch sử** để xem các revision đã công bố bất biến. Khôi phục một snapshot hợp lệ tạo revision mới. **Thao tác tài nguyên** gồm đổi tên, sao bản, lưu trữ, khôi phục, loại khỏi danh mục và xuất. Lưu trữ hoặc loại bỏ sẽ bị chặn nếu còn bên dùng. Tải lại revision đã lưu khi cửa sổ khác đã sửa cùng tài nguyên.

Dùng **Gán phạm vi** để gán tài nguyên đã công bố cho một binding repository. Trong **Context hiệu lực**, chọn binding và chế độ. Hồ sơ đang chọn ở Thư viện được đưa vào phần xem trước. Kết quả hiển thị kích thước đầu vào tiếng Anh, kỹ năng và tham chiếu được hoãn nạp, nguồn, revision và mục bị loại. Đây là mô phỏng. Phạm vi runtime và các chặn cứng vẫn áp dụng.

## Nguồn chuẩn và ranh giới

Dịch vụ soạn thảo cục bộ lưu văn bản, bản nháp, đường dẫn tệp ảo, revision đã công bố bất biến và audit trong một nguồn SQLite qua DAL của Dashboard. Thao tác công bố chuyển con trỏ và ghi audit trong cùng giao dịch SQL. Phiên bản này chưa có binding cho phép ghi vào tệp governance của checkout. Nhập tệp xem nội dung là dữ liệu đầu vào, còn xuất tạo snapshot đã xem. Tệp trong repository và chỉ dẫn nhập từ công cụ khác không có quyền của operator.

Runtime stdio kiểm tra giao thức trước khi dispatch. Các thao tác ghi dùng định danh yêu cầu và revision kỳ vọng. Lần thử lại với dữ liệu giống nhau dùng lại định danh. Kênh Desktop cục bộ xác định ranh giới operator ở cấp tài khoản hệ điều hành hiện tại. Kiểm tra toàn vẹn SQLite và digest revision không xác thực một người duyệt khác hoặc cô lập tiến trình dùng cùng tài khoản.

Mục Tích hợp có kiểm tra loopback chỉ đọc và phân tích dialect nhập. Biên nhận capability mô tả bề mặt đã kiểm tra và giới hạn. Xem [khả năng adapter và bằng chứng nguồn](adapter-capabilities.md).
