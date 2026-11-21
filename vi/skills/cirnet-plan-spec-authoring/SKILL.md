---
name: cirnet-plan-spec-authoring
description: Dùng khi soạn hoặc sửa đặc tả kiến trúc, kế hoạch triển khai hoặc tiêu chí nghiệm thu tương ứng của CIRNET.
---

# Soạn Plan hoặc Spec cho CIRNET

Lưu mỗi tài liệu trong repo sở hữu các yêu cầu hoặc công việc tương ứng.

| Tài liệu hoặc bản ghi | Trách nhiệm |
| --- | --- |
| Spec | Yêu cầu, hành vi quan sát được, bất biến, ràng buộc và tiêu chí nghiệm thu đo được |
| Plan | Trình tự triển khai, phụ thuộc, mốc và các bước kiểm chứng |
| Work Prompt | Nhiệm vụ thực thi cụ thể, đã được cấp quyền cho Coding Agent |
| Task Continuity | Tiến độ, quyết định, bằng chứng và trạng thái khôi phục |

1. **Tìm chủ sở hữu và tài liệu có thẩm quyền.** Kiểm tra chỉ dẫn cục bộ
   áp dụng, hướng dẫn về quyền sở hữu, các Plan và Spec hiện có cùng những
   liên kết trỏ đến chúng. Xác định phiên bản tài liệu liên quan và phạm vi
   đã được duyệt trước khi soạn. Cập nhật trực tiếp tài liệu có thẩm quyền
   nếu cùng mục đích. Dùng quy ước của chủ sở hữu cho đầu ra có mục đích riêng.
2. **Viết đúng trách nhiệm của từng tài liệu.** Trong Spec, nêu các điều kiện
   phải đáp ứng, gồm các ca âm tính và tiêu chí nghiệm thu có kết quả kỳ vọng
   quan sát được. Trong Plan, liên kết đến Spec và sắp xếp công việc theo
   phụ thuộc. Gắn mỗi mốc với một bước kiểm chứng và ánh xạ từng tiêu chí
   nghiệm thu tới phép kiểm của chủ sở hữu cùng kết quả kỳ vọng dùng để xác
   nhận tiêu chí đó. Dùng `cirnet-work-prompt-authoring` khi được yêu cầu
   soạn nhiệm vụ thực thi, và `cirnet-task-continuity` khi khôi phục việc
   bị gián đoạn hoặc bàn giao checkpoint.
3. **Rà soát thay đổi.** Đối chiếu bản thảo với yêu cầu, kiến trúc, phạm vi
   và tiêu chí nghiệm thu đã được duyệt. Xin quyền trước khi áp dụng đề xuất
   thay đổi các quyết định đó. Bảo đảm có thể truy vết kết quả kiểm chứng tới
   tiêu chí, revision source và bằng chứng quan sát được. Đồng bộ bản dịch
   áp dụng theo chính sách của chủ sở hữu.

Dùng revision áp dụng trong checkout đã được duyệt.
Quy ước Plan (`vi/plans/README.md`) ·
Quy ước Spec (`vi/specs/README.md`) ·
Chính sách kiến trúc (`vi/policies/architecture.md`) ·
Chính sách tài liệu (`vi/policies/documentation.md`)

<!-- source: en/skills/cirnet-plan-spec-authoring/SKILL.md -->
<!-- source-sha256: 5919e3e6319023af0ad82574c30bf57a3dc2956c2197e5fdd72b84d7adbd8804 -->
