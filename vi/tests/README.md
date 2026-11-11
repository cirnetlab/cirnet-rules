<!-- doc-id: TESTS -->
<!-- revision: 2 -->
<!-- source-sha256: 24c4fd466bffc25ea8d902b1dbfea255edf0ee2ce9a2fb4e4669fc04a6fd408f -->
<!-- translation-status: synchronized -->
<a id="verification"></a>

# Kiểm thử governance

Chạy [các lệnh kiểm chứng cục bộ](../references/verification.md) từ thư mục
gốc governance. Bộ test Rust kiểm tra chính sách và bản dịch, quyền của task,
định danh binding, kiểm soát lease, độ mới của bằng chứng, xuất cấu hình theo
phạm vi và MCP STDIO. Cổng CI có
[quy trình duyệt bằng chứng ứng viên](../references/shared-task-coordination.md) riêng.

Dùng fixture dùng một lần cho các thao tác ghi khi test và tuân theo
[quy tắc kiểm thử](../rules/testing.md) khi ghi kết quả.
