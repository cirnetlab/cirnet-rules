<!-- doc-id: CNR-NATIVE-SMOKE-20261112-01-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: b8615b519ba5b14707af432a13a7e67a2d744950fa33678897fc4ace0f40ddf5 -->
<!-- translation-status: synchronized -->
<a id="scope"></a>

# Kiểm thử chỉnh tài liệu bằng Codex Desktop

Task `codex-native-smoke-20261112-01` đang chờ người vận hành phê duyệt và thực thi trong phiên native. Đích đề xuất là worktree của `cirnet-interface` có binding `codex-e2e-20261112-01` tại `f1ff70b9f22554c0680b92679e9e44a5544247ff`.

Phiên Codex Desktop dùng context hiện hành của task và làm rõ hai mục lục tài liệu bằng cách nêu đầu vào Contracts đã được xem xét và máy chủ phát triển Vite. Hướng dẫn hiện có đã mô tả hai nội dung này. Phạm vi đọc, ghi và kiểm chứng gồm đúng `docs/en/index.md` và `docs/vi/index.md`.

| Đường dẫn | Dòng hiện tại | Dòng đề xuất |
| --- | --- | --- |
| `docs/en/index.md` | `- [Getting started](guides/getting-started.md) covers dependency setup and Vite.` | `- [Getting started](guides/getting-started.md) covers installing reviewed Contracts artifacts and starting the Vite development server.` |
| `docs/vi/index.md` | `- [Bắt đầu](guides/getting-started.md) hướng dẫn cài gói phụ thuộc và chạy Vite.` | `- [Bắt đầu](guides/getting-started.md) hướng dẫn cài các gói Contracts đã được xem xét và khởi động máy chủ phát triển Vite.` |

- [ ] Lời gọi thật từ Codex Desktop lấy trạng thái task và context hiện hành của binding đã chọn trước khi sửa.
- [ ] Kiểm tra quyền ghi hiện hành đạt cho từng đường dẫn trước khi editor native sửa hai dòng. Kiểm tra một đường dẫn thứ ba bị chặn.
- [ ] Diff trong phạm vi chứa đúng các thay đổi trên so với baseline của task. Bước kiểm tra `interface-diff` đạt và MCP native kiểm tra digest của snapshot hiện hành.
- [ ] Bản ghi audit liên kết revision của task với biên nhận kiểm chứng. Việc nghiệm thu độc lập thuộc về reviewer được cấp quyền.

Hai mục lục tài liệu là đầu ra của task. Task này không có đích tài liệu riêng sau nghiệm thu. Đầu ra setup thuộc task cấu hình riêng và được ghi vào baseline trước khi task này bắt đầu.

[Kế hoạch](../../../plans/cirnet-interface/codex-native-smoke-20261112-01/plan.md)
