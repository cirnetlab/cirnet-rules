<!-- doc-id: CNR-OPERATOR -->
<!-- revision: 9 -->
<!-- source-sha256: 4da0df9cdd0a62f4fb389c3ba0765d79cb9f981822eb1cb50be00dc9d871e0f9 -->
<!-- translation-status: synchronized -->
<a id="cnr-cli"></a>

# Hướng dẫn vận hành CNR

CNR là CLI CIRNET RULES cho governance, môi trường và công cụ phát triển.
Build bằng Rust và Cargo đã ghim của repository:

~~~sh
cargo build --locked --bin cnr
./target/debug/cnr --help
./target/debug/cnr --project . env plan --json
~~~

Trên Windows dùng `target/debug/cnr.exe`. Chạy `cnr` hoặc `cnr tui` trong
terminal tương tác. Bố cục rộng hiển thị lệnh đang chọn bên cạnh danh sách.
`/` hoặc Ctrl+K tìm trong menu. `?` mở trợ giúp theo ngữ cảnh.

Gõ để mở ô Lệnh hoặc dùng `:` để mở lại bản nháp chưa gửi. Gợi ý hiện
bên dưới ô nhập. Menu rộng đặt ô Lệnh ở cột mô tả. Terminal nhỏ đặt ô
Lệnh dưới danh sách menu và giảm số dòng gợi ý.
Tab nhận và duyệt gợi ý. Shift+Tab duyệt ngược. Enter gửi nội dung trong
ô Lệnh. Ở menu chính, Esc đóng ô Lệnh, còn phím mũi tên đóng ô và di
chuyển mục đang chọn. Gợi ý phiên bản lấy từ pin dự án và bản cài do
CNR quản lý đã được xác thực.

Menu giữ thanh phím tắt ở đáy. Màn hình kết quả hiển thị trạng thái,
mã thoát và thời gian thực tính bằng mili giây ngay trên ô Lệnh.
Trong kết quả, PgUp/PgDn và bánh xe chuột cuộn đầu ra. Ctrl+Home và
Ctrl+End chuyển đến đầu và cuối. Ctrl+F tìm trong đầu ra. F3 và Shift+F3
chuyển giữa các vị trí khớp. Mũi tên lên và xuống gọi lại các lệnh chỉ
đọc gần đây, còn Ctrl+R tìm theo tiền tố. Ctrl+C hủy lệnh đang chạy hoặc
thoát khi rảnh. Esc trở về menu từ màn hình kết quả khi lệnh đã dừng.

`--lang vi` chọn nhãn terminal và mô tả lệnh tiếng Việt. `--no-banner` dùng
tiêu đề gọn. `NO_COLOR` hiển thị văn bản và chữ khối đơn sắc, bỏ bóng đổ.
Lượt chạy phi tương tác in trợ giúp. Phiên bản, trợ giúp và danh mục chạy
được khi chưa có dự án. Mã nguồn executable ở `src/cli/` và test tích hợp ở `src/tests/cnr.rs`.

<a id="environment"></a>

## Cấu hình dự án

- `cnr init` tạo `cnr.toml` và giữ cấu hình hiện có. Chọn thư mục bằng
  `--project PATH` hoặc dấu hiệu dự án gần nhất.
- `cnr pin rust@1.99.1` ghim cho dự án. `cnr use rust@1.99.1` chọn
  phiên bản trong profile hiện hành. Xem và chỉnh bằng `config show`,
  `config get KEY`, `config set KEY VALUE` hoặc `config unset KEY`. Pin đã bỏ
  vẫn được lock cũ bảo vệ cho đến khi cập nhật lock. Các khóa gồm
  `profile`, `tools.rust` hoặc `profiles.dev.rust`.
- `cnr env plan` liệt kê yêu cầu native, phiên bản đã phân giải, xung đột
  và pin chưa phân giải. `cnr env lock` ghi phiên bản cụ thể và mã băm
  manifest native vào `cnr.lock`. `--locked` yêu cầu lock khớp và kiểm
  sai lệch. Lockfile dependency native vẫn giữ thẩm quyền.
- `cnr env sync --dry-run` lập kế hoạch theo phiên bản cụ thể đã chọn.
  Thêm `--yes` để xác nhận kế hoạch trong phạm vi dự án sau khi rà soát.
  Có thể dùng lại bước đã hoàn thành nếu bước sau lỗi. Dùng `env profiles`,
  `env export --json` và `env import PATH` để trao đổi cấu hình dự án.
  Lưu JSON xuất riêng với bộ công cụ đã cài.

<a id="tool-providers"></a>

## Provider và quyền sở hữu

`search QUERY`, `list` và `info TOOL` đọc cùng danh mục. Lọc bằng
`--type runtime|toolchain|sdk|package-manager|development-tool`,
`--installed` hoặc `--available`. Khả dụng ở đây là có trong danh mục
cục bộ. `doctor` báo điều kiện cục bộ và trạng thái môi trường dự án.

Rust, Python và Node lần lượt dùng `rustup`, `uv` hoặc `fnm` native
đã có. Cài đặt dùng phiên bản cụ thể và kho cục bộ của dự án. Rà soát
`cnr install TOOL@VERSION --dry-run`, rồi xác nhận bằng `--yes`.
`cnr update` xem trước. `update TOOL@VERSION` chọn một đích cụ thể.
`update --all` chọn các phiên bản cụ thể của môi trường hiện tại.
`remove TOOL[@VERSION]` kiểm tra quyền sở hữu, tham chiếu và lượt dùng
đang hoạt động. Thay đổi phạm vi người dùng hoặc hệ thống cần installer
được cấp quyền riêng.

Danh mục báo kết quả phát hiện cục bộ và trạng thái hỗ trợ. Dùng installer được phê duyệt nếu thao tác ghi native
chưa khả dụng. `--offline` giới hạn cài đặt vào thao tác đã thỏa mãn.
Dùng `cache` và `audit` để xem trạng thái và biên nhận cục bộ. Installer
ngoài có thể để lại bản ghi phục hồi khi lỗi. Chạy lại thao tác đã duyệt
hoặc xem thông tin phục hồi được báo.

<a id="execution"></a>

## Thực thi native và governance

Dùng `build`, `run` và `test` với engine được hỗ trợ, ví dụ
`cnr build rust --locked --offline --dry-run`. Dùng `package ENGINE ACTION`
cho thao tác dependency native được hỗ trợ và `exec -- PROGRAM ARG...`
cho executable tường minh. Rà soát kế hoạch rồi xác nhận thay đổi và
thực thi bằng `--yes`. Đối số sau dấu phân cách được truyền dưới dạng
giá trị, có kiểm tra nền tảng và chính sách.

Dùng `venv create NAME [VERSION]`, `venv list`, `venv info NAME`,
`venv run NAME -- ARG...` và `venv remove NAME`. Lượt chạy dùng
interpreter trong tiến trình con. Môi trường ảo gắn với dự án và interpreter.

Tùy chọn dependency native và thời hạn áp dụng khi engine đã chọn có thể
thực thi chúng. Build hook, script vòng đời package và lệnh tường minh
có thể hành động với quyền của tiến trình local. Bộ quản lý tiến trình
giới hạn output thu được, quản lý hủy và dọn dẹp.

Dùng `cnr workspace discover` và `cnr task show --task ID` để vận hành governance.
`--governance-root PATH` chọn checkout authority. Dấu phân cách tương thích
vẫn nhận tùy chọn bảo trì như `cnr task show -- --task ID`.
Với lệnh bảo trì, `--project PATH` chọn workspace sản phẩm.
Xem [quy trình vận hành](cirnetrules-local.md) về cấp quyền và kiểm chứng.
Release cần ủy quyền riêng.

`--json` trả một tài liệu JSON trên stdout. Output tiến trình native có
giới hạn và được gửi riêng vào stderr. Mã thoát 0 là thành công, 1 là lỗi
và 2 là bị chặn hoặc chưa chạy. Mã 130 là đã hủy. `child_exit_code` báo
mã của tiến trình con riêng. `cnr completion SHELL` xuất hoàn thành lệnh cho Bash, Zsh,
Fish hoặc PowerShell.

<a id="system-checks"></a>

## Kiểm tra hệ thống

Mở `cnr` rồi chọn System checks hoặc nhập `cnr check all` trong ô lệnh.
Chọn một mục như `cnr check bindings` để giới hạn phạm vi kiểm tra. Khi chạy
headless, `cnr check` mặc định dùng `all`. Dùng `--repo`, `--binding`, `--task`
và `--agent` để thu hẹp phạm vi bằng chứng. Kết quả phân biệt checkout phát
hiện được với binding đã xác nhận, cũng như giao thức MCP đã kiểm chứng với
phiên Agent native. Xem lý do và bước tiếp theo cho từng mục bắt buộc còn thiếu
bằng chứng.

Dùng `cnr check all --json --non-interactive` để nhận báo cáo có phiên bản.
Mã thoát 1 báo lỗi, mã 2 báo thiếu kiểm tra bắt buộc hoặc có cảnh báo ở chế độ
nghiêm ngặt, mã 130 báo đã hủy. Mã 0 cho biết các mục bắt buộc đã đạt và vẫn
có thể có mục tùy chọn chưa chạy. Check chỉ đọc trạng thái hiện có. Binding,
xuất bundle, đồng bộ và đăng ký được xử lý bằng các thao tác thiết lập riêng.

<a id="guided-agent-setup"></a>

## Thiết lập Agent có hướng dẫn

Trong `cnr`, chọn Agent setup hoặc nhập `cnr setup agent`. Chọn Codex, Claude
Code hoặc Antigravity, rồi rà soát Governance root, workspace, repository và
binding. Checkout tìm thấy cần có binding được rà soát. Nếu có nhiều binding,
hãy chọn rõ một binding. Lệnh headless `cnr setup agent` yêu cầu `--agent` và
báo ngữ cảnh đã chọn.

Xem trước và xác nhận bundle hướng dẫn ở thư mục mới nằm ngoài Git và cấu hình
Agent native. Manifest liệt kê profile, skill và tài liệu tham chiếu được chọn.
Khi chạy headless, `cnr agent export --dry-run` nhận các lựa chọn xuất và đường
dẫn đầu ra. Có thể áp dụng bằng digest `--expect` vừa duyệt. Bundle xuất với
`--binding` cố định binding đã chọn và Git HEAD của worktree đó. Truyền cùng
binding vào `agent check` hoặc `agent sync` nếu muốn chọn tường minh.

Rà soát kế hoạch đồng bộ cho đúng worktree. Nếu `AGENTS.md` hoặc `CLAUDE.md`
của người dùng chặn strict sync, chọn xem trước managed block. Bản xem trước
hiển thị nội dung block đề xuất và digest dữ liệu gốc, không in nội dung người
dùng. Đọc hướng dẫn mới cùng hướng dẫn local trước khi duyệt. Lần tiếp nhận đầu
thêm một block riêng theo ký tự xuống dòng đầu tiên tìm thấy, hoặc dùng LF nếu
chưa có xuống dòng. UTF-8 BOM hiện có và toàn bộ byte ban đầu được giữ nguyên.
Có thể chọn chuyển đổi từ biên nhận quản lý toàn file đã kiểm chứng. Lần cập
nhật chỉ sửa block đã xác minh. Marker sai cấu trúc hoặc bị sửa cần rà soát
local. Skill và tài liệu tham chiếu vẫn dùng cơ chế quản lý file nghiêm ngặt.
Codex và Antigravity dùng chung một block `AGENTS.md` và không nhân đôi đường
dẫn `.agents/skills` khi nội dung giống nhau.

Chọn task hiện hành và operator đã khai báo trước khi xác nhận ghi. Bản xem
trước managed block gắn với revision của task, worktree đích, bundle và dữ liệu
gốc. Khi chạy headless, dùng `--entry-mode managed-block --dry-run --task` với
ID task và `--actor` với operator, sau đó áp dụng digest đã duyệt qua
`--yes --expect`. Dùng `agent rollback` với biên nhận rồi rà soát bản xem trước
rollback. Nếu entry thay đổi sau khi đồng bộ, rollback dừng và giữ nguyên byte
hiện tại để rà soát local. Xem `cnr audit agent --receipt` với ID biên nhận
trước khi phục hồi thao tác bị gián đoạn.

Xuất bundle khởi chạy MCP, rồi rà soát đăng ký phạm vi project hoặc làm theo
hướng dẫn nhập thủ công cho Agent và phiên bản ứng dụng đã chọn. Chạy kiểm tra
giao thức trong CNR. Việc Agent native phát hiện skill, nạp cấu hình và gọi
tool cần bằng chứng riêng từ phiên Agent đã chọn.

<a id="mcp-coordination"></a>

## MCP và phối hợp chung

Xem [MCP STDIO](mcp.md) để cấu hình Agent và chọn profile tool.
[Quy trình phối hợp](shared-task-coordination.md) mô tả binding có tên, lease,
báo cáo, audit local và gate candidate có thể di chuyển.
