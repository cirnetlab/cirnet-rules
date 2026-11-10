<!-- doc-id: TOOLS -->
<!-- revision: 3 -->
<!-- source-sha256: 6d27850e7db712ecb876c3e3e40070c8785aaabfcc044e9dec1cc45e69eeaddf -->
<!-- translation-status: synchronized -->

<a id="commands"></a>

# Công cụ quản trị Rust

Dùng [CNR](../references/cnr.md) để quản lý runtime, SDK và công cụ phát triển.
Binary bảo trì vẫn phục vụ các task governance có phạm vi.

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
src/tests/cli.rs. Schema ở en/schemas. Xem [quy trình local](../references/cirnetrules-local.md)
để tạo, thực thi, nghiệm thu task và chọn chế độ release tường minh.
Các lệnh context, doctor, version và drift cũ vẫn dùng cho audit lịch sử.
Luồng thường dùng scoped health và task context.
