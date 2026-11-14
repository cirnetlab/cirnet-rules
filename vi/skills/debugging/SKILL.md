---
name: debugging
description: Dùng khi cần chẩn đoán lỗi, hồi quy, kiểm thử lúc đạt lúc không hoặc sự khác biệt giữa hành vi kỳ vọng và hành vi quan sát được.
---

# Gỡ lỗi

Làm việc trong phạm vi đã được cho phép và theo chỉ dẫn của repository đích.

1. Ghi nhận hành vi kỳ vọng và thực tế, revision liên quan, đầu vào, lỗi và môi
   trường, đồng thời che giá trị nhạy cảm. Tái hiện trong luồng kiểm thử đã được chấp thuận.
2. Lần theo ranh giới xảy ra lỗi. So sánh với trường hợp hoạt động đúng và đưa ra
   một giả thuyết có thể bác bỏ bằng kiểm chứng. Kiểm tra bằng chứng trước khi đổi
   phần triển khai hoặc thời gian chờ.
3. Chạy thử nghiệm an toàn nhỏ nhất có thể phân biệt các nguyên nhân. Nếu không thể
   tái hiện, báo điều đã quan sát, đầu vào còn thiếu và thử nghiệm hữu ích tiếp theo;
   ghi rõ nguyên nhân chưa được xác nhận.
4. Khi đã có bằng chứng hỗ trợ, sửa nguyên nhân trong phạm vi hẹp và thêm kiểm tra
   hồi quy khi khả thi. Giữ nguyên kiểm tra an toàn, hủy, dọn dẹp và tương thích.
5. Cập nhật đồng thời tài liệu tiếng Anh/tiếng Việt bị ảnh hưởng theo chính sách
   cục bộ; phân biệt nguyên nhân đã xác nhận với giả thuyết chưa xác nhận trong cả hai bản.
6. Chạy lại bước tái hiện và bộ kiểm thử bị ảnh hưởng. Báo nguyên nhân, thay đổi,
   lệnh, kết quả PASS/FAIL và mọi kiểm tra NOT_RUN hoặc BLOCKED. Tuân thủ cách cấp
   phát đầu ra cục bộ.

Ví dụ: Stop hết thời gian chờ sau một yêu cầu đã bị hủy. Lần theo quá trình hủy
và xác nhận dọn dẹp trước khi tăng thời gian chờ; xác minh cả hành vi dọn dẹp hoàn
tất lẫn dọn dẹp thất bại. Lần thử lại đạt không xác lập được lý do lỗi đã xảy ra.

<!-- source: en/skills/debugging/SKILL.md -->
<!-- source-sha256: c2e94b0c4988a7ec5e7d7c7d70f5d22aab1a46262f79eae59c64feddbff50f20 -->
