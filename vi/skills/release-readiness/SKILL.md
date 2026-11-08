<!-- doc-id: SKILL-release-readiness -->
<!-- revision: 1 -->
<!-- source-sha256: 862233d8b49a1590911ae689fe201a3a7850bef867151ec6d6aa28d8a4aec48a -->
<!-- translation-status: synchronized -->

<a id="workflow"></a>

# Sẵn sàng release

| Phần | Yêu cầu |
| --- | --- |
| Kích hoạt | Có artifact đề xuất cho một release train |
| Đầu vào | Receipt producer bất biến, release-set của owner, tổ hợp và gate đã duyệt |
| Bước | Xác minh định danh và digest chính xác, ref sạch và mọi gate producer/consumer. Xác minh rollback độc lập. Tách bằng chứng ứng viên và đã công bố, rồi xin ủy quyền. |
| Đầu ra | Kết quả sẵn sàng kèm mọi blocker. |
| Khi lỗi | Thiếu phê duyệt, lệch tổ hợp hoặc capability chưa kiểm chặn kích hoạt. |

Áp dụng [quy tắc kiểm chứng](../../rules/testing.md).
