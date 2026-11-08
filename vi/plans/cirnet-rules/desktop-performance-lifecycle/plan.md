<!-- doc-id: CGACP-DESKTOP-PERFORMANCE-LIFECYCLE-PLAN -->
<!-- revision: 1 -->
<!-- source-sha256: 54de64b794e13ebaed6d061786cc32bf425c937d6fcb3ea9435cffefd19786b7 -->
<!-- translation-status: synchronized -->
<a id="steps"></a>

# Kế hoạch hiệu năng và vòng đời dữ liệu Desktop

[Phạm vi và nghiệm thu](../../../specs/cirnet-rules/desktop-performance-lifecycle/spec.md)

1. Đo bản Windows hiện tại và workload SQLite riêng. Ghi quan sát từ video riêng với số đo.
2. Giới hạn công việc IPC Qt và chuyển thao tác lưu trữ dài sang worker có thể hủy.
3. Thêm giao dịch vòng đời có kiểu dữ liệu, migrate database thử nghiệm và nối thao tác hội thoại với phân trang.
4. Nối Cấu hình, Lưu trữ, Hội thoại đã lưu trữ và Giao diện với capability thực của runtime.
5. Kiểm chứng Windows, SQLite, IPC và governance. Ghi số đo so sánh, kết quả từng ca và giới hạn còn lại.
