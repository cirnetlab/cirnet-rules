<!-- doc-id: SKILL-audit -->
<!-- revision: 1 -->
<!-- source-sha256: fe18dceedb2777639b3bbf9369fd17a8821ffc7d4be8f80de15e24536c92bbcc -->
<!-- translation-status: synchronized -->

<a id="workflow"></a>

# Audit

| Phần | Yêu cầu |
| --- | --- |
| Kích hoạt | Repo mới hoặc có tuyên bố sẵn sàng |
| Đầu vào | Origin/ref Git chính xác, manifest và bằng chứng release |
| Bước | Đọc toàn bộ inventory tracked, không cắt. Xem nguồn chuẩn liên quan và ghi dữ kiện thiếu trước khi đề xuất. |
| Đầu ra | Audit có ngày, phân biệt commit/tree, ghi path đã đọc và gate bị chặn. |
| Khi lỗi | Thiếu ref hoặc không đọc được nguồn sẽ chặn kết luận liên quan. |

Áp dụng [quy tắc kiểm chứng](../../rules/testing.md).
