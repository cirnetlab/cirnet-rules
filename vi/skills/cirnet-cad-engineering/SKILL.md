---
name: cirnet-cad-engineering
description: Dùng khi đặc tả, dựng, kiểm tra, kết xuất, tinh chỉnh hoặc chuẩn bị mô hình CAD và 3D cho CIRNET.
---

# Thiết kế và kiểm chứng mô hình CAD

Đọc `en/rules/cad-engineering.md` từ checkout `cirnet-rules` đã được phê duyệt.
Với Skill đã cài, dùng đường dẫn checkout và quyền đọc do người vận hành
cung cấp để truy cập Rule và `scripts/verify-cad-asset.mjs`. Nếu thiếu Rule,
báo quy trình CAD ở trạng thái `BLOCKED` và nêu nguồn hoặc quyền truy cập
còn thiếu. Với thiết kế workbench tương tác, chỉnh sửa, chọn đối tượng và lưu
trạng thái, chọn `cirnet-engineering-workspace-ui`. Khi tác vụ có cả hai
phần, dùng từng Skill cho phần tương ứng.

1. **Hiểu và đặc tả.** Xác nhận mục đích, tài liệu tham chiếu và hợp đồng của
   bên tiếp nhận. Ghi đơn vị, ràng buộc kỹ thuật đo được, ưu tiên thẩm mỹ,
   quyết định còn mở và phép kiểm nghiệm thu. Đánh dấu kích thước và dung sai
   tạm thời.
2. **Dựng và kiểm tra.** Xác minh danh tính, phiên bản, lệnh khả dụng và điều
   kiện tiên quyết của công cụ CAD được duyệt. Dùng CLI hoặc tích hợp MCP
   mà text-to-cad hỗ trợ khi có sẵn. Tạo mã tham số có thể chỉnh sửa trong
   workspace được duyệt và kiểm tra hình học, kích thước, cấu trúc lắp ráp
   thực tế.
3. **Kết xuất, đối chiếu và tinh chỉnh.** Chụp các góc nhìn kỹ thuật phù hợp
   và kết xuất ảnh trình bày khi có yêu cầu. So sánh ảnh với yêu cầu thiết kế,
   ghi lỗi cụ thể, tinh chỉnh chi tiết liên quan và chạy lại các phép kiểm
   bị ảnh hưởng.
4. **Kiểm chứng, rà soát và bàn giao.** Khi chuẩn bị tài sản cho Interface,
   chạy phép kiểm trước tích hợp trong checkout bằng Node và tuân theo hợp
   đồng hiện hành của chủ sở hữu. Giữ quyền xác định kết nối điện ở
   `CircuitGraph` và bảo toàn điểm neo tương tác, phép biến đổi và phương án
   dựng mô hình dự phòng bằng mã. Dùng `cirnet-verification` để ghi nguồn gốc
   và kết quả quan sát của từng bước. Dùng Task Continuity cho công việc kéo
   dài. Báo riêng bằng chứng cho phép kiểm kỹ thuật, thực thi native và phê
   duyệt trực quan của con người. Ghi `BLOCKED` hoặc `NOT_RUN` cho phép kiểm
   không khả dụng hoặc chưa chạy, tùy trường hợp.

<!-- source: en/skills/cirnet-cad-engineering/SKILL.md -->
<!-- source-sha256: 1c60ce5be5d38fbc4246e84c93f8a3f2907e36af806f6a1e31be27ed55e1332a -->
