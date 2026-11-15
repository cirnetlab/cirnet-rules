# Nhật ký thay đổi

## 0.2.0 (candidate)

- Rà soát Markdown trùng lặp và tệp mới đề xuất bằng lệnh `check-docs` chỉ đọc.
  Tìm nguồn tài liệu hiện có trước khi tạo trang.
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
<!-- source-sha256: 363688e8c9e3aac9cf2677cfcb58eddb8d5b564422ece71a5ed9e5b211241f38 -->
