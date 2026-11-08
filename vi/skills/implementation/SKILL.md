<!-- doc-id: SKILL-implementation -->
<!-- revision: 1 -->
<!-- source-sha256: 72448ed4c6eea755bd43ba0b8dbce20be19ea27ea84890e654b2936c1d22970a -->
<!-- translation-status: synchronized -->

<a id="workflow"></a>

# Triển khai

| Phần | Yêu cầu |
| --- | --- |
| Kích hoạt | Công việc có phạm vi đã duyệt |
| Đầu vào | ID yêu cầu, plan, owner và phép nghiệm thu |
| Bước | Đọc hành vi hiện tại, tái hiện rủi ro, sửa đúng tệp đã được phép, kiểm tra đường chính và ca từ chối. |
| Đầu ra | Diff nhỏ và bằng chứng có giới hạn rõ. |
| Khi lỗi | Lệch phạm vi, thiếu phép kiểm hoặc tác dụng phụ ngoài dự kiến chặn hoàn tất. |

Áp dụng [quy tắc kiểm chứng](../../rules/testing.md).
