<!-- doc-id: CNR-CLI-PLAN -->
<!-- revision: 4 -->
<!-- source-sha256: bb43c713cfb250be77848098974c8f6079675cf0d35f1483a7ff9e93baad78f8 -->
<!-- translation-status: synchronized -->
<a id="cnr-plan"></a>

# Kế hoạch triển khai CNR

[Tiêu chí nghiệm thu](../../../specs/cirnet-rules/cnr-cli/spec.md)

1. Bổ sung fixture cho entrypoint headless, chính sách tùy chọn, khởi tạo dự án và thực thi.
2. Di chuyển helper build và đóng gói, bảo toàn hành vi bảo trì trong lõi chung và triển khai CLI cùng giao diện terminal.
3. Tích hợp phân giải dự án, lập kế hoạch provider và thao tác theo quyền sở hữu, có kiểm tra phiên bản và quyền tường minh.
4. Kết nối thực thi với bộ quản lý tiến trình có giới hạn. Kiểm tra tương thích, phục hồi lỗi và phát hiện bằng fixture riêng.
5. Kiểm tra lệnh CI liên quan, đồng bộ tài liệu song ngữ và báo cáo giới hạn theo tiêu chí nghiệm thu.

## Làm mới terminal và bố trí mã nguồn

1. Tái hiện việc xóa toàn màn hình khi điều hướng bàn phím trong pseudoconsole Windows.
2. Chuyển executable và test vào cây mã nguồn Rust, đồng thời cập nhật Cargo, định danh build và chính sách vị trí mã Rust.
3. Tách trạng thái menu, bố cục và đầu ra terminal. Dựng chữ và bóng đổ lệch bằng ký tự nửa ô. Chỉ vẽ các dòng thay đổi. Hiển thị tìm kiếm, nhập lệnh và trợ giúp tùy chọn ở cuối khi được kích hoạt.
4. Kiểm tra màn hình rộng và gọn, nhãn tiếng Việt, thực thi lệnh, thay đổi kích thước, `NO_COLOR`, các dòng giữ nguyên và CLI phi tương tác.

## Công việc dự kiến cho terminal tương tác

1. Xác định ranh giới tương tác terminal và thực thi lệnh, gồm vector đối số tách biệt, mặc định phi tương tác, kiểm tra quyền hiện có và xác nhận tường minh cho thao tác ghi.
2. Hiển thị đầu ra đã giới hạn và làm sạch trong ứng dụng; hỗ trợ hủy, cuộn, xóa và quay lại menu.
3. Thêm gợi ý lệnh, hoàn thành bằng Tab và lịch sử, đồng thời giữ bố cục gọn đáp ứng kích thước màn hình.
4. Tinh chỉnh chữ và bóng lệch tạo chiều sâu theo hình tham khảo được cung cấp; giữ khả năng đọc ở chế độ gọn và `NO_COLOR`.
5. Kiểm tra thực thi được phép và bị từ chối, hủy, giới hạn và làm sạch đầu ra, hoàn thành lệnh, lịch sử, xóa, cuộn và đổi kích thước. Quan sát kết quả hiển thị và ghi nhận giới hạn kiểm tra tương tác hoặc nền tảng.
