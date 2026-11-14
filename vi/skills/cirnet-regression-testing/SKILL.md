---
name: cirnet-regression-testing
description: Dùng khi một lỗi hoặc thay đổi hành vi cần test tập trung phát hiện hồi quy.
---

# Kiểm thử hồi quy

Dùng để thêm coverage cho rủi ro đã biết. Dùng debugging để xác định nguyên nhân chưa rõ.

Tối thiểu cần: Triệu chứng, hành vi công khai mong đợi, fixture, đường chạy test và quy định output.

1. Chọn ca tái hiện đại diện nhỏ nhất. Kiểm tra kết quả người dùng thấy và ca bất lợi liên quan.
2. Nếu có thể, chạy trên hành vi lỗi và xác nhận assertion đúng lý do thất bại. Sửa triển khai rồi chạy lại.
3. Giữ fixture xác định, tránh snapshot không liên quan. Xác minh test chạy trong bộ phù hợp.

Báo test bảo vệ điều gì và kết quả lỗi/đạt đã thấy hoặc baseline không có.

Ví dụ: Parser chấp nhận frame cụt. Kiểm tra input bị từ chối và không sinh output dở. Crash không liên quan không đủ bằng chứng.

<!-- source: en/skills/cirnet-regression-testing/SKILL.md -->
<!-- source-sha256: 7108971710d79cbf4aa388808a845a2df76e78edb35f753620936c355eee8c94 -->
