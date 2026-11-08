<!-- doc-id: CGACP-SPEC -->
<!-- revision: 2 -->
<!-- source-sha256: 951349907a8a12a14d7bb1c34c94e4f30a382bce5dae3d6684a62da9a97fc74a -->
<!-- translation-status: synchronized -->
<a id="scope"></a>

# Đặc tả triển khai CGACP

CLI governance song ngữ hiện có kiểm tra release và tạo gói adapter.
Audit lịch sử sáu sản phẩm vẫn là snapshot bằng chứng. Control plane
local bổ sung nhiệm vụ độc lập, định tuyến theo phạm vi và bằng chứng.

| Yêu cầu | Nghiệm thu |
| --- | --- |
| CGACP-01–07 | Liên kết Git worktree local, định tuyến đường dẫn đã chọn, kiểm soát mở rộng rủi ro và dùng context nhạy với thay đổi nguồn. |
| CGACP-08–15 | Tách task state, quyền, bước chạy, kiểm chứng, review, nghiệm thu và đồng bộ tài liệu có kiểm tra xung đột. |
| CGACP-16–20 | Chọn policy release tường minh, giữ test lịch sử, ghi giới hạn adapter, kiểm soát ngân sách và đặt Rust dưới src. |

Repo này là đích triển khai. Thao tác sản phẩm khi kiểm chứng dùng fixture
dùng một lần. Các nguồn release và contract giữ chủ sở hữu hiện tại.
Khám phá Agent native và cô lập máy chủ cần bằng chứng riêng.
