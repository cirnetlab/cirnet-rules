<!-- doc-id: CIRNETRULES-INTERACTION-STATE -->
<!-- revision: 3 -->
<!-- source-sha256: 97cd64bdc938d41e8be81fba192a9f5a8d406fe3327e903df7366a3a404af416 -->
<!-- translation-status: synchronized -->
<a id="interaction-state"></a>

# Trạng thái tương tác Desktop

> Hồ sơ lịch sử cho [source lưu trữ](https://github.com/cirnetlab/cirnet-rules/blob/52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c/vi/references/desktop-interaction.md). Xem [quy trình CLI hiện hành](cirnetrules-local.md) để vận hành.

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
Nền hàng, trạng thái rê chuột và lựa chọn phủ hết chiều rộng vùng bảng.
Giữ độ rộng cột đã lưu và để nguyên nền phần trống bên dưới các hàng.

Danh mục Cài đặt mở đúng chức năng và chỉ tìm tên tùy chọn local. Giảm
chuyển động thuộc nhóm tùy chọn trình bày có revision. Toast xếp xuống từ
góc trên phải trong Hội thoại và xếp lên từ góc dưới phải ở các trang khác.
Giữ quyết định qua hộp thoại riêng với thông báo và dành chỗ cho ô soạn
tin cùng nút Thao tác của Hội thoại.

[Hướng dẫn vận hành](dashboard.md) · [Kiến trúc runtime](dashboard-architecture.md)
