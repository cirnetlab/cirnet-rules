<!-- doc-id: CGACP-DESKTOP-PLAN -->
<!-- revision: 1 -->
<!-- source-sha256: 35975c9ba810c5a4ab4fe9ae56b662130f2726272c1aec081ebc05fec07b5854 -->
<!-- translation-status: synchronized -->
<a id="delivery"></a>

# Kế hoạch bàn giao Dashboard CGACP

[Phạm vi và điều kiện](../../../specs/cirnet-rules/dashboard/spec.md)

1. Giữ các điểm vào policy và kiểm chứng hiện có. Tách chọn release sản phẩm khỏi intake Dashboard.
2. Thêm catalog SQLite và giao dịch bền vững cho intake/job. Kiểm tra Git metadata, xung đột relink và khôi phục trên checkout tạm.
3. Kết nối Qt Desktop bất đồng bộ qua pipe do chủ tiến trình kiểm soát. Kiểm tra chọn thư mục/file, lịch sử chat và chuyển trạng thái automation.
4. Kiểm tra build native, IPC và gate an toàn trên fixture. Ghi khoảng trống năng lực trước khi chuẩn bị phát hành ứng dụng.
