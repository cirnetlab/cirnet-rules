---
name: cirnet-contract-change
description: Dùng khi đổi schema chuẩn, serialization, protocol hoặc ABI do producer sở hữu.
---

# Thay đổi contract

Dùng cho thay đổi dữ liệu hoặc wire của producer. Dùng API compatibility cho semantics operation HTTP.

Tối thiểu cần: Nguồn chuẩn và owner, định dạng hiện tại và đề xuất, reader/writer được chọn và lệnh codegen.

1. So encoding, giới hạn, định danh field/enum, byte order, đơn vị và ABI layout khi áp dụng.
2. Kiểm tra reader/writer cũ và mới, dữ liệu lỗi và fixture lưu theo các chiều cần thiết. Sinh binding từ nguồn owner.
3. Ghi version, deprecation, migration dữ liệu và giới hạn rollback. Phối hợp thay đổi owner trong phạm vi được phép.

Báo revision contract, output sinh tự động, các chiều đã so, kết quả và chứng cứ thiếu.

Ví dụ: Giá trị enum mới tới reader cũ nghiêm ngặt. Kiểm tra luồng fallback/từ chối thật trước khi đổi phiên bản producer.

<!-- source: en/skills/cirnet-contract-change/SKILL.md -->
<!-- source-sha256: 3c8f533ddc9de7dce1b5ba7c51a5d43abe401a7e63bf4963f8ec1eee27700973 -->
