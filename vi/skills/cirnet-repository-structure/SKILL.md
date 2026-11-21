---
name: cirnet-repository-structure
description: Dùng khi rà soát cách tổ chức repository hoặc module, quyền sở hữu tệp, đầu ra sinh tự động hoặc việc di chuyển tệp có phạm vi.
---

# Cấu trúc repository

Dùng để xác định hoặc chỉnh ranh giới thực tế. Dùng refactoring khi chỉ đổi
cấu trúc code. Giữ layout theo ngôn ngữ nếu đang phù hợp.

Khi rà soát cách tổ chức, tham khảo đề xuất tại
`en/rules/repository-organization.md` trong checkout `cirnet-rules` đã duyệt.
Dùng chỉ dẫn hiện hành của chủ sở hữu và báo nếu thiếu quyền đọc đề xuất.
Bundle đã cài cung cấp nội dung Skill này.

Tối thiểu cần: Mục đích repo, entrypoint, owner, path sinh tự động/được bảo vệ, policy test/output và file được phép.

1. Xác định chủ sở hữu repository, miền, tính năng và module chịu trách nhiệm
   cho mã nguồn, test, tài liệu, tệp sinh tự động và đầu ra. Trước khi thêm
   đường dẫn, tìm đường dẫn, symbol, heading, chủ đề, mã issue và liên kết
   trỏ tới trong repo được phép. Xem bên gọi trực tiếp trước khi mở rộng
   module sẵn có hoặc giải thích lý do cần module riêng.
2. Trước khi di chuyển hoặc hợp nhất, lần theo import, liên kết trỏ đến tệp,
   script, CI, đóng gói và bên sử dụng phân biệt chữ hoa chữ thường. Xác định
   cách chuyển đổi cần thiết để giữ fixture, mẫu, giấy phép, phiên bản phục
   vụ tương thích, bằng chứng lịch sử và đường dẫn truy cập. Giữ riêng kế
   hoạch và đặc tả có mục đích khác nhau.
3. Áp dụng thay đổi được phép và cập nhật tham chiếu cùng lúc. Xác minh phụ thuộc,
   điểm vào và vị trí đầu ra. Dùng skill đồng bộ tài liệu cho nội dung song ngữ.

Báo bản đồ path mới, reference đã kiểm tra, artifact được bảo vệ và migration hoặc giới hạn.

Ví dụ: Chỉ di chuyển nguồn của binding sinh tự động sau khi tìm generator và mọi consumer. Sinh lại output bằng lệnh của owner.

<!-- source: en/skills/cirnet-repository-structure/SKILL.md -->
<!-- source-sha256: c563be9e3fd8f0eafb35a53f5afddc2fe90d9b2be9f3f749718e197ba29e7a85 -->
