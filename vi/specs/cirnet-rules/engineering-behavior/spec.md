# Nâng cấp hành vi kỹ thuật

Ngày 2026-11-15. Worktree ban đầu là `rules-common` tại
`a24c94aeb43d95dbf05f8d53f8676676dccba2e2`.

Tích hợp làm rõ, đơn giản, sửa đúng phạm vi và kết quả kiểm chứng được vào
baseline, coding rule và sáu skill hiện có. Giữ 12 ID skill, tài nguyên portable,
cặp EN/VI và chỉ dẫn an toàn. Đóng gói bốn ví dụ Before/After giả lập tự viết trong
skill implementation.

Định nghĩa đúng tám ca giả lập với rubric riêng và kiểm tra khách quan.
Dùng fixture mới và profile tạm cùng ca và thiết lập cho baseline và candidate.
Tách kết quả native khỏi kiểm tra harness offline. Xác thực, sandbox và simulator
có thể chặn đánh giá native. Đo văn bản toolkit, báo đúng bằng chứng và giới hạn.

Phạm vi ghi được phép gồm worktree toolkit và output riêng của kiểm thử.
Repo sản phẩm, cấu hình client thật và Git refs toolkit được bảo vệ.

<!-- source: en/specs/cirnet-rules/engineering-behavior/spec.md -->
<!-- source-sha256: 243a9caa1af932526280a77d9a57ce8839bfec4aec27a927c4b31fc93bd84e4e -->
