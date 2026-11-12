<!-- doc-id: CNR-TUI-READABILITY-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: 57719593dd0c025954d2fdd8bcb63748287dc6f183125edf8238e59fd27b9d1f -->
<!-- translation-status: synchronized -->
<a id="scope"></a>

# Trình bày kết quả Check và nhập liệu terminal CNR

CNR dùng finding có kiểu để điều hướng và giữ hợp đồng báo cáo cùng mã thoát hiện tại. Người vận hành thấy mức độ, lý do và bước tiếp theo ở terminal hẹp. Bằng chứng và thao tác liên quan hiển thị theo finding đang chọn.

## Quyết định giao diện

Bảng cố định phù hợp với giá trị ngắn ở chiều rộng ổn định. Lý do và đường dẫn dài làm bảng khó đọc ở terminal hẹp. Danh sách theo nhóm giữ lý do cạnh finding và hiển thị số lượng trước khi mở nhóm. Bố cục danh sách và chi tiết giữ finding đã chọn trong tầm nhìn đồng thời hiển thị bằng chứng khi đủ chỗ. CNR kết hợp hai bố cục này. Terminal hẹp chuyển qua lại giữa danh sách và một finding; terminal rộng có thể hiển thị cả hai.

Editor dùng chung hỗ trợ chọn cụm ký tự, thay thế và undo có giới hạn. Clipboard chỉ được truy cập sau thao tác của người vận hành. Vùng chọn kết quả chỉ đọc, và thao tác clipboard loại trừ dữ liệu nhạy cảm hoặc báo cáo thô. Dán chỉ sửa bản nháp và cần một sự kiện Enter riêng để gửi.

Cải tiến dùng TUI và vòng đời terminal hiện tại của CNR. Thiết lập của terminal quyết định các sự kiện phím và chuột do host xử lý. Phạm vi gồm trình bày và nhập liệu. Governance, MCP, quyền task, verification và audit giữ hợp đồng hiện tại.
