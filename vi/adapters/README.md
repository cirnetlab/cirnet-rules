<!-- doc-id: adapters -->
<!-- revision: 5 -->
<!-- source-sha256: 0dd74a1179a8aea7f9691f1a64fa0955ad0f780f859c780b9d20b860cfb998a2 -->
<!-- translation-status: synchronized -->

# Adapter khởi tạo ngữ cảnh Agent

[English](../../en/adapters/README.md)

<a id="adapter-scope"></a>
## Phạm vi

Lệnh bootstrap Rust chuẩn bị gói bàn giao có thể rà soát cho một checkout
CIRNET, nhiệm vụ và mức rủi ro. Bộ phân giải nhận diện sản phẩm qua Git remote
và gắn với commit đầy đủ của sản phẩm và governance. Adapter kiểm tra lại
digest từng tài liệu tiếng Anh đã chọn trước khi tạo gói. Biên nhận ghi nhận
bytes được bàn giao cho người vận hành. Việc Agent tự tìm chỉ dẫn, chạy hook
và làm theo quy tắc cần bằng chứng riêng.

Dùng thư mục phiên tuyệt đối đã tồn tại, dành riêng cho tác vụ và nằm ngoài
các checkout Git lẫn thư mục cấu hình Agent. Chỉ người vận hành adapter được
quyền ghi vào thư mục phiên hoặc đổi tên đường dẫn của thư mục đó. Adapter
giả định không có tiến trình khác chuyển hướng đường dẫn khi lệnh đang chạy.
Cấp quyền đọc checkout và nguồn tiếng Anh qua cơ chế quyền của harness. Với
audit sản phẩm, đặt quyền chỉ đọc cho cả sản phẩm và governance. Chỉ dẫn
Markdown và hook bổ trợ cho quyền hệ thống tệp và CI.

<a id="adapter-lifecycle"></a>
## Vòng đời cô lập

Từ checkout governance, đặt `CIRNET_AUDIT_WORKSPACE` thành đường dẫn tuyệt
đối tới thư mục chứa các checkout sản phẩm và `CIRNET_ADAPTER_SESSION`
thành thư mục phiên cô lập đã tồn tại. Dùng `claude` hoặc `antigravity`
thay `codex` cho các môi trường còn lại.

```powershell
cargo run --locked --bin cirnet-governance -- bootstrap --workspace "$env:CIRNET_AUDIT_WORKSPACE" --repo cirnet-remote --task audit --risk normal --agent codex --session "$env:CIRNET_ADAPTER_SESSION" --dry-run
cargo run --locked --bin cirnet-governance -- bootstrap --workspace "$env:CIRNET_AUDIT_WORKSPACE" --repo cirnet-remote --task audit --risk normal --agent codex --session "$env:CIRNET_ADAPTER_SESSION" --action install
cargo run --locked --bin cirnet-governance -- bootstrap --workspace "$env:CIRNET_AUDIT_WORKSPACE" --repo cirnet-remote --task audit --risk normal --agent codex --session "$env:CIRNET_ADAPTER_SESSION" --action uninstall
```

1. Phân giải ngữ cảnh cho checkout đích. Rà soát định danh Git, danh sách
   tài liệu tiếng Anh và phạm vi rủi ro. Chạy bootstrap ở chế độ dry-run và
   xác nhận thư mục quản lý cùng tên tệp dự kiến.
2. Cài gói trong thư mục phiên ngoài repo đã duyệt. Rà soát tệp đầu vào,
   `context.json` và `receipt.json`, rồi cung cấp tệp đầu vào và ngữ cảnh
   một cách tường minh cho Agent đã chọn. Lặp lại cùng nội dung không tạo
   thay đổi. Gói đang dùng khác nội dung hoặc đã bị sửa sẽ chặn việc thay thế.
3. Uninstall chuyển gói còn nguyên vẹn vào thư mục sao lưu mang digest,
   nằm cạnh thư mục đang dùng trước đó. Cài lại cùng ngữ cảnh sẽ phục hồi
   bản sao lưu. Tệp không nhận dạng, biên nhận bị sửa và trùng đích sao lưu
   sẽ chặn thao tác để người vận hành kiểm tra.
4. Dừng nếu nguồn đổi, thiếu quyền đọc hoặc harness không có sẵn. Adapter
   giữ lại gói chuẩn bị dở dang khi cài đặt thất bại. Rà soát thủ công vùng
   chuẩn bị còn lại hoặc khóa cũ, rồi dùng phiên cô lập mới để tiếp tục và
   giữ bằng chứng cũ.

<a id="config-export"></a>
## Cấu hình repository

Dùng thư mục bundle riêng ngoài Git, với thư mục cha đã tồn tại. Chỉ định
worktree đích. Thay các đường dẫn mẫu trong lệnh bên dưới.

```sh
cnr agent export --repo cirnet-interface --agents codex,claude,antigravity --output NEW_BUNDLE --json
cnr agent check --bundle NEW_BUNDLE --json
cnr agent sync --bundle NEW_BUNDLE --target BOUND_WORKTREE --dry-run --json
cnr agent sync --bundle NEW_BUNDLE --target BOUND_WORKTREE --yes --expect PLAN_DIGEST --task TASK --actor OPERATOR --json
cnr audit agent --receipt RECEIPT --json
cnr agent rollback --target BOUND_WORKTREE --receipt RECEIPT --dry-run --json
cnr agent rollback --target BOUND_WORKTREE --receipt RECEIPT --yes --expect ROLLBACK_DIGEST --task TASK --actor OPERATOR --json
```

Codex và Antigravity dùng chung chỉ dẫn repository trong `AGENTS.md`
và skill trong `.agents/skills`. Claude Code dùng `CLAUDE.md` và
`.claude/skills`. Manifest ghi digest theo byte và khả năng của đích.
Giữ đầu ra bundle ngoài Git. Xuất lại nếu nguồn đã chọn hoặc binding thay
đổi. Dry-run báo xung đột với tệp người dùng. Áp dụng thay đổi cần task hiện
hành với quyền ghi từng tệp chính xác. Rollback xác minh byte của tệp được
quản lý trước khi đảo thay đổi theo biên nhận mới nhất. Ghi nguyên tử theo
từng tệp. Sau gián đoạn, kiểm tra biên nhận trước khi chạy lại hoặc rollback.
Rollback có thể để lại thư mục cha rỗng.

Kiểm tra bundle cung cấp bằng chứng tĩnh. Native discovery cần quan sát Agent
nạp chỉ dẫn và skill tại worktree đích. Bundle và gói bàn giao đều giữ nguyên
quyền native và không đăng ký hook.

<a id="adapter-support"></a>
## Kiểm chứng trong client

Mở worktree đã chọn trong Codex Desktop, Claude Code hoặc Antigravity. Xác
nhận ngay trong client những chỉ dẫn và kỹ năng được nạp trước khi làm việc.
Với phiên di động, cung cấp rõ tệp đầu vào và ngữ cảnh đã tạo. Chỉ ghi nhận
việc nạp chỉ dẫn, dùng kỹ năng hoặc chạy hook sau khi quan sát trong client
đã cài.

Tài liệu Codex. [Chỉ dẫn](https://learn.chatgpt.com/docs/agent-configuration/agents-md) ·
[Skills](https://learn.chatgpt.com/docs/build-skills) · [Hooks](https://learn.chatgpt.com/docs/hooks) ·
[Tham chiếu CLI](https://learn.chatgpt.com/docs/cli/reference).

Tài liệu Claude Code. [Chỉ dẫn và thư mục bổ sung](https://code.claude.com/docs/en/memory) ·
[Skills](https://code.claude.com/docs/en/skills) · [Hooks](https://code.claude.com/docs/en/hooks) ·
[Tham chiếu CLI](https://code.claude.com/docs/en/cli-reference).

Tài liệu Antigravity. [Quy tắc](https://antigravity.google/docs/rules/) ·
[Cách tìm cấu hình](https://antigravity.google/docs/customization-discovery?tab=cli) ·
[Hooks](https://antigravity.google/docs/hooks?tab=ide) ·
[Quyền](https://antigravity.google/docs/permissions?tab=cli).

Với phiên cloud, cấp checkout nguồn đã duyệt trong môi trường từ xa và
phân giải ngữ cảnh ngay tại đó. Đường dẫn cục bộ chỉ nhận dạng máy đã tạo
ngữ cảnh. Cấu hình và hook cấp người dùng cần đợt triển khai được phê duyệt
riêng cùng kiểm thử cơ chế tự nạp trong môi trường cô lập.

<a id="mcp-launch"></a>

## Mẫu khởi chạy MCP

[Lệnh xuất MCP](../references/mcp.md) riêng tạo mẫu khởi chạy để review.
Sync chỉ dẫn và skill giữ hợp đồng quyền sở hữu hiện hành.
