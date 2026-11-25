# Nạp context theo nhiệm vụ

| Lớp | Nạp | Quyết định |
| --- | --- | --- |
| L0 | Một baseline common và chỉ dẫn áp dụng của repo đích | Kiểm tra scope thực tế của client, checkout và chỉ dẫn lồng nhau |
| L1 | Metadata skill do client cung cấp | Chọn workflow có trigger phù hợp |
| L2 | Nội dung skill đã chọn, reference liên quan, source và test của owner | Chỉ mở rộng theo dependency hoặc rủi ro |

Dùng [danh mục Skill](../skills/README.md) nếu client không liệt kê metadata.
Mở rule theo ràng buộc liên quan. Tìm ký hiệu và đường dẫn, rồi đọc đủ code
quanh đó và output lỗi để hiểu thay đổi. Quyết định theo từng vai trò:

- **Required** gồm baseline, chỉ dẫn local và lồng nhau áp dụng, thân Skill
  đã chọn và bằng chứng source hoặc nghiệm thu mang tính quyết định.
- **Conditional** gồm source, kiến trúc, log hoặc kiểm tra công cụ bổ sung
  khi phụ thuộc, rủi ro, lỗi đầu tiên hoặc quyết định còn mở đòi hỏi.
- **Already-loaded** gồm tài liệu đã rõ định danh và revision hoặc digest áp
  dụng, còn sẵn nội dung. Dùng lại trong cùng vai trò. Đọc lại khi revision
  đổi, phạm vi mở rộng hoặc xác định được khoảng trống bằng chứng.

Khi nhiệm vụ ảnh hưởng đến một quyết định kiến trúc CIRNET, dùng
[chọn kiến trúc](../policies/architecture.md) để tìm các phần áp dụng và
[định tuyến quyền sở hữu](../references/architecture-routing.md) để xác định
repo sở hữu. Sau khi cài đặt, tìm chính sách trong checkout `cirnet-rules`
được phép hoặc artifact kiến trúc đã xác minh.

Mở [ví dụ kỹ thuật](../skills/cirnet-implementation/references/engineering-examples.md)
khi Before/After tương tự giúp làm rõ thay đổi. Ví dụ đi cùng skill implementation
và nằm ngoài context khởi động.

Context cho Agent thường lấy từ tiếng Anh. Bản dịch Việt, plan cũ, lịch sử
và bản đồ sản phẩm dành cho review hoặc task có nhu cầu. Baseline root và
bản tiếng Anh là hai bản sao của cùng chỉ dẫn. Bỏ lần đọc bản sao thứ hai khi
đã xác minh chúng giống nhau. Import Claude nạp baseline một
lần. Adapter managed chỉ nên đưa vào baseline.

## Vòng đời mô hình

Chọn mô hình và effort khi chạy. Dùng [kiểm tra tương thích theo rủi ro](model-compatibility.md)
khi thay đổi ảnh hưởng đến quyền truy cập công cụ, quyền hạn hoặc hành vi kỹ thuật.

## Tình huống đọc thử

| Nhiệm vụ | Đọc chọn lọc | Nhận định dự kiến |
| --- | --- | --- |
| Sửa typo trong README | Chỉ dẫn local áp dụng và đoạn liên quan | Theo yêu cầu review local mà không phải nạp thư viện skill |
| Ghi lại bản sửa lỗi | Tìm chủ đề, issue và tài liệu sở hữu trong repository được phép | Cập nhật hướng dẫn chuẩn và liên kết tới bằng chứng có ngày. Theo [vòng đời tài liệu](../policies/documentation.md#vòng-đời-tài-liệu) |
| Đổi field response HTTP dùng chung | Quyền sở hữu theo Ecosystem, contract miền áp dụng, API compatibility và test của consumer trực tiếp | Xem SDK/default/null theo revision được chọn |
| Debug timing testbench FPGA | Chỉ dẫn lồng nhau FPGA, skill debugging, RTL và testbench liên quan. Đọc thêm phần an toàn Lab-First và kiến trúc FPGA khi thay đổi ranh giới an toàn Lab hoặc RTL | Báo kết quả mô phỏng cùng trạng thái kiểm chứng vật lý |

## Ngân sách văn bản

Mục tiêu 500 từ tách theo khoảng trắng và 4.000 byte UTF-8 cho baseline,
khoảng 300 từ cho index dự phòng, tối đa 240 ký tự cho mô tả skill và 900 từ
hoặc 250 dòng cho thân skill. Giữ quy tắc an toàn rõ ràng. Checker báo số đo
văn bản và cảnh báo tham khảo cho người bảo trì.

<!-- source: en/docs/context.md -->
<!-- source-sha256: 0e365820959dece3e845441d69afba7b4128b160bc9ba0b75e9b93f2d329f1a1 -->
