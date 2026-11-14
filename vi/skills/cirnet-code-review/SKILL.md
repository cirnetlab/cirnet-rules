---
name: cirnet-code-review
description: Dùng khi được yêu cầu review diff hoặc thiết kế để tìm vấn đề tính đúng, regression hay bảo mật.
---

# Review mã nguồn

Dùng để đánh giá thay đổi. Dùng implementation cho bản sửa đã thống nhất.
Chỉ sửa khi yêu cầu review bao gồm xử lý phát hiện.

Tối thiểu cần: Diff hoặc đề xuất, hành vi đích, caller bị ảnh hưởng và bằng chứng kiểm tra.

1. Đọc thay đổi và contract upstream. Theo luồng bình thường và lỗi thực tế.
2. Kiểm tra xử lý dữ liệu, đồng thời, tương thích và coverage khi liên quan. Tách lỗi có chứng cứ khỏi câu hỏi.
3. Ưu tiên theo tác động tới người dùng. Chỉ đoạn file nhỏ nhất liên quan và điều kiện làm lộ lỗi.

Trả về phát hiện có thể xử lý trước, rồi khoảng trống test và giới hạn. Nếu không có phát hiện, nêu phạm vi đã review.

Ví dụ: Interceptor retry cả ghi không idempotent. Chỉ tình huống gây tác dụng lặp và nơi gọi bị ảnh hưởng.

<!-- source: en/skills/cirnet-code-review/SKILL.md -->
<!-- source-sha256: f3799ad80da6b7cfd36eddd2d7ec74d858331b1e76913f81317da97705c9e2a2 -->
