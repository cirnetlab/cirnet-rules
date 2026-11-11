<!-- doc-id: INDEX -->
<!-- revision: 4 -->
<!-- source-sha256: 5fbe8732dc6e4626bf0cd9f29d9d2777956a3713a0f21ef4ccb47b5c23a60bf5 -->
<!-- translation-status: synchronized -->
<a id="index"></a>

# CIRNET RULES

[English](../en/README.md) · [Điểm vào cho agent](AGENTS.md) · [Registry](../governance.yaml)

CIRNETRULES cung cấp lõi governance bằng Rust, CLI `cnr` và máy chủ MCP
qua STDIO. Chạy phiên lập trình độc lập trong Codex Desktop, Claude Code
hoặc Antigravity. Dùng registry và chính sách tiếng Anh để xác định thẩm
quyền của task, rồi dùng kho task cục bộ để theo dõi công việc và bằng chứng.

| Phạm vi | Tài liệu |
| --- | --- |
| Registry, chính sách và quy tắc | [Registry](../governance.yaml), [chính sách và chuyển trạng thái](policies/governance.md), [kiến trúc](rules/architecture.md), [lập trình](rules/coding.md), [bảo mật](rules/security.md), [tài liệu](rules/documentation.md) |
| Kỹ năng và hồ sơ vai trò | [Kỹ năng](skills/), [hồ sơ vai trò](profiles/), [xuất cấu hình repository](adapters/README.md) |
| Tham chiếu và hook | [Tham chiếu](references/), [danh mục lệnh](commands/README.md), [hướng dẫn hook](hooks/README.md), [công cụ](tools/README.md) |
| Định tuyến repository | [Workspace và binding](routing/README.md), [quyền sở hữu repository](profiles/product-architecture.md) |
| Quản lý task và quyền | [Quy trình vận hành](references/cirnetrules-local.md), [phối hợp dùng chung](references/shared-task-coordination.md), [bàn giao cho agent](references/handoff.md) |
| Kiểm chứng và audit | [Hướng dẫn kiểm chứng](references/verification.md), [kiểm thử Rust](tests/README.md), [schema](schemas/README.md) |
| CNR CLI và máy chủ MCP | [Tham chiếu CLI](references/cnr.md), [MCP STDIO và xuất cấu hình khởi chạy](references/mcp.md) |

[Kế hoạch phát triển lõi](plans/cirnet-rules/) · [Đặc tả lõi](specs/cirnet-rules/) ·
[Nhật ký thay đổi](CHANGELOG.md) · [Khôi phục dữ liệu cũ](references/cirnetrules-identity.md)
