# Nhật ký thay đổi

## 0.2.0 (candidate)

- Thêm [Rule giao diện không gian làm việc kỹ thuật](../rules/engineering-workspace-ui.md)
  và [Skill có thể chọn cài](../skills/cirnet-engineering-workspace-ui/SKILL.md)
  cho editor và workbench lấy đối tượng kỹ thuật làm trung tâm.

- Thêm ba Skill có thể chọn cài cho Task Continuity, soạn Plan/Spec và
  đánh giá Single Candidate. Hướng dẫn cài cùng các Skill mặc định.

- Dùng một phiên Candidate hiện tại và một danh mục cho JavaScript, TypeScript,
  Python, Rust và SystemVerilog. Niêm phong bằng chứng vai trò để rà chất lượng
  độc lập và giữ công cụ A/B cũ tại các revision đã ghi.

- Tập trung bộ công cụ vào Work Prompt có phạm vi, quyền sở hữu theo Lab-First
  và thay đổi mã đã kiểm chứng. Đóng băng checkpoint và A/B quy mô lớn cho
  việc chạy thủ công. Giữ số liệu chẩn đoán vai trò và bỏ chấm điểm hiệu năng.

- Giữ giao thức stage kiểm chứng an toàn khi lọc output; nhận marker lịch sử
  với mã thoát con dạng số và chặn PASS giả khi có error/signal. Bao phủ cả
  hai kiểu xuống dòng và giữ nguyên bằng chứng chấm lịch sử.

- Khôi phục nhiệm vụ gốc được cung cấp bên trong yêu cầu biên soạn trước khi
  kết luận thiếu mục tiêu; giữ riêng bằng chứng benchmark của ứng viên đóng
  băng không đạt.
- Giữ thao tác bàn giao rõ ràng và dùng lại context riêng theo vai trò. Ưu tiên
  đọc cấu trúc có giới hạn và giữ giới hạn riêng tư trong bàn giao bị chặn.
- Tách danh tính ứng viên sửa lỗi khỏi bằng chứng B2 lịch sử. Mở rộng audit
  giá trị escape và gắn nghiệm thu Native với bằng chứng đã rà soát.
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
<!-- source-sha256: 1da2a61f743e660063a5d3ee3a818a2bafc719e14e55ce79c5ced9407fd97083 -->
