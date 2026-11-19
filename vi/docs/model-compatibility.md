# Kiểm tra tương thích mô hình theo rủi ro

Skill biên soạn và một cặp mẫu EN/VI của CIRNET định nghĩa một giao ước hành vi
qua các lần phát hành model. Chọn model và mức reasoning tại runtime. Chọn
phép kiểm theo hành vi và ranh giới bị ảnh hưởng bởi thay đổi.

## Chọn phép kiểm theo rủi ro

| Thay đổi | Kiểm chứng bắt buộc | Phạm vi bằng chứng |
| --- | --- | --- |
| Chỉ đổi model hoặc effort, giữ công cụ và quyền | E01 và E06 Candidate-only | Các ca smoke đã thực hiện |
| API công cụ, sandbox, prompt hoặc ánh xạ công cụ | Kiểm tra adapter hoặc CLI và các ca native liên quan | Hành vi công cụ đã thay đổi |
| Hành vi của Skill biên soạn hoặc template | Regression tập trung và kiểm tra kiến trúc, tài liệu | Hành vi bị ảnh hưởng |
| Kiến trúc, contract, ranh giới an toàn hoặc nhiều Skill | Kiểm tra của bên sở hữu, test contract và đường lỗi, cùng các ca tập trung được cấp quyền cho rủi ro còn lại | Các ranh giới đã kiểm chứng |
| Thiếu điều kiện, fixture đối tác hoặc quota | Ghi `BLOCKED` hoặc `NOT_RUN` | Phạm vi kiểm chứng còn thiếu |

Chọn thêm ca qua [quy trình đánh giá hành vi](behavioral-evaluation.md) hiện có
khi ranh giới bị ảnh hưởng cần kiểm chứng. Ví dụ E04 cho contract, E07 cho giới
hạn tích hợp và E08 cho an toàn RTL. Kiểm chứng vật lý cần quyền của bên sở hữu
và bằng chứng quan sát được. Dùng cấu hình runtime đã được kiểm chứng gần nhất
làm phương án dự phòng.

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
dùng Skill thực thi hiện tại.

## Ghi nhận mức tương thích

Báo mức hoàn thành tác vụ, tuân thủ kiến trúc, độ bao phủ kiểm chứng và quyền
riêng tư cho các ca đã chạy. Ghi snapshot, model và effort yêu cầu, cùng thiết
lập thực tế khi runtime có báo. Kết quả shim bao phủ bộ khung chạy kiểm tra.
Tương thích Native cần bằng chứng chạy thật và rà độc lập. Số liệu sử dụng và
thời gian trên host của từng vai trò có thể hỗ trợ chẩn đoán lỗi.

Giữ bằng chứng A/B/B2 lịch sử với cách chấm, model, effort, revision và hash
fixture gốc trong [kho lưu trữ benchmark](../../reports/work-prompt-benchmark-history.md).

<!-- source: en/docs/model-compatibility.md -->
<!-- source-sha256: 4bfa578f028960a1e8aa69128a55a4d08ef2338935f13f6b0a65f72f78eb8e09 -->
