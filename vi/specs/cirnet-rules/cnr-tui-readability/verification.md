<!-- doc-id: CNR-TUI-READABILITY-VERIFICATION -->
<!-- revision: 1 -->
<!-- source-sha256: 692571f0a6bf1c905c14161f8588e87378c0c524b9130a8fb4a7ea266630a122 -->
<!-- translation-status: synchronized -->
<a id="evidence"></a>

# Kiểm chứng Check và editor CNR

Lượt chạy ngày 2026-11-12 dùng Windows, Rust và Cargo 1.99.1, trên working tree của `master` tại HEAD `aca6afdbbb81f4996c98a0b22ab8d09e29aef58b`.

| Lệnh | Mã thoát cuối | Bằng chứng |
| --- | --- | --- |
| `cargo fmt --all --check` | 0 | Kiểm tra định dạng |
| `cargo clippy --locked --all-targets -- -D warnings` | 0 | Kiểm tra lint |
| `cargo check --locked --all-targets` | 0 | Kiểm tra mọi target |
| `cargo build --locked --bins` | 0 | Binary debug trong thư mục đích riêng |
| `cargo test --locked --all-targets` | 0 | 258 đạt, 1 bỏ qua |
| `cargo build --locked --release --bins` | 0 | Binary tối ưu |
| `git diff --check` | 0 | Kiểm tra khoảng trắng của diff |

Check ở baseline dùng dòng bốn cột với tối đa 201 ký tự trên workspace hiện tại. Báo cáo text tiếng Việt mới dài tối đa 84 ký tự mỗi dòng trong lượt này và không có ANSI. Cùng phép kiểm báo 41 mục gồm 16 PASS, 6 BLOCKED và 19 NOT_RUN. Tổng thể WARNING và mã thoát 0 khớp với JSON schema 1. Bindings báo BLOCKED, mã thoát 2. Agent-config báo ba NOT_RUN, mã thoát 2. MCP báo giao thức PASS và còn thiếu bằng chứng native.

Regression tests kiểm tra khung 40x12, 60x20, 80x24, 120x30 và 160x40, đổi kích thước và 135 finding có Unicode. Các test cũng kiểm tra lọc trạng thái và repository, chi tiết finding, loại JSON thô khỏi thao tác clipboard, vùng chọn theo cụm ký tự, undo giới hạn, dán quá giới hạn hoặc đa dòng, lỗi clipboard, vùng chuột và hủy. Test bỏ qua cần uv và Python bên ngoài.

Windows Terminal 1.24.12741.0 chạy binary release ở chế độ headless với `WT_SESSION`. `cnr check registry --json --non-interactive` trả PASS, schema 1 và mã thoát 0. Console PTY Windows riêng với `NO_COLOR` đã chạy danh sách tiếng Việt, chi tiết, bộ lọc trạng thái, JSON, tìm kiếm và khôi phục terminal khi thoát. API clipboard native có test codec và lỗi mô phỏng. Chưa quan sát việc host chuyển phím clipboard desktop, chuột phải vật lý, vùng chọn do host quản lý, IME hay Windows Terminal Preview. Runtime Linux và WSL chưa được chạy.

Lượt test baseline đến validator khi hash của kế hoạch mới còn đang được cập nhật. Bộ test cuối dùng tài liệu đã đồng bộ. Windows từ chối thay executable debug mặc định hiện hữu, nên lượt build debug và test đầy đủ thành công dùng thư mục đích mới.

[Phạm vi và quyết định giao diện](spec.md)
