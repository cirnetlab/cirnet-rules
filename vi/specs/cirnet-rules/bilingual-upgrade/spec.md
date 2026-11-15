# Đặc tả nâng cấp tài liệu song ngữ

Bắt đầu ngày 2026-11-14 từ `rules-common` sạch tại
`17f1738ccd4312f284b61372a3c507a65a26588a`.

- Dùng `en/` làm nguồn chuẩn và `vi/` làm bản tương ứng cho người đọc. Soát nghĩa
  trước khi ghi đường dẫn tiếng Anh và SHA-256. Cập nhật cả hai ngôn ngữ của nội
  dung bị ảnh hưởng và ghi giới hạn kiểm chứng hoặc phần dịch còn thiếu.
- Giữ bản sao `AGENTS.md` độc lập tại root, adapter `CLAUDE.md`, điều hướng
  song ngữ và ba skill tùy chọn ở giai đoạn này. Ghép cặp hướng dẫn, hooks,
  policies, plans, specs, references, rules, skills, templates và docs.
- Dùng hook Python tùy chọn, chỉ đọc đã được duyệt cho giai đoạn này để kiểm
  tra cặp nguồn, liên kết, điểm vào và metadata skill. Chạy kiểm thử tình huống
  lỗi và `git diff --check`. Kích hoạt hook native ở trạng thái NOT_RUN.

Giữ bằng chứng chuyển đổi có ngày, refs sao lưu, LICENSE, quy tắc ignore và ghi công.
Phạm vi là worktree hiện có, loại trừ sửa sản phẩm, cấu hình toàn cục, stage và
commit Git, cài hook native và hạ tầng runtime.

<!-- source: en/specs/cirnet-rules/bilingual-upgrade/spec.md -->
<!-- source-sha256: ea4112a5301bcd03f88b6897a504bd3b1c6c1f0ef6f685505eb1c9733718f72a -->
