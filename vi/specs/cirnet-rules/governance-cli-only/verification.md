<!-- doc-id: CNR-CLI-ONLY-VERIFICATION -->
<!-- revision: 2 -->
<!-- source-sha256: 2c3933e8d2676e485ae4ba6a94963859f3de0ab749795be129f4e9d22d1a0f31 -->
<!-- translation-status: synchronized -->
<a id="verification"></a>

# Kiểm chứng CLI-only

## Phạm vi kiểm chứng

Hồ sơ ghi lượt kiểm chứng triển khai CLI-only trên Windows x64 MSVC với
Rust và Cargo 1.99.1. Commit gốc của đợt triển khai là
`52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c`. Số lượng test và biên nhận bên
dưới thuộc lượt chạy đó. Xem [hướng dẫn kiểm chứng hiện hành](../../../references/verification.md)
để chạy các bước hiện tại.

## Kết quả Windows

Các lệnh sau đều thoát mã 0 với toolchain đã pin.

```text
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo check --locked --all-targets
cargo build --locked --bins
cargo test --locked --all-targets
cargo build --release --locked --bin cnr
```

Bộ mặc định có 101 test đạt và một test opt-in bị bỏ qua. Fixture `uv` opt-in
chạy riêng ở chế độ offline đã đạt. Validator tương thích và kiểm tra bản
dịch đều xác minh 109 cặp EN/VI. Help, version JSON, kiểm kê authority, phân
giải context và audit tooling của CNR chạy thành công. Fixture thiếu nguồn
SQLite tường minh trả `NOT_FOUND` với mã thoát 2.

Bộ migration đạt bảy test. Fixture WAL schema 7 bảo toàn bốn asset, ba
revision và một draft. Fixture cũng giữ assignment, default, hồ sơ audit
và snapshot lịch sử đã ghim. Byte của database, WAL và SHM nguồn khớp
trước và sau khi xuất. Các test preview, tiếp tục, rollback và xung đột
đường dẫn dùng fixture dùng một lần. Cả bảy tệp SQL lịch sử khớp digest byte
khi preflight.

Xuất và kiểm tra bundle chung cho ba Agent đều thoát mã 0 và ghi
`STATIC_EXPORT_VALIDATED`. Bundle staging gồm 18 tệp được quản lý và manifest.
Xuất lặp lại trả `UNCHANGED`. Sync và rollback dùng Git fixture đã bind
với grant task theo từng tệp.

## Giới hạn bằng chứng

Kiểm thử migration dùng SQLite và Git fixture dùng một lần. Migration trên
dữ liệu thật, nạp cấu hình trong agent, chạy hook, chấp nhận có xác thực và
kích hoạt release ở trạng thái `NOT_RUN`. Lượt này ghi `NOT_RUN` cho CI từ
xa, Linux cục bộ, fixture symlink Unix và kiểm tra junction thật trên Windows.
