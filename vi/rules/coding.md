# Chọn cách triển khai nhỏ nhất nhưng nhất quán

Dùng phần bổ trợ này cho quyết định thiết kế hoặc tái cấu trúc sau khi đọc mã bị
ảnh hưởng và [bộ quy tắc nền tảng](../AGENTS.md).

- Lần theo đầu vào qua ranh giới hiện có đến đầu ra. Ưu tiên mở rộng luồng đã có
  thay vì thêm nguồn dữ liệu chuẩn thứ hai.
- Tách thay đổi hành vi cần thiết khỏi việc dọn dẹp không liên quan. Tái cấu trúc
  phải giữ nguyên hành vi quan sát được, trừ khi nhiệm vụ yêu cầu rõ ràng thay đổi nó.
- Đọc các bên gọi trực tiếp trước khi sửa giao diện. Giữ nguyên ngữ nghĩa xử lý lỗi,
  hủy, hết thời gian chờ và dọn dẹp mà các bên gọi đang dựa vào.
- Sinh tệp dẫn xuất từ nguồn và lệnh do chủ sở hữu quy định. Nếu bên cung cấp hoặc
  bộ sinh không sẵn có, báo phần còn thiếu thay vì sửa đầu ra sinh tự động như thể
  đó là nguồn chuẩn.
- Đánh giá phụ thuộc mới so với khả năng sẵn có, chi phí bảo trì và chính sách phụ
  thuộc của repo. Không chuẩn hóa ngôn ngữ hoặc phiên bản công cụ trên phạm vi toàn cục.
- Cập nhật tài liệu bị ảnh hưởng cùng thay đổi hành vi. Quy tắc cục bộ quyết định
  yêu cầu dịch thuật, vị trí plan/spec và các quyết định kiến trúc.

<!-- source: en/rules/coding.md -->
<!-- source-sha256: b2745a5e183aeb8ffea94361074bff43ac89865ac891eef8904b597257d7248c -->
