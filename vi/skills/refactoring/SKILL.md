<!-- doc-id: SKILL-refactoring -->
<!-- revision: 1 -->
<!-- source-sha256: e0d79fcb69eac426fde7bd9ea904d0851ea590b944af99113ddf364c7a7c6634 -->
<!-- translation-status: synchronized -->

<a id="workflow"></a>

# Tái cấu trúc

| Phần | Yêu cầu |
| --- | --- |
| Kích hoạt | Đổi cấu trúc nội bộ |
| Đầu vào | Caller hiện tại, interface công khai và phạm vi test |
| Bước | Ánh xạ path cũ và mới. Giữ điểm vào thường dùng và hành vi, chạy hồi quy biên rồi loại nguồn chuẩn cũ sau khi xác minh nguồn thay thế. |
| Đầu ra | Ánh xạ di chuyển và hành vi đã kiểm chứng. |
| Khi lỗi | Consumer chưa rõ hoặc thiếu parity thì chưa dọn phần cũ. |

Áp dụng [quy tắc kiểm chứng](../../rules/testing.md).
