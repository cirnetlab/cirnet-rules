---
name: cirnet-implementation
description: Dùng khi triển khai tính năng hoặc thay đổi hành vi đã rõ kết quả mong đợi.
---

# Triển khai

Dùng cho thay đổi đã thống nhất. Dùng debugging trước khi chưa rõ nguyên nhân.

Tối thiểu cần: File được phép sửa, hành vi đích, interface của owner và test gần đó.

1. Đọc luồng bị ảnh hưởng, nơi gọi trực tiếp, contract của bên sở hữu và test.
   Nêu giả định ảnh hưởng đến hành vi, tương thích, dữ liệu hoặc an toàn. Làm
   rõ điểm mơ hồ quan trọng hoặc quyền còn thiếu trước khi sửa phần liên quan.
   Tiếp tục với giả định thông thường có thể đảo ngược nếu nhiệm vụ cho phép.
2. Viết tiêu chí nghiệm thu quan sát được. Theo quy tắc lập kế hoạch cục bộ và
   dùng dạng `Bước → phép kiểm tra` cho việc không đơn giản.
3. Sửa trong luồng hiện có ở mức nhỏ nhất đủ đáp ứng yêu cầu. Giữ xử lý lỗi
   và các biện pháp an toàn cần thiết. Sinh đầu ra dẫn xuất từ đầu vào của bên
   sở hữu.
4. Bổ sung coverage có ý nghĩa khi cần. Cập nhật cặp tài liệu bị ảnh hưởng và
   chạy các phép kiểm tra gắn với nghiệm thu.

Báo diff, hành vi, kiểm tra và giới hạn. Đối chiếu mỗi tuyên bố nghiệm thu
với kết quả quan sát được và chỉ rõ phần còn thiếu.

Mở [Ví dụ kỹ thuật](references/engineering-examples.md#ví-dụ-kỹ-thuật) khi có
thể tham khảo ca tương tự về API, Vue, testbench RTL hoặc kiểm thử hồi quy.

<!-- source: en/skills/cirnet-implementation/SKILL.md -->
<!-- source-sha256: 79388b6e10fbbdcc1d080425d94686ce5529539248701338d67a5117171feadd -->
