# Nạp context theo nhiệm vụ

| Lớp | Nạp | Quyết định |
| --- | --- | --- |
| L0 | Một baseline common và chỉ dẫn áp dụng của repo đích | Kiểm tra scope thực tế của client, checkout và chỉ dẫn lồng nhau |
| L1 | Metadata skill do client cung cấp | Chọn workflow có trigger phù hợp |
| L2 | Nội dung skill đã chọn, reference liên quan, source và test của owner | Chỉ mở rộng theo dependency hoặc rủi ro |

Dùng [danh mục Skill](../skills/README.md) nếu client không liệt kê metadata.
Mở rule theo ràng buộc liên quan. Tìm ký hiệu và đường dẫn, rồi đọc đủ code
quanh đó và output lỗi để hiểu thay đổi. Đọc lại khi revision, scope hoặc context thiếu.

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
bản tiếng Anh là hai bản sao của cùng chỉ dẫn. Import Claude nạp baseline một
lần. Adapter managed chỉ nên đưa vào baseline.

## Chat-to-Work

Khi được yêu cầu chuẩn bị Prompt cho Agent khác, chọn Skill biên soạn và mở
mẫu tiếng Anh từ checkout đã được phép. Chỉ đọc kiến trúc, chỉ dẫn của chủ
sở hữu và bằng chứng sẵn có liên quan. Trả bản bàn giao đúng phạm vi trong
chat. Yêu cầu sửa mã trực tiếp dùng các Skill thực thi thông thường.
[Skill biên soạn](../skills/cirnet-work-prompt-authoring/SKILL.md) ·
[Mẫu](../templates/chat-to-work-prompt.md) · [Truy cập từ bundle](distribution.md#soạn-work-prompt)

## Tình huống đọc thử

| Nhiệm vụ | Đọc chọn lọc | Nhận định dự kiến |
| --- | --- | --- |
| Sửa typo trong README | Chỉ dẫn local áp dụng và đoạn liên quan | Theo yêu cầu review local mà không phải nạp thư viện skill |
| Ghi lại bản sửa lỗi | Tìm chủ đề, issue và tài liệu sở hữu trong repository được phép | Cập nhật hướng dẫn chuẩn và liên kết tới bằng chứng có ngày. Theo [vòng đời tài liệu](../policies/documentation.md#vòng-đời-tài-liệu) |
| Đổi field response HTTP dùng chung | Quyền sở hữu theo Ecosystem, contract miền áp dụng, API compatibility và test của consumer trực tiếp | Xem SDK/default/null theo revision được chọn |
| Debug timing testbench FPGA | Chỉ dẫn lồng nhau FPGA, skill debugging, RTL và testbench liên quan. Đọc thêm phần an toàn Lab-First và kiến trúc FPGA khi thay đổi ranh giới an toàn Lab hoặc RTL | Báo kết quả mô phỏng cùng trạng thái kiểm chứng vật lý |

Đây là tình huống đọc tài liệu. Bộ [đánh giá hành vi](behavioral-evaluation.md)
riêng dùng dự án giả lập và ghi quan sát Codex native khi đủ điều kiện.

## Ngân sách văn bản

Mục tiêu 500 từ tách theo khoảng trắng và 4.000 byte UTF-8 cho baseline,
khoảng 300 từ cho index dự phòng, tối đa 240 ký tự cho mô tả skill và 900 từ
hoặc 250 dòng cho thân skill. Giữ quy tắc an toàn rõ ràng. Checker báo số đo
và cảnh báo tham khảo. Số token phụ thuộc phần client nạp và tokenizer.

<!-- source: en/docs/context.md -->
<!-- source-sha256: 28837cde97ad6005aa44ea458d1395168bec19df98ffda4dccd3c085451e7d2e -->
