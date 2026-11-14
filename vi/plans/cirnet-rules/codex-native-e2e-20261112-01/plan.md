<!-- doc-id: CNR-CODEX-NATIVE-E2E-PLAN -->
<!-- revision: 1 -->
<!-- source-sha256: 3cc1c7efd5eae485e4fb0b99ecf692fbe3890b4e957f47cf84b2d455ed87588b -->
<!-- translation-status: synchronized -->
<a id="implementation"></a>

# Kế hoạch tích hợp Codex Desktop native

1. CP1 xác minh checkout và bản build đã ghim, tái hiện lỗi validation của proposal setup và mở giao diện vận hành CNR.
2. CP2 xác minh repo Interface thật, tạo và bind worktree riêng, rồi chuẩn bị phạm vi chính xác cho task setup và task sản phẩm.
3. CP3 xuất, kiểm launcher MCP chỉ đọc và probe STDIO. Đăng ký cấp dự án cần phê duyệt task và xác minh trust.
4. CP4 cung cấp phạm vi task đã xác thực trong context dùng chung và quan sát lời gọi native khi có phiên Desktop ở target.
5. CP5 áp dụng chỉnh sửa tài liệu đã review từ phiên Desktop ở target sau đánh giá grant mới. Xác minh diff và bằng chứng audit qua core.
6. CP6 chạy cổng regression và build, ghi từng tầng bằng chứng và lưu checkpoint khi gặp ranh giới host hoặc phê duyệt.

[Phạm vi](../../../specs/cirnet-rules/codex-native-e2e-20261112-01/spec.md)
