<!-- doc-id: CNR-OPERATOR -->
<!-- revision: 5 -->
<!-- source-sha256: c972b2fb9400682b934de79b202f21615f8004b0b9d9e9c5d69ba86222f26b38 -->
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
`/` mở tìm kiếm và `?` mở trợ giúp theo ngữ cảnh ở cuối màn hình.
Khi một lệnh nhận đầu vào, ví dụ xuất hiện ở đó. Để trống trường nhập để
dùng ví dụ. Có thể dùng bàn phím để di chuyển trong danh sách, chọn lệnh,
đóng trường đang mở hoặc thoát.

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

CMake, Ninja, Qt, MSVC, Windows SDK và các công cụ khác có phát hiện và
trạng thái hỗ trợ. Dùng installer được phê duyệt nếu thao tác ghi native
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

<a id="mcp-coordination"></a>

## MCP và phối hợp chung

Xem [MCP STDIO](mcp.md) để cấu hình Agent và chọn profile tool.
[Quy trình phối hợp](shared-task-coordination.md) mô tả binding có tên, lease,
báo cáo, audit local và gate candidate có thể di chuyển.
