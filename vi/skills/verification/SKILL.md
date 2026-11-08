<!-- doc-id: SKILL-verification -->
<!-- revision: 1 -->
<!-- source-sha256: 6551ae72bd03990398e21ac7f79380a080e3ae02d640fd30d6f09d8e97542a8b -->
<!-- translation-status: synchronized -->

<a id="workflow"></a>

# Kiểm chứng

| Phần | Yêu cầu |
| --- | --- |
| Kích hoạt | Trước hoàn tất, phê duyệt hoặc release |
| Đầu vào | Đầu vào chính xác, exit code kỳ vọng và môi trường kiểm thử |
| Bước | Chạy phép kiểm của owner trong môi trường cô lập. Đọc toàn bộ kết quả, phân biệt phạm vi mẫu, ca bỏ qua và lỗi thật. |
| Đầu ra | Receipt nối đầu vào, lệnh, kết quả và gate. |
| Khi lỗi | Thiếu bằng chứng chặn gate tương ứng. |

Áp dụng [quy tắc kiểm chứng](../../rules/testing.md).
