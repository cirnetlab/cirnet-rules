<!-- doc-id: CNR-MCP-STDIO -->
<!-- revision: 3 -->
<!-- source-sha256: 2a7a9e9781188037712eece7a6605ef29ced39dfb981a48251ecfa2aaeb3cda4 -->
<!-- translation-status: synchronized -->
<a id="stdio"></a>

# CIRNETRULES MCP qua STDIO

Khởi chạy server với các thư mục gốc tuyệt đối và chuẩn hóa. Thay các giá trị
mẫu bằng đường dẫn thật. Đặt đường dẫn chứa khoảng trắng trong một đối số.

```sh
cnr mcp serve --governance-root AUTHORITY --workspace WORKSPACE --access read-only
cnr agent mcp-config preview-export --governance-root AUTHORITY --workspace WORKSPACE --binary CNR_EXECUTABLE --output NEW_BUNDLE --json
cnr agent mcp-config export --governance-root AUTHORITY --workspace WORKSPACE --binary CNR_EXECUTABLE --output NEW_BUNDLE --expect BUNDLE_PLAN_DIGEST --json
cnr agent mcp-config check --governance-root AUTHORITY --bundle NEW_BUNDLE --json
```

Thư mục đầu ra nằm ngoài Git và vùng cấu hình native. Review các tệp local
và manifest. Ghép bảng trong `codex.toml` vào cấu hình dự án Codex cần dùng,
dùng `claude.mcp.json` làm mẫu MCP dự án Claude Code, hoặc dùng
`antigravity.mcp_config.json` trong trình chỉnh sửa MCP của Antigravity. Các
tệp chứa đường dẫn tuyệt đối local và đối số khởi chạy chỉ đọc. Giữ tệp riêng
của máy ngoài Git. Manifest mới gắn launcher với digest của CNR, định danh
checkout Git, HEAD và snapshot governance. Xuất lại khi các đầu vào này thay đổi.

Dùng **Thiết lập Agent** trong CNR để xem trước đăng ký cấp dự án. Thao tác
ghi tự động cần worktree đã xác nhận, bundle đã duyệt từ CNR đang chạy, quyền
ghi đúng tệp từ task hiện hành và xác nhận của người vận hành. Xác nhận mức
tin cậy của dự án và phê duyệt server riêng trong giao diện Agent.

| Agent | Cấu hình dự án | Phạm vi người dùng và nhập thủ công |
| --- | --- | --- |
| Codex | `.codex/config.toml` trong dự án tin cậy | Ghép `codex.toml` bằng giao diện native. Cấu hình hiệu lực phụ thuộc host và `CODEX_HOME`. |
| Claude Code | `.mcp.json` | Ghép `claude.mcp.json` qua cấu hình MCP của client đã cài. Claude Code quản lý phê duyệt dự án. |
| Antigravity | `.agents/mcp_config.json` khi chọn rõ CLI hoặc IDE | Nhập `antigravity.mcp_config.json` qua trình quản lý MCP cho App, phạm vi người dùng hoặc bề mặt chưa xác nhận. |

Trên dòng lệnh, xem trước bằng `cnr agent mcp-config install --agent AGENT
--repo REPO --bundle BUNDLE --target WORKTREE --task TASK --actor OPERATOR
--dry-run`. Với Antigravity CLI hoặc IDE, thêm `--surface cli` hoặc
`--surface ide`. Duyệt tệp đích và mẫu khởi chạy rồi thay `--dry-run` bằng
`--yes --expect PLAN_DIGEST --trust-reviewed`. Xem biên nhận bằng
`cnr audit agent --receipt RECEIPT`. Xem trước hoàn tác bằng
`cnr agent mcp-config rollback --agent AGENT --target WORKTREE --receipt
RECEIPT --task TASK --actor OPERATOR --dry-run`, rồi xác nhận bằng digest
trả về. Nếu mục `cirnetrules` hoặc tệp đã cài thay đổi, thao tác bị chặn
để người dùng rà soát. Đăng ký giữ nguyên các byte khác trong tệp. Tệp được
ghi và biên nhận có quyền truy cập giới hạn cho chủ sở hữu. Biên nhận chỉ
lưu đoạn đã chèn và digest tệp để hoàn tác.

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

<a id="task-context"></a>

## Context và phạm vi chính xác của task

Đọc `cirnet_task_status` trước. Dùng revision nhận được làm
`expected_revision` cho `cirnet_task_context`. Task cần quyền đọc còn hiệu
lực. CLI `cnr task context` và MCP trả cùng metadata bên cạnh các tệp nguồn
được định tuyến.

- `primary_repo`, `spec_revision`, `descriptor_sha256` và `approval_epoch`
  xác định task cùng trạng thái thẩm quyền.
- `task_scope` chứa đầy đủ các ánh xạ `read_scope`, `write_scope`,
  `verification_scope` và `documentation_scope`. `acceptance` và
  `allowed_command_ids` đến từ cùng descriptor.
- `execution_bindings` xác định các binding đã kiểm tra theo repository,
  ID binding, danh tính và commit. Trường `frozen` cho biết task đã ghi
  nhận binding để thực thi. Đường dẫn checkout tuyệt đối chỉ dùng cục bộ.
- `authorization_summary` báo các thao tác đã khai báo và hạn dùng.
  Đánh giá quyền ghi gồm `WRITE_NOT_AUTHORIZED`, `WRITE_REQUIRES_ACTIVE_TASK`
  hoặc `WRITE_REQUIRES_GRANT_EVALUATION`. Chạy `cnr task grant-evaluate`
  với đúng đường dẫn và actor trước khi sửa.

Trường `scope` hiện hữu vẫn là chuỗi `read-only context`. Toàn bộ metadata
được tính vào ngân sách byte của context. CLI báo chi phí này tại
`metrics.task_metadata_bytes`, kể cả trong phản hồi delta. Context trả
`BLOCKED` khi metadata chính xác chưa thể được công bố đầy đủ. Với
`cirnet_diff_check`, lấy digest dự kiến từ lần chạy `cnr task diff-check`
hiện hành sau khi sửa.

`cnr check mcp` thử CNR đang chạy qua đường ống riêng với profile chỉ đọc.
Phép thử kiểm tra cả hai lifecycle, đúng sáu tool và các lời gọi có cấu trúc
để đọc chính sách và repo. Nếu có task, phép thử đọc trạng thái và context
tại revision vừa nhận. Phép thử có thời hạn và dọn tiến trình do nó khởi chạy.

`STATIC_CONFIG_VALIDATED` xác nhận tệp đã xuất, `CONFIG_REGISTERED` xác nhận
mục cấu hình dự án khớp, và `MCP_PROTOCOL_VERIFIED` xác nhận probe qua đường
ống CNR. `NATIVE_CLIENT_VERIFIED` cần quan sát riêng kết nối của từng Agent
đã cài tại worktree được chọn.

[Quy trình phối hợp](shared-task-coordination.md) ·
[Rust SDK](https://docs.rs/rmcp/3.6.0/rmcp/) ·
[Đặc tả MCP](https://modelcontextprotocol.io/specification/2026-07-28) ·
[Codex](https://developers.openai.com/codex/mcp) ·
[Claude Code](https://code.claude.com/docs/en/mcp) ·
[Antigravity](https://antigravity.google/docs/mcp)
