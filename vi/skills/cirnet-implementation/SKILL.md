---
name: cirnet-implementation
description: Dùng khi triển khai tính năng hoặc thay đổi hành vi đã rõ kết quả mong đợi.
---

# Triển khai

Dùng cho thay đổi đã thống nhất. Dùng debugging trước khi chưa rõ nguyên nhân.

Tối thiểu cần: File được phép sửa, hành vi đích, interface của owner và test gần đó.

1. Khi cần quyết định kiến trúc CIRNET, mở `en/policies/architecture.md` từ
   nguồn được phép nêu trong baseline. Chọn kiến trúc áp dụng và bên sở hữu.
   Đọc luồng bị ảnh hưởng, nơi gọi, contract, artifact và test. Nêu giả định
   ảnh hưởng đến hành vi, tương thích, dữ liệu hoặc an toàn. Làm rõ điểm mơ hồ
   quan trọng hoặc quyền còn thiếu trước khi sửa phần liên quan.
   Tiếp tục với giả định thông thường có thể đảo ngược nếu nhiệm vụ cho phép.
2. Với việc xuất/tiết lộ dữ liệu, dùng yêu cầu, mã và contract đáng tin cậy để xác
   lập ai thực hiện, họ được xuất bản ghi của ai và những trường nào được phép.
   Phân biệt dữ liệu của mình, người dùng khác, tenant/nhóm và phạm vi hàng loạt.
   Quyền sửa mã, đăng nhập, ID hay public projection không xác lập quyền xuất.
   Làm rõ người nhận/nơi nhận, cách tải/lưu và che dữ liệu khi ảnh hưởng đến tiết
   lộ thông tin. Hỏi cụ thể hoặc chỉ rõ quyết định còn thiếu về quyền với bản ghi
   hay phạm vi trường trước khi sửa phần liên quan; dùng lại quyết định đã rõ.
   Tiếp tục đọc có mục tiêu hoặc đưa đề xuất ngắn chưa triển khai, không tạo
   endpoint xuất dữ liệu, migration hay tệp xuất khi quyền chưa rõ. Đọc mã
   hàm/projection và tên thuộc tính trong kho dữ liệu trước; lọc giá trị nhạy cảm
   được ghi trực tiếp trong mã trước khi xuất đoạn source. Đọc toàn bộ tệp ra
   output lệnh sẽ tiết lộ giá trị lưu trong đó, kể cả khi câu trả lời cuối không
   nhắc lại. Giữ các giá trị đó khỏi ví dụ và log mới.
3. Viết tiêu chí nghiệm thu quan sát được. Theo quy tắc lập kế hoạch cục bộ và
   dùng dạng `Bước → phép kiểm tra` cho việc không đơn giản.
4. Sửa trong luồng hiện có ở mức nhỏ nhất đủ đáp ứng yêu cầu. Giữ xử lý lỗi
   và các biện pháp an toàn cần thiết. Sinh đầu ra dẫn xuất từ đầu vào của bên
   sở hữu.
5. Bổ sung coverage có ý nghĩa khi cần. Cập nhật cặp tài liệu bị ảnh hưởng và
   chạy các phép kiểm tra gắn với nghiệm thu, gồm kiểm tra kiến trúc của bên
   sở hữu đối với ranh giới bị ảnh hưởng.

Với công việc có Task Continuity, lưu trước khi sửa, sau khi sửa trực tiếp bằng
agent và khi tiến độ hoặc quyết định thay đổi. Chạy các mốc lệnh đã được phép
bằng `scripts/task-continuity.ps1 run` từ checkout toolkit đã duyệt. Trong
phiên mới, dùng `resume`, đọc lại tham chiếu áp dụng và xác nhận quyền độc
lập. Xử lý vướng mắc trước khi dùng `-ConfirmContinuation` và chỉ sửa khi
`CanContinue` cho phép.

Báo diff, hành vi, kiểm tra và giới hạn. Đối chiếu mỗi tuyên bố nghiệm thu
với kết quả quan sát được và chỉ rõ phần còn thiếu.

Mở [Ví dụ kỹ thuật](references/engineering-examples.md#ví-dụ-kỹ-thuật) khi có
thể tham khảo ca tương tự về API, Vue, testbench RTL hoặc kiểm thử hồi quy.

<!-- source: en/skills/cirnet-implementation/SKILL.md -->
<!-- source-sha256: 4789fbd55aa89922bec8eadd45a1ef30544249222f1783bec97f18f69f6cb9e8 -->
