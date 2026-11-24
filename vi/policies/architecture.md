# Tuân thủ kiến trúc và ưu tiên Lab-First

## Chọn nguồn có thẩm quyền theo tác vụ

Dùng [bảng định tuyến repository](../references/architecture-routing.md) để tìm
repo sở hữu và kiến trúc liên quan. Đọc các mục tiếng Anh chi phối thay đổi cùng
chỉ dẫn, hợp đồng, artifact và kiểm thử hiện hành của repo sở hữu. Dùng checkout
được phép hoặc artifact đã ghim và có thể xác minh nguồn gốc. Bộ cài agent cung
cấp chỉ dẫn tra cứu. Đọc kiến trúc từ checkout hoặc artifact đó khi cần.

| Phạm vi | Nguồn có thẩm quyền |
| --- | --- |
| Ranh giới tổ chức, dịch vụ dùng chung và tích hợp | [Kiến trúc Ecosystem](../../docs/tai-lieu-kien-tuc-he-thong/cirnetlab-architecture-vi.md), đặc biệt §§4–5 về quyền sở hữu. |
| Lab ảo và vật lý, an toàn điện và thực thi thí nghiệm | [Kiến trúc Lab-First](../../docs/tai-lieu-kien-tuc-he-thong/cirnet-architecture-lab-first-vi.md). |
| RTL, thiết bị đo, định tuyến và phần cứng bo FPGA của Lab | [Kiến trúc FPGA](../../docs/cirnetlab-FPGA-architecture-vi.md) trong giới hạn an toàn Lab-First. |
| Triển khai, lệnh và kiểm thử | Chỉ dẫn local áp dụng, mã nguồn và kiểm thử của repository. Dùng schema do repo sở hữu hợp đồng công bố và phiên bản bên tiêu thụ đã chọn. |

Khi có xung đột thực sự, dẫn cả hai quyết định kèm số mục, mô tả ranh giới bị ảnh
hưởng và yêu cầu các bên sở hữu rà soát. Tiếp tục phần việc độc lập trong phạm vi.
Xem quyền sở hữu mục tiêu là quyết định thiết kế và xác minh bên ghi dữ liệu, bên
phát hành định danh, bản phát hành được chọn và bằng chứng trước khi đổi ranh giới.

Xác định tác vụ → chọn kiến trúc → xác định repository sở hữu → kiểm tra hợp đồng
và artifact → lập kế hoạch → triển khai → kiểm tra tuân thủ kiến trúc → báo cáo
kết quả thực tế và giới hạn.

## Ưu tiên phát triển

`LAB_FIRST_PRIMARY` là mặc định. Ưu tiên `cirnet-interface`, `cirnet-remote`,
`cirnet-contracts`, `cirnet-hardware`, `cirnet-simulator` và `cirnet-embedded-hardware`.
Trước hết cần chứng minh luồng thật Web → Rust Remote → Rust Hardware → C++
Simulator → Web. Trong phạm vi vận hành production của nền tảng, dùng Rust cho
Backend, runtime Contracts và Hardware, C++ cho Simulator mặc định, TypeScript
cho Web và Verilog/SystemVerilog cho FPGA. Dùng Python trong kiểm thử, benchmark
và xác minh lịch sử tách biệt.

- Tích hợp qua hợp đồng do bên sở hữu công bố và ghim chính xác artifact. Xác minh
  tính tương thích và nguồn gốc của bộ đã chọn. Giữ nguyên
  [các cổng VD-01–VD-10 ở mục §6.5 của Lab-First](../../docs/tai-lieu-kien-tuc-he-thong/cirnet-architecture-lab-first-vi.md#65-acceptance-gates-virtual-digital-e2e)
  làm tiêu chí nghiệm thu Virtual Digital E2E.
- Tách kết quả Safety, Topology và Functional. An toàn và khả năng thực thi là
  điều kiện để vận hành vật lý. Gắn endpoint theo topology vật lý quan sát được.
  Physical Compiler chỉ định tuyến hạ tầng được phép, giữ nguyên dây sinh viên
  đã lắp. Trả `UNAVAILABLE` cho khả năng chưa hỗ trợ. Không thay bằng số đo
  hay kết quả giả.
- Dùng [trách nhiệm kiểm thử ở mục §19 của Lab-First](../../docs/tai-lieu-kien-tuc-he-thong/cirnet-architecture-lab-first-vi.md#19-verification-strategy-và-definition-of-done).
  Remote sở hữu kiểm thử phân quyền, lập kế hoạch và vòng đời thao tác. Hardware
  và Simulator kiểm tra Stop với worker thật và các tình huống lỗi. Remote,
  Hardware và FPGA kiểm tra gắn kết theo quan sát và giới hạn định tuyến vào hạ
  tầng tại ranh giới của mình. Hardware và FPGA cần HIL trên bo và phê duyệt của
  bên sở hữu để vận hành vật lý. Kiểm tra tĩnh chỉ xác minh thuộc tính của tài
  liệu hoặc mã nguồn. Xem các kiểm thử cụ thể trong
  [bằng chứng theo repo sở hữu](../docs/toolkit-verification.md#tuân-thủ-kiến-trúc).

Dùng `PARALLEL_ALLOWED` để mở rộng có phối hợp khi các bên sở hữu có hồ sơ chứng
minh một luồng Lab-First xuyên suốt và ổn định, gồm tương thích hợp đồng liên
quan, chuỗi thực thi thật, Stop/hủy, kiểm thử tình huống lỗi và nguồn gốc artifact
chính xác. Ghi phạm vi đã có bằng chứng và giới hạn. Đây là điều kiện phát triển.
Ghi từng kết quả VD riêng theo §6.5 và bộ artifact đã chọn. Dùng
`LAB_FIRST_PRIMARY` cho bộ sản phẩm vừa kiểm tra. IDE, LMS, Studio,
tính toán khoa học và các phần khác của Ecosystem có thể làm độc lập trong phạm
vi của mình khi Lab-First vẫn được ưu tiên, miễn là giữ luồng Lab hoạt động, tôn
trọng quyền sở hữu hiện có và giữ một nguồn có thẩm quyền cho mỗi miền.

## Nguồn chuẩn và các bản chuyên biệt

Hai cặp EN/VI của Ecosystem và Lab-First trong `docs/tai-lieu-kien-tuc-he-thong/`
là nguồn chuẩn của toolkit. Xem [revision nguồn và mã kiểm tra](../../docs/architecture-sources.tsv).
Giữ bốn bản EN/VI trùng tên trong
`docs/tai-lieu-kien-tuc-chuyen-biet/` tại checkout nguồn. Các bản chia sẻ quyết
định cốt lõi và mô tả với mức chi tiết khác nhau. Dùng nguồn chuẩn để định tuyến
và chuyển đề xuất sửa quyết định đến bên sở hữu.

| Quyết định | Bản hệ thống chuẩn | Bản chuyên biệt |
| --- | --- | --- |
| Chuyển quyền sở hữu Ecosystem | §§4–5 và §21 xác định chủ sở hữu mục tiêu, yêu cầu kiểm thử chuyển giao và giữ miền lease và an toàn Lab ở Remote. | §3 chỉ rõ thẩm quyền standalone và tích hợp theo tenant và loại bản ghi, đối chiếu qua đọc song song, khóa ghi ở hệ thống cũ và hoàn tác. |
| Tổ hợp phát hành Ecosystem | §16 giữ quyền chọn bộ phát hành Lab ở Remote và để ngỏ chủ sở hữu manifest toàn hệ sinh thái. | §10 nêu Platform là bên có thể sở hữu danh mục triển khai liên ứng dụng, trong đó tham chiếu khóa phát hành Lab. |
| Quét và bảo vệ trong Lab | §§8.2 và 10 yêu cầu trạng thái an toàn đã xác minh, kích thử có giới hạn, bằng chứng quan sát và vận hành có bảo vệ. | §6.2 dùng quyền kích quét riêng có giới hạn, giữ nguồn DUT và kích thí nghiệm tắt, đồng thời kiểm tra sơ bộ nguồn dư và nguồn ngoài. |
| Bằng chứng Virtual Digital của Lab | §§6.4–6.5 quy định ghim chính xác và đầy đủ điều kiện VD-01–VD-10, tách tích hợp Windows khỏi xác nhận Mini PC Linux. | §§5 và 12 nêu lại các cổng nghiệm thu, chỉ rõ chuỗi tham chiếu AND/LED và thứ tự nghiệm thu trên Linux mục tiêu. |

Phân biệt các quyết định trên với trạng thái triển khai đã quan sát. Dùng
[bằng chứng có ngày của sáu repository](../references/repository-map.md) để tra
cứu ban đầu và xác minh trạng thái hiện tại từ checkout repo sở hữu và bộ đã chọn.

<!-- source: en/policies/architecture.md -->
<!-- source-sha256: 8a8f9bf1f98e8e5452bdae4579462f4df059c4dfc149d393ba8b97505ce1c39b -->
