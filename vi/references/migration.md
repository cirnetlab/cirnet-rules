<!-- doc-id: MIGRATION-MAP -->
<!-- revision: 2 -->
<!-- source-sha256: 77d287df0d8761e141b2408f225e8f90cc48b656d832449586d6a79fead10ad0 -->
<!-- translation-status: synchronized -->

<a id="migration-map"></a>

# Ánh xạ đường dẫn cũ sang mới

Checkout governance gốc có tiêu đề README, giấy phép và placeholder được
Git theo dõi. Đợt di chuyển giữ giấy phép và lịch sử Git.

| Đường dẫn cũ | Đích hoặc cách xử lý |
| --- | --- |
| AGENTS và README gốc | Bootstrap mỏng và điều hướng song ngữ. |
| rules và profiles gốc | Quy tắc và hồ sơ tương ứng ở Anh và Việt. |
| Placeholder YAML trong policies gốc | Chính sách chung và một registry ở root. |
| schemas gốc | Schema chạy được trong en/schemas và một hướng dẫn tiếng Việt. |
| Placeholder Python trong tools gốc | Đã được thay bằng engine Rust trong đợt di chuyển trước. |
| Rust cũ trong en/tools/src và en/tests | Binary, module và test chuyển về src. EN/VI giữ hướng dẫn. |
| Cặp cutover/audit trước đây | Hồ sơ audit và kế hoạch migration lịch sử. Việc mới dùng thư mục repo/task tập trung. |
| Placeholder adapter, hook, test và references gốc | Thư mục chức năng dưới en và vi. |
| Placeholder commands theo repo | Hướng dẫn lệnh dùng chung. |
| plans và specs gốc | Plan/spec sáu repo dưới en và vi. Placeholder CLI phụ trợ ngoài baseline. |
| skills gốc | Quy trình chung tiếng Anh và bản tương ứng tiếng Việt. |

Discovery dùng manifest tiếng Anh được chọn. Nguồn lịch sử vẫn truy cập
qua Git. Liên kết được kiểm tra ở hai ngôn ngữ. [Quy tắc tài liệu](../rules/documentation.md)
