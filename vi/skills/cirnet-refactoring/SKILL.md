---
name: cirnet-refactoring
description: Dùng khi đổi cấu trúc nội bộ và giữ hành vi quan sát được cùng input hỗ trợ.
---

# Tái cấu trúc

Dùng để tái cấu trúc giữ hành vi. Xử lý hành vi mới như thay đổi triển khai riêng.

Tối thiểu cần: Hành vi hiện tại, caller, ranh giới, bất biến có coverage và file được phép.

1. Xác định input, output, thay đổi trạng thái, lỗi và thứ tự có thể quan sát.
2. Ghi phép kiểm tra tập trung cho các bất biến trước khi sửa. Đổi từng bước
   nhỏ và cập nhật cùng lúc các tham chiếu bị ảnh hưởng. Dùng cấu trúc sẵn có
   khi phù hợp.
3. Gắn từng đoạn diff với việc tái cấu trúc hoặc phép kiểm chứng của nó. Xóa
   import hoặc helper trở nên thừa do tái cấu trúc sau khi kiểm tra nơi gọi,
   tham chiếu được sinh tự động và cách sử dụng động. Giữ mã thừa có từ trước
   và định dạng không liên quan.
4. So hành vi trước và sau bằng cùng đầu vào và assertion. Giữ tên công khai,
   dạng lưu dữ liệu và ý nghĩa fixture theo cam kết.

Báo thay đổi cấu trúc, hành vi được bảo vệ và kiểm tra đã quan sát. Nêu rõ khác biệt hành vi nếu có.

Khi tách helper cho parser, đối chiếu lỗi đầu vào không hợp lệ và offset trên
cùng các fixture đã có.

<!-- source: en/skills/cirnet-refactoring/SKILL.md -->
<!-- source-sha256: 527279731917fbee7061d7cb63ddf8fff76fda83fc5e6737e69f760430ec7910 -->
