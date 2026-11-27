---
name: cirnet-engineering-workspace-ui
description: Dùng khi thiết kế, triển khai hoặc rà soát editor tương tác hay workbench cho mạch, CAD/3D, IDE hoặc Analyze của CIRNET.
---

# Xây dựng giao diện không gian làm việc kỹ thuật

Với tác vụ phù hợp, đọc Rule chuẩn `en/rules/engineering-workspace-ui.md` từ
checkout `cirnet-rules` đã duyệt. Khi dùng Skill đã cài, lấy đường dẫn checkout
và quyền đọc từ người vận hành. Rule nằm trong checkout đó. Nếu không đọc
được Rule tại đó, báo quy trình này là `BLOCKED` và nêu checkout hoặc quyền
đọc đang thiếu. Áp dụng chỉ dẫn, mã nguồn và contract của repo sở hữu trong
phạm vi ràng buộc áp dụng tại `en/policies/architecture.md`.

1. **Phân loại tác vụ.** Xác định chủ sở hữu, vai trò người dùng, đối tượng
   kỹ thuật đang xử lý và các tác vụ chính. Với màn hình đăng nhập, tài khoản,
   LMS, lập lịch và quản trị thông thường, chọn quy trình phù hợp với tác vụ
   của từng màn hình.
2. **Mô tả một tương tác thực.** Theo dõi đối tượng → vùng chọn → chế độ →
   thuộc tính hoặc lệnh → phản hồi → hủy, hoàn tác hoặc lưu. Xác định trạng
   thái tài liệu lưu lâu dài, trạng thái UI tạm thời, trạng thái thao tác
   bên ngoài và các đường lỗi. Đặt tiêu chí thành công quan sát được cho
   tác vụ chịu ảnh hưởng.
3. **Bố cục quanh đối tượng chính.** Ưu tiên vùng editor, viewport, đồ thị
   hoặc vùng mã. Xác định các điều khiển xung quanh hỗ trợ bước nào trong
   quy trình. Dùng lại component, token và thao tác đầu vào của chủ sở hữu.
   Giữ panel gọn, có thể thu gọn hoặc đổi kích thước khi được hỗ trợ, và
   giữ nhận diện CIRNET nhất quán.
4. **Giữ quyền sở hữu và tính chính xác kỹ thuật.** Với CIRNET-Interface,
   xem `docs/en/architecture/workbench.md` hiện hành của chủ sở hữu cùng
   các component hoặc controller chịu ảnh hưởng. Giữ đúng vai trò của
   `SingleWorkspaceShell`, `CircuitGraph` và lớp hiển thị Three.js. Dựa vào
   ngữ nghĩa đồ thị cho kết nối và trạng thái đã được chủ sở hữu xác nhận
   cho thao tác và thiết bị đo. Chỉ triển khai trong phạm vi sản phẩm và
   các thao tác được phép.
5. **Kiểm chứng tương tác.** Dùng phép kiểm chức năng của chủ sở hữu và
   tương tác trên trình duyệt thật khi có quyền và điều kiện. Thử chọn,
   chỉnh sửa, hủy hoặc hoàn tác, lưu, focus, đổi kích thước và các trạng thái
   không khả dụng hoặc lỗi liên quan đến thay đổi. Dùng tiêu chí nghiệm thu
   của Rule để kiểm tra cách trở lại vùng thao tác khi panel che khuất, tính
   nhất quán giữa tài liệu và scene, và trạng thái đã lưu khi áp dụng.
   Kiểm tra thao tác bằng bàn phím và thiết bị trỏ, viewport nhỏ và phản hồi
   vận hành thực tế khi phù hợp. Báo kết quả rà soát
   giao diện, kiểm thử chức năng và tương tác trình duyệt đã quan sát kèm
   bằng chứng. Đánh dấu các phép kiểm chưa chạy là `NOT_RUN`.

Trước khi sửa, xem tham chiếu đã duyệt, điểm vào lệnh thực tế và nguồn gốc tài
sản. Áp dụng Rule cho các yêu cầu thiết kế bắt buộc. Trên ứng dụng đang chạy,
chụp trình chỉnh sửa chính, thuộc tính đối tượng đã chọn, từng trạng thái ẩn
panel, ẩn cả hai panel và Focus Mode. So sánh 1280×720 và 1920×1080 khi có thể.
Rà đơn vị, focus, cắt nội dung, lý do không khả dụng và độ tương phản. Thử các
luồng đặt, sửa, lỗi và lưu bị ảnh hưởng. Lưu ảnh vào Build Output đã cấp và
mô tả kết quả quan sát để người vận hành xem xét.

<!-- source: en/skills/cirnet-engineering-workspace-ui/SKILL.md -->
<!-- source-sha256: 9d404e7dcd2f364fb4eec18f80677061b614d2fc451e2d2714c3eca53a31f37d -->
