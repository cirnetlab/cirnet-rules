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

<!-- source: en/specs/cirnet-rules/adaptive-stable/spec.md -->
<!-- source-sha256: 89deff6fd71896244daae22332a02cbbdc6846a16ceeb6c168d85cb255c3a2d0 -->
