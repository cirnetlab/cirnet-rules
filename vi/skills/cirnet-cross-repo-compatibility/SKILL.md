---
name: cirnet-cross-repo-compatibility
description: Dùng khi kiểm tra revision hoặc artifact producer so với dependency repo consumer đang chọn.
---

# Tương thích liên repo

Dùng để so producer và consumer trực tiếp. Dùng contract change khi sửa định dạng chuẩn thuộc quyền sở hữu.

Tối thiểu cần: Owner và consumer, revision/hash artifact được chọn, ranh giới bị ảnh hưởng và repository được phép.

1. Với ranh giới CIRNET, dùng `en/policies/architecture.md` và bảng định tuyến
   ở đó để xác định nguồn có thẩm quyền, producer và các consumer trực tiếp.
   Đọc manifest của bên sở hữu và dependency selector thực tế của consumer.
   Phân biệt source local, candidate và system selection.
2. Kiểm tra từng chiều cần thiết và consumer trực tiếp theo test lane của nó. Ghi chính xác input và giới hạn.
3. Mô tả thay đổi owner cần làm và xác định quyền trước khi sửa repo khác.

Báo artifact producer, pin consumer, chiều, kết quả, migration và phần chưa biết cho từng cặp.

Ví dụ: UI pin SDK cũ trong khi producer có candidate mới. Kiểm tra SDK đang pin và server được chọn riêng với candidate.

<!-- source: en/skills/cirnet-cross-repo-compatibility/SKILL.md -->
<!-- source-sha256: c41727b2bcd8e36b5c30962635359bc8d499412ce103812e815a28f27a8c137a -->
