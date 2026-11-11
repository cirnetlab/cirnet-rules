<!-- doc-id: CNR-CLI-ONLY-VERIFICATION -->
<!-- revision: 1 -->
<!-- source-sha256: 010d2f7414699a2a1e3b07c39705e833ed87666e9e59b4b5682d6f54545a9117 -->
<!-- translation-status: synchronized -->
<a id="verification"></a>

# Kiểm chứng CLI-only

## Baseline

Nhánh lưu trữ và nhánh triển khai ban đầu ở 52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c. Windows x64 MSVC báo rustc 1.99.1 (58cf8f9c3) và cargo 1.99.1 (5f94df478). Lượt cargo test baseline thoát mã 1 sau ba lỗi probe loopback tích hợp cũ và một lỗi kỳ vọng ACL Windows của runtime riêng. Các test sau đó chưa được quan sát trong lượt này. Tệp zip bị ignore và thư mục build nằm ngoài thay đổi source. Không tìm thấy database SQLite và tệp phụ trợ của đời hiện tại và hai đời trước tại vị trí mặc định. Chưa kiểm kê vị trí dữ liệu tùy chỉnh.

## Các test được thay thế

| Fixture runtime trước đây | Độ phủ trong kiến trúc CLI-only |
| --- | --- |
| Draft, khởi động lại, công bố, rollback và gián đoạn ghi SQL | Archive giữ các revision và bản nháp. Staging và digest đã review kiểm soát việc lưu đề xuất. Git giữ lịch sử nguồn đã duyệt. |
| Traversal, ghi stale, dependency, nội dung có cấu trúc và công bố nâng quyền | Kiểm tra catalog, freshness nội dung, archive migration và các cổng phạm vi task hiện có. |
| Ghim profile và binding cho hội thoại | Archive giữ snapshot đã ghim để review. Phân giải hiện hành kiểm tra Git binding và phạm vi task. |
| Xuất revision đã công bố và preview nhập chính xác | Archive kiểm kê mọi revision và bước nhập so digest đã review. |
| Alias và tệp đính kèm thực thi | Review legacy báo tài nguyên không an toàn. Bundle Agent bảo vệ đường dẫn có quản lý và metadata thực thi. |
| Schema giao tiếp riêng và tương tác desktop | Các giao diện này ngừng cùng runtime. Test tương thích CLI giữ envelope bảo trì. |
| Probe engine, hội thoại và lịch chạy, tùy chọn, UI và IPC | Gỡ cùng các dịch vụ tương ứng. Nhánh lưu trữ giữ phần triển khai và test cũ. |

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

Ba vị trí database mặc định ở trạng thái `NOT_FOUND`. Chưa biết vị trí tùy
chỉnh. Migration thật ở trạng thái `NOT_RUN`, nên chưa có backup hay biên nhận
nhập thật. Ref Git lưu trữ vẫn trỏ tới baseline.

Kiểm tra phiên bản ghi nhận Codex `0.167.0-alpha.3.2` và launcher Antigravity IDE
`1.107.0`. Claude Code không có trên PATH. Native discovery, chạy hook, chấp
nhận có xác thực và kích hoạt release ở trạng thái `NOT_RUN`. Kiểm thử Linux
local bị chặn bởi `Wsl/E_ACCESSDENIED`. Hosted CI, fixture symlink Unix và
kiểm thử junction thật trên Windows ở trạng thái `NOT_RUN`.
