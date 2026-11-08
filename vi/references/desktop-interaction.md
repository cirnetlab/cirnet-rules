<!-- doc-id: CGACP-INTERACTION-STATE -->
<!-- revision: 1 -->
<!-- source-sha256: ef5ee4ecac993437f8f3c085a418411fc262ced2c49a90a92a1f37fac9caa865 -->
<!-- translation-status: synchronized -->
<a id="interaction-state"></a>

# Trạng thái tương tác Desktop

| Trạng thái | Hành vi trên Qt Desktop |
| --- | --- |
| Viền | Dùng đường viền để nhận diện vùng nhập hoặc đích bàn phím hiện tại. Action thông thường và dòng điều hướng dùng khoảng cách và nền. |
| Nền | Giữ nền của action chính, vùng nhập, giá trị đang chọn hoặc vùng nội dung. |
| Rê chuột và nhấn | Đổi nền action trong lúc tương tác rồi trở về trạng thái thường khi nhả và rời con trỏ. |
| Focus bàn phím | Hiện chỉ báo ở đích bàn phím hiện tại. Focus chuột trên action không biểu thị trang đang chọn. Vùng nhập vẫn có dấu hiệu focus. |
| Trang đang mở | Suy ra tô nền điều hướng từ trang đang hiển thị và ID hội thoại đã mở. Lựa chọn trong cache hoặc phản hồi đang chờ không đổi trang. |
| Giá trị đã chọn | Giữ dấu tick và radio đang chọn. Chủ đề và các tùy chọn khác độc lập với điều hướng. |
| Đang xử lý | Dựa vào trạng thái request thật. Giữ phạm vi và cơ chế phê duyệt của runtime. |

Dùng giao diện dùng chung và hình học control Qt gốc cho mũi tên, dấu hiệu
menu và nút tăng giảm số hoặc giờ. Giữ tên truy cập, thao tác bàn phím và
trạng thái vô hiệu hóa thực khi chỉnh control.

Danh mục Cài đặt mở đúng chức năng và chỉ tìm tên tùy chọn local. Giảm
chuyển động thuộc nhóm tùy chọn trình bày có revision. Toast nằm trong
vùng ứng dụng và xếp lên từ góc dưới phải. Giữ quyết định qua hộp thoại
riêng với thông báo và dành chỗ cho ô soạn tin.

[Hướng dẫn vận hành](dashboard.md) · [Kiến trúc runtime](dashboard-architecture.md)
