<!-- doc-id: SKILL-debugging -->
<!-- revision: 1 -->
<!-- source-sha256: 8416229ba9ad2a37e09115a795263edbe0c5bdb52a18077d14f5a8f651fc5c5a -->
<!-- translation-status: synchronized -->

<a id="workflow"></a>

# Chẩn đoán

| Phần | Yêu cầu |
| --- | --- |
| Kích hoạt | Có lỗi hoặc sai khác |
| Đầu vào | Fixture, log, revision và kết quả kỳ vọng chính xác |
| Bước | Tái hiện trước. Lần theo biên và kiểm tra từng giả thuyết. Giữ yêu cầu an toàn và timeout. |
| Đầu ra | Nguyên nhân có bằng chứng, sửa hẹp và kết quả hồi quy. |
| Khi lỗi | Khi chưa tái hiện có bằng chứng, chỉ báo quan sát, không đổi gate. |

Áp dụng [quy tắc kiểm chứng](../../rules/testing.md).
