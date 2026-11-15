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
   Tìm trong repository được phép theo đường dẫn, heading, chủ đề, mã issue
   và liên kết trỏ tới.
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
<!-- source-sha256: 864ec697c95f7bf1b780ae01832f99c21c3b52f4db20431164ea890d7ef1b082 -->
