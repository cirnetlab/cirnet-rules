---
name: cirnet-engineering-workspace-ui
description: Dùng khi thiết kế, triển khai hoặc rà soát editor tương tác hay workbench cho mạch, CAD/3D, IDE hoặc Analyze của CIRNET.
---

# Xây dựng giao diện không gian làm việc kỹ thuật

Đọc `vi/rules/engineering-workspace-ui.md` từ checkout `cirnet-rules` đã duyệt.
Áp dụng chỉ dẫn, mã nguồn và contract của repo sở hữu trong phạm vi ràng buộc
áp dụng tại `vi/policies/architecture.md`.

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
   không khả dụng hoặc lỗi liên quan đến thay đổi. Kiểm tra thao tác bằng
   bàn phím và thiết bị trỏ, panel che khuất, viewport nhỏ, tính nhất quán
   giữa tài liệu và scene, cùng phản hồi vận hành thực tế khi phù hợp. Báo kết quả rà soát
   giao diện, kiểm thử chức năng và tương tác trình duyệt đã quan sát kèm
   bằng chứng. Đánh dấu các phép kiểm chưa chạy là `NOT_RUN`.

<!-- source: en/skills/cirnet-engineering-workspace-ui/SKILL.md -->
<!-- source-sha256: 8aef884fbe1fa94441e514f18981c39dc9303a15cc0e1dfe75f29165ff79f685 -->
