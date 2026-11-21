# Nguyên tắc giao diện không gian làm việc kỹ thuật

Áp dụng quy tắc này cho editor kỹ thuật và workbench tương tác. Chọn biểu
mẫu, bảng hoặc dashboard khi phù hợp với các tác vụ như đăng nhập, tài
khoản, LMS, lập lịch và quản trị.

## Các bất biến thiết kế

1. **Vùng làm việc chính.** Dành vùng thao tác chính cho mạch, sơ đồ, mô
   hình, mã, đồ thị hoặc dữ liệu thí nghiệm đang xử lý. Trong editor mạch
   CIRNET-Interface, giữ viewport 3D tương tác làm vùng chỉnh sửa chính.
2. **Quyền sở hữu theo miền.** Tuân theo chỉ dẫn, mã nguồn và contract của
   chủ sở hữu sản phẩm trong phạm vi [kiến trúc CIRNET](../policies/architecture.md)
   áp dụng. Trong CIRNET-Interface, `SingleWorkspaceShell` chứa workspace,
   các lệnh `CircuitGraph` quản lý kết nối điện, chỉnh sửa tài liệu và lịch sử,
   còn scene Three.js hiển thị đồ thị. Chỉ xác định kết nối điện dựa trên
   ngữ nghĩa đồ thị. Chuyển động camera, giao cắt trên hình và trạng thái
   panel thuộc phần hiển thị.
3. **Điều khiển theo ngữ cảnh.** Sắp xếp thuộc tính và lệnh theo đối tượng,
   vùng chọn, chế độ và tác vụ hiện tại. Xác định mỗi vùng lớn hỗ trợ công
   việc nào. Dùng cây đối tượng, inspector, thanh công cụ, panel tác vụ,
   lịch sử và gizmo khi có ích. Ưu tiên panel gọn hoặc thích ứng để vùng
   làm việc chính và phản hồi quan trọng luôn dễ tiếp cận.
4. **Chỉnh sửa có thể hoàn tác.** Giữ các thao tác chọn, chọn nhiều, hủy,
   xác nhận, hoàn tác/làm lại và xử lý thay đổi chưa lưu mà sản phẩm hỗ trợ.
   Hiển thị chế độ hiện hành và bản xem trước tách khỏi trạng thái tài liệu
   đã xác lập.
5. **Phản hồi kỹ thuật.** Phân biệt các trạng thái đã yêu cầu, đang chờ, đã
   chấp nhận, đã áp dụng, thất bại, không khả dụng và đã đo khi phù hợp.
   Dựa vào contract và bằng chứng vận hành của chủ sở hữu để xác định khả
   năng, mức sẵn sàng, kết quả và xác nhận Run/Stop. Với thao tác Lab, giữ
   quyền cấp phép phần cứng ở Remote và Hardware. Hiển thị đơn vị, dải đo,
   nguồn gốc và độ bất định của phép đo khi có liên quan.
6. **Nhận diện trực quan.** Dùng mật độ, kiểu chữ, thứ bậc, tương phản và
   biểu tượng để thông tin kỹ thuật dễ đọc và thao tác chính xác. Chọn
   thành phần và chủ đề giao diện theo lợi ích cho tác vụ. Giữ nhận diện
   riêng của CIRNET. Tham khảo các mẫu tương tác chung từ công cụ khác.
7. **Khả dụng và hiệu năng.** Giữ khả năng thao tác bằng bàn phím và thiết
   bị trỏ, chỉ báo focus, lệnh dễ tìm, nhãn rõ nghĩa, hành vi khi đổi kích
   thước, vòng đời đồ họa và trạng thái đang tải, lỗi hoặc không hỗ trợ.

## Phạm vi nghiệm thu

Chạy ít nhất một tác vụ đại diện khi thay đổi editor. Tìm và chọn đối tượng,
xem hoặc đổi thuộc tính, rồi xác minh trạng thái kết quả và thao tác hủy,
hoàn tác hoặc lưu nếu áp dụng. Thử các cách thao tác bằng bàn phím và thiết
bị trỏ được hỗ trợ, các trạng thái không khả dụng hoặc lỗi, và kiểm tra
vùng làm việc khi có panel. Xác nhận tác vụ thành công qua tương tác quan
sát được và dùng ảnh chụp để rà soát phần hiển thị. Dùng bằng chứng trên
trình duyệt thật khi có quyền và điều kiện. Giữ các phép kiểm chức năng
cùng ranh giới an toàn của chủ sở hữu.

Dùng [Skill giao diện không gian làm việc kỹ thuật](../skills/cirnet-engineering-workspace-ui/SKILL.md)
cho quy trình triển khai và rà soát.

<!-- source: en/rules/engineering-workspace-ui.md -->
<!-- source-sha256: 82af979d4876b80aa64d44fc984d731de63f6506b50e8a111b6df32e4c2fa342 -->
