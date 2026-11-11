<!-- doc-id: CIRNETRULES-IDENTITY -->
<!-- revision: 6 -->
<!-- source-sha256: 4b48fdff6a83e40ce1c44f375e4f0efba74c6284aeb6315ed74b3ee374201855 -->
<!-- translation-status: synchronized -->
<a id="identity-cutover"></a>

# Định danh CIRNETRULES và khôi phục dữ liệu

[Metadata ứng dụng](../../application-identity.json) quy định package, tên
hiển thị, tổ chức và repository. Bản build Rust đã ghim kiểm tra phiên bản
và định danh executable công khai.

Đường dẫn database CIRNETRULES mặc định trên Windows là
`%LOCALAPPDATA%\CIRNETLAB\CIRNETRULES\cirnetrules.sqlite3`. Với bản cài
đặt trước đây hoặc thư mục dữ liệu tùy chỉnh, truyền đường dẫn database
gốc qua `--source`. [Hướng dẫn khôi phục lưu trữ](https://github.com/cirnetlab/cirnet-rules/blob/52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c/vi/references/cirnetrules-identity.md#identity-cutover)
ghi các đường dẫn mặc định trước đây. Giữ database gốc, WAL và các bản
sao lưu. Dừng tiến trình ghi trước khi di chuyển hoặc khôi phục dữ liệu gốc.

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
để khôi phục. Lưu đề xuất vào staging cần schema 7. Schema chưa hỗ trợ
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

<a id="adoption"></a>

## Xác nhận tiếp nhận trong Git

Archive bảo toàn byte đời trước và đề xuất staging ghi ánh xạ đã review.
Áp dụng tệp và mục catalog đã duyệt qua thay đổi Git có ủy quyền riêng.
Kiểm tra authority sau thay đổi và ghi commit đã review.

```sh
cnr governance migrate adoption --expected-digest PROPOSAL_DIGEST --authority-commit REVIEWED_COMMIT --actor OPERATOR --json
```

Biên nhận tiếp nhận kiểm tra nội dung tracked và provenance catalog.
Đề xuất staging và đề xuất tiếp nhận một phần có trạng thái riêng.
Lệnh ghi biên nhận local và giữ grant cùng tệp authority hiện hành.
