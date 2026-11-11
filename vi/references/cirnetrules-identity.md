<!-- doc-id: CIRNETRULES-IDENTITY -->
<!-- revision: 4 -->
<!-- source-sha256: 434904d837544b04cd16d7808b238de0b3d8e513dbdeb781c181fb77a35306de -->
<!-- translation-status: synchronized -->
<a id="identity-cutover"></a>

# Danh tính và khôi phục dữ liệu cũ

[Metadata ứng dụng](../../application-identity.json) quy định package, tên
hiển thị, tổ chức và repository. Bản build Rust đã pin kiểm tra phiên bản và
danh tính executable công khai. Source và hướng dẫn vận hành cũ nằm trên
[nhánh lưu trữ](https://github.com/cirnetlab/cirnet-rules/tree/52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c).

Trên Windows, kiểm tra tệp database trong thư mục dữ liệu local của hệ điều
hành dưới CIRNETLAB hoặc dùng đường dẫn database từ thư mục dữ liệu tùy chỉnh
ban đầu. Kiểm tra riêng các đời CIRNETRULES, OPENRULES và CGACP. Giữ nguyên
database gốc, WAL và các backup trước đó. Đóng tiến trình ghi cũ trước khi khôi
phục hoặc di chuyển dữ liệu gốc.

```text
%LOCALAPPDATA%\CIRNETLAB\CIRNETRULES\cirnetrules.sqlite3
%LOCALAPPDATA%\CIRNETLAB\OPENRULES\openrules.sqlite3
%LOCALAPPDATA%\CIRNETLAB\CGACP\cgacp.sqlite3
```

```sh
cnr governance migrate inspect --source PATH_TO_DATABASE --json
cnr governance migrate export --source PATH_TO_DATABASE --destination NEW_ARCHIVE --json
cnr governance migrate validate --archive NEW_ARCHIVE --json
cnr governance migrate import --archive NEW_ARCHIVE --dry-run --preview-out REVIEW_DIR --json
cnr governance migrate import --archive NEW_ARCHIVE --yes --expected-digest PLAN_DIGEST --json
cnr audit migration --receipt PLAN_DIGEST --json
cnr governance migrate rollback --expected-digest PLAN_DIGEST --yes --json
```

Export dùng API backup của SQLite cho WAL hiện hành khi đọc được WAL và
tệp bộ nhớ dùng chung. Nếu database đã dừng, đã checkpoint đầy đủ và không
còn cả hai tệp phụ trợ, export xác minh bản sao byte trong staging. Inspect
hướng dẫn dùng export cho trường hợp này. Thiếu một tệp phụ trợ hoặc không
truy cập được WAL sẽ chặn thao tác.

Giữ archive trong thư mục cha riêng tư ngoài Git hoặc vùng local bị ignore.
Archive bảo toàn toàn bộ database, gồm hồ sơ riêng. Archive schema 7 chứa
các dòng governance và lịch sử. Có thể bảo toàn schema 1 đến 6 đã kiểm tra
để khôi phục, với dữ liệu không có đề xuất governance. Schema chưa hỗ trợ
hoặc lỗi kiểm tra toàn vẹn sẽ chặn xuất. Giữ nguyên các nguồn đó để khôi phục
bằng ứng dụng gốc.

`--preview-out` tùy chọn lưu đề xuất để review vào một thư mục riêng mới.
Dry-run mặc định báo ánh xạ qua stdout. Import đã xác nhận lưu đề xuất
trong kho migration bị Git ignore. Dùng
`--additional-archive` để review nguồn thứ hai mà không ghi đè nguồn thứ nhất. Đối chiếu
tài nguyên, bản dịch, dependency và ánh xạ repository được đề xuất với
registry trước khi tiếp nhận. Xác nhận digest lưu đề xuất và biên nhận để
review. Thay đổi Git và quyền task vẫn theo quy trình review và ủy quyền
thông thường. Rollback chỉ thu hồi đề xuất khớp digest và giữ dữ liệu archive.

Với task trong kho tệp đời trước, giữ kho gốc và theo
[quy trình khôi phục task](cirnetrules-local.md#recover-maintenance-task).

<a id="identity-namespaces"></a>

## Định danh tương thích

[Kiểm kê danh tính](../../identity-legacy-allowlist.json) liệt kê ngoại lệ chính
xác cho khôi phục, guard, fixture và provenance lịch sử.
