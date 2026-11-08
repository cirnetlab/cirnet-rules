<!-- doc-id: CGACP-DESKTOP-PERFORMANCE-LIFECYCLE-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: 925040c030f8d9f75ebdce3eeff0c3c5667c5ad55acb9b55137c88a9338fc062 -->
<!-- translation-status: synchronized -->
<a id="acceptance"></a>

# Hiệu năng, vòng đời dữ liệu và Cài đặt Desktop

Giữ CGACP v1 và ứng dụng 0.1.0. Chỉ kiểm chứng trên workspace và thư mục
SQLite dùng một lần. Báo cáo từng ca theo môi trường kiểm thử.

- [x] Ghi bằng chứng baseline và sau thay đổi có thể so sánh.
- [x] Kiểm chứng IPC Qt có giới hạn và worker lưu trữ hỗ trợ hủy.
- [x] Kiểm chứng vòng đời SQLite, khởi động lại và bảo vệ lượt chạy đang hoạt động.
- [x] Kiểm chứng các chức năng Cài đặt đã triển khai và phạm vi quyền của chúng.
- [x] Đồng bộ hướng dẫn vận hành và ghi các điều kiện chưa đạt.
- [ ] Hoàn tất các ca còn mở trong ma trận 54 ca nghiệm thu.

Các ca còn mở gồm dọn file theo ownership, xóa hàng loạt, ghi cấu hình
native, các dịch vụ còn phụ thuộc SQLite và kiểm thử chèn lỗi, tài nguyên
bổ sung. Sẵn sàng phát hành là gate riêng.
