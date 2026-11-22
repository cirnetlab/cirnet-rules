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
mẫu tiếng Anh từ bản cài đã xác minh hoặc checkout đã được phép. Chọn Work Skills áp dụng từ metadata,
chỉ đọc thân các Skill đó, kiến trúc, chỉ dẫn owner và bằng chứng liên quan.
Chọn LIGHT, STANDARD hoặc DEEP theo độ phức tạp, rủi ro, phụ thuộc, bất định
và quyền đọc source. Xét đủ 12 trường ngữ nghĩa, gom thông tin áp dụng vào
các heading nhiệm vụ cần.

Tham chiếu nguồn Work đọc được và ghi trực tiếp ràng buộc riêng của nhiệm vụ.
Nếu Work thiếu quyền đọc source, đưa dữ kiện đáng tin cậy thiết yếu, định danh
nguồn, ràng buộc và nghiệm thu, hoặc chặn phần phụ thuộc khi thiếu thẩm quyền.
Trả một bản bàn giao đúng phạm vi trong chat. Sửa source trực tiếp dùng Skills
thực thi.
Theo dõi Required / Conditional / Already-loaded riêng cho từng vai trò.
Có thể dẫn dữ kiện Author đã xem; context trong bộ nhớ của Author không phải cache
của Work. Giao Work lần đọc đầu tối thiểu, chỉ mở lại bằng chứng quyết định đã đổi
hoặc còn thiếu.
[Skill biên soạn](../skills/cirnet-work-prompt-authoring/SKILL.md) ·
[Mẫu](../templates/chat-to-work-prompt.md) · [Truy cập từ bundle](distribution.md#soạn-work-prompt)

### Kiểm tra bàn giao offline

Bắt đầu từ yêu cầu và phạm vi người dùng đã cấp cho repo chủ sở hữu.
Xác định đúng checkout và lưu các byte UTF-8 của yêu cầu gốc. Với tác vụ
thông thường không nhạy cảm, Author chuẩn bị bản bàn giao cục bộ.

```powershell
./scripts/validate-work-prompt.ps1 -Prepare -Autonomous -OriginalTask request.txt -Repository $ownerRoot -AuthorizedScope Edit -Risk LIGHT
```

Chọn `ReadOnly` hoặc `Edit` theo quyền được cấp. Dùng `-ReadPath` hoặc
`-WritePath` khi quyền giới hạn ở các tệp cụ thể. Dùng `-SourcePath` để
chọn nguồn bằng chứng được phép đọc.
Bước chuẩn bị định vị tệp được nêu trực tiếp, ghim các byte đã chọn và giao
chi tiết hoặc phép kiểm chưa biết cho Work kiểm tra trước. Author chọn mức
rủi ro và rà ý nghĩa, ràng buộc của yêu cầu. Phép kiểm offline phát hiện
xung đột tường minh. Nội dung tự do vẫn cần rà ý nghĩa.

Với `-Autonomous`, mã thoát 0 và `authoringStatus = PASS` nghĩa là
`PROMPT_READY`. Trả `prompt` có thể sao chép và điều kiện còn chờ. Khi
chưa có phê duyệt độc lập, `authorizationStatus = PENDING` và
`handoffStatus = NOT_RUN`. Work có thể thực hiện bước kiểm tra trước được
phép và chuẩn bị danh sách tác động, phép kiểm để duyệt. Lưu yêu cầu, pin
và chẩn đoán trong bằng chứng cục bộ. Nếu đã có phê duyệt độc lập, dùng
`-Authorization` và `-AuthorizationSha256` thay cho các cờ phạm vi bản nháp.
Validator kiểm tra bản ghi, tạo lại bản bàn giao chuẩn và trả
`authorizationStatus = RECORD_CHECKED`. Thực thi và nghiệm thu vẫn
`NOT_RUN`. Tạo lại một lần sau khi sửa đầu vào kỹ thuật. Thay đổi ý nghĩa
hoặc chỉ dẫn đã duyệt cần được rà độc lập.

Để thực thi, nhận digest cấp quyền qua kênh tin cậy từ người phê duyệt.
Bản ghi, digest do Author tạo, checkpoint hay chú thích mã nguồn không cấp
quyền. Work cần đọc được đầu vào đã duyệt theo cùng phạm vi quyền. Thiết lập
kiểm soát phòng ngừa đối với tác vụ nhạy cảm.
Runner hiện tại thiếu cơ chế cưỡng chế quyền đọc có giới hạn nên chặn trước khi
mở bản nháp, nhiệm vụ gốc hay nguồn. Tác vụ hẹp hơn chỉ được dùng bản dữ liệu
đã lọc, cô lập và được phê duyệt độc lập.

Để chuẩn bị phiếu cấp quyền theo quy trình cũ cho đầu vào không nhạy cảm, dùng
`validate-work-prompt.ps1 -Prepare -OriginalTask request.txt -Repository $ownerRoot -SourcePath label.js`
để lấy các dòng được đánh số và `proposal` chứa pin của yêu cầu, HEAD và nguồn.
Mã thoát 2 cho biết đang chờ duyệt. Giữ đầu ra cục bộ. Rà từng ánh xạ, điền
quyền và phép kiểm rõ ràng, rồi lưu đối tượng `proposal` đã được duyệt độc lập
làm tệp phê duyệt. Nhận riêng SHA-256 từ người phê duyệt. Bước chuẩn bị để
trống các quyền.
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

Dùng `create:`, `modify:` hoặc `delete:` trong ánh xạ thao tác khi cần chỉ
rõ tác động lên tệp. Di chuyển cần quyền xóa nguồn và tạo đích. Ánh xạ
`write:` hiện có cho phép tạo mới hoặc sửa tệp tại đúng đường dẫn. Rà tác
động phụ của lệnh và dùng đường dẫn tạm đã được duyệt. Đặt
`"preventiveScope": true` trong bản phê duyệt khi tác vụ cần cơ chế giới hạn
tác động từ trước. Runner offline trả `BLOCKED` cho đầu vào này. Thiết lập
cơ chế phù hợp trước khi tiếp tục.

Mở đầu bản nháp bằng trạng thái đã duyệt, rồi đặt `Original: <request digest>`
và `Authorization: <approval digest>` trên hai dòng riêng. Với bàn giao có
thể thực hiện, giữ nguyên văn các dòng đã ánh xạ về mục tiêu, kết quả mong đợi,
phạm vi, ca âm và nghiệm thu theo thứ tự gốc. Chỉ dùng `Task#N` cho dòng
bổ sung mà Work mở lại được và người rà độc lập đã kiểm tra. Ghi rõ
`Owner: <approved repository root>`, từng tác động đã ánh xạ, phép kiểm và
nguồn trên các dòng hiển thị. Với ví dụ trên, thêm `write: label.js`,
`Verify Task#3: test: node --test` và `Source: label.js`. Dùng
`Write scope: []` khi không có tác động ghi tệp. Thêm dòng báo cáo chung:

`Report: deliverables, changed paths, actual verification, failures, and unresolved limitations.`

Gom nhóm các dòng theo LIGHT, STANDARD hoặc DEEP. Rà độc lập phần diễn giải
bổ sung và ý nghĩa của yêu cầu cùng quyền được cấp. Tiêu đề của Candidate
chỉ áp dụng cho fixture benchmark.

```powershell
./scripts/validate-work-prompt.ps1 -OriginalTask request.txt -Authorization approval.json -AuthorizationSha256 $approvedPin -Prompt handoff.md -Repository $ownerRoot
```

Chạy validator từ checkout toolkit được duyệt trước khi giao và lúc Work tiếp nhận,
đối chiếu cùng đầu vào đã duyệt
và SHA-256 của bản nháp. Mã thoát 0 (`PASS`) áp dụng cho phần nội dung và pin
đã kiểm. Mã 1 (`FAIL`) yêu cầu sửa, 2 (`REVIEW_REQUIRED`) yêu cầu rà độc lập
với yêu cầu gốc và quyền, 3 (`BLOCKED`) dừng triển khai. Giữ thực thi ở
trạng thái chặn trong khi còn chờ rà độc lập. Sau khi người rà độc lập
duyệt bản nháp theo cùng đầu vào, người vận hành có thể truyền digest của bản nháp
qua `-ReviewedPromptSha256` để ghi nhận quyết định. Digest xác định đúng byte
đã rà. Vẫn phải bổ sung nội dung còn thiếu hoặc sửa lỗi toàn vẹn và phạm vi.
Người vận hành đánh giá tương đương ngữ nghĩa, cấp quyền công cụ và rà bằng
chứng thực tế của Work.

Dùng cùng đầu vào đã duyệt ở từng giai đoạn:

1. Xác thực bàn giao và yêu cầu `handoffStatus = PASS`.
2. Với tác động lên tệp hoặc lệnh có thể ghi như test/build, duyệt kiểm kê và
   băm bằng `"scopeAudit": true`. Chạy `-CaptureScope` và yêu cầu
   `baselineStatus = PASS`. Lưu nguyên JSON ngoài worktree của owner và giữ
   riêng SHA-256 của tệp.
3. Ngay trước khi chạy Work, dùng `-ExecutionPreflight -ScopeSnapshot $snapshot -ScopeSnapshotSha256 $snapshotPin`
   và yêu cầu `executionStatus = PASS`. Tác vụ chỉ đọc dùng
   `-ExecutionPreflight`. Rà tác động thật của lệnh và liệt kê mọi đường dẫn
   đầu ra được duyệt trong ánh xạ thao tác. Chụp lại baseline cũ hoặc hết hiệu
   lực sau khi đối chiếu được phép.
4. Sau khi Work chạy, dùng `-ScopeSnapshot $snapshot -ScopeSnapshotSha256 $snapshotPin`
   để đối chiếu và yêu cầu `scopeAuditStatus = PASS` khi phải audit. Rà độc lập
   bằng chứng kiểm thử thực tế và sản phẩm bàn giao.

CLI giữ `acceptanceStatus` ở `NOT_RUN`. Mỗi worktree của owner cần phê duyệt
và bằng chứng riêng. Audit so sánh HEAD, nhánh, index, cấu hình Git cục bộ
và cấu hình worktree, các ref của repo, tệp exclude và attributes cục bộ,
byte tệp được và chưa được theo dõi, cùng tối đa 1.024 tệp bị bỏ qua với tổng
dung lượng 32 MiB. Chụp lại baseline được tạo trước khi có bằng chứng metadata Git.
Đầu vào không có hoặc không được hỗ trợ sẽ chặn giai đoạn tương ứng. Audit
ghi nhận chênh lệch cuối cùng. Bảo vệ vùng quản trị Git bằng quyền ở runtime.
Git hook, thiết lập toàn cục và repo khác cần cơ chế riêng. Agent bên ngoài
phải tuân thủ kết quả preflight và dùng các giới hạn
công cụ được cấp quyền riêng. Validator không điều khiển việc thực thi đó.

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
<!-- source-sha256: 88641f040a91887df07be43fa19af56696da16513f09986ea07fa8a2aac8abd3 -->
