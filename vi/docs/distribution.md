# Bundle và nâng cấp

`VERSION` là phiên bản toolkit. Checker Cargo và mục release đầu ở changelog
mỗi ngôn ngữ dùng cùng version. Phiên bản sản phẩm tách riêng. Version 0.2.0
đổi tên skill mặc định và bổ sung định dạng bundle local; bản cài cũ cần
migration được rà soát.

## Quy tắc tên release và kiểm tra trước phát hành

Git tag và tên GitHub Release của CIRNETLAB dùng `<repo_name>-<version>`,
với đúng tên repository và phiên bản đã duyệt, không tự thêm tiền tố `v`.
Với toolkit này, cả tag và tên release là `cirnet-rules-0.2.0`.
Giữ `VERSION` và `bundle.json.version` là `0.2.0`. Tên ZIP có thể
chứa SHA ngắn của mã nguồn để truy vết; đó không phải tên release.

Trước khi phát hành, xác nhận commit `master` đã duyệt và checkout sạch,
`VERSION`/Cargo/changelog thống nhất, CI Windows/Linux đạt trên commit đó,
license/notices và bundle đã kiểm tra cùng các bài test install/restore.
Xác nhận tag và release chưa tồn tại. Xem [bằng chứng kiểm chứng toolkit](toolkit-verification.md)
để biết giới hạn: CI chưa chứng minh client AI thật đã nạp quy tắc.

Chỉ tạo tag sau khi commit cuối đã review nằm trên `master` và có quyền
phát hành. Đóng gói checkout sạch với `-Revision cirnet-rules-0.2.0`,
kiểm tra bundle đã giải nén, rồi đính kèm ZIP và ghi chú đã review vào
GitHub Release tên `cirnet-rules-0.2.0`. Việc đóng gói thông thường không
được tự tạo tag hay phát hành.

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
<!-- source-sha256: 755c2b11e9f6cd9abea6af9dd250220d59d6e542ce557a783c94d4912ad1e4cb -->
