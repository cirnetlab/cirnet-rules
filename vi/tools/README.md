<!-- doc-id: TOOLS -->
<!-- revision: 1 -->
<!-- source-sha256: f02306439c3435f2966763b51b29fdd55d1f9926c3e4e61acb9ec6ab150a01f0 -->
<!-- translation-status: synchronized -->

<a id="commands"></a>

# Công cụ quản trị Rust

Dùng Rust và Cargo 1.99.1, Git và workspace sản phẩm có sáu checkout cùng cấp
đọc được. MSRV trong metadata Cargo khác với toolchain chính xác. Lệnh trả
JSON. Exit 0 là đạt phép kiểm yêu cầu, 1 là lỗi chính sách hoặc đầu vào, 2 là
bị chặn hoặc chưa chạy.

~~~sh
cargo run --locked -- validate
cargo run --locked -- check-translations
cargo run --locked -- doctor --workspace ..
cargo run --locked -- context --workspace .. --repo cirnet-remote --task audit
cargo run --locked -- check-versions --workspace ..
cargo run --locked -- check-drift --workspace ..
cargo run --locked -- check-release --workspace ..
~~~

Chạy từ checkout governance hoặc truyền path bằng tùy chọn governance.
Lệnh sản phẩm chỉ đọc và không tải hoặc build sản phẩm. Context ghi commit,
trạng thái dirty và hash bytes tiếng Anh đã chuẩn hóa. Đặt target và cache
Cargo bên ngoài khi chạy cô lập. Xem [adapter](../adapters/README.md) và
[schema phát hành](../schemas/README.md).
