<!-- doc-id: CIRNETRULES-CHANGELOG -->
<!-- revision: 3 -->
<!-- source-sha256: 8c8e8cd6cb251fd5e96cb6afd3cf872db9421adfd6026cd98619e2b8b9a5cffe -->
<!-- translation-status: synchronized -->
<a id="unreleased"></a>

# Nhật ký thay đổi

## Chưa phát hành

### Bổ sung

- Không gian Cấu hình native quản lý bản nháp song ngữ, tệp văn bản, duyệt, công bố và khôi phục qua runtime Rust và SQLite.
- Có thể gán tài nguyên đã công bố cho binding repository và đưa vào bản xem trước context xác định. Hội thoại và lượt automation mới ghim metadata governance.
- Probe chỉ đọc kiểm tra endpoint phiên bản đã pin của OpenHands và OpenCode. Kiểm tra nhập báo cáo tập con được hỗ trợ của định dạng upstream đã chọn.

### Thay đổi

- Desktop CIRNET RULES hiện hành dùng manifest chung `application-identity.json`, IPC riêng revision 3, tên runtime và kho dữ liệu mới.
- Dashboard nâng cấp kho cục bộ đời trước qua backup đã kiểm tra và journal riêng có thể tiếp tục.

- Quy trình bảo trì hiện có có thể khôi phục task đã rà soát thành đề xuất chưa được cấp quyền, kèm biên nhận nguồn lưu bền vững.

### Sửa lỗi

- Migrator nâng cấp và kiểm chứng schema staging trước khi báo sẵn sàng. Lỗi từ chối truy cập lưu trữ giữ journal có thể tiếp tục và các bản sao lưu đã kiểm chứng.

Các thay đổi trước đó trong cùng chu kỳ chưa phát hành


- Desktop, runtime, tài nguyên và namespace thuộc ứng dụng dùng OPENRULES. Tên release vẫn là `cirnet-rules-0.1.0`.
- Kho SQLite cũ cần xem trước, xác nhận, sao lưu đã kiểm tra và di trú trước khi sử dụng. Nguồn cũ được giữ để khôi phục offline.
- IPC riêng của Desktop dùng giao thức 2 và tiền tố method `openrules.`. Client và runtime đời trước cần binary ứng dụng tương ứng.
