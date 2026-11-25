# Bằng chứng độ tin cậy CIRNET

## Phạm vi và quyền sở hữu

Bổ sung ba phép kiểm nhỏ cho công việc kỹ thuật có phạm vi trong `cirnet-rules`.
Dùng [quy trình và kiến trúc của chủ sở hữu](../../../policies/architecture.md)
để xác định lệnh sản phẩm, contract, vị trí đầu ra và nghiệm thu. Dùng
[Task Continuity](../../../docs/distribution.md#task-continuity) hiện có khi
phù hợp. Giữ schema và hành vi thực thi của cơ chế này, hợp đồng đóng gói
và bằng chứng Candidate đã niêm phong.

Người vận hành duyệt phạm vi, tác động và nguồn. Chủ sở hữu cung cấp quan sát
từ lệnh và độ bao phủ assertion. Agent kiểm chứng tuyên bố bằng bằng chứng
đó và báo phạm vi. Môi trường thực thi cung cấp giới hạn hệ thống tệp và
mạng. Đối chiếu phạm vi đo thay đổi ròng. Các phép kiểm dưới đây bổ trợ
những cơ chế đó ở các bước riêng của quy trình.

## Hành vi mong đợi

1. **Đánh giá tuyên bố.** Đọc JSON kỳ vọng và quan sát của chủ sở hữu đã được
   ghim độc lập. Gắn tuyên bố với thư mục gốc repo được duyệt, revision Git
   hiện hành và digest của đầu vào đã khai báo. Kiểm tra chính xác ID bước,
   mã thoát nguyên đã quan sát, số assertion đã chạy và thất bại, cùng mức
   tối thiểu bắt buộc. Đánh giá bước thực tế độc lập với mã thoát của wrapper.
   Phân biệt bằng chứng cũ hoặc thiếu với bước đã chạy và thất bại. Giữ bước
   đã được bỏ qua rõ ràng, lý do và mức bắt buộc trong kết quả.
2. **Xem trước phép kiểm của chủ sở hữu.** Đọc khai báo npm, Cargo, CMake và
   RTL được chỉ định rõ. Trả ID ứng viên, digest nguồn, lý do lựa chọn và giới
   hạn về điều kiện tiên quyết và tác động. Yêu cầu lựa chọn rõ ràng. Rà soát
   script, tác động, công cụ và đường dẫn đầu ra thực tế của chủ sở hữu trước
   khi cấp quyền và thực thi riêng. Giữ nội dung lệnh và bằng chứng thô ngoài
   bản tóm tắt.
3. **Đánh giá lượt sửa.** Dùng phép kiểm đang thất bại và lịch sử lượt thử đã
   quan sát cho một cụm lỗi. Mặc định cho phép tối đa hai lượt sửa đã khởi
   chạy mỗi cụm và một lượt thử lại không đổi cho một định danh phép kiểm,
   với lý do lỗi tạm thời được ghi lại. Lượt sửa cần giả thuyết khác biệt.
   Phải xác minh quyền, an toàn và tiến triển. Dừng khi lặp giả thuyết, hết
   ngân sách hoặc thiếu bằng chứng. Xem quyết định là tư vấn khi tiếp nhận.
   Mọi thao tác do quy trình đã được cấp quyền của chủ sở hữu thực hiện.

## Nghiệm thu

- Hiển thị `PASS`, `FAIL`, `BLOCKED` và `NOT_RUN` cho tuyên bố, đồng thời
  giữ `SKIPPED` ở cấp bước. Bước bắt buộc bị bỏ qua hoặc thiếu quan sát
  ngăn kết quả chung PASS. Công bố phạm vi tùy chọn bị bỏ qua. Dùng fixture
  có kiểm soát để kiểm tra bước lỗi dù wrapper thoát thành công, assertion
  chưa đủ, thiếu mã thoát con, revision hoặc đầu vào cũ và thiếu điều kiện.
- Xem trước các phép kiểm được chọn rõ trong bố cục giả lập của chủ sở hữu
  được hỗ trợ. Xác minh thực thi vẫn là `NOT_RUN`. Từ chối đầu vào thiếu
  hoặc thiếu an toàn và lựa chọn không xác định. Để chủ sở hữu rà các bố cục
  chưa khai báo.
- Cho thấy quan sát từ lệnh thất bại, một cơ chế phòng lỗi hẹp và hồi quy
  đạt. Xác minh quyết định dừng sau các lượt lỗi lặp không đổi và hai lượt
  sửa, cũng như khi thiếu phạm vi, an toàn hoặc tiến triển.
- Giữ cặp EN/VI và hash, phạm vi, tương thích CI và đóng gói. Báo phát hiện
  Skill trên agent thật, E2E sản phẩm, bằng chứng phần cứng và hiệu năng so
  sánh theo phạm vi đã kiểm tra.

Ghi kết quả rà soát, quyết định tái sử dụng và quan sát trong
[kế hoạch triển khai](../../../plans/cirnet-rules/reliability-evidence/plan.md).

<!-- source: en/specs/cirnet-rules/reliability-evidence/spec.md -->
<!-- source-sha256: f648dc003ddb8d1e1a0ce978be85db9cd55cd745988f8aa08c3b15d386a24573 -->
