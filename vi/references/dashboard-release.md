<!-- doc-id: CIRNETRULES-DESKTOP-RELEASE -->
<!-- revision: 2 -->
<!-- source-sha256: a2d503b17f77d049158a74bc5573bd617c0ddfb00e0d0fb833ea991f2b91c271 -->
<!-- translation-status: synchronized -->
<a id="candidate"></a>

# Ứng viên ứng dụng Dashboard

> Hồ sơ lịch sử cho [source lưu trữ](https://github.com/cirnetlab/cirnet-rules/blob/52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c/vi/references/dashboard-release.md). Xem [quy trình CLI hiện hành](cirnetrules-local.md) để vận hành.

| Định danh | Ứng viên |
| --- | --- |
| Tên hiển thị | CIRNET RULES |
| ID sản phẩm | `cirnetrules` |
| Phiên bản ứng dụng | 0.1.0 |
| Tên release | `cirnet-rules-0.1.0` |
| Tag đề xuất | `v0.1.0` |
| Trạng thái phát hành | `RELEASE_BLOCKED` |

Ứng viên bổ sung chọn repository bằng Qt native, lưu binding Git worktree,
lưu hội thoại và intake theo phạm vi, sao lưu SQLite, lưu trữ dữ liệu cũ đã
rà và scheduler automation ở backend. Đăng ký catalog không cấp quyền thực
thi hay ghi file. Relink làm revision binding cũ mất hiệu lực. Gói nhắm đến
Windows x64 MSVC.

Tên tệp nén ứng viên dùng tên sản phẩm. Định danh release trong manifest
vẫn dùng tên kho mã.

Manifest archive ghi revision nguồn gốc, fingerprint cây nguồn đang dùng,
digest từng file cài và SHA-256 archive. Giữ manifest và checksum cạnh zip.
Kiểm tra cả hai trước khi thử. Giải nén vào thư mục riêng và chạy Dashboard
trong bin. Dùng thư mục dữ liệu dùng một lần và [quy trình vận hành](dashboard.md)
để kiểm chứng.

Gate phát hành còn cần thực thi provider native, cô lập đã kiểm chứng,
review độc lập có xác thực, cài Windows sạch và rà điều kiện phân phối.
Ứng viên local phục vụ kiểm thử. Lịch sử nguồn và thẩm quyền release sản
phẩm được bảo toàn. Xem [kiến trúc và giới hạn](dashboard-architecture.md).
