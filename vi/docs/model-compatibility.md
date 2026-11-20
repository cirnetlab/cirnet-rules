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

## Chọn một phiên Candidate

Dùng [benchmark một phiên](behavioral-evaluation.md) với fixture E01 và E06
gốc khi thay đổi runtime cần kiểm tra tập trung. E01 kiểm tra làm rõ quyền xuất
dữ liệu trong chế độ chỉ đọc và E06 kiểm tra refactor nhãn đúng phạm vi.
Preview đóng băng nguồn hiện tại và lập kế hoạch tối đa bốn lượt vai trò.
Chạy Native cần quyền riêng và giới hạn lượt gọi chính xác.

Chọn E05 để kiểm tra thứ tự hồi quy, `python-retry-count-refactor` để kiểm
tra hành vi lỗi và bảo toàn dữ liệu người dùng, hoặc `rust-timeout-bound`
để kiểm chứng unit Rust. Danh mục vẫn nhận `P01` làm bí danh lịch sử của ca
Python. Có thể chọn nhiều ca trong một phiên khi dùng cùng cấu hình runtime.

Yêu cầu bằng chứng E05 thất bại trước rồi đạt sau và bộ kiểm của owner trước
và sau khi sửa Python. Xác minh E06 giữ các sửa đổi người dùng, tệp chưa
theo dõi và bị bỏ qua, cùng helper dùng chung. Rà byte handoff, kết quả chấm
objective và test thực tế trước khi nghiệm thu hành vi áp dụng.

## Ghi nhận mức tương thích

Báo mức hoàn thành tác vụ, tuân thủ kiến trúc, độ bao phủ kiểm chứng và quyền
riêng tư cho các ca đã chạy. Ghi snapshot, model và effort yêu cầu, cùng thiết
lập thực tế khi runtime có báo. Kết quả shim bao phủ bộ khung chạy kiểm tra.
Tương thích Native cần bằng chứng chạy thật và rà độc lập. Rà lượt gọi công
cụ, thời gian trên host và lỗi của từng vai trò. Đo đầu vào lần đầu theo
[quy trình đánh giá](behavioral-evaluation.md).

Giữ bằng chứng A/B/B2 lịch sử với cách chấm, model, effort, revision và hash
fixture gốc trong [kho lưu trữ benchmark](../../reports/work-prompt-benchmark-history.md).

<!-- source: en/docs/model-compatibility.md -->
<!-- source-sha256: 60cf4fe9e8f2752fa0067f492090bb86fb98a63503eb593a02298c3434e476cd -->
