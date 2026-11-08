<!-- doc-id: SKILL-contract-change -->
<!-- revision: 1 -->
<!-- source-sha256: 83270af2dc4efb6fd36cdc467fa10d54e254208ddd98e6d41d9934060a308243 -->
<!-- translation-status: synchronized -->

<a id="workflow"></a>

# Đổi contract

| Phần | Yêu cầu |
| --- | --- |
| Kích hoạt | Đổi shape request, encoding, ID hoặc ngữ nghĩa |
| Đầu vào | Nguồn producer, schema hiện hành, consumer và digest đóng băng |
| Bước | So sánh trường bắt buộc, enum, giới hạn số, đơn vị, mặc định và lỗi. Rà số protobuf đã dành, giới hạn frame và bằng chứng wire. Sinh mã một lần từ nguồn chuẩn. |
| Đầu ra | ADR, revision contract và generator chính xác, ca thuận và từ chối giữa consumer. |
| Khi lỗi | Phá tương thích chưa duyệt hoặc đầu ra sinh lệch chặn lựa chọn. |

Áp dụng [quy tắc kiểm chứng](../../rules/testing.md).
