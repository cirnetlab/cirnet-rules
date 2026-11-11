<!-- doc-id: CNR-MCP-COORDINATION-VERIFICATION -->
<!-- revision: 1 -->
<!-- source-sha256: 78cd0d421f65a0dc2cd4aad4cf1733a911bf520a0bc5861ea295eaa32ad97741 -->
<!-- translation-status: synchronized -->
<a id="verification"></a>

# Kiểm chứng phối hợp task dùng chung

## Baseline và bảo toàn

Linked checkout triển khai ở
328848abc3dda7e5fce8024d601c4eeeaa83cd83. Ref lưu trữ bắt đầu và kết thúc ở
52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c. Nhiệm vụ để trống index Git và
giữ ZIP lưu trữ cùng thay đổi ignore-file đồng thời.

Trên Windows x64 MSVC, rustc 1.99.1 và cargo 1.99.1 đã pin chạy đạt 101
test baseline. Đã tái hiện quan sát SQL của source audit. Cả hai Git blob
của migration 007 đều là biến thể LF 3.283 byte. Fixture schema 7 chấp nhận
checksum Windows mixed và LF đã biết. Checksum lạ và schema bị sửa đều bị từ chối.

Ba vị trí SQLite mặc định của đời hiện tại và hai đời trước trả
`NOT_FOUND`. Chưa biết vị trí tùy chỉnh. Archive, staging đề xuất và
tiếp nhận dữ liệu thật ở trạng thái `NOT_RUN`.

## Triển khai và kiểm chứng

Các lệnh sau thoát mã 0 trong linked worktree. Cache dependency Cargo
được cô lập cho nhiệm vụ.

```text
cargo fmt --all --check
cargo clippy --locked --offline --all-targets -- -D warnings
cargo check --locked --offline --all-targets
cargo build --locked --offline --bins
cargo test --locked --offline --all-targets
cargo build --release --locked --offline --bin cnr
```

Bộ mặc định đạt 128 test. Fixture `uv` opt-in chạy riêng ở chế độ offline
cũng đạt. Test mới bao phủ linked worktree có tên, phát hiện binding và branch
stale, xung đột writer cùng worktree, tiến trình claim và cập nhật đồ thị tranh
chấp, fencing khi hết hạn và khôi phục, biên nhận idempotent, source drift,
redaction báo cáo, migration hồ sơ cũ, pin và tiêu thụ artifact producer.

Bộ STDIO Windows đạt chín test với `rmcp` 3.6.0 và Tokio 1.54.1.
Initialize trước đây và discovery hiện hành chạy qua tiến trình CNR thật.
Fixture phối hợp dùng CLI và hai subprocess MCP trên một kho task, niêm phong
một tệp đổi, xuất bằng chứng có thể di chuyển và xác minh Git candidate sau đó.
Các fixture wire khác kiểm tra chặn ghi trong profile chỉ đọc, frame lỗi và
quá lớn, phân trang, hủy, EOF và stdout không được đọc. Xuất và kiểm tra tĩnh
ba mẫu khởi chạy đều thoát mã 0, ghi `STATIC_CONFIG_VALIDATED`. Trạng thái
fixture giao thức là `MCP_PROTOCOL_VERIFIED`.

CNR help, validate và inspect authority, kiểm tra bản dịch, danh sách Agent,
MCP help và danh sách binding có tên đều thoát mã 0. Catalog giữ 32 asset.
Trạng thái task và sự kiện audit dùng chung `.cirnetrules/tasks`. Credential
phiên được chuẩn bị nằm trong thư mục phiên local. Gate CI trả biên nhận JSON
gắn với bundle đã review, authority tin cậy và candidate thực tế. Patch triển
khai đổi 57 tệp, kèm sáu tài liệu task EN/VI ở thư mục trung tâm.

## Giới hạn bằng chứng

Kiểm tra phiên bản ghi nhận Codex CLI `0.167.0-alpha.3.2` và launcher
Antigravity IDE `1.107.0`, cùng cảnh báo quyền OS. Claude Code và Antigravity
CLI không có trên PATH. Kết nối và discovery native ở trạng thái `NOT_RUN`.
Các mẫu khởi chạy dùng để review.

Kiểm tra WSL trả `Wsl/E_ACCESSDENIED`. Linux local và hosted CI ở trạng thái
`NOT_RUN`. Gate tái sử dụng kiểm tra một candidate task lá đã niêm phong theo
digest review cấp riêng và commit authority tin cậy, sạch. Build sản phẩm,
HIL, kích hoạt release, chấp nhận độc lập có xác thực và triển khai branch
protection ở trạng thái `NOT_RUN`.
