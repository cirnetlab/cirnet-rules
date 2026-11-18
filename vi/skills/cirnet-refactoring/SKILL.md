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
   Khi xóa import hoặc helper, xóa dòng trống chỉ dùng để ngăn cách khai báo đó.
   Rà diff trong phạm vi, kể cả khoảng trắng, và giữ nguyên các byte lân cận.
4. So hành vi trước và sau bằng cùng đầu vào và assertion. Giữ tên công khai,
   dạng lưu dữ liệu và ý nghĩa fixture theo cam kết.
   Giữ loại lỗi, mã lỗi và nguyên văn thông báo được tài liệu hóa. Xem thông
   báo được kiểm thử bằng assertion hoặc được caller sử dụng là một phần
   của hợp đồng. Báo khác biệt về câu chữ do runtime sinh ra mà không mở
   rộng phạm vi sửa.

Báo thay đổi cấu trúc, hành vi được bảo vệ và kiểm tra đã quan sát. Nêu rõ khác biệt hành vi nếu có.

Khi tách helper cho parser, đối chiếu lỗi đầu vào không hợp lệ và offset trên
cùng các fixture đã có.

<!-- source: en/skills/cirnet-refactoring/SKILL.md -->
<!-- source-sha256: 53bcbab35e15dbb9fe9bae5ccfcbc1841886bf6226c92ecd12cd73c7a8b4e2b3 -->
