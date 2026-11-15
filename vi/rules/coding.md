# Chọn cách triển khai nhỏ nhất nhưng nhất quán

Dùng phần bổ trợ này cho quyết định thiết kế hoặc tái cấu trúc sau khi đọc mã bị
ảnh hưởng và [bộ quy tắc nền tảng](../AGENTS.md).

- Lần theo đầu vào qua ranh giới hiện có đến đầu ra. Đọc nơi gọi trực tiếp,
  contract và test để giải quyết điểm chưa rõ. Nêu các giả định ảnh hưởng đến
  tương thích, dữ liệu hoặc an toàn. Nếu vẫn còn cách hiểu hợp lý cho kết quả
  khác biệt đáng kể, tóm tắt các đánh đổi và hỏi về quyết định chưa rõ. Với chi
  tiết thông thường, nêu giả định có thể đảo ngược rồi tiếp tục.
- Ưu tiên mở rộng luồng đã có trước khi thêm nguồn dữ liệu chuẩn thứ hai. Dùng
  helper nhỏ nếu đủ đáp ứng yêu cầu. Khi thêm phụ thuộc, cấu hình hoặc điểm mở
  rộng, căn cứ vào nhu cầu hiện tại và cân nhắc chi phí bảo trì. Tối ưu dựa
  trên vấn đề đã quan sát hoặc mục tiêu đã xác định. Ưu tiên mã dễ đọc hơn
  việc giảm số dòng.
- Giữ kiểm tra đầu vào không tin cậy, phân quyền, xử lý hết thời gian chờ, hủy,
  dọn dẹp, kiểm tra toàn vẹn và cơ chế liên động an toàn ở nơi cần thiết. Giữ
  nguyên lỗi mà nơi gọi nhận được và các hành vi quan sát được khác, trừ phần
  nhiệm vụ yêu cầu thay đổi.
- Gắn từng nhóm trong diff với yêu cầu, lỗi quan sát được hoặc kiểm tra và tài
  liệu cần cho thay đổi. Theo phong cách có sẵn. Giữ nguyên cách đặt tên, định
  dạng, kiến trúc không liên quan và mã thừa có từ trước. Trước khi xóa import,
  biến hoặc helper trở nên thừa do thay đổi này, kiểm tra nơi gọi, quy trình
  sinh mã và cách sử dụng động nếu có.
- Sinh tệp dẫn xuất từ nguồn và lệnh do chủ sở hữu quy định. Nếu bên cung cấp hoặc
  bộ sinh không sẵn có, báo phần còn thiếu thay vì sửa đầu ra sinh tự động như thể
  đó là nguồn chuẩn.
- Cập nhật tài liệu hiện có làm nguồn chuẩn khi thay đổi hành vi. Tìm theo mục
  đích và chủ sở hữu trước khi thêm Markdown. Tuân thủ [chính sách tài liệu](../policies/documentation.md)
  và quy tắc cục bộ về dịch thuật, vị trí kế hoạch/đặc tả, phụ thuộc và kiến trúc.
  Chọn ngôn ngữ và phiên bản công cụ trong phạm vi repository được phép.

<!-- source: en/rules/coding.md -->
<!-- source-sha256: 7baa403f5849e15a03080e3c7c72a3fabd9d44177154c39463e52a1bed8ffa68 -->
