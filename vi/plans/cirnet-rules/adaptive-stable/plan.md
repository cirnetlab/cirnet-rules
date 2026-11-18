# Kế hoạch tích hợp Adaptive Stable

Phạm vi là checkout CIRNET rules và gói cập nhật được đọc. [Đặc tả](../../../specs/cirnet-rules/adaptive-stable/spec.md)
quy định hành vi và giới hạn nghiệm thu.

| Bước | Phép kiểm |
| --- | --- |
| Xem gói cập nhật và checkout | Đối chiếu từng điểm chèn với source hiện tại và ghi trạng thái Git ban đầu. |
| Cập nhật Skill biên soạn, template và hướng dẫn | Rà mỗi cặp EN/VI, xác minh hash source theo byte và đếm đủ 13 Skill. |
| Nối smoke Candidate-only với runner vai trò | Chạy preview và kiểm tra luồng Author sang Work cùng xử lý lỗi bằng shim offline. |
| Rà diff và kết quả kiểm chứng | Chạy bộ kiểm tra tài liệu, Cargo và regression offline. So đầu vào và bằng chứng lịch sử với trạng thái ban đầu. |

## Kết quả kiểm chứng

Bắt đầu tích hợp trên `master` sạch tại `fdfe0949`. Đối chiếu và tích hợp gói
thủ công để cả hai hướng dẫn context đều có phần vòng đời model. Smoke offline
đã đạt preview, đối chiếu byte bàn giao, chốt bảo vệ dữ liệu giả lập, xử lý
lỗi objective và thiếu bằng chứng. Bộ kiểm tra source xác nhận 13 Skill, một
cặp mẫu Work Prompt và các source hash khớp.

Cargo format, test, Clippy, bộ chọn tài liệu thay đổi, fixture đóng gói và
cài đặt, cùng các kiểm tra hành vi offline, follow-up và shim A/B đã qua. Snapshot kiến
trúc và 5.525 file lịch sử trong `.superpowers` khớp trạng thái ban đầu.
Không thể mô phỏng RTL vì thiếu `iverilog` và `vvp`. Windows bỏ qua các ca
kiểm tra liên kết cần quyền tạo symlink hoặc reparse point.

Smoke native, rà hành vi độc lập và benchmark native đầy đủ đều `NOT_RUN`.
Chưa đo usage native. Cần quyền riêng cho đợt chạy native và phải rà độc lập
bằng chứng từ đợt chạy trước khi kết luận tương thích.

<!-- source: en/plans/cirnet-rules/adaptive-stable/plan.md -->
<!-- source-sha256: 5ca088a12b26278c593fb2349898641e76e19899c71c037362acd261b5634775 -->
