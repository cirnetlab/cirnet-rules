---
name: verification
description: Dùng khi đánh giá mức hoàn tất, hành vi đã thay đổi hoặc kết luận rà soát cần bằng chứng mới.
---

# Xác minh

Xác minh kết luận được yêu cầu trong phạm vi đã được cho phép; chỉ dẫn cục bộ quy
định kiểm tra bắt buộc và vị trí đầu ra. Việc xác minh không cho phép xuất bản.

1. Xác định chính xác diff/đầu vào và kết luận nghiệm thu. Gắn mỗi kết luận với một
   bước kiểm tra quan sát được, phân biệt rà soát tĩnh, kiểm thử đơn vị, tích hợp và
   bằng chứng vật lý.
2. Kiểm tra tác dụng phụ và điều kiện tiên quyết của lệnh. Dùng cơ chế cấp phát của
   repo và công cụ đã cài đặt; bảo vệ fixture, sản phẩm đầu ra hiện có và bằng chứng trước đó.
3. Chạy kiểm tra liên quan và đọc cả đầu ra lẫn mã thoát. Xác định trường hợp bị bỏ
   qua, lỗi có sẵn và lỗi đã thay đổi. Không thay kiểm tra còn thiếu bằng loại bằng
   chứng khác mà không nêu giới hạn.
4. Xác minh tài liệu tiếng Anh/tiếng Việt bị ảnh hưởng thống nhất về hành vi, lệnh,
   tiến độ plan/spec, kết quả và giới hạn. Chạy hook tài liệu cục bộ khi sẵn có.
   Mã băm khớp chỉ chứng minh bản dịch theo đúng phiên bản nguồn; phải kiểm tra ý nghĩa
   bản dịch. Bản đối ứng bắt buộc bị thiếu hoặc lỗi thời ngăn việc tuyên bố hoàn tất toàn bộ.
5. Xem xét diff cuối cùng và trạng thái để kiểm tra phạm vi, đầu ra vô tình tạo ra
   và dữ liệu nhạy cảm.
6. Báo bước kiểm tra, lệnh/ngữ cảnh, kết quả và giới hạn. PASS nghĩa là đã chạy và
   đạt kỳ vọng; FAIL nghĩa là đã chạy nhưng không đạt kỳ vọng; NOT_RUN nghĩa là chưa
   chạy; BLOCKED nghĩa là thiếu điều kiện tiên quyết hoặc quyền cho phép bắt buộc.
   Giới hạn kết luận cho phù hợp.

Ví dụ: mô phỏng RTL đạt nhưng không có bo mạch được kết nối. Báo mô phỏng PASS và
xác minh vật lý NOT_RUN; không tuyên bố sẵn sàng về điện, cũng không nạp chương
trình hoặc cấp điện phần cứng khi chưa có sự cho phép riêng và bằng chứng từ chủ sở hữu.

<!-- source: en/skills/verification/SKILL.md -->
<!-- source-sha256: c90e2e37bac6f7e8e9e5e08957fdaa2c273568b776c0873b61ef97971a010974 -->
