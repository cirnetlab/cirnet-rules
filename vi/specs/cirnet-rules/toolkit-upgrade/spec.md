# Nâng cấp Common Rules Toolkit

Ngày 2026-11-14. Baseline: `35e677b1264958839a7e0cb883c4477eb24da6df`.

Giữ baseline tiếng Anh tự đủ và nạp skill, tài liệu bổ sung theo nhu cầu.
Cung cấp 12 skill có tên riêng, bản Việt cho người review, template tương thích
và repository, cùng bundle offline có phiên bản. Dùng checker Rust hiện có,
giữ kiểm tra song ngữ, liên kết và entrypoint.

Lệnh PowerShell 7 cho user scope mặc định xem trước. Khi apply rõ ràng, dùng bundle
local đã review, bảo toàn byte cấu hình cá nhân, backup đường dẫn bị ảnh hưởng
và hỗ trợ restore có kiểm tra xung đột. Kiểm thử trong home giả và bản đã giải nén.
Nghiệm thu gồm các ca lỗi installer/bundle, tình huống nạp ngữ cảnh, số đo văn bản,
bộ test Rust và CI chỉ cho toolkit. Quan sát loader native có kết quả riêng.

Phạm vi ghi là worktree `rules-common` hiện có. Sản phẩm, release, cài client thật
và thay đổi Git nằm ngoài task này.

<!-- source: en/specs/cirnet-rules/toolkit-upgrade/spec.md -->
<!-- source-sha256: c99ee8eda43a43efef2903609599e99717128bec791e0a836a98d55b952f59e5 -->
