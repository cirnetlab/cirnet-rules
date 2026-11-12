<!-- doc-id: CNR-GUIDED-VERIFICATION -->
<!-- revision: 1 -->
<!-- source-sha256: 7d9f4887ca7ceaacbd4a5c53f864c7219cc32ae6df88aef0bf9e08ca0577a14c -->
<!-- translation-status: synchronized -->
<a id="evidence"></a>

# Xác minh kiểm tra và thiết lập Agent trong CNR

Lượt chạy ngày 2026-11-12 dùng Windows, Rust và Cargo 1.99.1, trên working tree chưa commit của `master` tại HEAD `f14da9a6a8e41f89570f1ecf1d6d7a5a93511aeb`.

| Lệnh | Mã thoát | Kết quả |
| --- | --- | --- |
| `cargo fmt --all --check` | 0 | Định dạng đạt |
| `cargo check --locked --all-targets` | 0 | Kiểm tra mọi target đạt |
| `cargo clippy --locked --all-targets -- -D warnings` | 0 | Kiểm tra lint đạt |
| `cargo build --locked --bins` | 0 | Build được binary debug |
| `cargo test --locked --all-targets` | 0 | 233 đạt, 1 bỏ qua |
| `cargo build --locked --release --bins` | 0 | Build được binary tối ưu |
| `git diff --check` | 0 | Diff đạt kiểm tra khoảng trắng |

Baseline tái hiện `--target is required`. Đường dẫn Unicode đã đặt trong dấu nháy đi đến bước xác minh thư mục. Regression tests bao phủ trường thiếu có cấu trúc, gợi ý, dấu nháy, hủy, freshness của binding, bảo toàn nội dung entry, xuất theo named binding, từ chối thiếu grant, đăng ký có phạm vi và rollback trên ba adapter. Test bị bỏ qua yêu cầu uv và Python bên ngoài.

Binary mới báo `WARNING` và mã thoát 0 cho `cnr check all` trong workspace hiện tại, với sáu mục binding cần review. `cnr check mcp --agent all` trả mã thoát 0 và bằng chứng giao thức qua STDIO thật. Hai vòng đời giao thức MCP được hỗ trợ đều đạt wire tests. Trong terminal Windows, trình hướng dẫn tiếng Việt với `NO_COLOR` mở được, chọn Agent, chạy probe, xem chi tiết báo cáo và thoát bình thường.

Native discovery và tuân thủ trong phiên Codex, Claude Code và Antigravity vẫn là `NOT_RUN`. Cơ chế ghi cấu hình dự án được chạy trong fixture. Phạm vi người dùng và Antigravity App dùng nhập thủ công có hướng dẫn. Runtime Ubuntu và kích hoạt release ngoài phạm vi xác minh này.

[Phạm vi](spec.md)
