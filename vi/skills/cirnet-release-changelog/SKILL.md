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

Duy trì tệp tiếng Anh theo phiên bản chính trong `docs/changelogs/`. Dùng chỉ
mục ở thư mục gốc và thư mục changelog để điều hướng. Dùng Unreleased và phiên
bản Semantic Versioning đầy đủ với ngày công bố hoặc trạng thái có bằng chứng.
Xem chính sách tài liệu trong checkout Rules đã duyệt.

Cung cấp đề xuất version, changelog và ghi chú nâng cấp/rollback kèm giới hạn kiểm chứng.

Ví dụ: Tên skill đổi. Liệt kê tên cũ/mới và migration an toàn cho bản cài chưa rõ ownership. Việc tạo tag thuộc thao tác release được phép.

<!-- source: en/skills/cirnet-release-changelog/SKILL.md -->
<!-- source-sha256: 0204690c9a128bf9e29b83e5f1d3b0d47db7dd6ec246f5f1812bdb237e8dcc0c -->
