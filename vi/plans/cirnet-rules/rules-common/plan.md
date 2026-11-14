# Kế hoạch chuyển đổi Common Rules

Ngày 2026-11-14. Bản đối chiếu của
[kế hoạch tiếng Anh](../../../../en/plans/cirnet-rules/rules-common/plan.md).
Đây là hồ sơ chuyển đổi, không phải workflow bắt buộc cho task sau này.

- [x] Xác minh đúng worktree/branch, backup và trạng thái ban đầu.
- [x] Đọc sáu repository, ghi SHA, ownership và giới hạn chức năng.
- [x] Viết baseline độc lập, adapter, bốn topic guide, ba skill và mẫu local.
- [x] Viết setup và repository map; giữ giấy phép và attribution phù hợp.
- [x] Xóa riêng legacy tracked đã có trong backup.
- [x] Kiểm tra file, ba tình huống trên giấy và review độc lập.
- [x] Đối chiếu refs/status cuối cùng và báo cáo kết quả.

Ban đầu có 290 file tracked; không có thay đổi, untracked hoặc ignored được báo.
HEAD, `master`, `governance-cli-only` cùng ở
`646afa38516bbfd276b0aecb7a138ab427df5fa9`.
Adapter/setup/map/skills mới chưa tồn tại; bootstrap cũ phụ thuộc governance.

Quyết định: dùng worktree hiện tại và thiết kế người dùng đã cung cấp; không mở
vòng xin phép hoặc tạo commit. Giữ plan/spec ở thư mục trung tâm hiện có.
Kiểm tra tĩnh và tình huống trên giấy phù hợp đầu ra Markdown;
native client E2E được ghi riêng là `NOT_RUN` khi chưa thực thi.
