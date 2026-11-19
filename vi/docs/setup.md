# Thiết lập một client

Chọn bundle local đã review và một client. Giữ chỉ dẫn local áp dụng của repo
sản phẩm. Baseline tiếng Anh độc lập và 13 thư mục skill cài được riêng.
Bản Việt dành cho dịch và review. Xem [nạp context](context.md) để chọn phần đọc.

## Cài từ bundle local đã giải nén

Dùng PowerShell 7 và script trong checkout toolkit đã review. Đặt `$bundle`
là thư mục đã giải nén chứa `bundle.json` và chọn `$agent` là
`Codex`, `Claude` hoặc `Antigravity`. Lệnh đầu xem trước đường dẫn chính xác.
Sau review, người dùng có thể chạy lệnh apply.

```powershell
./scripts/install.ps1 -Source $bundle -Agent $agent
./scripts/install.ps1 -Source $bundle -Agent $agent -Apply
./scripts/verify-install.ps1 -Source $bundle -Agent $agent
```

Mặc định chọn `cirnet-implementation`, `cirnet-debugging` và
`cirnet-verification`. Dùng cùng mảng `-Skills` khi preview, apply và verify
một lựa chọn khác. Ghi rõ tên bổ sung. Preview và `-Apply -WhatIf` giữ nguyên
đích. Khi test hoặc dùng vị trí riêng, truyền `-HomeRoot` rõ ràng và
`-CodexHome` nếu cần.
Dùng `-Skills @()` trong PowerShell để chỉ chọn baseline và adapter.

| Client | Đích baseline user | Thư mục skill user |
| --- | --- | --- |
| Codex | `$CODEX_HOME/AGENTS.md` hoặc `~/.codex/AGENTS.md` | `~/.agents/skills/` |
| Claude Code | `~/.claude/CLAUDE.md` import `@cirnet/AGENTS.md` | `~/.claude/skills/` |
| Antigravity | `~/.gemini/AGENTS.md` | `~/.gemini/config/skills/` |

Với Codex, `-CodexHome` rõ ràng được ưu tiên. Khi truyền `-HomeRoot`,
mặc định dùng `.codex` bên trong và tách home test hoặc home tùy chọn khỏi
`CODEX_HOME` môi trường. Nếu thiếu cả hai, script dùng override môi trường
hoặc đích mặc định. `AGENTS.override.md` không rỗng của Codex được ưu tiên.
Xem [chỉ dẫn Codex](https://developers.openai.com/codex/guides/agents-md)
và [skill Codex](https://developers.openai.com/codex/skills).

Claude giải import managed tương đối từ file chứa import. Kiểm tra discovery
theo version và surface thực tế. Xem [import Claude](https://code.claude.com/docs/en/memory)
và [skill Claude](https://code.claude.com/docs/en/skills).

Với Antigravity, kiểm tra discovery và thứ tự ưu tiên của App, CLI hoặc IDE.
Tài liệu dẫn mô tả đích user ở trên và ranh giới discovery workspace.
Đọc chỉ dẫn lồng nhau FPGA rõ ràng nếu surface đã chọn không tự nạp.
Xem [quy tắc Antigravity](https://antigravity.google/docs/rules/)
và [skill Antigravity](https://antigravity.google/docs/skills?app=antigravity).

Xác định home trong môi trường client chạy, kể cả WSL, container và phiên remote.
Với project scope, cùng owner điều chỉnh [mẫu chỉ dẫn local](../templates/AGENTS.md)
và dùng quy ước skill project theo version client.

## Đọc checkout local được cho phép

Yêu cầu client đọc `AGENTS.md` root của checkout local rồi đọc chỉ dẫn áp dụng
của repo đích. Cung cấp path và quyền đọc khi cần. Chọn rõ một skill tiếng Anh
nếu hữu ích. Lặp yêu cầu ở phiên mới chưa có baseline cấu hình.

Với công việc Codex hoặc Claude Code dùng Task Continuity, cung cấp cho phiên
checkout toolkit đã duyệt, worktree sở hữu, Task ID và vị trí checkpoint. Xem
[thiết lập và lệnh Task Continuity](distribution.md#task-continuity) để chọn
kho cục bộ được Git bỏ qua và khôi phục công việc sau gián đoạn.

## Kiểm tra discovery thực tế

Xác minh file kiểm tra đích được chọn, nội dung managed, reference và xung đột
user scope đã biết. Các nguồn khác như plugin và project override cần review
trong client thực tế. Discovery native và hành vi task có kết quả riêng.
Mở phiên mới trên surface thực tế của client,
xác nhận discovery baseline và skill, rồi thử phần chỉ đọc của task có phạm vi.

| Client | Quan sát thủ công |
| --- | --- |
| Codex | Kiểm tra chuỗi chỉ dẫn và metadata skill trong CLI/App/IDE thực tế. Thử override và scope lồng nhau trong fixture. |
| Claude Code | Xác nhận một import giải được và skill đã chọn có mặt trong project dự kiến. |
| Antigravity | Xác nhận rule user và Skills ở surface đã chọn, yêu cầu context lồng nhau liên quan nếu cần. |

Với cả ba, chọn một skill liên quan, kiểm tra rule local/lồng nhau và báo phép
kiểm tra thực sự chạy. Task này quan sát hành vi file trong home giả.
Discovery và hành vi native là NOT_RUN. Xem [phân phối](distribution.md)
để cập nhật và restore.

<!-- source: en/docs/setup.md -->
<!-- source-sha256: 2f976be112dcba7850f2ae0543eddfed8b23c6986ae814aae43aa29e3604dc93 -->
