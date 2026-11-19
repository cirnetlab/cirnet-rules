# Tương thích mô hình với Adaptive Stable

Skill biên soạn và một cặp mẫu EN/VI của CIRNET định nghĩa một giao ước hành vi
qua các lần phát hành model. Chọn model và mức reasoning tại runtime. Giữ bằng
chứng A/B/B2 lịch sử kèm model, effort, revision và hash fixture gốc.

## Chọn phép kiểm theo rủi ro

| Thay đổi | Kiểm chứng bắt buộc | Phạm vi bằng chứng |
| --- | --- | --- |
| Chỉ đổi model hoặc effort, giữ công cụ và quyền | E01 và E06 Candidate-only | Các ca smoke đã thực hiện |
| API công cụ, sandbox, prompt hoặc ánh xạ công cụ | Kiểm tra adapter hoặc CLI và các ca native liên quan | Hành vi công cụ đã thay đổi |
| Hành vi của Skill biên soạn hoặc template | Regression tập trung và kiểm tra kiến trúc, tài liệu | Hành vi bị ảnh hưởng |
| Kiến trúc, contract, ranh giới an toàn hoặc nhiều Skill | Kiểm tra của bên sở hữu và so sánh A/B đã khóa với quy mô phù hợp, khi được cấp quyền | Điều kiện nghiệm thu đã công bố |
| Thiếu điều kiện, fixture đối tác hoặc quota | Ghi `BLOCKED` hoặc `NOT_RUN` | Phạm vi kiểm chứng còn thiếu |

Chọn thêm ca qua [quy trình đánh giá hành vi](behavioral-evaluation.md) hiện có
khi ranh giới bị ảnh hưởng cần kiểm chứng. Ví dụ E04 cho contract, E07 cho giới
hạn tích hợp và E08 cho an toàn RTL. Kiểm chứng vật lý cần quyền của bên sở hữu
và bằng chứng quan sát được. Chỉ phát hành model mới thì giữ source, template
và kết quả A/B lịch sử ở revision hiện tại. Dùng cấu hình runtime đã được kiểm
chứng gần nhất làm phương án dự phòng.

## Chạy smoke Candidate-only

Smoke dùng `scripts/eval-codex.ps1` với các ca giả lập E01 và E06 gốc. Preview
tạo session mới dưới OS temp và ghi SHA-256 theo byte trong checkout của bộ
công cụ và đầu vào bộ chấm được Git theo dõi, hướng dẫn này và các file runner.
Dùng tên session mới cho mỗi lần chạy.

```powershell
$root = [IO.Path]::GetTempPath()
$authHome = '<thư mục xác thực dành riêng đã được phép dưới OS temp>'
./scripts/eval-codex-model-smoke.ps1 -Session smoke-preview -Model '<model-id-được-cli-hỗ-trợ>' -Effort high -OutputRoot $root
# Chỉ chạy khi có quyền riêng và phương thức xác thực đã được phép.
./scripts/eval-codex-model-smoke.ps1 -Session smoke-native -Model '<model-id-được-cli-hỗ-trợ>' -Effort high -OutputRoot $root -AuthHome $authHome -Execute -MaxNativeCalls 4
```

Mỗi session native có thể gọi tối đa bốn vai trò, một Author và một Work cho
mỗi ca. Nếu Author lỗi hoặc bản bàn giao không hợp lệ, dừng ca trước Work.
Điểm kiểm tra Candidate-only xác minh trạng thái, nguyên văn nhiệm vụ gốc,
phạm vi ghi của fixture và các mục có nội dung để rà. E01 dùng `PROMPT_BLOCKED`
và chỉ cho Work làm rõ thẩm quyền xuất dữ liệu trong sandbox chỉ đọc. E06 dùng
`PROMPT_READY` và phạm vi ghi gốc của ca. Work xác minh SHA-256 của Author và
các trường bắt buộc trên chính byte đầu vào. Runner ghi
`compatibility-summary.json` sau khi chạy. Mã thoát 1
khi một ca lỗi, thiết lập session lỗi hoặc kiểm tra toàn vẹn lỗi. Mã thoát 2
khi thiếu bằng chứng và không có ca lỗi. Mã thoát 0 cho preview, mock đạt bộ
chấm objective hoặc cả hai ca native đạt `OBJECTIVE_PASS_REVIEW_REQUIRED`.
Preview gọi 0 vai trò. Shim `.ps1` offline không dùng thông tin xác thực và
được đánh dấu `TEST_SHIM`.

Rà độc lập trace thực tế, phạm vi được cấp quyền, diff source và các kiểm tra
objective trước khi kết luận tương thích. So nội dung mục yêu cầu và mục kiểm
chứng với nhiệm vụ gốc và rubric, gồm ca âm tính và quyền riêng tư. Sandbox
ghi của E06 bao phủ fixture và bộ chấm objective phát hiện thay đổi ngoài
file được phép. Giữ trace gốc trong session tạm
thuộc phạm vi quản lý và chỉ chia sẻ bằng chứng đã lọc. Phát hiện canary giả
lập E01 là lỗi. Bộ rà quyền riêng tư hiện có phát hiện việc lộ dữ liệu và giới
hạn cách ly quyền đọc filesystem vẫn áp dụng.

## Kiểm tra khả năng tổng quát hóa riêng

Dùng `-CaseId E05` cho ca gỡ lỗi và hồi quy hiện có hoặc `-CaseId P01`
cho ca refactor Python độc lập. Mỗi preview đóng băng bộ đánh giá được
chọn, Skill hiện tại của Author và Work, cùng runner. Lượt chạy được cấp
quyền dùng `-Execute -MaxNativeCalls 2` và áp dụng cùng kiểm tra handoff như
smoke mặc định. P01 cần Python được duyệt và truy cập được từ sandbox.
Khi cần, dùng `-ToolPath` với trình thông dịch portable đã xác minh trong
thư mục tạm của hệ điều hành.

Rà mức prompt đã chọn và phép kiểm thực tế. Với E05, regression mới phải
thất bại vì lỗi thứ tự trước khi sửa source và đạt sau sửa. Với P01, lệnh
unittest do owner công bố phải đạt trên fixture thực tế trước và sau sửa,
đồng thời giữ hợp đồng lỗi và dữ liệu người dùng. Các lượt Candidate-only
dùng Skill thực thi hiện tại. Lane A/B legacy dùng bộ Skill thực thi chung
đã đóng băng cho cả hai cohort.
Khi dùng Work rules hiện tại, E06 Work ở cả hai cohort A/B áp dụng cách đọc
thay thế có giới hạn và hướng dẫn lọc tóm tắt test như Candidate-only.

## Tiếp tục kiểm chứng Adaptive Stable

Chạy bộ điều phối từ checkout của rules. Giữ cùng ID phiên và thư mục đầu ra
để tiếp tục với ngân sách tích lũy. Mặc định đầu ra nằm trong thư mục tạm của
hệ điều hành.

```powershell
./scripts/eval-adaptive-stable.ps1 -Session adaptive-stable-acceptance
# Chỉ dùng lệnh này khi owner đã duyệt giới hạn ban đầu 124 lượt vai trò.
./scripts/eval-adaptive-stable.ps1 -Session adaptive-stable-acceptance -Execute `
  -ApprovedRoleCalls 124 -AuthHome $authHome -PythonToolPath $pythonTool -RtlToolPath $rtlTool
```

Lệnh đầu chạy các phép kiểm offline và chuẩn bị preview cho bước tiếp theo.
Các bước native dùng `gpt-6.2-sol`, `xhigh` và 300 giây mỗi vai trò. Lịch ban
đầu gồm E01/E06, lần lặp E01/E06 độc lập, E05, P01, A/B pilot bốn ca và A/B
đầy đủ với ba lần lặp. Các bước lần lượt dành trước 4, 4, 2, 2, 16 và 96 lượt
vai trò. Dùng hồ sơ đăng nhập dành riêng cho đánh giá. Có thể bỏ đường dẫn
công cụ nếu Python và cả hai chương trình Icarus đã có trên `PATH`.

Xem `checkpoint.json` trong thư mục phiên để biết hash nguồn, hash preview,
bước kế tiếp và số lượt đã tính. Khi gặp `REVIEW_REQUIRED`, rà byte handoff,
trace các vai trò, thay đổi nguồn và kết quả kiểm chứng thực tế. Lưu review
Candidate thành `candidate-review.json` theo `candidate-review-template.json`
của bước đó. Ghi lý do cho từng tiêu chí và trích dẫn đường dẫn tương đối tới
bằng chứng đã niêm phong trong phiên. Với A/B, điền `review-template.json`
của runner và lưu thành `ab-review.json` trong phiên tương ứng. Bộ điều phối
chạy lại reporter hiện có và đối chiếu bằng chứng đã lập chỉ mục.

Khi tác vụ lỗi, chẩn đoán ca chạy và kiểm chứng hồi quy tập trung trước khi
tiếp tục. Thay đổi đầu vào nguồn sẽ bắt đầu kiểm chứng trên snapshot mới.
Mỗi bước chỉ dùng lại bằng chứng khi đầu vào đóng băng và dấu vân tay runtime
vẫn khớp.
Dùng `-ResumeBlocked` sau khi đã khắc phục điều kiện môi trường tạm thời
hoặc xác nhận lượt bị gián đoạn đã dừng. A/B continuation giữ lại các ca đã
hoàn tất và lập chỉ mục.
Các lượt chưa xác định vẫn giữ mức dành trước, và các lượt trên snapshot cũ
vẫn được tính. Xin thêm quyền từ owner nếu bước kế vượt giới hạn đã duyệt.

Pilot kiểm bốn cặp có thể đối chiếu và đã được rà độc lập. Báo cáo đầy đủ
phải đạt G0–G6 và có kết quả đánh giá hiệu quả G7. Bộ điều phối chỉ xuất báo
cáo cuối khi tất cả bước bắt buộc đạt. Owner phê duyệt phát hành riêng.

## Đánh giá điều kiện phát hành

Dùng **Adaptive Stable** cho dòng phát hành duy nhất khi đạt các điều kiện áp
dụng về Task Success, Architecture Compliance, Verification Coverage và Privacy.
Revision chưa được nghiệm thu là **Adaptive Candidate**. Smoke tập trung chỉ
bao phủ các ca đã thực hiện. Nghiệm thu A/B đầy đủ, kể cả đợt native 96 lượt
vai trò, cần quyết định và cấp quyền riêng.

Báo `modelRequested`, `effortRequested`, token input, output và cached của từng
vai trò, số lượt gọi công cụ và thời gian trên host khi quan sát được. Ghi
`NOT_AVAILABLE` khi thiếu usage và giữ trạng thái chưa xác minh nếu thiếu thiết
lập hiệu lực. Runner đánh dấu usage native là `ACTUAL` và usage mock là
`TEST_SHIM`. Usage đo được có thể hỗ trợ chẩn đoán kể cả khi một đợt chạy lỗi.
Kết luận về chi phí và chất lượng cần bằng chứng riêng.

<!-- source: en/docs/model-compatibility.md -->
<!-- source-sha256: 60cd5d88183dfd8fffc86e42f0891525309db21057ba6a6aaac907593e219b0b -->
