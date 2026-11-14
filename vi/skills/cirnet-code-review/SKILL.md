---
name: cirnet-code-review
description: Dùng khi được yêu cầu review diff hoặc thiết kế để tìm vấn đề tính đúng, regression hay bảo mật.
---

# Review mã nguồn

Dùng để đánh giá thay đổi. Dùng implementation cho bản sửa đã thống nhất.
Chỉ sửa khi yêu cầu review bao gồm xử lý phát hiện.

Tối thiểu cần: Diff hoặc đề xuất, hành vi đích, caller bị ảnh hưởng và bằng chứng kiểm tra.

1. Đọc thay đổi và contract upstream. Lần theo luồng bình thường và luồng
   lỗi có thể xảy ra. Gắn mỗi vùng thay đổi với yêu cầu, lỗi quan sát được
   hoặc phép kiểm tra.
2. Kiểm tra xử lý dữ liệu, đồng thời và tương thích khi liên quan. Chỉ ra
   các lớp trừu tượng chưa có nhu cầu, thay đổi không liên quan và bằng chứng
   còn thiếu có ảnh hưởng đến nhiệm vụ. Xem assertion có bắt được lỗi đã báo.
3. Ưu tiên theo tác động tới người dùng. Tách lỗi có chứng cứ khỏi câu hỏi.
   Chỉ đoạn file nhỏ nhất liên quan và nêu điều kiện làm lộ lỗi.

Trả về phát hiện có thể xử lý trước, rồi khoảng trống test và giới hạn. Nếu
không có phát hiện, nêu phạm vi đã review. Chỉ sửa trong phạm vi được phép và
ghi nhận các vấn đề khác trong phần phát hiện.

Interceptor thử lại cả thao tác ghi không idempotent. Nêu tình huống gây tác
dụng lặp và nơi gọi bị ảnh hưởng.

<!-- source: en/skills/cirnet-code-review/SKILL.md -->
<!-- source-sha256: eaa99729f85baf3de68064d2855ad2a954e99428206e4ee326c0512753563512 -->
