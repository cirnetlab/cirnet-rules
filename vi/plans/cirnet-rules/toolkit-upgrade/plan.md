# Kế hoạch nâng cấp toolkit

- [x] Ghi worktree, kích thước baseline và kết quả Rust. Giảm ngữ cảnh khởi
  động, mở rộng phát hiện skill và thêm 12 skill cùng mẫu.
- [x] Xây dựng và kiểm thử gói, cài đặt, xác minh và khôi phục tại repo. Ghi
  cách quản lý phiên bản và nâng cấp, đo ngữ cảnh và thêm CI cho toolkit.
- [x] Ghi bằng chứng nghiệm thu và giới hạn bằng hai ngôn ngữ.

## Rà soát Markdown dư thừa

- [x] Xác nhận worktree `rules-common` sạch tại `5d6cc26` và đọc mã checker.
- [x] Mở rộng checker hiện có và đạt tám test về trùng lặp, ngoại lệ và tệp mới.
- [x] Cập nhật hướng dẫn EN/VI hiện có và kiểm chứng checkout bằng checker offline.

## Hoàn thiện rà soát Markdown dư thừa

- [x] Xác nhận worktree `rules-common` sạch tại `66d2c8a`.
- [x] Kiểm thử Rust mức lỗi của tệp trùng và mức cảnh báo của tệp gần trùng.
- [x] Truyền danh sách tệp Git thêm vào checker trong CI và xác minh bằng fixture cục bộ.
- [x] Kiểm tra kích thước baseline, đồng bộ EN/VI và bằng chứng đã lưu.

[Đặc tả](../../../specs/cirnet-rules/toolkit-upgrade/spec.md) ·
[Kiểm chứng](../../../docs/toolkit-verification.md)

<!-- source: en/plans/cirnet-rules/toolkit-upgrade/plan.md -->
<!-- source-sha256: 968facd190e0113b19317bf858342a6b93ec5eb8f97e9954ac5d94993ba264b7 -->
