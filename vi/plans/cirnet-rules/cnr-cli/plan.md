<!-- doc-id: CNR-CLI-PLAN -->
<!-- revision: 3 -->
<!-- source-sha256: cdd677450724bdf98c848c62662c25418d63fb3cd2fe5a65215e8227328b6708 -->
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
