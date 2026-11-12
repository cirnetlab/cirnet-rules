<!-- doc-id: CNR-GUIDED-PLAN -->
<!-- revision: 1 -->
<!-- source-sha256: 6600450278ee031bfe4b84c8f3942cd6786f9900ecc1c6e85458081e07171903 -->
<!-- translation-status: synchronized -->
<a id="implementation"></a>

# Kế hoạch kiểm tra và thiết lập Agent trong CNR

1. **CP1** Ghi danh tính Git hiện tại và tái hiện trường hợp thiếu hoặc sai đầu vào. Dùng chung metadata hành động cho kiểm tra trước khi chạy và gợi ý. Giữ tương thích lệnh bảo trì.
2. **CP2** Tạo báo cáo chỉ đọc có kiểu, dùng lại policy, binding, task, audit và bằng chứng xác minh. Kiểm thử authority lỗi, thiếu binding và bằng chứng Agent theo phạm vi.
3. **CP3** Nối trình hướng dẫn với chọn ngữ cảnh, xuất bundle, xem trước sync theo khối quản lý và phục hồi. Yêu cầu xác nhận tường minh và giữ quyền ghi hiện có.
4. **CP4** Xác minh hoặc đăng ký MCP theo phạm vi và kiểm tra server CNR hiện tại qua STDIO có giới hạn. Kiểm thử từng adapter với fixture cách ly.
5. **CP5** Chạy regression tests, build debug và release bằng toolchain đã ghim. Đồng bộ tài liệu và ghi nhận phép thử native riêng.

[Phạm vi](../../../specs/cirnet-rules/cnr-guided/spec.md)
