<!-- doc-id: CNR-CLI-ONLY-PLAN -->
<!-- revision: 1 -->
<!-- source-sha256: 4b1142e197a97d76a054d3fb9ed42f8ca5e61d057703e6608d2f25f22cd96fac -->
<!-- translation-status: synchronized -->
<a id="implementation"></a>

# Kế hoạch triển khai CLI-only

Checkout và nhánh lưu trữ ban đầu cùng ở 52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c. Worktree không có tệp staged, đã sửa hoặc chưa được theo dõi. Kết quả build bị ignore và tệp zip local nằm ngoài thay đổi này. Không có database hiện tại hoặc hai database đời trước tại các vị trí mặc định, kể cả tệp SQLite phụ trợ. Chưa biết các vị trí dữ liệu tùy chỉnh.

| Thao tác | Ranh giới source và phụ thuộc |
| --- | --- |
| Giữ | Lõi Rust, kho task, liên kết Git, giới hạn ngữ cảnh, kiểm tra có quản lý, cổng tài liệu và release, công cụ và menu terminal CNR, CLI tương thích. |
| Tách | SQL lịch sử có kiểm tra, snapshot không mất dữ liệu và ánh xạ để review. Dùng fixture catalog và migration cho các bất biến governance từ Desktop. |
| Thay thế | Dùng authority Git có version để biên soạn và xuất nguồn được chọn cho ba môi trường Agent, cùng thao tác sync có phạm vi riêng. |
| Gỡ | Source Desktop, runtime và IPC riêng, chat, lịch chạy, probe engine, trình khởi chạy, đóng gói và CI Qt. |

1. Kiểm kê dữ liệu local, chạy test baseline và bảo toàn SQL migration.
2. Triển khai và kiểm thử snapshot SQLite, archive và đề xuất migration.
3. Gỡ target Desktop và viết lại tài liệu đang hướng dẫn sử dụng.
4. Nối kiểm tra registry, resolver chung, freshness, CLI và tra cứu biên nhận.
5. Kiểm thử xuất và ghi có quản lý trong Git repo fixture, rồi chạy lại các kiểm tra Windows.
