# Nâng cấp Common Rules Toolkit

Baseline ngày 2026-11-14 là `35e677b1264958839a7e0cb883c4477eb24da6df`.

- Cung cấp baseline EN độc lập, ngữ cảnh theo nhu cầu, 12 skill có namespace
  và bản VI, mẫu tương thích/repo, gói offline có phiên bản. Giữ kiểm tra Rust
  cho cặp ngôn ngữ, liên kết và điểm vào.
- PowerShell 7 mặc định xem trước ở phạm vi người dùng. Chỉ áp dụng gói local
  đã duyệt khi được chỉ định, giữ nguyên byte cá nhân, sao lưu đường dẫn bị
  ảnh hưởng và kiểm tra xung đột khi khôi phục.
- Thử lỗi trong home giả và gói đã giải nén. Rà soát cách nạp ngữ cảnh, đo
  văn bản, chạy test Rust và CI toolkit. Ghi riêng kết quả nạp native.

Chỉ ghi vào `rules-common`. Thao tác trên sản phẩm, phát hành, cài client thật
và thay đổi Git nằm ngoài phạm vi.

<!-- source: en/specs/cirnet-rules/toolkit-upgrade/spec.md -->
<!-- source-sha256: cf579c4c30483576de2806b7457f1fbfb2426fae30410b1bec30081b012db457 -->
