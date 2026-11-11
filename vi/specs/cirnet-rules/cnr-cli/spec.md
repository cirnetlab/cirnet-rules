<!-- doc-id: CNR-CLI-SPEC -->
<!-- revision: 4 -->
<!-- source-sha256: fe8fd679d1f3ed3f764f2a2612fcee20ac2815d8494b542c6a66c59096bb9165 -->
<!-- translation-status: synchronized -->
<a id="cnr-acceptance"></a>

# Tiêu chí nghiệm thu CNR

- [x] Binary riêng dùng chung lõi governance. Dạng kết quả của các lệnh bảo trì giữ tính tương thích.
- [x] Menu terminal thao tác được bằng bàn phím và cung cấp trợ giúp thuần văn bản ngoài TTY. CLI xác minh phạm vi tùy chọn, trả JSON sạch và hỗ trợ hoàn thành lệnh.
- [x] Dự án có thể khởi tạo, chọn profile và phiên bản công cụ cụ thể, khóa đầu vào native, lập kế hoạch và đồng bộ môi trường, xuất hoặc nhập cấu hình.
- [x] Một danh mục provider phục vụ phát hiện và lập kế hoạch cài đặt. Thao tác ghi đòi hỏi quyền, nằm trong phạm vi sở hữu dự án, có chạy thử và lưu biên nhận có thể phục hồi.
- [x] Lệnh build, run, test, exec và package áp dụng chính sách môi trường và thực thi. Kiểm tra giới hạn output, timeout, hủy, xung đột phiên bản và an toàn đối số Windows.
- [x] Kiểm chứng Rust, governance, bản dịch và tương thích build CLI. Ghi nhận các ca nền tảng và provider chưa có bằng chứng fixture hoặc native.

## Tiêu chí nghiệm thu terminal

- Terminal rộng hiển thị chữ `CIRNET RULES` màu cam Claude với cùng kích thước, đường chân chữ và bóng đổ lệch màu sẫm. Terminal hẹp hoặc thấp dùng tiêu đề gọn.
- Điều hướng bằng phím mũi tên giữ màn hình ổn định và chỉ vẽ các dòng thay đổi. Khi đổi kích thước, bố cục được tính lại. Khi trở về từ một lệnh, đầu vào và đầu ra terminal được khôi phục.
- Giao diện mặc định hiển thị danh sách lệnh. Phần cuối dành cho tìm kiếm, nhập lệnh và trợ giúp tùy chọn khi được kích hoạt.
- `cnr`, `--no-banner`, `--lang vi`, `NO_COLOR` và CLI phi tương tác giữ vai trò hiện có. Bộ phân tích lệnh, kiểm tra quyền và điểm vào MCP dùng lõi hiện hành.

## Tiêu chí dự kiến cho terminal tương tác

- Hiển thị kết quả lệnh trong ứng dụng, giới hạn và làm sạch đầu ra, hỗ trợ cuộn và hủy. Menu vẫn sử dụng được sau khi lệnh kết thúc hoặc bị hủy.
- Chạy lệnh bằng các vector đối số tách biệt và mặc định phi tương tác. Không tự cấp quyền; giữ nguyên các bước kiểm tra quyền và xác nhận tường minh đối với thao tác ghi.
- Cung cấp gợi ý lệnh và hoàn thành bằng Tab, gọi lại lịch sử lệnh và thao tác xóa đầu ra đang hiển thị. Các điều khiển vẫn sử dụng được trong bố cục gọn, đáp ứng kích thước màn hình.
- Tinh chỉnh chữ `CIRNET RULES` trên terminal rộng với bóng lệch tạo chiều sâu theo các hình tham khảo được cung cấp. Giữ chữ dễ đọc, tiêu đề thay thế cho màn hình hẹp và chế độ `NO_COLOR`.
- Kiểm tra điều phối lệnh, từ chối và hủy, giới hạn và làm sạch đầu ra, hoàn thành lệnh, lịch sử, xóa, cuộn, đổi kích thước, chế độ đơn sắc và hình thức hiển thị. Ghi nhận giới hạn khi không thể kiểm tra tương tác hoặc nền tảng.
