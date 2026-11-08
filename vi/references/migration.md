<!-- doc-id: MIGRATION-MAP -->
<!-- revision: 1 -->
<!-- source-sha256: 13eee21b4321c36f07db9f45c8474826b84d9d6a25b4c1847d37d35f768efa9f -->
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
| Placeholder Python trong tools gốc | Engine Rust trong en/tools và hướng dẫn tiếng Việt. |
| Placeholder adapter, hook, test và references gốc | Thư mục chức năng dưới en và vi. |
| Placeholder commands theo repo | Hướng dẫn lệnh dùng chung. |
| plans và specs gốc | Plan/spec sáu repo dưới en và vi. Placeholder CLI phụ trợ ngoài baseline. |
| skills gốc | Quy trình chung tiếng Anh và bản tương ứng tiếng Việt. |

Discovery dùng manifest tiếng Anh được chọn. Nguồn lịch sử vẫn truy cập
qua Git. Liên kết được kiểm tra ở hai ngôn ngữ. [Quy tắc tài liệu](../rules/documentation.md)
