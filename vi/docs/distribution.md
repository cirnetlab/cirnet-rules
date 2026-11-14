# Bundle và nâng cấp

`VERSION` là phiên bản toolkit. Checker Cargo và mục release đầu ở changelog
mỗi ngôn ngữ dùng cùng version. Phiên bản sản phẩm tách riêng. Version 0.2.0
là candidate vì đổi tên skill mặc định và thêm định dạng phân phối local.

## Chuẩn bị bundle local

Chạy checker toolkit trước. Dùng PowerShell 7, Git và thư mục output do bạn sở hữu.
Với bundle release, chọn revision đã review trùng HEAD sạch của checkout.
Ví dụ thứ hai tạo bundle development từ checkout dirty.
Script mặc định preview. Thêm `-Apply` để tạo archive.

```powershell
./scripts/package.ps1 -Output $output -Revision $reviewedRevision -Apply
./scripts/package.ps1 -Output $output -AllowDirty -Apply
```

`$output` là thư mục archive đích và `$reviewedRevision` là mã commit Git đủ
40 ký tự hoặc tag local. Chọn lệnh hợp trạng thái nguồn. Manifest development ghi revision
và trạng thái dirty thực tế. Xem `Archive` trong kết quả lệnh, giải nén vào
thư mục khác, rồi đặt `$bundle` là thư mục đã giải nén chứa manifest.

```powershell
./scripts/package.ps1 -Verify -Source $bundle
```

Định dạng 1 chứa path và hash SHA-256 từng file, phiên bản toolkit, revision
Git và trạng thái dirty. Payload gồm baseline, license, thông báo, cùng skill
tiếng Anh và tài nguyên local liên kết. Checksum khớp giúp phát hiện thay đổi
nội dung. Chọn bundle từ nguồn tin cậy. Manifest mô tả byte payload.
Timestamp và metadata container ZIP có thể khác giữa hai lần build.

## Cập nhật và restore

Giữ bundle đã review trước đó và receipt trả về khi apply. Dùng cùng lệnh
install cho bundle mới đã review, xem trước đích, rồi apply và verify.
Lặp lại bundle và lựa chọn giống nhau không phát sinh ghi. Ownership cài đặt
áp dụng cho file managed. Đổi lựa chọn giữ lại skill đã cài trước đó.
File liên kết không còn dùng trong skill được cập nhật vẫn nằm trên đĩa và
được báo để review.
Review ownership trước khi tự gỡ `implementation`, `debugging` và
`verification` cũ để tránh workflow trùng.

Đặt `$receipt` là đường dẫn từ dòng `Backup receipt` và dùng cùng agent,
home và Codex home ban đầu. Xem trước restore trước khi apply.

```powershell
./scripts/restore.ps1 -Backup $receipt -Agent $agent
./scripts/restore.ps1 -Backup $receipt -Agent $agent -Apply
```

Receipt ghi hash byte trước/sau của file bị ảnh hưởng. Restore kiểm tra trạng
thái sau cài hiện tại trước khi phục hồi. Nếu người dùng sửa đích sau apply,
review xung đột và khôi phục thủ công từ backup. Update và restore thao tác
nhiều file. Lỗi ghi sẽ báo vấn đề rollback để bạn kiểm tra file được nêu.
Receipt và thư mục rỗng có thể còn lại sau khi restore thành công.

Đọc [changelog](changelog.md) để xem thay đổi và [thiết lập](setup.md) để xem
đích client. Chọn revision ổn định đã review để dùng thường xuyên và giữ bundle
đã thử trước đó để phục hồi.

<!-- source: en/docs/distribution.md -->
<!-- source-sha256: ef893a644b1200747c29316c05dd153a4538d07ec1b447c1cba09831802c4951 -->
