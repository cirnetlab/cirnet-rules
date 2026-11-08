<!-- doc-id: GLOSSARY -->
<!-- revision: 1 -->
<!-- source-sha256: 6b2ab88bd1da53a6aeec2e6b71db4655b1f62684a5614ff8c0bdca75c213439a -->
<!-- translation-status: synchronized -->

<a id="glossary"></a>

# Thuật ngữ và bản dịch

| Định danh | Nghĩa |
| --- | --- |
| `releaseTrain` | Mục tiêu và tổ hợp release hệ thống chính xác. |
| `family` | Nhãn tương thích CIRNET dạng chuỗi. Cần tổ hợp chính xác đã duyệt. |
| `contractRevision` | Nguồn và digest cụ thể của contract. Độc lập với hậu tố path API hoặc phiên bản gói. |
| Commit, tree, blob, SHA-256 | Commit Git nhận dạng lịch sử và tree. Tree nhận dạng đồ thị snapshot. Blob nhận dạng đối tượng tệp. SHA-256 ở đây nhận dạng bytes artifact. |
| `OBSERVED` / `VERIFIED` | Quan sát từ nguồn / đã thực hiện phép kiểm tra với bằng chứng có phạm vi. |
| `TARGET` / `PROPOSED` | Đích người dùng chọn / quyết định thiết kế chờ phê duyệt. |
| `UNKNOWN` / `BLOCKED` | Thiếu dữ kiện / gate không thể tiếp tục với bằng chứng hiện có. |

Rà soát mức bắt buộc, phiên bản và từ ngữ an toàn ở hai ngôn ngữ. Giữ các
định danh như schema key, URL, enum, operation ID và lỗi.
[Quy tắc tài liệu](../rules/documentation.md)
