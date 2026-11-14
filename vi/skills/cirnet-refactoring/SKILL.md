---
name: cirnet-refactoring
description: Dùng khi đổi cấu trúc nội bộ và giữ hành vi quan sát được cùng input hỗ trợ.
---

# Tái cấu trúc

Dùng để tái cấu trúc giữ hành vi. Xử lý hành vi mới như thay đổi triển khai riêng.

Tối thiểu cần: Hành vi hiện tại, caller, ranh giới, bất biến có coverage và file được phép.

1. Xác định input, output, thay đổi trạng thái, lỗi và thứ tự có thể quan sát.
2. Chọn kiểm tra tập trung bảo vệ bất biến. Chuyển từng bước nhỏ và cập nhật reference cùng lúc.
3. So hành vi trước/sau. Giữ tên công khai, dạng lưu dữ liệu và ý nghĩa fixture theo cam kết.

Báo thay đổi cấu trúc, hành vi được bảo vệ và kiểm tra đã quan sát. Nêu rõ khác biệt hành vi nếu có.

Ví dụ: Tách helper parser, giữ lỗi và offset khi input sai. Cập nhật caller và so fixture hiện có.

<!-- source: en/skills/cirnet-refactoring/SKILL.md -->
<!-- source-sha256: 2d2935b08ba2ff4f704e89ecfcaddd617920d45e702544f135be355332481eaf -->
