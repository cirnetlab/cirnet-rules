---
name: cirnet-regression-testing
description: Dùng khi một lỗi hoặc thay đổi hành vi cần test tập trung phát hiện hồi quy.
---

# Kiểm thử hồi quy

Dùng để thêm coverage cho rủi ro đã biết. Dùng debugging để xác định nguyên nhân chưa rõ.

Tối thiểu cần: Triệu chứng, hành vi công khai mong đợi, fixture, đường chạy test và quy định output.

1. Chọn ca tái hiện nhỏ, cho kết quả lặp lại ổn định. Viết assertion về hành vi
   quan sát được mà lỗi vi phạm, chẳng hạn kết quả ở biên hoặc thứ tự chính xác.
2. Nếu có thể, chạy test mới trên mã còn lỗi. Kiểm tra assertion cụ thể thất bại
   với đúng triệu chứng đã báo. Sửa đúng phần lỗi rồi chạy lại cùng test và bộ
   kiểm tra bị ảnh hưởng.
3. Giữ assertion, thời gian chờ và ranh giới thực đang được kiểm tra. Loại các
   thay đổi kỳ vọng, bỏ qua test hoặc mock làm che khuất lỗi. Xác minh bộ test
   thực sự chạy phép kiểm tra mới.

Báo test bảo vệ điều gì và kết quả thất bại và đạt đã quan sát được. Chỉ rõ
lượt nào chưa chạy được và lý do.

Parser chấp nhận frame bị cắt cụt. Kiểm tra việc từ chối và không ghi đầu ra
dở dang. Xác nhận lỗi test đến từ chính hành vi của parser đang được kiểm tra.

<!-- source: en/skills/cirnet-regression-testing/SKILL.md -->
<!-- source-sha256: 7dbd6f370508ddf0e86fdca37f5be0be30b469fca03809f53dd407c839be7d36 -->
