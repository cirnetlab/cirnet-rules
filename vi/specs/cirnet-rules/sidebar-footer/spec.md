<!-- doc-id: CGACP-SIDEBAR-FOOTER-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: ab75df0971e1fd0b581a5180079ed05d83d8b5adfed0bb94b50efd0497533a63 -->
<!-- translation-status: synchronized -->
<a id="acceptance"></a>

# Kiểm chứng sidebar Dashboard

- [x] Bỏ nút phân trang hội thoại riêng và kiểm chứng vẫn tải được hội thoại cũ bằng cách cuộn hoặc dùng menu ngữ cảnh.
- [x] Kiểm chứng footer Local và Cài đặt gọn, hoạt động khi mở rộng hoặc thu gọn sidebar.
- [x] Kiểm chứng build Qt Windows và tài liệu vận hành đồng bộ.

Fixture phân trang tải 30, 30 và 5 dòng trong ba request. Phím End tải trang thứ hai và menu ngữ cảnh tải trang thứ ba. Nhấn End thêm lần nữa ở trang cuối ngắn không gửi request mới. Build Qt 6.11.2 trên Windows, kiểm tra native ở 125%, CTest và kiểm chứng governance đều đạt.
