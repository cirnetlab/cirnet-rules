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

Khi cần chứng minh tuân thủ kiến trúc CIRNET, ghi revision của nguồn áp dụng và
bên sở hữu từ `en/policies/architecture.md` cùng bảng định tuyến ở đó. Gắn
các ràng buộc bị ảnh hưởng về an toàn điện và dây nối của sinh viên với test
của bên sở hữu. Kiểm tra tĩnh xác nhận tính hợp lệ của tài liệu và metadata.
Dùng artifact được pin và test tích hợp của bên sở hữu làm bằng chứng cho
chuỗi tiến trình đã thực thi.

Liệt kê từng lệnh và PASS, FAIL, NOT_RUN hoặc BLOCKED cùng phạm vi. Nếu có
khoảng trống, nêu điều kiện còn thiếu hoặc phép kiểm tra chưa giải quyết.

Với pipeline compile/simulation, ghi mã thoát thực tế của từng tiến trình trước
khi lệnh khác, kể cả Git hoặc dọn dẹp, ghi đè nó. Nếu wrapper chỉ cung cấp kết
quả tổng hợp, đọc luồng xử lý và quan sát kết quả tiến trình con trong một lượt
thực thi mới được phép. Giữ nguyên tham số, đầu vào, môi trường, cách xử lý
standard I/O, timeout, điều kiện chạy bước phụ thuộc và cleanup. Giữ chẩn đoán
trong phạm vi ghi. File output, marker thành công hoặc exit tổng thể không cung
cấp được mã thoát còn thiếu của một bước; báo rõ thiếu capture và chưa đóng nghiệm thu.

Với FPGA/RTL, câu trả lời cuối phải nêu riêng trạng thái `RTL compile` và
`RTL simulation`, cùng `Compile exit code` và `Simulation exit code` từ những
quan sát đó. Ghi NOT_RUN cho bước phụ thuộc chưa chạy; không tự điền thành công
hoặc mã số cho chúng. Liệt kê riêng từng safety assertion bắt buộc, chẳng hạn
reset, permit loss, watchdog expiry và fault khi có, với PASS/FAIL/NOT_RUN dựa
trên thực thi. Nếu chưa kiểm chứng vật lý, thêm chính xác dòng
`physical verification: NOT_RUN`. Simulation không xác lập mức sẵn sàng về
điện hoặc an toàn nạp chương trình; các điều đó cần bằng chứng riêng được bên
sở hữu cho phép.

<!-- source: en/skills/cirnet-verification/SKILL.md -->
<!-- source-sha256: a2b7f350751d5dec896eff065f45b8b277a1cf96bf748975c41743ccdc06a1b1 -->
