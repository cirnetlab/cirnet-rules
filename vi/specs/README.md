# Đặc tả

Spec ghi hành vi mong muốn, ranh giới ownership, phần ngoài mục tiêu và nghiệm thu.
Với thay đổi common-rules không đơn giản, dùng
`en/specs/cirnet-rules/<task>/spec.md` cùng tài liệu tiếng Việt tương ứng.
Bắt đầu từ [mẫu spec](../templates/spec.md) khi cần; không tạo spec chỉ cho sửa
rất nhỏ, trừ khi chính sách local yêu cầu.

Tách yêu cầu khỏi kết quả kiểm chứng. Nếu triển khai cho thấy cần đổi thiết kế,
cập nhật spec Anh và bản dịch, giải thích quyết định và kiểm tra nghiệm thu bị ảnh hưởng.
Không âm thầm viết lại yêu cầu cũ để biến kết quả lỗi thành đạt.

Hồ sơ lịch sử giữ ngày/phạm vi ban đầu. Yêu cầu hiện tại thay quyết định thiết kế
cũ phải nêu rõ; plan lịch sử không phải căn cứ mới.
Repo sản phẩm giữ thư mục spec và owner nghiệm thu riêng.

Hồ sơ task: [rebuild ban đầu](cirnet-rules/rules-common/spec.md) (lịch sử),
[nâng cấp song ngữ](cirnet-rules/bilingual-upgrade/spec.md),
[nâng cấp toolkit](cirnet-rules/toolkit-upgrade/spec.md),
[hành vi kỹ thuật](cirnet-rules/engineering-behavior/spec.md),
[khôi phục E01/E08](cirnet-rules/e01-e08-recovery/spec.md).

<!-- source: en/specs/README.md -->
<!-- source-sha256: 76644eeda248c8cafc9bce8a8981b3d2dff18d5a730cb00e5b48d530772db436 -->
