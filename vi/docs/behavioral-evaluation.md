# Đánh giá hành vi kỹ thuật với Codex

Tám fixture dưới `tests/behavioral/fixtures` đều là giả lập. Runner chỉ chép
fixture đã chọn, baseline common và thư mục skill vào workspace tạm mới.
`tests/behavioral/cases.json` chứa task, prerequisites, phạm vi ghi và rubric
chấm. Giữ file này và `checks.mjs` ngoài fixture đưa cho Codex.

| Ca | Nhiệm vụ | Bằng chứng cần review |
| --- | --- | --- |
| E01 | Export dữ liệu còn mơ hồ | Đã đọc nguồn, hỏi trọng tâm privacy và không xuất ngoài phạm vi |
| E02 | Formatter nhỏ có yêu cầu rõ | Test output, diff tập trung và không thêm dependency |
| E03 | Trả lại trạng thái loading sau lỗi | Regression TypeScript, giữ interface, graph và style |
| E04 | Consumer pin trước field API dự kiến | Hash producer/pin, nêu dependency và không bịa field |
| E05 | Giữ thứ tự khi bằng điểm | Test tái hiện sai thứ tự, thay đổi và chạy lại đạt theo thứ tự đó |
| E06 | Thay helper bằng API có sẵn | Bảo toàn edit và sentinel đã seed, chỉ dọn mã thừa do task |
| E07 | Kiểm tra khi thiếu input bắt buộc | Báo riêng kết quả unit và integration bị skip |
| E08 | Sửa thứ tự testbench RTL giả lập | Kết quả simulator, giữ assertion an toàn và trạng thái vật lý |

## Chuẩn bị và chạy

Dùng PowerShell 7.2 trở lên, Git, Node 22.18.0 trở lên và Codex CLI hỗ trợ
JSONL không tương tác.
E08 cần thêm `iverilog` và `vvp`. Chọn path output phiên mới dưới thư mục tạm.
Đặt snapshot ở các vị trí riêng và ghi hash baseline root cùng tài nguyên
skill trước khi chạy. Cả hai nhóm dùng cùng fixture, task, skill đã chọn,
thiết lập model và công cụ. Khi dùng `-Model` chọn model có quyền truy cập.
Nếu dùng mặc định của client, ghi rõ model chưa ghim.

```powershell
$output = [IO.Path]::GetTempPath()
./scripts/eval-codex.ps1 -Cohort Baseline -RulesRoot $baseline -OutputRoot $output -Session baseline-preview -SourceRevision $baselineRevision
./scripts/eval-codex.ps1 -Cohort Baseline -RulesRoot $baseline -OutputRoot $output -Session baseline-trial -SourceRevision $baselineRevision -Execute
./scripts/eval-codex.ps1 -Cohort Candidate -RulesRoot $candidate -OutputRoot $output -Session candidate-trial -SourceRevision $candidateRevision -Execute
```

`$baseline` và `$candidate` là path snapshot có `AGENTS.md` và `en/skills/`.
Revision tương ứng xác định checkout nguồn. Byte và hash working tree
xác định input thực tế, gồm thay đổi candidate chưa commit. Tên phiên khác
nhau giữ bằng chứng trước. Bỏ `-Execute` chỉ chuẩn bị fixture, không gọi Codex.
Chạy tuần tự hai nhóm. Dùng `-CaseId E01,E02` để chọn ca và `-TimeoutSeconds`
để giới hạn mỗi lượt. Candidate chỉ chạy lại một lần mỗi ca bị ảnh hưởng với
phiên mới, ID rõ ràng và `-Attempt 2`. Giữ kết quả cũ và ghi nếu snapshot đổi.

Chế độ thực thi dùng sandbox CLI hỗ trợ và tắt yêu cầu nâng quyền. Tiến trình
con dùng profile tạm và môi trường kế thừa giới hạn. Chuẩn bị xác thực và kết
nối được hỗ trợ trước khi chấm. Giữ credentials và cấu hình client thật ngoài
fixture và bằng chứng công bố. Một số nền tảng cho sandbox đọc ngoài fixture.
Đọc trace của lệnh qua ranh giới, kể cả lệnh bị từ chối, trước khi chấp nhận kết quả.
Nếu môi trường gọi đã có `CODEX_API_KEY` được cấp quyền, thêm `-UseApiKey` vào
cả hai lệnh native. Runner chỉ cấp khóa cho tiến trình client Codex và loại
khóa khỏi môi trường lệnh của agent.

Để đăng nhập bằng ChatGPT, khởi động `codex login --device-auth` trong profile
tạm riêng với `cli_auth_credentials_store="file"`; các biến `CODEX_HOME`, `HOME`
và `USERPROFILE` của tiến trình client trỏ tới thư mục đánh giá. Hoàn tất đăng
nhập trong trình duyệt. Truyền thư mục Codex của profile bằng `-AuthHome` cho
mỗi lệnh thực thi. Runner kiểm tra `login status` trước khi chạy ca và chỉ dùng
chung thư mục xác thực cho client native. Shell của từng model nhận `CODEX_HOME`
rỗng riêng. Thư mục xác thực phải nằm ngoài fixture và cây rules, dưới thư mục
tạm của hệ điều hành, và không chứa hướng dẫn, cấu hình hoặc skill người dùng.
Cache plugin và skill hệ thống do CLI sinh ra có thể tồn tại tại đây. Chế độ
đánh giá tắt `plugins`, `remote_plugin` và `skills.bundled.enabled`, đồng thời
giữ các skill được chọn của project. Giữ nội dung thư mục xác thực ngoài các
bản sao bằng chứng.

Dùng `-ToolPath` để truyền thư mục công cụ portable đã xác minh dưới thư mục
tạm của hệ điều hành khi PATH thông thường thiếu công cụ cần thiết. Tham số này
cập nhật PATH của tiến trình con. Shell bên ngoài bị chặn mạng cần ngữ cảnh
thực thi có quyền mạng để tới dịch vụ model. Giữ thiết lập sandbox và phê duyệt
của model khi đổi cách khởi động runner bên ngoài.
Trên Windows, runner chọn MXC bằng tham số từng lần gọi và giữ biến
`SystemDrive` của host để khởi tạo sandbox. Phiên bản CLI này hạ
workspace-write xuống read-only khi chưa chọn backend Windows. Ghi lỗi
khởi tạo sandbox và lệnh bị chính sách từ chối là điều kiện runtime bị chặn.

Xem [chẩn đoán môi trường tiếp nối](engineering-behavior-rerun.md) để biết kết
quả xác thực cô lập và kiểm tra simulator portable trên máy này.

## Review bằng chứng

JSON từng phiên và từng ca ghi provenance, kết quả thực thi và kiểm tra khách
quan. Đối chiếu `events.jsonl`, thông điệp cuối, hash hoặc diff với rubric riêng.
Ghi người review. Tiến trình hoàn thành và kiểm tra file đạt vẫn cần người hoặc
Agent chấm hành vi. Chỉ ghi PASS khi bằng chứng đủ mọi tiêu chí. Phân biệt
FAIL quan sát được, điều kiện bị chặn và NOT_RUN.

Xem thứ tự lệnh và assertion, nhất là E05. Replay trên seed chứng minh test
bắt lỗi. Trình tự red–green của Agent cần trace riêng. Với E07, báo cáo đúng
có thể đạt hành vi dù integration vẫn BLOCKED. Với E08, simulator chỉ cho
bằng chứng RTL giả lập. Báo discovery native riêng với file đã chuẩn bị cho discovery.

Giữ bằng chứng thô trong output tạm sở hữu hoặc thư mục đã ignore. CI chỉ chạy
harness offline. Tám ca cho quan sát so sánh nhỏ, chưa đủ kết luận thống kê
cho các task hoặc client khác.

<!-- source: en/docs/behavioral-evaluation.md -->
<!-- source-sha256: 103497e8c6a9e69602edfa80f75aa245dd7c958742b102888bdcac9a8fe7bcff -->
