<!-- doc-id: CIRNETRULES-DESKTOP-ASYNC-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: dcc53fef84a6af8a00cc908c59e4e2d84d205c270ddc37565c2f39f37ebbff51 -->
<!-- translation-status: synchronized -->
<a id="acceptance"></a>

# Điều phối request và hiệu năng Desktop

Kiểm chứng bản Release trên Windows bằng fixture hội thoại, Git và SQLite dùng một lần. Giữ Desktop, CLI, envelope IPC v3, thư mục dữ liệu, quyền hạn, kiểm tra revision và biên nhận ghi hiện hành.

Khi đổi hội thoại, giao diện hiển thị lựa chọn và trạng thái tải ngay, đồng thời thay thế các lượt đọc chi tiết và sự kiện đã lỗi thời. Năng lực runtime tùy chọn cho phép hủy các lượt đọc đó. Watermark bao quát sự kiện thấy được trong cùng transaction đọc SQLite. GUI hiển thị cửa sổ transcript có giới hạn và giữ định danh tin nhắn, bản nháp cùng vòng đời biên nhận ghi.

Runtime tiếp nhận lượt đọc tương tác tách khỏi công việc có thể chờ Git, hệ thống tệp hoặc quyền ghi SQLite. Đường phản hồi có giới hạn và deadline đóng kết nối kiểm soát client đọc chậm. Connection owner duy trì khóa ngoại, WAL, quyền cấp preview và thứ tự ghi.

- [x] Ghi trace native trước và sau cùng workload, số mẫu và phân vị.
- [x] Kiểm tra đổi hội thoại nhanh, phản hồi lỗi thời, race snapshot/sự kiện, nhập liệu và phân trang transcript gia tăng.
- [x] Kiểm tra hủy, tách tác vụ chậm, phân trang SQLite và bảo toàn biên nhận ghi.
- [x] Đo tài nguyên sau warm-up, ghi giới hạn và đường thu hồi.
- [x] Chạy build đã pin, bộ test Rust và Qt, kiểm tra CLI và governance. Ghi riêng ca bỏ qua.
- [ ] Đạt mục tiêu heartbeat hỗn hợp và thời gian apply trên workload kiểm chứng.

[Kiểm chứng và bằng chứng còn thiếu](verification.md)
