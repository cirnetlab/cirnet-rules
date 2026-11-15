---
name: cirnet-contract-change
description: Dùng khi đổi schema chuẩn, serialization, protocol hoặc ABI do producer sở hữu.
---

# Thay đổi contract

Dùng cho thay đổi dữ liệu hoặc wire của producer. Dùng API compatibility cho semantics operation HTTP.

Tối thiểu cần: Nguồn chuẩn và owner, định dạng hiện tại và đề xuất, reader/writer được chọn và lệnh codegen.

1. Với contract CIRNET, dùng `en/policies/architecture.md` và bảng định tuyến
   ở đó để xác định nguồn có thẩm quyền và repo sở hữu. Đọc schema chuẩn và
   chính xác các phiên bản được chọn trước khi lập kế hoạch.
2. So encoding, giới hạn, định danh field/enum, byte order, đơn vị và ABI layout khi áp dụng.
3. Kiểm tra reader/writer cũ và mới, dữ liệu lỗi và fixture lưu theo các chiều cần thiết. Sinh binding từ nguồn owner.
4. Ghi version, deprecation, migration dữ liệu và giới hạn rollback. Phối hợp thay đổi owner trong phạm vi được phép.

Báo revision contract, output sinh tự động, các chiều đã so, kết quả và chứng cứ thiếu.

Ví dụ: Giá trị enum mới tới reader cũ nghiêm ngặt. Kiểm tra luồng fallback/từ chối thật trước khi đổi phiên bản producer.

<!-- source: en/skills/cirnet-contract-change/SKILL.md -->
<!-- source-sha256: 7b2948734c617dfcff6095ad6186c85dab523e150b632093392b7d8fb3bb1f4c -->
