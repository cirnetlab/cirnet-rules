<!-- doc-id: CIRNETRULES-IDENTITY -->
<!-- revision: 3 -->
<!-- source-sha256: 0129256518926c70132a891eb3b7e09490a691501c1002ae8349e7423ade1bed -->
<!-- translation-status: synchronized -->
<a id="identity-cutover"></a>

# Định danh CIRNET RULES và nâng cấp dữ liệu cục bộ

[Metadata ứng dụng](../../application-identity.json) quy định tên hiển thị,
ID kỹ thuật, tổ chức, định danh repository và revision IPC riêng. Rust và Qt
dẫn xuất tên runtime và namespace từ cùng metadata. Cargo và `VERSION` cùng
xác định ứng dụng 0.1.0. Tên release dự kiến là `cirnet-rules-0.1.0`. IPC
revision 3 dùng `cirnetrules.`. Desktop kiểm tra checksum của runtime cùng thư
mục, rồi xác minh sản phẩm và giao thức trước khi mở thao tác. Thay đổi này
thuộc IPC riêng của Desktop. Giao thức engine ngoài và schema SQLite revision
7 có định danh riêng.

## Nâng cấp từ Dashboard

1. Đóng Desktop cũ và mọi tiến trình runtime của nó. Mở CIRNET RULES và xem
   thế hệ nguồn, kích thước database và WAL nguồn, cùng đường dẫn đích.
   Vị trí mặc định thuộc vùng dữ liệu ứng dụng cục bộ của hệ điều hành trong
   `CIRNETLAB`. Tham số `--data-dir` dùng chung thư mục và đặt tên database mới.
2. Chọn **Sao lưu và nâng cấp**. Runtime khóa thư mục dữ liệu mới và lấy quyền
   truy cập SQLite độc quyền vào nguồn. Sao lưu online bao gồm các giao dịch
   đã commit trong WAL. Runtime giữ backup gốc và nâng cấp schema trên một
   bản staging riêng. Runtime kiểm database đã nâng cấp, khóa ngoại, lịch sử
   migration và digest của bản staging trước khi kích hoạt. Nếu gặp lỗi
   quyền, dung lượng hoặc SQL, xử lý lỗi được báo rồi mở lại Desktop để thử
   lại. Bản staging chưa được cài nằm ngoài kho đang hoạt động.
3. Có thể tạm dừng trước khi kích hoạt. Mở lại Desktop để tiếp tục. Bản staging
   chưa được cài sẽ được dựng lại từ nguồn đã khóa. Bản đã cài trước khi quá
   trình kích hoạt bị gián đoạn được đối chiếu với journal và backup. Mất
   database đích sau khi kích hoạt cần xử lý khôi phục.
4. Nếu có các kho độc lập, Desktop báo xung đột. **Dùng dữ liệu hiện tại** chỉ
   hiện khi đã có database đích và runtime sẽ kiểm tra trước khi mở. Các kho
   vẫn được giữ riêng. Nếu chỉ có hai nguồn đời cũ độc lập, đóng Desktop và
   xác định kho cần dùng trước khi thử lại.

Database hiện hành là `cirnetrules.sqlite3`. Migrator tìm `openrules.sqlite3`
và `cgacp.sqlite3` trong các vị trí đã biết hoặc thư mục chỉ định. Khi có cả
hai, biên nhận kích hoạt cũ và backup đã kiểm tra phải xác nhận kho CGACP thuộc
lần chuyển sang OPENRULES. Journal mới `identity-cutover-v2` ghi thế hệ, đường
dẫn nguồn và đích, revision schema và digest snapshot riêng. Runtime đối chiếu
kho đời cũ với snapshot trước để chặn các ghi phân kỳ sau lần chuyển cũ.

Di trú bảo toàn nội dung người dùng, nội dung nhập, ID, bằng chứng và revision.
Database nguồn, backup và biên nhận cũ được giữ để khôi phục offline. Giữ runtime
cũ đóng sau kích hoạt để tránh ghi dữ liệu phân kỳ. Lịch sử đã công bố và byte
của các migration trước được giữ nguyên.

Kho task dạng tệp `.openrules` và `.cgacp` giữ nguyên byte. Cổng lưu trữ lịch
sử đưa bằng chứng task và binding đã rà soát vào SQLite.
[Quy trình khôi phục bảo trì](cirnetrules-local.md#recover-maintenance-task)
có thể khôi phục từng task thành đề xuất với cùng ID và biên nhận nguồn.
Cần cấp quyền mới và xác minh binding Git trước khi thực thi. Cache có thể
dựng lại trong `.cirnetrules`.

<a id="identity-namespaces"></a>

## Định danh cũ

[Danh sách ngoại lệ](../../identity-legacy-allowlist.json) ghi phạm vi chính xác
cho migrator, guard, fixture, lịch sử bất biến và nguồn gốc.
