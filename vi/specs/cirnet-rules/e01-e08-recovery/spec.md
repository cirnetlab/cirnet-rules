# Khôi phục E01/E08

Khởi đầu: `rules-common`, `b8981de1793995ab11f372cfe912771b0147c7a7`, worktree sạch,
2026-11-15. Giữ nguyên E01 FAIL và E08 BLOCKED lịch sử.

Làm rõ quyền export trong baseline, quy tắc bảo mật, chính sách quyền và skill
triển khai hiện có. Xác định phạm vi actor, bản ghi và trường dữ liệu từ ngữ cảnh
đáng tin trước khi hỏi các quyết định còn thiếu. Quyền sửa mã và public projection
không xác lập quyền truy cập dữ liệu. Giữ 12 định danh skill, baseline ngắn,
ngữ cảnh độc lập và các bản EN/VI đã rà soát.

Chẩn đoán Icarus bằng đúng toolchain/profile của runner trong Windows MXC với
workspace-write và approval never. Yêu cầu biên dịch mới thành công trước khi
mô phỏng. Crash vẫn là lỗi dù sinh được image. Giữ nguyên sequence seed E08,
DUT, assertions và thời hạn; chỉ bản sao chẩn đoán dùng một lần được chứa
bản sửa sequence đã biết.

Đóng băng candidate và dùng Candidate / Attempt 2, chỉ định rõ E01 hoặc E08.
Chạy E08 native đòi hỏi preflight MXC: HDL tối thiểu đạt, seed gốc biên dịch được
và thất bại đúng assertion khởi động, bản sửa chẩn đoán biên dịch được và đạt
cả bốn guard. Đối chiếu lệnh, output và thay đổi file thực với rubric riêng giữ
nguyên; xác minh vật lý là NOT_RUN.

Không dispatch hoặc replay E02–E07, đổi đầu vào của chúng, chấm lại kết quả
lịch sử, sửa cấu hình Agent thật/repo sản phẩm, cài tool global hoặc commit/publish.
Giữ bằng chứng cũ và báo cáo kết quả follow-up riêng. Nếu MXC vẫn lỗi, ghi
reproducer và ranh giới đã chứng minh với trạng thái BLOCKED.

<!-- source: en/specs/cirnet-rules/e01-e08-recovery/spec.md -->
<!-- source-sha256: a6c3778ac6b9c852aef4582271832c0f71c1a797d44122ccfd8d45b0405ffbb9 -->
