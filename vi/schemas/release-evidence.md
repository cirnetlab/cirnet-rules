<!-- doc-id: schema-release-evidence -->
<!-- revision: 1 -->
<!-- source-sha256: a314c7df495620c1eabbeb04bc31bb02eef71b87aa78535b041aec1424bb3c08 -->
<!-- translation-status: synchronized -->
# Định dạng đầu vào và bằng chứng phát hành native

<a id="authority"></a>
## Nguồn chuẩn và kích hoạt

Các schema đề xuất xác định cổng kiểm tra cục bộ cho baseline phần mềm Windows
MSVC. Remote sở hữu `integration/release-set.json` và
`integration/compatibility-matrix.json`. Governance lưu digest của ma trận,
danh mục bằng chứng và manifest phát hành đã duyệt. Schema v1 lịch sử vẫn đọc
được dưới trạng thái cutover bị chặn. Release
train `0.2.4` và họ chuỗi `"0.2"` tách khỏi revision hợp đồng, phiên bản
toolchain và commit Git governance.

Ứng viên có dữ liệu cục bộ nhất quán và chưa có danh mục bằng chứng đã rà soát
ghi `integrity_validation: PASS` với `status: BLOCKED`. Sau khi danh mục được
duyệt, `candidate_validation: PASS` vẫn có trạng thái `BLOCKED` cho đến khi
baseline được kích hoạt và manifest của repo sở hữu được phát hành và phê
duyệt. Bên sở hữu release xác minh nguồn gốc tag và artifact trên hệ thống
phát hành trước khi pin SHA-256 của chính xác bytes manifest đã phát hành.
Công cụ ngoại tuyến kiểm tra digest này, bytes release-set đã commit và đích
tag cục bộ chính xác.

<a id="selection"></a>
## Lựa chọn chính xác

[Schema phát hành](../../en/schemas/native-release.schema.json) xác định sáu
commit thành phần, tên artifact có phiên bản, đích, kích thước, SHA-256,
định danh toolchain, đầu vào build, khả năng và bytes hợp đồng. Digest lựa chọn
băm JSON UTF-8 gọn, sắp xếp đệ quy khóa đối tượng, giữ thứ tự mảng và chỉ dùng
các trường `contracts`, `family`, `releaseTrain`, `repositories` và
`rollback`. Các mảng artifact là một phần của lựa chọn đã rà soát.
[Ma trận tương thích](../../en/schemas/compatibility-matrix.schema.json) chỉ
cho phép các digest lựa chọn chính xác. Bytes JSON gốc của ma trận phải khớp
digest đã duyệt trong registry.

Git cung cấp bytes đầu vào build từ từng commit đã chọn. Đầu vào khóa là
`Cargo.lock` cho Rust, `package-lock.json` cho Interface, `CMakeLists.txt` cho
Simulator và `integration/toolchain-lock.json` cho FPGA. Công cụ so sánh pin
nguồn Rust Contracts với commit Contracts được chọn và lựa chọn SDK Interface
với bytes gói npm Contracts đã chọn, gồm SHA-256 của artifact và SHA-512 từ
npm lock. Các đích native phải khớp nhau. Đích web và mã
nguồn FPGA lần lượt là `web-static` và `portable-source`.

<a id="evidence"></a>
## Bằng chứng

Giữ bằng chứng đã rà soát trong workspace sản phẩm, bên ngoài các checkout
repo, dưới `cirnet-release-evidence/<selectionSha256>/`.

- `artifacts/<repository>/<filename>` chứa bytes thành phần được chọn.
- `contracts/<domain>/<filename>` chứa bytes hợp đồng được chọn.
- `gates/<gate>.json` chứa [bản ghi cổng](../../en/schemas/release-evidence.schema.json) chính xác.
- `results/<resultFile>` chứa kết quả bằng chứng tương ứng.

Mỗi bản ghi cổng gắn commit repo, commit governance, digest lựa chọn, lệnh,
phiên bản tool, nền tảng, thời điểm UTC, digest kết quả, mã thoát và giới hạn.
Công cụ của bên sản xuất tạo bản ghi trong phiên kiểm chứng được phép. Sau
khi rà soát, `release_policy.approved_evidence` lưu `governance_commit` của
chính sách dùng để kiểm chứng và `index_sha256` của ánh xạ `evidence` trong
manifest, tuần tự hóa thành JSON UTF-8 gọn với khóa đối tượng được sắp xếp đệ
quy. Bản ghi phải dùng commit chính sách đó. Commit phải là tổ tiên của
checkout governance hiện tại hoặc chính là commit hiện tại. Thay bản ghi hay
kết quả làm đổi chuỗi hash đã duyệt và cần rà soát lại. Các đường dẫn Python
hoặc legacy đang hoạt động
trong checkout đã chọn chặn cổng, còn kiểm thử, benchmark và tài liệu lịch sử
được loại khỏi quét đường dẫn Python. Cổng `legacy_rejection` kiểm tra hành vi
từ chối riêng. Bằng chứng mô phỏng chỉ bao phủ phạm vi phần mềm đã rà soát.
Khả năng vật lý và firmware cần rà soát HIL riêng ngoài công cụ này.
