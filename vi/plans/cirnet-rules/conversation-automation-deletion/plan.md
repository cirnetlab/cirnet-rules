<!-- doc-id: CGACP-CONVERSATION-AUTOMATION-DELETION-PLAN -->
<!-- revision: 1 -->
<!-- source-sha256: 6276fd2ea51ab24d81e4bce1442cf0c55c8b106f42816076c93d6d219c44f914 -->
<!-- translation-status: synchronized -->
<a id="steps"></a>

# Kế hoạch xóa Hội thoại và Automation

[Phạm vi và kiểm chứng](../../../specs/cirnet-rules/conversation-automation-deletion/spec.md)

1. Xác định Automation từ lượt chạy sinh hội thoại và hiển thị định nghĩa bị ảnh hưởng trong bản xem trước.
2. Xóa định nghĩa Automation trong cùng giao dịch SQLite khi xóa cả Hội thoại. Thêm thao tác xóa riêng cho các thẻ cũ đã tách liên kết.
3. Kiểm chứng bản xem trước hết hiệu lực, lượt chạy đang hoạt động, tên trùng, thử lại, khởi động lại và giao diện native trên dữ liệu tạm.
