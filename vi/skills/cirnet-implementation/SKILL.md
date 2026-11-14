---
name: cirnet-implementation
description: Dùng khi triển khai tính năng hoặc thay đổi hành vi đã rõ kết quả mong đợi.
---

# Triển khai

Dùng cho thay đổi đã thống nhất. Dùng debugging trước khi chưa rõ nguyên nhân.

Tối thiểu cần: File được phép sửa, hành vi đích, interface của owner và test gần đó.

1. Đọc luồng bị ảnh hưởng và caller trực tiếp. Nêu kiểm tra có thể quan sát.
2. Chọn thay đổi nhỏ nhất đủ giải quyết yêu cầu. Theo quy định lập kế hoạch local và sinh output từ input của owner.
3. Bổ sung coverage khi cần thiết. Cập nhật cặp tài liệu bị ảnh hưởng và chạy kiểm tra liên quan.

Báo diff, hành vi, kiểm tra và giới hạn. Hoàn tất cần kết quả nghiệm thu đã quan sát hoặc nêu rõ phần thiếu.

Ví dụ: Consumer cần một field payload. Kiểm tra contract đang được chọn, rồi chỉ ra thay đổi ở owner nếu thiếu field. Dừng tại ranh giới ghi.

<!-- source: en/skills/cirnet-implementation/SKILL.md -->
<!-- source-sha256: 9ebbdaca65509919509da05004db586a4115e7e0d36cf197bd99c8ddc17c1791 -->
