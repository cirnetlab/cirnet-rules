---
name: cirnet-plan-spec-authoring
description: Dùng khi soạn hoặc sửa đặc tả kiến trúc, kế hoạch triển khai hoặc tiêu chí nghiệm thu tương ứng của CIRNET.
---

# Soạn Plan hoặc Spec cho CIRNET

Dùng quy trình này cả khi thiếu Superpowers. Xác định repo sở hữu, phạm vi
đã cấp quyền, yêu cầu đã duyệt và quy tắc CIRNET áp dụng.

1. Với tác vụ thực thi không đơn giản, khởi tạo Task Continuity trước khi sửa.
   Ghi tác vụ gốc và giữ tiêu đề hiện có trong `task.md`. Đặt quyết định
   thiết kế, các bước theo thứ tự và phụ thuộc trong `Plans & Specs`. Ghi
   tệp bị ảnh hưởng, contract, ràng buộc, hành vi mong đợi và trường hợp lỗi.
2. Đặt kết quả kiểm tra được cho mỗi mốc. Ánh xạ tiêu chí nghiệm thu với phép
   kiểm của chủ sở hữu và bằng chứng bắt buộc. Ghi việc đã làm, việc còn lại,
   vướng mắc, bước tiếp theo và quan sát trong `progress.md`. Công bố qua Task
   Continuity. Rà thay đổi phạm vi hoặc tiêu chí trước khi áp dụng.
3. Giữ đặc tả và quyết định có giá trị dài hạn tại repo sở hữu, ghim revision
   và hash liên quan. Nếu Superpowers cần đầu vào ổn định, dẫn xuất từ tác vụ
   hiện có bên ngoài snapshot niêm phong và đối chiếu sau thay đổi đã duyệt.

Dùng revision áp dụng trong checkout đã được duyệt.
Task Continuity và Superpowers (`vi/docs/distribution.md#task-continuity`) ·
Vòng đời tài liệu (`vi/policies/documentation.md#vòng-đời-tài-liệu`)

<!-- source: en/skills/cirnet-plan-spec-authoring/SKILL.md -->
<!-- source-sha256: c63c1e34111ceb3c91b97e961c5a297ac2e4cd13793fbae789c21d2a0de2886e -->
