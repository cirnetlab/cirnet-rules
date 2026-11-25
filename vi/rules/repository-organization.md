# Tiêu chuẩn tổ chức repository và module CIRNETLAB

- **Trạng thái** — PROPOSED (đề xuất)
- **Phạm vi** — Toàn bộ repository CIRNETLAB

Dùng đề xuất này để rà soát quyền sở hữu, tính gắn kết của tính năng và
trách nhiệm của module. Ưu tiên [chính sách kiến trúc](../policies/architecture.md)
hiện hành cùng chỉ dẫn, contract và quy ước của repository sở hữu. Áp dụng
thay đổi cấu trúc qua quy trình rà soát và cấp quyền của chủ sở hữu.

## Quyền sở hữu và tính gắn kết

Dùng **Chủ sở hữu repository → Miền → Tính năng → Module → Tệp** làm các
cấp logic. Tạo thư mục khi việc đó làm rõ quyền sở hữu, phụ thuộc hoặc
cách kiểm chứng. Mỗi thư mục liên quan cần có trách nhiệm và miền hoặc
module sở hữu dễ tìm. Xác định chủ sở hữu và mục đích của module, tệp,
artifact sinh tự động và bằng chứng. Dựa vào cấu trúc mã, ranh giới module,
tài liệu hoặc bản đồ repository hiện có của chủ sở hữu. Chỉ thêm README
cục bộ hoặc manifest chủ sở hữu khi repo có nhu cầu cụ thể. Chỉ bổ sung
cấu trúc khi cần cho năng lực, ranh giới sở hữu, kiểm thử hoặc khả năng bảo trì.

- **Miền** sở hữu hành vi, dữ liệu và bất biến của một phạm vi nghiệp vụ
  hoặc kỹ thuật. Dùng [bảng định tuyến chủ sở hữu](../references/architecture-routing.md)
  để tìm chủ sở hữu theo kiến trúc đích. Xác minh thẩm quyền hiện tại từ bên
  thực sự ghi dữ liệu, contract đã phát hành hoặc artifact đang được chọn
  cùng các bên sử dụng.
- **Tính năng** cung cấp một khả năng hoặc quy trình hoàn chỉnh. Dùng tên
  tính năng nhất quán hoặc bản đồ chủ sở hữu để dễ tìm mã, test và tài liệu.
- **Module** có một trách nhiệm gắn kết và ranh giới kiểm thử được. Tách
  module khi trách nhiệm, phụ thuộc, vòng đời hoặc ranh giới an toàn cần
  phân biệt. Giữ phần triển khai nhỏ thành một khối gắn kết.

## Vị trí và nguồn gốc

| Loại | Nội dung đặt theo vị trí hiện hành của repository sở hữu |
| --- | --- |
| Mã nguồn | Mã chạy thực tế trong cây nguồn theo quy ước ngôn ngữ, nhóm theo miền hoặc tính năng |
| Kiểm thử | Test đặt cạnh mã hoặc trong cây kiểm thử như `tests/`, `e2e/` hay `native/tests/` |
| Tài liệu | Giải thích, kế hoạch, đặc tả và bằng chứng, có cặp EN/VI khi áp dụng |
| Script và cấu hình | Công cụ build, test, migration hoặc vận hành có mục đích rõ, cùng manifest theo quy ước công cụ hoặc cấu hình đúng phạm vi |
| Contract | Giao diện có phiên bản và fixture tương thích do chủ sở hữu được chỉ định phát hành |
| Tệp sinh tự động | Đầu ra khai báo được, có thể truy đến bộ sinh, đầu vào và revision nguồn |
| Asset và fixture | Tài nguyên sản phẩm hoặc tính năng có nguồn gốc, giấy phép và mục đích sử dụng rõ |
| Báo cáo và bằng chứng | Lịch sử đã duyệt cho bằng chứng bất biến và thư mục đầu ra được cấp cho log dùng tạm |
| Migration | Thay đổi có phiên bản và tham chiếu lịch sử hỗ trợ migration có kiểm soát và rollback |

Chỉ tạo nhóm khi công việc cần đến. Dùng tên mô tả hành vi hoặc trách nhiệm
và theo quy ước ngôn ngữ, framework tại repo. Giữ một tài liệu chuẩn cho
mỗi mục đích và bảo toàn vị trí hồ sơ công việc riêng của repo, gồm quy
ước `.superpowers/plans/` và `.superpowers/specs/` hiện có. Theo
[vòng đời tài liệu](../policies/documentation.md) và dùng bên sinh có
thẩm quyền để tạo lại đầu ra dẫn xuất.

## Tìm trước khi tạo

1. Xác định khả năng cần bổ sung, chủ sở hữu và phạm vi được phép.
2. Tìm đường dẫn, tên tệp, symbol công khai, trách nhiệm, tài liệu và test
   trong phạm vi đó. Xem phần triển khai hiện tại và nơi gọi trực tiếp
   trước khi chọn vị trí.
3. Dùng lại hoặc mở rộng phần triển khai chuẩn khi phù hợp. Nêu trách nhiệm
   riêng nếu cần module mới và giải thích mọi phần trùng lặp.
4. Xác minh khả năng đó có một chủ sở hữu có thẩm quyền. Nếu quyền sở hữu
   còn chưa xác minh hoặc đang tranh chấp, báo phần thiếu bằng chứng và làm
   rõ với chủ sở hữu trước khi tạo hoặc sửa các đường dẫn phụ thuộc vào đó.

Chỉ dùng `utils/`, `common/`, `shared/` hoặc `helpers/` ở phạm vi toàn repo
khi có trách nhiệm dùng chung ổn định, bên sử dụng rõ và ranh giới kiểm thử
được. Giữ helper riêng của tính năng trong miền của nó kể cả khi nhiều nơi gọi.
Ưu tiên tên như
`capture/`, `validation/` hoặc `routing/` cho trách nhiệm theo miền.
Giữ các thư mục dùng chung có chủ sở hữu rõ và tách các trách nhiệm không
liên quan thành module riêng.

## Quy ước theo ngôn ngữ và repository

- **TypeScript và Vue.** Giữ các trách nhiệm về logic miền, điều phối ứng
  dụng, trình bày, trạng thái, đồ họa và test trình duyệt nếu đã có. Với
  CIRNET-Interface, giữ [vai trò hiện hành](engineering-workspace-ui.md) của
  ngữ nghĩa tài liệu CircuitGraph, thao tác ứng dụng, trạng thái UI và lớp
  hiển thị Three.js.
- **Rust.** Theo Cargo workspace và crate hiện có. Dùng module theo miền
  khi làm rõ phụ thuộc, giữ phần triển khai riêng ở phạm vi nội bộ, và
  dùng bằng chứng khi đề xuất tách workspace hoặc thêm tiến trình. Giữ đồ
  thị phụ thuộc module không có chu trình.
- **C++.** Giữ target CMake, điểm vào worker và ranh giới ABI hoặc giao
  thức. Phân biệt logic bộ giải, truyền tải của worker, giới hạn tài nguyên
  và adapter tích hợp khi trách nhiệm mở rộng.
- **FPGA và RTL.** Tổ chức theo khối phần cứng, giao diện và trách nhiệm
  kiểm chứng. Phân biệt RTL, ràng buộc, testbench, cấu hình tổng hợp và
  đầu ra nạp được sinh tự động. Bảo toàn tên module và top-level, giao
  diện clock/reset, ràng buộc và manifest tích hợp, trừ phần có migration
  tương thích đã duyệt. Dùng mô phỏng để kiểm chứng RTL và bằng chứng trên
  bo mạch được chủ sở hữu duyệt để nghiệm thu vật lý.
- **Repository chưa có triển khai.** Giữ README, giấy phép và tệp quản trị
  áp dụng. Thêm mã, thư mục kiến trúc và test khi một tính năng đã duyệt
  cần đến.

## Ranh giới liên repository

Dùng giao diện có phiên bản và tài liệu hoặc artifact đã duyệt để tái sử
dụng liên repo. Xác định bên phát hành, revision được chọn và bên dùng
trực tiếp qua [định tuyến chủ sở hữu](../references/architecture-routing.md)
và [quy tắc liên repo](cross-repo.md). Khi cần, rà soát các ranh giới dưới đây.

| Ranh giới | Trách nhiệm |
| --- | --- |
| Studio / Interface | Ghép không gian làm việc và chỉnh sửa mạch |
| Learn / LMS | Trình bày nội dung học tập và xử lý nghiệp vụ học tập |
| Project / Format-Project / Contracts | Vòng đời dự án, tuần tự hóa dự án để trao đổi và giao diện dùng chung chuẩn |
| Interface / Remote / Hardware | Trình bày Lab, cấp quyền thao tác và thực thi tại trạm |
| Analyze / Numerics / Simulator | Trải nghiệm phân tích, nhân tính toán tái sử dụng và mô phỏng điện tử |
| IDE / Toolchains / Devices | Chỉnh sửa, môi trường build, hồ sơ và adapter thiết bị |
| Rules / Repository sản phẩm | Hướng dẫn agent dùng chung, triển khai và ràng buộc của từng chủ sở hữu |

Giữ khả năng truy nguồn schema sao chép, SDK sinh tự động và adapter của bên dùng
đến chủ sở hữu có thẩm quyền.

## Tái cấu trúc an toàn và nghiệm thu

Trước khi di chuyển, lập danh sách chủ sở hữu nguồn và đích, import và
export, symbol và API công khai, bên gọi trực tiếp, ranh giới package và
crate, đầu vào build và bộ sinh mã, script, test, fixture, snapshot, độ
bao phủ, CI, đóng gói, triển khai, release, liên kết tài liệu và bản EN/VI.
Kiểm tra chữ hoa/thường, mã hóa và hành vi theo nền tảng. Xác định asset
sinh tự động, bằng chứng lịch sử và nhu cầu rollback. Áp dụng thay đổi nhỏ
nhất trong phạm vi được phép và cập nhật tham chiếu cùng lúc. Giữ chỉnh
sửa của người dùng, tệp bị bỏ qua hoặc chưa theo dõi, release đã đóng
băng, bằng chứng và nguồn gốc. Xin quyền rõ ràng trước khi di chuyển
liên repo, dọn dẹp hàng loạt hoặc sửa lịch sử theo cách phá hủy dữ liệu.

Xác minh chủ sở hữu và mục đích của từng đường dẫn thay đổi, một phần
triển khai có thẩm quyền, import, phụ thuộc, điểm vào build, test,
fixture, đóng gói và tham chiếu CI chịu ảnh hưởng. Sinh lại hoặc xác
minh đầu ra liên quan qua bên sinh có thẩm quyền. Kiểm tra liên kết tài
liệu và lịch sử, tính tương thích của contract công khai và release đã
ghim hoặc migration đã duyệt, rồi rà diff cuối. Báo kết quả quan sát
được dưới dạng `PASS`, `FAIL`, `BLOCKED` và `NOT_RUN`. Dùng bằng chứng
để chỉ ra cải thiện về quyền sở hữu, phần trùng lặp, phụ thuộc hoặc quy trình.

Dùng [Skill cấu trúc repository](../skills/cirnet-repository-structure/SKILL.md)
cho quy trình rà soát và kiểm chứng.

<!-- source: en/rules/repository-organization.md -->
<!-- source-sha256: b7aa3b937d91fabc7e50db8c7193108f0729251b73f317809970272912d9128f -->
