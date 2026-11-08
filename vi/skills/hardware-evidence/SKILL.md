<!-- doc-id: SKILL-hardware-evidence -->
<!-- revision: 1 -->
<!-- source-sha256: 71c8b6f99d09a4ab3423776ab9d5d0b85f34d0b0a0b254d06296c59413e4a1fd -->
<!-- translation-status: synchronized -->

<a id="workflow"></a>

# Rà bằng chứng phần cứng

| Phần | Yêu cầu |
| --- | --- |
| Kích hoạt | Có tuyên bố về board, permit hoặc capability vật lý |
| Đầu vào | Phạm vi RTL/scaffold, owner an toàn, định danh board/firmware và hồ sơ test được phép |
| Bước | Tách mô phỏng, tổng hợp, timing và bằng chứng board. Rà safe state, watchdog, shutdown và ca từ chối peer mà không cấp điện. |
| Đầu ra | Quyết định capability giới hạn theo bằng chứng thật. |
| Khi lỗi | Thiếu ủy quyền phần cứng hoặc bằng chứng HIL giữ nghiệm thu vật lý ở trạng thái mở. |

Áp dụng [quy tắc kiểm chứng](../../rules/testing.md).
