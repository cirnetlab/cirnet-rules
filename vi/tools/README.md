<!-- doc-id: TOOLS -->
<!-- revision: 2 -->
<!-- source-sha256: 8a4b48db525c732e2b52875c83dae8eb4c4061d33f15ccb8f4534cf51ba38f3d -->
<!-- translation-status: synchronized -->

<a id="commands"></a>

# Công cụ quản trị Rust

Dùng Rust và Cargo 1.99.1 cùng Git. Task WORK cần các checkout đã liên kết
trong phạm vi. Build một lần rồi chạy binary cho các bước sau. Mã 0 là
PASS, 1 là lỗi chính sách hoặc đầu vào và 2 là bị chặn hoặc chưa chạy.

~~~sh
cargo build --locked
./target/debug/cirnet-governance workspace-discover
./target/debug/cirnet-governance validate
./target/debug/cirnet-governance check-translations --paths en/routing/README.md
~~~

Trên Windows binary có đuôi exe. Cargo target là src/main.rs và
src/tests/cli.rs. Schema ở en/schemas. Xem [quy trình local](../references/openrules-local.md)
để tạo, thực thi, nghiệm thu task và chọn chế độ release tường minh.
Các lệnh context, doctor, version và drift cũ vẫn dùng cho audit lịch sử.
Luồng thường dùng scoped health và task context.
