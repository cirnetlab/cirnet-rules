<!-- doc-id: CNR-CLI-SPEC -->
<!-- revision: 3 -->
<!-- source-sha256: 29d253c266069ab80e2d3ee7e56d24f03300bb445b6bd66acac66b01b67ea8dc -->
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
