---
name: cirnet-repository-structure
description: Dùng khi review ownership file, layout repository, output sinh tự động hoặc di chuyển file có phạm vi.
---

# Cấu trúc repository

Dùng để xác định hoặc chỉnh ranh giới thực tế. Dùng refactoring khi chỉ đổi
cấu trúc code. Giữ layout theo ngôn ngữ nếu đang phù hợp.

Tối thiểu cần: Mục đích repo, entrypoint, owner, path sinh tự động/được bảo vệ, policy test/output và file được phép.

1. Lập bản đồ ownership source, test, docs, file sinh tự động và output. Giữ fixture, license và bằng chứng lịch sử.
2. Trước khi di chuyển, tìm import, link, script, CI, packaging và bên phân biệt hoa thường. Xác định migration nếu cần.
3. Di chuyển trong phạm vi và cập nhật reference cùng lúc. Kiểm tra dependency, entrypoint và vị trí output.

Báo bản đồ path mới, reference đã kiểm tra, artifact được bảo vệ và migration hoặc giới hạn.

Ví dụ: Chỉ di chuyển nguồn của binding sinh tự động sau khi tìm generator và mọi consumer. Sinh lại output bằng lệnh của owner.

<!-- source: en/skills/cirnet-repository-structure/SKILL.md -->
<!-- source-sha256: 58f592d7d4f96ed53b44fc8fd8bc33e23665e6bd6ab3c1edb935c170c004f585 -->
