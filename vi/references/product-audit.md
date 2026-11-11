<!-- doc-id: AUDIT-PRODUCTS -->
<!-- revision: 2 -->
<!-- source-sha256: c0a91f20c294d5b532a8c8f161c73da487094cdd6c1c415ebb37a6b2ef3c6cc0 -->
<!-- translation-status: synchronized -->
# Kiểm toán sáu repo sản phẩm cho đợt phát hành 0.2.4

<a id="method"></a>

## Phạm vi và khả năng tái lập

Ảnh chụp bằng chứng local được thu ngày 2026-11-08 UTC trên Windows bằng Git 2.52.0.windows.1 và PowerShell 7.6.9. Sáu worktree local đều sạch và đang ở nhánh `master`. Truy vấn Git công khai độc lập xác nhận `master` là nhánh mặc định và trả về cùng commit. Truy vấn không thấy ref công khai `v0.2.4` hay `v0.2.2`. Điểm cuối phát hành công khai cho `v0.2.4` trả HTTP 404 ở cả sáu repo. Các bản nháp, phát hành riêng tư và bằng chứng CI riêng tư nằm ngoài phạm vi quan sát.

| Repository | Commit | Số đường dẫn Git theo dõi | Tag lịch sử công khai được chọn |
| --- | --- | ---: | --- |
| [cirnet-contracts](../specs/cirnet-contracts/audit.md#snapshot) | `4bfd0950f564aaf24abaf6b8b82399c2d54d0e39` | 334 | `v0.1.2` |
| [cirnet-remote](../specs/cirnet-remote/audit.md#snapshot) | `1a38c1331a07bdc3b5644c70eebb40590ae53452` | 629 | `v0.1.4` |
| [cirnet-hardware](../specs/cirnet-hardware/audit.md#snapshot) | `2139cccc8b36dc403c2f7182dfb670a97513858a` | 91 | `v0.1.2` |
| [cirnet-simulator](../specs/cirnet-simulator/audit.md#snapshot) | `c05be23bef71de44a9027319af945d3e635d0882` | 73 | `v0.1.2` |
| [cirnet-interface](../specs/cirnet-interface/audit.md#snapshot) | `f1ff70b9f22554c0680b92679e9e44a5544247ff` | 609 | `v0.1.4` |
| [cirnet-fpga](../specs/cirnet-fpga/audit.md#snapshot) | `bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0` | 56 | `v0.1.2` |

Báo cáo từng repo ghi ID cây gốc thực tế và digest danh mục thô. Sáu giá trị tra cứu ban đầu là đối tượng commit. Cây gốc ghi đường dẫn và định danh từng mục; blob định danh một đối tượng Git. SHA-256 artifact được tính trên byte artifact, độc lập với các ID Git.

Chạy các lệnh sau trong repo sản phẩm. Để tính digest danh mục, băm luồng byte nguyên gốc của lệnh Git cuối, kể cả các dấu phân cách NUL, trước khi phân tích bản ghi đường dẫn. Thay `HEAD` bằng commit đã ghim để tái lập lần kiểm toán này. Tránh băm pipeline đã được PowerShell giải mã.

```text
git --no-optional-locks status --porcelain=v1 --untracked-files=all
git rev-parse HEAD HEAD^{tree}
git --no-optional-locks -c credential.helper= ls-remote --symref origin HEAD refs/heads/master refs/tags/v0.2.4 refs/tags/v0.2.2 refs/tags/v0.1.4 refs/tags/v0.1.2
git --no-optional-locks ls-tree -r -z --full-tree HEAD
```

Đã lập danh mục đủ 1.792 đường dẫn Git theo dõi. Báo cáo từng repo ghi phần mã nguồn, tài liệu và hướng dẫn kiểm chứng đã đọc, cùng các nhóm chỉ được lập danh mục. Đã rà `README.md`, `AGENTS.md`, changelog và Git history liên quan của từng checkout. Chỉ dẫn Agent FPGA nằm trong `fpga/AGENTS.md`. Kiểm toán quản trị lần này chưa chạy build sản phẩm, máy sạch, CI đám mây hoặc phép thử vật lý. Byte artifact lưu trữ và hash biên nhận được đối chiếu riêng.

<a id="identity"></a>

## Định tuyến repository

Dùng [registry](../../governance.yaml) để xác định quyền sở hữu repository,
định tuyến task và quan hệ phụ thuộc có kiểu. Bảng trên ghim source đã audit
cho quyết định phát hành được ghi. Kiểm tra binding Git worktree thực tế
khi phân giải task mới.

<a id="contract-inventory"></a>

## Bằng chứng phụ thuộc

Audit từng repository xác định ranh giới bên cung cấp, bên sử dụng và phần
kiểm chứng tương thích còn cần cho họ `0.2` đã ghi nhận. Ghi revision
source và governance, đầu vào hợp đồng đã rà soát, digest artifact và biên
nhận chính xác. Owner sản phẩm quản lý chi tiết triển khai trong
repository của mình.

<a id="historical-evidence"></a>

## Tổ hợp đã ghi nhận

[Tổ hợp đã phát hành tại thời điểm audit](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/integration/release-set.json)
có SHA-256 `90a8ef4d30ddc19d3024dcb3790fef5d5e041f45fa47922bfd5e34c2fd19c419`.
Tính lại digest của 15 artifact lưu giữ cho kết quả khớp với tổ hợp đã ghi.

<a id="candidate-evidence"></a>

## Quyết định kiểm chứng

Audit ghi nhận **PRODUCT_0.2.4_CUTOVER_READINESS · BLOCKED**. Ứng viên cụ
thể cần source sạch, biên nhận tương thích, đóng gói tái lập và bằng chứng
chức năng, an toàn, rollback cho chính ứng viên đó. Biên nhận chức năng
trước đây áp dụng cho artifact gốc. Xem
[biên nhận release bị chặn](../../en/references/evidence/check-release-blocked.json) và
[audit gốc cùng chi tiết bằng chứng](https://github.com/cirnetlab/cirnet-rules/blob/52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c/vi/references/product-audit.md#candidate-evidence).
