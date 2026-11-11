<!-- doc-id: CIRNETRULES-ROUTING -->
<!-- revision: 3 -->
<!-- source-sha256: 57df030f27ee1fb53c31eaa79af4973677384cff6d33ed9455b510becdafc2a0 -->
<!-- translation-status: synchronized -->
<a id="routing"></a>

# Workspace cục bộ và định tuyến repository

Chạy `cnr workspace discover` để tìm checkout từ metadata giới hạn của
workspace. Kiểm tra ID repository đã đăng ký rồi bind checkout bằng
`cnr workspace bind --repo REPO --path WORKTREE --actor OPERATOR`.
Binding ghi Git root, thư mục admin, thư mục common và commit mốc. Thay
binding cần định danh trước đó. Dùng `cnr workspace health --repo REPO`
để kiểm tra.

Để làm song song trên cùng repository, dùng
[quy trình worktree có tên](../references/shared-task-coordination.md).
Chọn binding trong descriptor của task trước khi duyệt. Kiểm tra binding
khi bàn giao. Đổi binding cần revision mới của task và duyệt lại.

Dùng danh sách tệp chính xác cho phạm vi đọc và ghi. Hồ sơ module và phụ
thuộc trực tiếp mô tả ảnh hưởng. Bộ phân giải trả về tệp sản phẩm và hồ sơ
task dưới dạng dữ liệu, rồi nạp chỉ dẫn từ các mục governance tiếng Anh.
Khi thêm repository, kiểm tra quyền sở hữu, phân loại và quan hệ có kiểu
trong [registry](../../governance.yaml) trước.

`cnr task context --task ID --session TOKEN` phân giải điểm vào, spec, plan,
kỹ năng và source sản phẩm đã chọn. Dùng lại token để lấy phần thay đổi.
Tạo token mới cho phiên mới hoặc sau khi rút gọn ngữ cảnh. Bộ phân giải
kiểm tra các đường dẫn đã sửa, bị thiếu hoặc chưa được Git theo dõi trong
phạm vi đã chọn, rồi ghi số lượng đầu vào và cache. Xem
[tham chiếu CLI](../references/cnr.md#execution) để quản lý cache.
