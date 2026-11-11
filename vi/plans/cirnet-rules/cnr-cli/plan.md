<!-- doc-id: CNR-CLI-PLAN -->
<!-- revision: 2 -->
<!-- source-sha256: 2282749ef648db8f0a481ddcaded8957551ecef56e72b6e7c5bd7853c2cb3ea6 -->
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
3. Tách trạng thái menu, bố cục và đầu ra terminal. Dựng chữ bằng ký tự nửa ô và chỉ vẽ các dòng thay đổi. Đặt tìm kiếm, nhập lệnh và trợ giúp tùy chọn ngay trên dòng trạng thái.
4. Kiểm tra màn hình rộng và gọn, nhãn tiếng Việt, thực thi lệnh, thay đổi kích thước, `NO_COLOR`, các dòng giữ nguyên và CLI phi tương tác.
