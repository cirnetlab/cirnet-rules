---
name: cirnet-api-compatibility
description: Dùng khi đổi HTTP API hoặc bề mặt SDK công khai được dùng ngoài thành phần sở hữu.
---

# Tương thích API

Dùng cho semantics API client thấy. Dùng contract change khi đổi wire/schema chuẩn.

Tối thiểu cần: Operation công khai hoặc method SDK, policy phiên bản, producer hiện tại và phiên bản consumer chọn.

1. So request/response, optional/default/null, lỗi, auth, pagination, timeout và hành vi SDK.
2. Phân loại tác động source, wire, dữ liệu và hành vi. Thử consumer đang được chọn, gồm validator nghiêm ngặt.
3. Đề xuất phiên bản hoặc deprecation/migration theo policy owner. Kiểm tra tác dụng phụ của retry và rollback.

Nêu operation thay đổi, chiều tương thích client, bằng chứng và migration. Biến thể thiếu thông tin để unknown.

Ví dụ: Response thêm field tùy chọn. Chạy client đang pin có kiểm tra additional properties trước khi kết luận tương thích.

<!-- source: en/skills/cirnet-api-compatibility/SKILL.md -->
<!-- source-sha256: ff1e928cddc6dbabcb1f962e20fe30f94ae20aca0966fe99795ac00859fb0313 -->
