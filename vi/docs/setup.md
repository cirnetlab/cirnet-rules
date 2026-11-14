# Thiết lập thủ công

Dùng **một agent được chọn** tại repository đích. Repo common không tự được đọc từ
GitHub hoặc một checkout khác. Chỉ dẫn có tác dụng khi được đặt đúng scope hoặc
được đọc rõ ràng trong phiên có quyền truy cập. Không cần tải rules qua mạng mỗi task.

Tài liệu được đối chiếu ngày **2026-11-14**. Discovery có thể khác theo phiên bản,
App/CLI/IDE, cấu hình override và môi trường local/remote. Đây là hướng dẫn cài sau
này; lần rebuild ban đầu không cài gì vào user scope. Native loader/E2E của cả ba client:
**NOT_RUN**. Xem [kết quả kiểm tra rebuild ban đầu](rebuild-verification.md).

Trong repository này, soạn baseline tại `en/AGENTS.md` và giữ `AGENTS.md` ở root
là bản phân phối giống hệt. Cài đặt user/global tiếp tục dùng bản ở root này.
Ngữ cảnh hỗ trợ cho agent nằm trong `en/`; `vi/` chứa bản dịch để con người review,
không phải chỉ dẫn bổ sung cho agent. Xem quy trình song ngữ hiện tại và hook kiểm
tra tài liệu của repository trong [chính sách tài liệu](../policies/documentation.md).
Hook là kiểm tra của repository; hướng dẫn này không cài hook vào vòng đời native
của client.

## A. Copy/sync có kiểm soát vào user scope

1. Chọn checkout và revision đã review. Đọc diff của bản common và các thay đổi
   local trước khi dùng. Không tự pull hoặc cập nhật theo nhánh chưa review.
2. Kiểm tra file đích, override, preferences và policy đang có. Tạo bản backup
   tên riêng có ngày/giờ; xác nhận backup đọc được trước khi sửa.
3. Chuẩn bị bản hợp nhất ở file tạm, xem diff. Giữ preferences riêng, bỏ phần common
   cũ đã được thay thế để tránh hai baseline mâu thuẫn. Chỉ áp dụng sau khi review.
4. Giữ nguyên `AGENTS.md`/adapter của repo sản phẩm. Không copy common đè lên nó.
5. Ghi lại source commit, ngày và SHA-256 của file đã review trong ghi chú cài đặt
   cá nhân. Với source chưa commit, ghi rõ working-tree hash, không chỉ ghi HEAD.
6. Mở phiên mới trong repo đích, kiểm tra nguồn chỉ dẫn được nạp và một task chỉ đọc.
   Nếu không nạp được, dùng cách B và kiểm tra tài liệu của đúng client.

Các lệnh dưới đây **chỉ đọc**, chạy từ checkout common. Chúng không cài cấu hình:

```powershell
git rev-parse HEAD
git status --short
git diff -- en/AGENTS.md AGENTS.md
Get-FileHash -Algorithm SHA256 -LiteralPath .\AGENTS.md
```

Trên Linux/macOS, dùng `sha256sum AGENTS.md` hoặc `shasum -a 256 AGENTS.md`
nếu đã có sẵn. Không cần cài tool chỉ để xem file.

### Codex

User scope mặc định là `~/.codex/AGENTS.md`; `CODEX_HOME` có thể đổi vị trí.
`AGENTS.override.md` không rỗng ở cùng scope được ưu tiên. Repo-local được nạp
theo đường từ root tới thư mục làm việc; kiểm tra cả nested instructions cần thiết.
Xem [hướng dẫn AGENTS.md chính thức](https://developers.openai.com/codex/guides/agents-md)
và [prompting guide](https://developers.openai.com/cookbook/examples/gpt-5/codex_prompting_guide).

### Claude Code

Dùng `CLAUDE.md` làm điểm nạp rõ ràng; không dựa vào riêng `AGENTS.md` khi chưa
kiểm tra discovery của phiên bản/cấu hình đang dùng. Tài liệu loader có thể khác
giữa các phiên bản. Adapter vẫn dùng chung một nguồn nội dung.
Trong checkout này, [CLAUDE.md](../../CLAUDE.md) import `@AGENTS.md`.
Với user scope, đặt bản common đã review tại `~/.claude/AGENTS.md`,
rồi hợp nhất dòng sau vào `~/.claude/CLAUDE.md`:

```text
@AGENTS.md
```

Import tương đối tính từ file chứa import. **Phải có AGENTS.md cạnh CLAUDE.md**;
copy adapter một mình sẽ trỏ tới file thiếu. Giữ preferences hiện hữu bên cạnh
import, không nhân bản toàn bộ common. Khi được phép sửa adapter tại repo đích,
import phải trỏ tới đúng chỉ dẫn local của repo ấy. Không cần symlink.
Xem [Claude memory/imports](https://code.claude.com/docs/en/memory).

### Antigravity

Tài liệu đã đối chiếu nêu `~/.gemini/AGENTS.md` cho global và `AGENTS.md` cho workspace.
Không tạo thêm `GEMINI.md` trùng nội dung khi client hỗ trợ AGENTS.
Kiểm tra đúng surface và danh sách Rules trong client; không suy ra mọi bản cũ
có cùng discovery. Theo tài liệu discovery, mở repo root không tự nạp
`fpga/AGENTS.md` bên dưới: hãy đọc file đó rõ ràng khi làm FPGA.
Xem [Rules](https://antigravity.google/docs/rules/) và
[Customization discovery](https://antigravity.google/docs/customization-discovery/).
Lần đối chiếu này đọc được nội dung qua kết quả tìm kiếm chính thức; tải trực tiếp
hai trang Rules/discovery trả cache miss/404. Native behavior chưa được xác minh.

### Vị trí trên hệ điều hành

| Client | Windows, home mặc định | Linux/macOS |
| --- | --- | --- |
| Codex | `%USERPROFILE%\.codex\AGENTS.md` | `~/.codex/AGENTS.md` |
| Claude | `%USERPROFILE%\.claude\CLAUDE.md` và `AGENTS.md` cạnh nó | `~/.claude/CLAUDE.md` và `~/.claude/AGENTS.md` |
| Antigravity | `%USERPROFILE%\.gemini\AGENTS.md` | `~/.gemini/AGENTS.md` |

Trong PowerShell, `$env:USERPROFILE` là cách tham chiếu home; `%USERPROFILE%`
ở bảng là ký hiệu đường dẫn Windows, không phải cú pháp PowerShell.
Với WSL/container/remote, xác định home của môi trường chạy agent, không mặc định
nó dùng cấu hình Windows host. Tuân theo override của client đang dùng.

## B. Đọc checkout local trong phiên

Không thay cấu hình global. Thay placeholder bằng đường dẫn thật của máy:

> Đọc `<common-checkout>/AGENTS.md` làm baseline, sau đó đọc chỉ dẫn local
> liên quan trong `<product-checkout>`. Chỉ sửa các file của task trong repo
> sản phẩm được cấp phép; checkout common chỉ đọc. Báo nếu không truy cập được.

Ở bố cục nhiều worktree, common có thể nằm tại
`<repo-root>/Branch/cirnet-rules/rules-common/AGENTS.md`.
Cấp quyền đọc file theo client nếu cần; quyền đọc không cấp quyền ghi.
Không dựa vào đường dẫn đĩa hoặc username của một PC cố định.
Cách này cần nhắc lại ở phiên mới nếu common chưa nằm trong scope tự nạp.

## Skills tùy chọn

Ba thư mục `en/skills/implementation`, `en/skills/debugging`, `en/skills/verification`
đều chỉ chứa một `SKILL.md` tự đủ nghĩa. Đọc trực tiếp theo đường dẫn khi cần,
hoặc review và cài thủ công **thư mục skill được chọn**, không bắt cài cả ba.
Chỉ cài skill tiếng Anh từ `en/skills/`. Các bản tương ứng trong `vi/skills/`
là bản dịch để con người review, không phải skill bổ sung để cài cho agent.

| Client | User scope được tài liệu mô tả | Project scope |
| --- | --- | --- |
| [Codex skills](https://developers.openai.com/codex/skills/) | `~/.agents/skills/<name>/SKILL.md` | `.agents/skills/<name>/SKILL.md` |
| [Claude skills](https://code.claude.com/docs/en/skills) | `~/.claude/skills/<name>/SKILL.md` | `.claude/skills/<name>/SKILL.md` |
| [Antigravity skills](https://antigravity.google/docs/skills?app=antigravity) | `~/.gemini/config/skills/<name>/SKILL.md` | `.agents/skills/<name>/SKILL.md` |

Kiểm tra hỗ trợ theo version/surface thực tế trước khi cài. Tên chung có thể trùng
skill đã có: preview/backup và quyết định bản dùng, không tự ghi đè.
`en/skills/` ở checkout common không tự xuất hiện trong workspace sản phẩm.
Không cần cài `en/rules/`, references hay task records vào global để baseline hoạt động.

## Ba tình huống sử dụng

- **Sửa typo:** đọc baseline đã nạp, quy tắc local và đoạn tài liệu liên quan;
  sửa hẹp, kiểm tra link/diff. Không quét sáu repo hoặc tạo plan riêng, trừ khi
  local yêu cầu plan/spec tại vị trí cụ thể.
- **Đổi API:** xem source API, Contracts canonical và version SDK consumer đang chọn.
  Không sửa schema hoặc generated SDK ngay trong consumer để bỏ qua owner;
  nếu cần thay repo khác, báo dependency và giới hạn quyền ghi.
- **Đổi FPGA:** đọc `fpga/AGENTS.md`, hướng dẫn verification và RTL liên quan.
  RTL PASS chỉ chứng minh hành vi mô phỏng đã kiểm tra; không chứng nhận an toàn
  điện, board programming hoặc physical firmware. Không cấp nguồn theo suy đoán.

<!-- source: en/docs/setup.md -->
<!-- source-sha256: 1b426c98eddb872dbbd9b3f9e7240b861d844213a063fc327816e2045f213549 -->
