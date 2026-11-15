---
name: cirnet-release-changelog
description: Dùng khi đề xuất phiên bản hoặc chuẩn bị changelog, bước nâng cấp và rollback cho candidate.
---

# Release và changelog

Dùng để chuẩn bị release. Dùng implementation khi task chỉ sửa code.
Phát hành cần quyền riêng.

Tối thiểu cần: Phiên bản đã phát hành gần nhất, policy owner, thay đổi người dùng thấy, artifact đã kiểm tra và giới hạn.

1. Phân loại tác động người dùng và miền phiên bản bị ảnh hưởng. Đề xuất tăng version theo policy owner.
2. Mô tả thay đổi, phần breaking, bước migration và rủi ro rollback. Giữ lịch sử đã phát hành và dữ kiện release được chọn.
3. Ghi revision/artifact đã thử, trạng thái candidate và kiểm tra release/nền tảng chưa chạy.
4. Với repository CIRNETLAB, dùng `<repo_name>-<version>` làm cả Git tag và tên GitHub Release. Dùng đúng tên repository và phiên bản đã duyệt; không tự thêm tiền tố `v` hay hậu tố commit. Ví dụ: `cirnet-rules-0.2.0`.
5. Kiểm tra tag và release chưa tồn tại, tag sẽ trỏ đúng commit đã review và CI cùng kiểm thử gói áp dụng cho commit đó. Tạo tag và phát hành cần quyền riêng.

Cung cấp đề xuất version, changelog và ghi chú nâng cấp/rollback kèm giới hạn kiểm chứng.

Ví dụ: Tên skill đổi. Liệt kê tên cũ/mới và migration an toàn cho bản cài chưa rõ ownership. Việc tạo tag thuộc thao tác release được phép.

<!-- source: en/skills/cirnet-release-changelog/SKILL.md -->
<!-- source-sha256: 6df08c80d677bc481b40bf95c49e4f89dd8d3821dc01b2124cd5014b573dfb94 -->
