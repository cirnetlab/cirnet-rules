---
name: cirnet-documentation-sync
description: Dùng khi tạo hoặc sửa tài liệu, gồm chỉ dẫn, ví dụ, điều hướng hoặc bản dịch.
---

# Đồng bộ tài liệu

Dùng chính sách tài liệu của repository cho phạm vi bị ảnh hưởng.

1. Xác định mục đích. Tìm trong repository được phép theo tên tệp, tiêu đề,
   heading, chủ đề, mã issue, nội dung liên quan và liên kết trỏ tới. Cập nhật
   tài liệu chuẩn tại chỗ. Xác định repo sở hữu và liên kết tới nguồn của repo đó. Chỉ tạo tệp cho mục đích riêng hoặc bằng chứng bắt
   buộc, rồi ghi lý do chọn vị trí đó.
2. Khi có phần trùng, đề xuất tệp chuẩn, nội dung cần giữ và liên kết cần
   cập nhật. Hợp nhất trong phạm vi được phép, giữ kế hoạch và đặc tả có
   mục đích riêng, kết quả có ngày, phiên bản phục vụ tương thích, cặp EN/VI,
   fixture và mẫu.
3. Cập nhật bản Anh trước. Soát nghĩa, lệnh, tham số, phiên bản, kết quả
   kiểm thử, khả năng tương thích, nguồn gốc và giới hạn an toàn trong bản
   Việt tương ứng trước khi cập nhật hash nguồn.
4. Chạy checker phù hợp và `git diff --check`. Thêm `--new-file PATH` cho mỗi
   đường dẫn Markdown mới mà `check-docs` hỗ trợ. Khi không có checker phù
   hợp cho một đường dẫn, rà soát thủ công trong phạm vi được phép về mục
   đích, phần trùng, liên kết và độ mới của bản dịch. Rà soát đề xuất trùng
   lặp rồi báo các cặp đã đổi, quyết định và giới hạn kiểm chứng.

Với bản lưu tài liệu kiến trúc, làm theo `en/policies/architecture.md`. Giữ
nội dung nguồn được cung cấp và xác nhận đường dẫn cùng digest của cặp EN/VI
trong `docs/architecture-sources.tsv`. Rà soát các bản có nội dung khác nhau
và giữ lịch sử khi đề xuất thay liên kết.

<!-- source: en/skills/cirnet-documentation-sync/SKILL.md -->
<!-- source-sha256: a8f91643f8e70b4e41b06e364c812fb9c48f3f778340124be992e6ba40af1adb -->
