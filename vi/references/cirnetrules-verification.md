<!-- doc-id: CIRNETRULES-VERIFICATION -->
<!-- revision: 2 -->
<!-- source-sha256: 363bda68b4bd87cef190346712f06efc52574490699374337b9fe1a914bfd7c0 -->
<!-- translation-status: synchronized -->
<a id="verification"></a>

# Kiểm chứng CIRNETRULES trên máy local

Triển khai bắt đầu từ commit governance `a83dd8b710d45a5ace1aabe9477ebd887c7f825e` và đang ở trạng thái thay
đổi local. Windows x64 MSVC dùng Rust và Cargo 1.99.1 qua alias stable có
sẵn. Các checkout kiểm thử là fixture dùng một lần. Release và audit cũ
được lưu làm bằng chứng lịch sử riêng.

Bộ lệnh Windows dùng `cargo +stable fmt --all --check`,
`cargo +stable clippy --locked --offline --all-targets -- -D warnings`,
`cargo +stable check --locked --offline --all-targets`,
`cargo +stable build --locked --offline` và
`cargo +stable test --locked --offline`. Kiểm chứng CLI dùng binary đã
build cho `validate`, `check-translations` theo phạm vi và
`workspace-discover`. Whitespace của Git diff và staging được kiểm tra riêng.

| Nghiệm thu | Kiểm thử hoặc bằng chứng | Phạm vi |
| --- | --- | --- |
| CGACP-01–04, CGACP-07 | Fixture workspace và CLI | Dò offline, liên kết tường minh, Git worktree liên kết có tên khác, một repo UI và context producer/consumer đúng phạm vi. |
| CGACP-05–06, CGACP-20 | Fixture runner và workspace | Diff chạm ranh giới hoặc ngoài phạm vi, đầu vào dirty/rename/delete/untracked, delta phiên, giới hạn context, timeout và hủy lease. |
| CGACP-08–10 | Test task và runner | Liên kết task và initiative, revision CAS, quyền chồng lấp, hết hạn, đổi thẩm quyền và đường dẫn không an toàn. |
| CGACP-11–14 | Fixture tiến trình, task và CLI | Từ chối lệnh không hỗ trợ, không xuất bí mật thô, chạy kiểm tra được duyệt, từ chối test stale và đóng task không cần release. Nghiệm thu rủi ro cao cần danh tính xác thực. |
| CGACP-15 | Fixture CLI và test Markdown | Worktree ràng buộc độc lập, preview không ghi, xung đột một phần, resume, áp dụng lặp, ranh giới tiêu đề và kiểm tra cuối. |
| CGACP-16–17 | Fixture release và workspace | Train/family theo cấu hình, duyệt matrix chính xác, từ chối đóng gói lại và tách release tường minh. Chưa chạy release sản phẩm. |
| CGACP-18 | Cargo và validator | Binary và test ở src, kiểm tra registry, kiểm tra EN/VI theo cặp và toàn bộ. |
| CGACP-19 | Fixture adapter và CLI | Vòng đời adapter và xem trước gói task. Khám phá native và hook vẫn NOT_RUN. |

## Số đo tài nguyên

Số đo fixture tính nội dung nguồn UTF-8, chưa gồm lớp bọc JSON. Thời gian
là mẫu local. Cache hit bỏ việc gửi lại nội dung nhưng vẫn đọc bytes trong
phạm vi để phát hiện thay đổi.

| Packet | Repo sản phẩm | File nguồn | Byte đọc | Byte gửi | Hit | Thời gian ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| UI lần đầu | 1 | 4 | 60 | 60 | 0 | 92 |
| UI có cache | 1 | 4 | 60 | 0 | 4 | 95 |
| API | 2 | 5 | 126 | 126 | 0 | 158 |
| Context RELEASE | 2 | 6 | 154 | 126 | 0 | 149 |

Context RELEASE giữ lại một bí mật fixture và chỉ chọn context. Ước lượng
token lấy byte gửi chia bốn rồi làm tròn lên. Số đo này không đại diện chi
phí token hay bộ nhớ model. Công cụ giới hạn byte context, số nguồn, kích
thước output và thời gian lệnh. Runner chỉ giữ một lượt chạy mỗi task và
không tự retry. Cô lập CPU và RAM thuộc môi trường bên ngoài.

## Cưỡng chế và rà soát

Runner từ chối executable, nhóm đối số và đường dẫn ngoài phạm vi. Docs
apply kiểm tra quyền, liên kết đã nghiệm thu và bytes cơ sở. Diff guard
phát hiện thay đổi theo phạm vi sau khi công cụ ngoài chỉnh sửa. File Git
ignore, thay đổi ngoài phạm vi không được Git báo và một số ranh giới ngữ
nghĩa cần kiểm soát riêng. Thư mục chỉ chủ sở hữu ghi được và harness đã
được cấp phép đảm nhiệm cô lập và xác thực bên ngoài lệnh được quản lý.

Một Agent rà soát khác đã kiểm tra lifecycle và phạm vi release, tái hiện
lỗi tiêu đề Markdown và báo các khoảng trống về biên nhận và worktree.
Test bao phủ các guard đã sửa. Nghiệm thu rủi ro thấp cần chủ thể ủy quyền
tách khỏi người thực hiện và bằng chứng công cụ còn hiệu lực. Nhãn người
thao tác local là khai báo. Khám phá Agent native, CI hosted và thử phần
cứng chưa được chạy.

[Quy trình vận hành](cirnetrules-local.md) · [Kiểm chứng lịch sử](verification.md)
