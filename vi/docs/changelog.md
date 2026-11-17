# Nhật ký thay đổi

## 0.2.0 (candidate)

- Giữ giao thức stage kiểm chứng an toàn khi lọc output; nhận marker lịch sử
  với mã thoát con dạng số và chặn PASS giả khi có error/signal. Bao phủ cả
  hai kiểu xuống dòng và giữ nguyên bằng chứng chấm lịch sử.

- Khôi phục nhiệm vụ gốc được cung cấp bên trong yêu cầu biên soạn trước khi
  kết luận thiếu mục tiêu; giữ riêng bằng chứng benchmark của ứng viên đóng
  băng không đạt.
- Giữ thao tác bàn giao rõ ràng và dùng lại context riêng theo vai trò. Ưu tiên
  đọc cấu trúc có giới hạn và giữ giới hạn riêng tư trong bàn giao bị chặn.
- Tách danh tính ứng viên sửa lỗi khỏi bằng chứng B2 lịch sử. Mở rộng audit
  giá trị escape và đặt nghiệm thu native trước đánh giá hiệu quả.
- Thêm Skill biên soạn Work Prompt có thể chọn cài và mẫu song ngữ để bàn
  giao nhiệm vụ đúng phạm vi cho Agent khác.
- Thêm cách chọn kiến trúc Lab-First, định tuyến chủ sở hữu repository và
  điều kiện bằng chứng để phát triển song hành. Xác minh bản chụp EN/VI đã
  ghim và bảng định tuyến bằng checker offline và CI hiện có.
- Tìm tài liệu sở hữu nội dung trước khi tạo Markdown. Rà soát tệp thêm và
  báo lỗi với bản trùng đã xác nhận ngoài diện bảo vệ qua `check-docs` và
  script Git diff trong CI.
- Làm rõ quyền xuất dữ liệu và cách đọc mã nguồn an toàn cho E01.
- Sửa đường dẫn tạm và I/O chuẩn của tiến trình RTL trong MXC. Ghi mã thoát
  thực tế của compile và simulation cùng trạng thái vật lý rõ ràng cho E08.
- Hỗ trợ xác thực Codex native cô lập và đường dẫn công cụ portable. Thêm
  kiểm tra harness tập trung và hợp nhất [nghiệm thu E01–E08](e01-e08-followup.md).
- Tích hợp bốn nguyên tắc hành vi kỹ thuật vào baseline và skill hiện có.
  Thêm ví dụ Before/After nạp khi cần và bộ đánh giá Codex giả lập có kiểm tra
  harness offline.
- Rút gọn baseline tự đủ và nạp thân skill cùng reference theo task.
- Thêm 12 skill có namespace cho triển khai, review, tương thích và bảo trì.
- Đổi `implementation`, `debugging` và `verification` sang tên tiền tố `cirnet-`.
- Thêm gói local, cài đặt có quản lý, xác minh và restore từ backup.
- Checker Rust khám phá skill động, kiểm tra tài nguyên portable, độ mới bản dịch
  và sự thống nhất phiên bản toolkit. Thêm CI cho toolkit.

Bản cài hiện có cần review khi nâng cấp vì tên thư mục skill mặc định đã đổi.
Xem [phân phối và nâng cấp](distribution.md).

## 0.1.0

Repo có baseline song ngữ, ba skill portable và checker tài liệu local.
Checker và test đã chuyển từ Python sang Rust.

<!-- source: en/docs/changelog.md -->
<!-- source-sha256: c1009944ae146acd9878ce89d7b27e730a51cf09274efdf3f132f384e79b4b9f -->
