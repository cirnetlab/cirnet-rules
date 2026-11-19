# Tích hợp source Adaptive Stable

Skill biên soạn và mẫu Work Prompt EN/VI định nghĩa một giao ước. Bản bàn giao
giữ mục tiêu, thẩm quyền, quyền riêng tư và yêu cầu kiểm chứng của nhiệm vụ.
Model và effort là thiết lập runtime đánh giá. [Hướng dẫn tương thích](../../../docs/model-compatibility.md)
chọn phép kiểm theo thay đổi về hành vi, khả năng hoặc rủi ro.

Smoke Candidate-only dùng `scripts/eval-codex.ps1` với hai ca giả lập E01 và
E06 gốc. Preview ghi hash của đầu vào đã khóa. Đợt chạy được cấp quyền riêng
có thể gọi tối đa bốn vai trò. Nếu Author lỗi hoặc bản bàn giao không hợp lệ,
dừng ca trước Work. Đối chiếu byte bàn giao với đầu vào Work và dùng bộ chấm
objective, bộ rà quyền riêng tư hiện có. Giữ bằng chứng vai trò trong session
tạm thuộc phạm vi quản lý. Báo số liệu quan sát được kèm nguồn, ghi
`NOT_AVAILABLE` cho số liệu thiếu.

Nghiệm thu offline gồm preview, mock Author và Work, lỗi quyền riêng tư và lỗi
objective, đủ 13 Skill, cặp EN/VI và source hash. Tương thích native cần bằng
chứng chạy thật và rà độc lập. Các điều kiện phát hành áp dụng bao gồm Task
Success, Architecture Compliance, Verification Coverage và Privacy. Giữ nguyên
fixtures, quy tắc chấm điểm, snapshot và kết quả lịch sử.

Khi khôi phục, Author giữ thao tác, đích, phép đổi và phạm vi đã nêu trước khi
khảo sát source. Sau khi đọc thất bại, chỉ dùng cách khác đã xác minh độc lập
trong cùng giới hạn truy cập và output nếu bằng chứng thiết yếu cho việc bàn
giao an toàn. Nếu không, chuyển chi tiết chưa xem sang preflight có phạm vi
của Work khi đã xác lập mục tiêu, owner và phạm vi an toàn. Các kiểm soát đọc
source và sandbox hiện có áp dụng cho cả hai vai trò.

Với E01/E06 Candidate-only, kiểm tra bàn giao Author theo ca đã đóng băng
trước khi gọi Work. Ba dòng đầu ghi trạng thái, nguyên văn nhiệm vụ gốc và
chính xác phạm vi ghi của fixture. Yêu cầu mục yêu cầu và mục kiểm chứng có
nội dung để rà độc lập. E01 yêu cầu `PROMPT_BLOCKED` và phạm vi rỗng. Work
có thể làm rõ thẩm quyền trong sandbox chỉ đọc. E06 yêu cầu `PROMPT_READY`
và danh sách file gốc được ghi. Bàn giao không hợp lệ kết thúc ca trước khi
gọi Work. Work kiểm SHA-256 dự kiến của Author và phần đầu trên chính byte nhận được.

Điểm kiểm tra xác minh cấu trúc và đầu vào đáng tin cậy của ca. Rà nội dung
chức năng, ca âm tính, quyền riêng tư và kiểm chứng theo trace và diff thực
tế. Sandbox E06 vẫn bao phủ fixture. Bộ chấm objective và kiểm tra bằng chứng
Git phát hiện vi phạm phạm vi. Điểm kiểm tra dùng runner hiện có. Các vai trò
lịch sử chạy theo quy trình và bằng chứng đã pin của chúng.

## Phạm vi bảo trì hiện tại

Luồng hằng ngày dùng baseline chung, kiến trúc và contract của repo sở hữu,
Work Prompt có phạm vi và kiểm chứng theo rủi ro. Các phép kiểm mặc định của
bộ công cụ bao phủ handoff và các hồi quy thiết yếu ở chế độ offline.
Thiết lập client hiện có và thực thi fixture cô lập thuộc phạm vi bảo trì.
Nhánh Governance phục vụ tra cứu lịch sử.

Bộ điều phối checkpoint và so sánh A/B quy mô lớn được giữ làm công cụ chạy
thủ công. Chạy Native cần quyền riêng. Các đợt rà nghiệm thu mới đánh giá mức độ hoàn
thành tác vụ, tuân thủ kiến trúc, độ bao phủ kiểm chứng và quyền riêng tư.
Số liệu sử dụng và thời gian đo được của từng vai trò phục vụ chẩn đoán.
Các phép so sánh lịch sử giữ nguyên đầu vào, báo cáo, cách chấm và revision nguồn.

<!-- source: en/specs/cirnet-rules/adaptive-stable/spec.md -->
<!-- source-sha256: 03fc8874ba1542c2518286c1c34c6110652aca60f25cc9719e309c2365067743 -->
