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

## Chat-to-Work

Khi được yêu cầu chuẩn bị Prompt cho Agent khác, chọn Skill biên soạn và mở
mẫu tiếng Anh từ checkout đã được phép. Chọn Work Skills áp dụng từ metadata,
chỉ đọc thân các Skill đó, kiến trúc, chỉ dẫn owner và bằng chứng liên quan.
Chọn LIGHT, STANDARD hoặc DEEP theo độ phức tạp, rủi ro, phụ thuộc, bất định
và quyền đọc source. Xét đủ 12 trường ngữ nghĩa, gom thông tin áp dụng vào
các heading nhiệm vụ cần.

Tham chiếu nguồn Work đọc được và ghi trực tiếp ràng buộc riêng của nhiệm vụ.
Nếu Work thiếu quyền đọc source, đưa dữ kiện đáng tin cậy thiết yếu, định danh
nguồn, ràng buộc và nghiệm thu, hoặc chặn phần phụ thuộc khi thiếu thẩm quyền.
Mẫu tiếng Anh vẫn cần quyền đọc checkout riêng như hướng dẫn bên dưới.
Trả một bản bàn giao đúng phạm vi trong chat. Sửa source trực tiếp dùng Skills
thực thi.
Theo dõi Required / Conditional / Already-loaded riêng cho từng vai trò.
Có thể dẫn dữ kiện Author đã xem; context trong bộ nhớ của Author không phải cache
của Work. Giao Work lần đọc đầu tối thiểu, chỉ mở lại bằng chứng quyết định đã đổi
hoặc còn thiếu.
[Skill biên soạn](../skills/cirnet-work-prompt-authoring/SKILL.md) ·
[Mẫu](../templates/chat-to-work-prompt.md) · [Truy cập từ bundle](distribution.md#soạn-work-prompt)

### Kiểm tra bàn giao offline

Trước khi khảo sát, người vận hành xác định yêu cầu gốc, checkout được duyệt và
bản cấp quyền riêng. Nhận SHA-256 của bản cấp quyền qua kênh tin cậy từ người
phê duyệt. Bản ghi, digest do Author tạo, checkpoint hay chú thích mã nguồn
không cấp quyền. Work cần đọc được các đầu vào này trong cùng phạm vi quyền.
Với tác vụ nhạy cảm, phải thiết lập cơ chế giới hạn công cụ trước khi đọc.
Runner hiện tại thiếu cơ chế cưỡng chế quyền đọc có giới hạn nên chặn trước khi
mở bản nháp, nhiệm vụ gốc hay nguồn. Tác vụ hẹp hơn chỉ được dùng bản dữ liệu
đã lọc, cô lập và được phê duyệt độc lập.

Yêu cầu gốc là văn bản UTF-8. Trước khi duyệt JSON dưới đây, người vận hành
đối chiếu từng dòng đánh số với quyền và phép kiểm nghiệm thu. Thay các giá trị
mẫu bằng đúng đường dẫn gốc, toàn bộ SHA của HEAD và digest tệp viết thường.
Trong ví dụ, dòng 1 yêu cầu sửa `label.js`, dòng 2 giới hạn thay đổi và giữ các
chỉnh sửa ngoài phạm vi, dòng 3 mô tả đầu ra cần quan sát.

```json
{"originalSha256":"<request digest>","repository":"<absolute root>","revision":"<commit>",
 "readiness":"PROMPT_READY","sensitive":false,
 "fields":{"action":[1],"target":[1],"transformation":[1],"outcome":[3],
           "scope":[2],"negative":[2],"acceptance":[3]},
 "permissions":{"read":["label.js"],"write":["label.js"],"test":["node --test"],
                "build":[],"export":[],"privileged":[]},
 "actions":{"1":["write: label.js"]},"checks":{"3":["test: node --test"]},
 "sources":{"label.js":"<source digest>"}}
```

Dùng đường dẫn tệp chính xác cho quyền đọc, ghi. Liệt kê riêng từng lệnh hoặc
tác động được duyệt. Với quyền xuất, ghi rõ chủ thể, bản ghi, trường và đích.
Mảng rỗng không cấp quyền nào. Chỉ để mảng `negative` rỗng khi yêu cầu gốc
không có ca âm. Ánh xạ từng dòng thao tác đến tác động được cấp quyền và từng
dòng nghiệm thu đến phép đọc, test hoặc build được duyệt.
Hash nguồn xác định đúng byte đã khảo sát trong working tree tại HEAD đã nêu.

Mở đầu bản nháp bằng trạng thái đã duyệt, sau đó đặt `Original: <request digest>`
và `Authorization: <approval digest>` trên hai dòng riêng. Nêu rõ thao tác, đối
tượng và phép biến đổi. Giữ thứ tự gốc, dùng nguyên văn từng dòng hoặc `Task#N`
cho các dòng khác có nội dung mà Work có thể mở lại. Với ví dụ trên, thêm
`Verify Task#3: test: node --test` và `Source: label.js`. Tham chiếu bản cấp
quyền xác định giới hạn. Dòng thao tác tùy chọn như `write: label.js` còn phải khớp
với thao tác, phép kiểm hoặc quyền đọc nguồn gắn với nhiệm vụ này. Markdown
thông thường và cách diễn đạt lại có thể được rà độc lập.
Tiêu đề của Candidate chỉ áp dụng cho fixture benchmark.

```powershell
./scripts/validate-work-prompt.ps1 -OriginalTask request.txt -Authorization approval.json -AuthorizationSha256 $approvedPin -Prompt handoff.md -Repository $ownerRoot
```

Chạy validator từ checkout toolkit được duyệt trước khi giao và lúc Work tiếp nhận,
đối chiếu cùng đầu vào đã duyệt
và SHA-256 của bản nháp. Mã thoát 0 (`PASS`) áp dụng cho phần nội dung và pin
đã kiểm. Mã 1 (`FAIL`) yêu cầu sửa, 2 (`REVIEW_REQUIRED`) yêu cầu rà độc lập
với yêu cầu gốc và quyền, 3 (`BLOCKED`) dừng triển khai. Giữ Work ở trạng thái
`PROMPT_BLOCKED` trong khi còn thiếu lần rà thiết yếu. Sau khi người rà độc lập
duyệt bản nháp theo cùng đầu vào, người vận hành có thể truyền digest của bản nháp
qua `-ReviewedPromptSha256` để ghi nhận quyết định. Validator không xác thực danh tính người
duyệt, diễn giải văn xuôi tùy ý, chạy các phép kiểm dự kiến hay cưỡng chế
quyền hệ thống tệp của Work.

## Vòng đời mô hình

Giao ước Work Prompt độc lập với các lần phát hành model. Chọn model và effort
tại runtime. Dùng [kiểm tra tương thích theo rủi ro](model-compatibility.md)
khi thay đổi ảnh hưởng đến hành vi bàn giao, quyền truy cập công cụ hoặc quyền thực thi.

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
<!-- source-sha256: f5621992a6506c38b982041a47749aa6e576746edd6ea16d20c23d11593bf7ad -->
