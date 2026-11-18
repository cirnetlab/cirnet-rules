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
Runner đối chiếu SHA-256 đã ghi của Author với file bàn giao và byte Work
thực nhận. Runner ghi `compatibility-summary.json` sau khi chạy. Mã thoát 1
khi một ca lỗi, thiết lập session lỗi hoặc kiểm tra toàn vẹn lỗi. Mã thoát 2
khi thiếu bằng chứng và không có ca lỗi. Mã thoát 0 cho preview, mock đạt bộ
chấm objective hoặc cả hai ca native đạt `OBJECTIVE_PASS_REVIEW_REQUIRED`.
Preview gọi 0 vai trò. Shim `.ps1` offline không dùng thông tin xác thực và
được đánh dấu `TEST_SHIM`.

Rà độc lập trace thực tế, phạm vi được cấp quyền, diff source và các kiểm tra
objective trước khi kết luận tương thích. Giữ trace gốc trong session tạm
thuộc phạm vi quản lý và chỉ chia sẻ bằng chứng đã lọc. Phát hiện canary giả
lập E01 là lỗi. Bộ rà quyền riêng tư hiện có phát hiện việc lộ dữ liệu và giới
hạn cách ly quyền đọc filesystem vẫn áp dụng.

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
<!-- source-sha256: 50819927a106d144661c26ab7f111f7cf03c637602e1926c21360df35f379af6 -->
