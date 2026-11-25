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

Cung cấp đề xuất version, changelog và ghi chú nâng cấp/rollback kèm giới hạn kiểm chứng.

Ví dụ: Tên skill đổi. Liệt kê tên cũ/mới và migration an toàn cho bản cài chưa rõ ownership. Việc tạo tag thuộc thao tác release được phép.

<!-- source: en/skills/cirnet-release-changelog/SKILL.md -->
<!-- source-sha256: 79e5006e10a28b9d660ec547093ec7e1dc1a85ebdbf47329162b5bfc8aa33c00 -->
