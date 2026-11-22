# Đặc tả kỹ thuật CAD CIRNET

## Phạm vi và quyền sở hữu

Cung cấp cho agent CIRNET quy trình lặp lại được để đặc tả, tạo, đánh giá
và bàn giao tài sản CAD. CIRNET sở hữu yêu cầu thiết kế, bằng chứng và nghiệm thu.
Công cụ CAD được duyệt phụ trách dựng mô hình tham số, kiểm tra bằng kernel
và xuất tệp. Interface sở hữu lựa chọn tài sản, hiển thị, tương tác khi chạy
và CircuitGraph. Thay đổi giới hạn trong `cirnet-rules`. Đọc checkout khác
để xác định hợp đồng hiện hành của từng chủ sở hữu.

Dùng [Quy tắc kỹ thuật CAD](../../../rules/cad-engineering.md) cho quy trình
vận hành và [kế hoạch](../../../plans/cirnet-rules/cad-engineering/plan.md)
cho bằng chứng triển khai. Skill CAD tùy chọn điều hướng công việc về tài
sản. Skill giao diện workspace kỹ thuật điều hướng thiết kế và kiểm thử tương tác.

## Tiêu chí nghiệm thu

1. Đặc tả nhiệm vụ ghi ca sử dụng, tham chiếu, đơn vị vật lý, hệ tọa độ,
   đặc trưng bắt buộc, lắp ghép, vật liệu, chuẩn tương tác, định dạng, ngân sách
   và quyết định còn mở. Yêu cầu và ưu tiên có bản ghi nghiệm thu riêng.
2. Workspace tạo mẫu đã được duyệt lưu mã tham số chỉnh sửa được và bằng chứng
   với xuất xứ chính xác của công cụ và đầu vào. Dựng, kiểm tra, xuất và
   rà soát trình bày báo kết quả riêng. Thiếu công cụ làm các bước phụ thuộc
   giữ `BLOCKED` hoặc `NOT_RUN`.
3. CLI tiếp nhận tĩnh đọc một kỳ vọng đã ghim độc lập và GLB từ thư mục gốc
   cục bộ được chọn rõ ràng. CLI báo SHA-256, số byte, số nút lưới trong
   cảnh kích hoạt, số tam giác và số primitive theo từng lần hiện diện.
   Tài sản sai pin, cấu trúc thuộc tập hỗ trợ nhưng không hợp lệ, tên bắt
   buộc thiếu hoặc trùng, hay vượt ngân sách đã duyệt đều thất bại trong phạm
   vi tiếp nhận. Cổng kiểm tra dữ liệu đỉnh hữu hạn và giá trị chỉ số thực của
   primitive đang dùng, gồm cận, cực trị POSITION đã khai báo và bố cục bộ đệm.
   Hỗ trợ khai báo `KHR_materials_unlit` hợp lệ dưới dạng tùy chọn hoặc bắt buộc.
   Điều kiện tiên quyết không khả dụng, tính năng ngoài hỗ trợ hoặc vượt giới
   hạn đọc, quét cứng làm lần kiểm tra bị chặn.
4. Quyết định dùng trong sản phẩm cần bằng chứng từ kernel và lắp ghép,
   rà soát ảnh thật, kiểm tra nhập, fallback và tương tác của chủ sở hữu.
   CLI tĩnh chỉ đánh giá các phép kiểm đã khai báo. CLI không cấp quyền
   tiếp nhận tài sản hoặc thay đổi kết nối điện.
5. Các cặp tài liệu song ngữ vượt qua checker. Việc chọn Skill, cấu hình
   cài mặc định và phân phối tùy chọn duy trì tương thích.

Dùng trực tiếp CLI text-to-cad hiện hành hoặc triển khai MCP được duyệt độc
lập. Bộ xác minh tĩnh gọn trong Rules cung cấp hợp đồng tiếp nhận tài sản,
không tích hợp Python hay kernel CAD vào bản build Interface.

<!-- source: en/specs/cirnet-rules/cad-engineering/spec.md -->
<!-- source-sha256: 47f39cb432528a127bf114d98d87263a6240875fe60d14d5ccb5dd1034e32d6f -->
