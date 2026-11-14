---
name: cirnet-verification
description: Dùng khi tuyên bố hoàn tất, tính đúng hoặc tương thích cần bằng chứng mới.
---

# Kiểm chứng

Dùng để đánh giá tuyên bố. Dùng regression testing để thiết kế coverage còn thiếu.

Tối thiểu cần: Tuyên bố, revision/diff chính xác, kết quả mong đợi và điều kiện kiểm tra local.

1. Gắn mỗi tuyên bố với phép kiểm tra có thể chứng minh nó, xét phạm vi và tác động.
2. Chạy kiểm tra và đọc exit code, output, phần bỏ qua. Đối chiếu chỉ dẫn EN/VI bị ảnh hưởng và diff cuối.
3. Phân biệt review source, chạy unit/integration, hành vi client và bằng chứng vật lý.

Liệt kê từng lệnh và PASS, FAIL, NOT_RUN hoặc BLOCKED cùng phạm vi. Hoàn tất khi tuyên bố khớp bằng chứng.

Ví dụ: Mô phỏng RTL pass. Báo hành vi đã được thử, ghi sẵn sàng điện và nạp chương trình chưa xác minh.

<!-- source: en/skills/cirnet-verification/SKILL.md -->
<!-- source-sha256: 9ebfd160111792bf4cd0ac74be2e06c227b3710e01bd9edb4d5b1cbe4eaf05a7 -->
