<!-- doc-id: CGACP-DESKTOP-ROW-HOVER-TOAST-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: 0bb80fb4091bc5e81c57913bd614c1248f0fbce3381f376f0457e7d0c09c3da0 -->
<!-- translation-status: synchronized -->
<a id="acceptance"></a>

# Nền hàng bảng và toast trong Hội thoại

Phạm vi gồm hai bảng Desktop là Kho mã và Lịch sử chạy tự động hóa.
Dùng dữ liệu tạm để kiểm chứng.

- [x] Rê chuột tô cả hàng có dữ liệu ở hai bảng, kể cả vùng sau cột cuối nếu có. Nền xen kẽ phủ cùng chiều rộng và khoảng trống bên dưới các hàng giữ nền thường.
- [x] Thao tác chọn, bàn phím, chiều rộng cột Kho mã đã lưu và hủy lượt chạy giữ đúng hành vi hiện có.
- [x] Toast chuyển lên góc trên phải khi mở Hội thoại và trở lại góc dưới phải ở các trang khác. Thẻ đang hiển thị được giữ khi chuyển trang theo thời gian hết hạn và không gian hiển thị.

Bài kiểm thử Qt lấy mẫu nền hàng trong cả hai chủ đề, xác nhận hủy lượt
chạy đã chọn bằng dữ liệu SQLite tạm và theo dõi cùng một toast khi
người dùng chuyển trang rồi bấm vào ô soạn tin.
