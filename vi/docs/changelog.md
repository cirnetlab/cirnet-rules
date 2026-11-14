# Nhật ký thay đổi

## 0.2.0 (candidate)

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
<!-- source-sha256: 4a6adf705f9e571456fead0eaa9838f115fa48791f9906d88c55a9edcf479709 -->
