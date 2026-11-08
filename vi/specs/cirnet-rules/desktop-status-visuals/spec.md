<!-- doc-id: CGACP-DESKTOP-STATUS-VISUALS-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: 8e0c64b4886ddf3884ecdcdb397effc2f3a87fae66162fe4525d4328edfe6c07 -->
<!-- translation-status: synchronized -->
<a id="acceptance"></a>

# Tinh chỉnh trạng thái và giao diện Desktop

Phạm vi công việc là lớp giao diện Qt. Dùng runtime và Git fixture riêng để kiểm chứng.

- [x] Local hiển thị trạng thái kết nối khi bấm chuột hoặc kích hoạt bằng bàn phím và không mở menu điều hướng.
- [x] Chọn repo tô hết chiều rộng hàng, kể cả vùng dư theo chiều ngang, đồng thời giữ định danh binding và các thao tác của bảng.
- [x] Refresh và các icon Desktop liên quan dùng nét vector nhất quán, dễ đọc trong hai chủ đề.
- [x] Icon, nút đóng và nội dung toast được căn chỉnh. Mức độ thông báo dễ nhận biết, giữ xuống dòng, lịch sử và xếp chồng.

Bốn kiểm tra hồi quy tái hiện lỗi trước khi sửa và đều đạt sau khi sửa
trên Qt 6.11.2 cho Windows. Fixture kho mã kiểm tra màu chọn và định danh
binding sau khi bấm trong ô và vùng dư. Kiểm tra icon bao phủ hai bảng màu
và kích thước pixel vật lý. Kiểm tra toast bao phủ bốn mức độ và xác minh
lượt tự động hóa đang chờ trong hàng đợi hiện thông báo thông tin.
