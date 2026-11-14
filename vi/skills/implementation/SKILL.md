---
name: implementation
description: Dùng khi triển khai tính năng hoặc thay đổi hành vi trong phạm vi xác định và cần đối chiếu mã nguồn, giao diện cùng kiểm thử hiện có.
---

# Triển khai

Dùng phạm vi đã được cho phép và chỉ dẫn áp dụng của repository làm đầu vào.
Với sửa đổi văn bản đơn giản, làm theo quy tắc cục bộ trực tiếp; quy trình này là tùy chọn.

1. Kiểm tra trạng thái, mã nguồn bị ảnh hưởng, các bên gọi trực tiếp, hợp đồng của
   bên cung cấp và kiểm thử liên quan gần đó. Nêu hành vi kỳ vọng và các kiểm tra
   nghiệm thu có thể quan sát được.
2. Với công việc không đơn giản, phác thảo thay đổi nhỏ nhất nhưng nhất quán. Chỉ
   viết plan/spec nếu quy tắc cục bộ hoặc nhiệm vụ yêu cầu, tại vị trí chủ sở hữu quy định.
3. Thêm kiểm thử hành vi có ý nghĩa khi cần, rồi triển khai trong các tệp được phép.
   Sinh đầu ra dẫn xuất qua quy trình chuẩn của bên cung cấp. Nếu thay đổi bắt buộc
   ở bên cung cấp nằm ngoài phạm vi, báo phụ thuộc đó.
4. Cập nhật tài liệu tiếng Anh bị ảnh hưởng và bản tiếng Việt tương ứng sau khi
   triển khai/build, theo chính sách ngôn ngữ cục bộ. Giữ ví dụ, lệnh, tiến độ plan/spec
   và giới hạn nhất quán. Nếu ý nghĩa tài liệu không thay đổi, giải thích lý do.
   Rà soát ý nghĩa bản dịch trước khi cập nhật siêu dữ liệu đồng bộ.
5. Chạy kiểm tra bị ảnh hưởng theo chính sách công cụ/đầu ra cục bộ. Xem xét lỗi và
   diff cuối cùng để phát hiện sửa đổi không liên quan, bí mật và sản phẩm sinh tự động.
6. Báo các tệp thay đổi, các cặp tài liệu đã cập nhật, hành vi, kiểm tra và giới hạn.
   Báo bản dịch bắt buộc còn thiếu là chưa hoàn tất, không phải đã xong.
   Dùng PASS cho kiểm tra đã chạy thành công, FAIL cho kiểm tra đã chạy không thành
   công, NOT_RUN cho kiểm tra chưa chạy và BLOCKED khi thiếu điều kiện tiên quyết.
   Không ngụ ý đã được chấp thuận ở cấp bản phát hành hoặc môi trường thực thi nguyên bản.

Ví dụ: bên sử dụng cần trường payload mới. Trước hết kiểm tra hợp đồng mà bên đó
đã khóa phiên bản; triển khai thay đổi tương thích ở bên sử dụng nếu có thể, nếu
không thì xác định thay đổi cần thiết phía chủ sở hữu. Đọc repo bên cung cấp không
cho phép sửa repo đó.

<!-- source: en/skills/implementation/SKILL.md -->
<!-- source-sha256: 50997d80198de71d4f285fef89a744a54ea4ff8ec6d51d6dc63419bc2a3f4b15 -->
