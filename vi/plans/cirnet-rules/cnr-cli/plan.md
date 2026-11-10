<!-- doc-id: CNR-CLI-PLAN -->
<!-- revision: 1 -->
<!-- source-sha256: c980de0bc61a9d6eba7c29f0daa6de3800789273923e3d53ee7d19d341367617 -->
<!-- translation-status: synchronized -->
<a id="cnr-plan"></a>

# Kế hoạch triển khai CNR

[Tiêu chí nghiệm thu](../../../specs/cirnet-rules/cnr-cli/spec.md)

1. Bổ sung fixture cho entrypoint headless, chính sách tùy chọn, khởi tạo dự án và thực thi.
2. Di chuyển helper build và đóng gói, bảo toàn hành vi bảo trì trong lõi chung và triển khai CLI cùng giao diện terminal.
3. Tích hợp phân giải dự án, lập kế hoạch provider và thao tác theo quyền sở hữu, có kiểm tra phiên bản và quyền tường minh.
4. Kết nối thực thi với bộ quản lý tiến trình có giới hạn. Kiểm tra tương thích, phục hồi lỗi và phát hiện bằng fixture riêng.
5. Kiểm chứng lệnh CI bị ảnh hưởng và Desktop, đồng bộ tài liệu song ngữ và báo giới hạn theo tiêu chí nghiệm thu.
