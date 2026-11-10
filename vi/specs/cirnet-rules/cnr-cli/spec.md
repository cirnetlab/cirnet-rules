<!-- doc-id: CNR-CLI-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: 90e0b416c4fdf9ebac295e5a4103681cb7c29c81eff48c464e045bdeed2c8234 -->
<!-- translation-status: synchronized -->
<a id="cnr-acceptance"></a>

# Tiêu chí nghiệm thu CNR

- [x] Binary riêng dùng chung lõi governance hiện hành. Envelope lệnh bảo trì và IPC Desktop giữ tương thích.
- [x] Menu terminal thao tác được bằng bàn phím và cung cấp trợ giúp thuần văn bản ngoài TTY. CLI xác minh phạm vi tùy chọn, trả JSON sạch và hỗ trợ hoàn thành lệnh.
- [x] Dự án có thể khởi tạo, chọn profile và phiên bản công cụ cụ thể, khóa đầu vào native, lập kế hoạch và đồng bộ môi trường, xuất hoặc nhập cấu hình.
- [x] Một danh mục provider phục vụ phát hiện và lập kế hoạch cài đặt. Thao tác ghi đòi hỏi quyền, nằm trong phạm vi sở hữu dự án, có chạy thử và lưu biên nhận có thể phục hồi.
- [x] Lệnh build, run, test, exec và package áp dụng chính sách môi trường và thực thi. Kiểm tra giới hạn output, timeout, hủy, xung đột phiên bản và an toàn đối số Windows.
- [x] Kiểm chứng Rust, governance, bản dịch và tương thích build Desktop. Ghi nhận các ca nền tảng và provider chưa có bằng chứng fixture hoặc native.
