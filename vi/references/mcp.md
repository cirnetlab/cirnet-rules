<!-- doc-id: CNR-MCP-STDIO -->
<!-- revision: 1 -->
<!-- source-sha256: eaf9b987d3e2ffb90a31941ee02c38c442b278a609244c35fc8749ec48af3a12 -->
<!-- translation-status: synchronized -->
<a id="stdio"></a>

# CIRNETRULES MCP qua STDIO

Khởi chạy server với các thư mục gốc tuyệt đối và chuẩn hóa. Thay các giá trị
mẫu bằng đường dẫn thật. Đặt đường dẫn chứa khoảng trắng trong một đối số.

```sh
cnr mcp serve --governance-root AUTHORITY --workspace WORKSPACE --access read-only
cnr agent mcp-config export --governance-root AUTHORITY --workspace WORKSPACE --binary CNR_EXECUTABLE --output NEW_BUNDLE --json
cnr agent mcp-config check --governance-root AUTHORITY --bundle NEW_BUNDLE --json
```

Thư mục đầu ra nằm ngoài Git và vùng cấu hình native. Review các tệp local
và manifest. Ghép bảng trong `codex.toml` vào cấu hình dự án Codex cần dùng,
dùng `claude.mcp.json` làm mẫu MCP dự án Claude Code, hoặc dùng
`antigravity.mcp_config.json` trong trình chỉnh sửa MCP của Antigravity. Các
tệp chứa đường dẫn tuyệt đối local và đối số khởi chạy chỉ đọc. Giữ tệp riêng
của máy ngoài Git.

Server dùng Rust MCP SDK chính thức `rmcp` 3.6.0 với Tokio 1.54.1. Transport
hỗ trợ discovery STDIO theo `2026-07-28` và lifecycle trước đây
`2025-11-25`. Danh sách tool:

| Profile | Tool |
| --- | --- |
| Chỉ đọc | `cirnet_repositories_discover`, `cirnet_policy_lookup`, `cirnet_task_context`, `cirnet_task_status`, `cirnet_diff_check`, `cirnet_audit_read` |
| Phối hợp bổ sung | `cirnet_task_claim`, `cirnet_task_renew`, `cirnet_task_release`, `cirnet_task_report` |

STDOUT dành cho frame giao thức. Các thư mục gốc và profile truy cập được
cố định khi khởi chạy. Dịch vụ cố định Git admin và common directory, HEAD, registry và schema lúc khởi chạy. Khởi động lại server nếu chúng thay đổi. Tool dùng
schema có giới hạn và nhóm worker cố định. Truy cập phối hợp cần tệp phiên
local do operator có quyền chuẩn bị. Truyền đường dẫn bằng `--session-file`
khi khởi chạy. Giữ tệp riêng tư. Mỗi phiên task được cấp quyền dùng một tệp.
Schema tool không chứa token và không cho caller chọn actor hay gốc khác.

`STATIC_CONFIG_VALIDATED` mô tả các tệp đã xuất. `MCP_PROTOCOL_VERIFIED`
mô tả kiểm thử giao thức qua tiến trình. `NATIVE_CLIENT_VERIFIED` cần quan sát
kết nối trong Agent đã cài. Quyền sở hữu tệp, quyền ghi OS và sandbox native
là các cơ chế kiểm soát riêng.

[Quy trình phối hợp](shared-task-coordination.md) ·
[Rust SDK](https://docs.rs/rmcp/3.6.0/rmcp/) ·
[Đặc tả MCP](https://modelcontextprotocol.io/specification/2026-07-28) ·
[Codex](https://developers.openai.com/codex/mcp) ·
[Claude Code](https://code.claude.com/docs/en/mcp) ·
[Antigravity](https://antigravity.google/docs/mcp)
