---
name: cirnet-verification
description: Dùng khi tuyên bố hoàn tất, tính đúng hoặc tương thích cần bằng chứng mới.
---

# Kiểm chứng

Dùng để đánh giá tuyên bố. Dùng regression testing để thiết kế coverage còn thiếu.

Tối thiểu cần: Tuyên bố, revision/diff chính xác, kết quả mong đợi và điều kiện kiểm tra local.

1. Gắn mỗi tuyên bố và tiêu chí nghiệm thu với phép kiểm tra có thể chứng
   minh điều đó. Xác nhận revision, điều kiện tiên quyết và tác động được phép.
2. Chạy các phép kiểm tra khả dụng rồi xem assertion, mã thoát, đầu ra và các
   ca bị bỏ qua. Dựa trên kết quả quan sát được để đánh giá tuyên bố hoàn tất.
   Đối chiếu chỉ dẫn EN/VI bị ảnh hưởng và xem diff cuối.
3. Phân biệt review mã nguồn, chạy unit test và integration test, hành vi
   thực tế của client và bằng chứng vật lý. Ca bị bỏ qua vẫn thiếu coverage
   ngay cả khi lệnh báo thành công.

Liệt kê từng lệnh và PASS, FAIL, NOT_RUN hoặc BLOCKED cùng phạm vi. Nếu có
khoảng trống, nêu điều kiện còn thiếu hoặc phép kiểm tra chưa giải quyết.

Khi mô phỏng RTL đạt, báo hành vi đã được thử. Tình trạng sẵn sàng về điện và
việc nạp chương trình cần bằng chứng riêng được bên sở hữu cho phép.

<!-- source: en/skills/cirnet-verification/SKILL.md -->
<!-- source-sha256: e83dbc9f5f5c5a9b41473a2fe41cca7514ebab99c881ff2af100c10964a8a02d -->
