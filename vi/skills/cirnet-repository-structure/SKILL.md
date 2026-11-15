---
name: cirnet-repository-structure
description: Dùng khi review ownership file, layout repository, output sinh tự động hoặc di chuyển file có phạm vi.
---

# Cấu trúc repository

Dùng để xác định hoặc chỉnh ranh giới thực tế. Dùng refactoring khi chỉ đổi
cấu trúc code. Giữ layout theo ngôn ngữ nếu đang phù hợp.

Tối thiểu cần: Mục đích repo, entrypoint, owner, path sinh tự động/được bảo vệ, policy test/output và file được phép.

1. Xác định chủ sở hữu mã nguồn, kiểm thử, tài liệu, tệp sinh tự động và đầu ra.
   Trước khi thêm đường dẫn, tìm chủ sở hữu và vị trí hiện có cho mục đích đó.
2. Trước khi di chuyển hoặc hợp nhất, lần theo import, liên kết trỏ đến tệp,
   script, CI, đóng gói và bên sử dụng phân biệt chữ hoa chữ thường. Xác định
   cách chuyển đổi cần thiết để giữ fixture, giấy phép, bằng chứng lịch sử và
   đường dẫn truy cập.
3. Áp dụng thay đổi được phép và cập nhật tham chiếu cùng lúc. Xác minh phụ thuộc,
   điểm vào và vị trí đầu ra. Dùng skill đồng bộ tài liệu cho nội dung song ngữ.

Báo bản đồ path mới, reference đã kiểm tra, artifact được bảo vệ và migration hoặc giới hạn.

Ví dụ: Chỉ di chuyển nguồn của binding sinh tự động sau khi tìm generator và mọi consumer. Sinh lại output bằng lệnh của owner.

<!-- source: en/skills/cirnet-repository-structure/SKILL.md -->
<!-- source-sha256: 7c66fec15806adae7df7363172e9e4451427a6e7b01c4f7b0e84cd9ed5f62ce9 -->
