---
name: cirnet-debugging
description: Dùng khi lỗi, regression hoặc kết quả chập chờn cần xác định nguyên nhân trước khi sửa.
---

# Chẩn đoán lỗi

Dùng khi triệu chứng chưa rõ nguyên nhân. Dùng implementation khi thay đổi và nguyên nhân đã rõ.

Tối thiểu cần: Kết quả mong đợi và thực tế, revision, input gây lỗi, môi trường và thông báo lỗi đã che dữ liệu nhạy cảm.

1. Tái hiện theo quy tắc test cục bộ và so sánh với ca chạy được. Ghi đúng đầu
   vào, triệu chứng quan sát được và ranh giới xảy ra lỗi đầu tiên.
   Với pipeline, ghi mã thoát thực tế của từng bước trước khi lệnh sau ghi đè;
   phân biệt bước lỗi với công việc phụ thuộc chưa hề chạy.
2. Tách điều quan sát được khỏi giả thuyết. Thử một cách giải thích bằng phép
   thử nhỏ nhất giúp phân biệt với các khả năng hợp lý khác. Ghi điều kiện xác
   nhận hoặc loại trừ giả thuyết.
3. Sửa hẹp sau khi có bằng chứng về nguyên nhân. Giữ hành vi dọn dẹp, hủy và
   tương thích, rồi chạy lại ca tái hiện và kiểm tra liên quan.

Báo bằng chứng, nguyên nhân đã hoặc chưa xác nhận, bản sửa và kết quả chạy lại.
Nêu điều cản tái hiện nếu bị chặn. Mỗi lần thử lại cần lý do chẩn đoán cụ thể.

Yêu cầu Stop hết thời gian chờ sau khi hủy. Theo dõi xác nhận dọn dẹp trước
khi đổi thời gian chờ. Coi một lần thử lại thành công là một quan sát.

<!-- source: en/skills/cirnet-debugging/SKILL.md -->
<!-- source-sha256: 0a5db8ad5301661d30b833d8920a1cb24cc219e1155c0b4209cc2a87213b6e75 -->
