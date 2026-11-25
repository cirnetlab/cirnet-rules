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

## Tuân thủ kiến trúc

- [x] Xác nhận `rules-common` tại `004c318` và đối chiếu bản chụp với revision nguồn.
- [x] So sánh các bản kiến trúc, định tuyến chủ sở hữu và ghi bằng chứng sản phẩm.
- [x] Kiểm chứng checker offline, cập nhật EN/VI và phân phối trong môi trường cô lập.

## Biên soạn Chat-to-Work

- [x] Kiểm tra checkout `rules-common`, các mẫu hiện có và cơ chế đóng gói.
  Rà soát đầu ra baseline với yêu cầu giả lập.
- [x] Thêm cặp mẫu và cặp Skill. Đồng bộ mục tra cứu và hướng dẫn nạp context.
- [x] Rà soát bản bàn giao của ứng viên, kiểm chứng tệp song ngữ và cài Skill
  theo lựa chọn. Ghi lỗi checker trên worktree và kết quả đạt ở bản sao cô lập.

[Đặc tả](../../../specs/cirnet-rules/toolkit-upgrade/spec.md) ·
[Kiểm chứng](../../../docs/toolkit-verification.md)

<!-- source: en/plans/cirnet-rules/toolkit-upgrade/plan.md -->
<!-- source-sha256: 8712c7f092401582ee8a7d2c1ddccbb9de01a1eafc7670e6ffbf59b1f3ccfb65 -->
