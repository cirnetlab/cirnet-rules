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

Chỉ đọc kiến trúc khi cần. Với Codex, Claude Code hoặc Antigravity, cung cấp
checkout toolkit hoặc artifact kiến trúc đã giải nén, có phê duyệt. Bắt đầu từ
[chính sách kiến trúc](../policies/architecture.md); giữ revision nguồn và đối
chiếu byte cần dùng với [`docs/architecture-sources.tsv`](../../docs/architecture-sources.tsv).
Bundle cài hướng dẫn, còn các nguồn kiến trúc nằm ngoài payload cài đặt.
Nếu không có nguồn tham chiếu được phép, báo giới hạn trước khi quyết định về kiến trúc.

## Soạn Work Prompt

Khi cài và xác minh, chọn Skill trong bundle bằng
`-Skills 'cirnet-work-prompt-authoring'` cùng các Skill khác cần dùng. Khi được
yêu cầu bàn giao, mở [mẫu tiếng Anh](../../en/templates/chat-to-work-prompt.md)
trong checkout được phép. Mẫu nằm ngoài bundle nên Skill đã cài cần quyền đọc
riêng. Nếu thiếu quyền này, Skill báo `PROMPT_BLOCKED`.

## Task Continuity

Chạy `scripts/task-continuity.ps1` cùng các module bằng PowerShell 7 từ checkout
`cirnet-rules` được phép ở revision đã duyệt. Bundle đã cài cung cấp baseline và
Skills. Cấp riêng quyền đọc checkout. Đặt `Repository` là thư mục gốc của worktree
sở hữu và giữ cùng `TaskId` viết thường và `AgentFamily` (`codex` hoặc `claude-code`).

Dùng quy ước lưu trữ cục bộ của bên sở hữu. Trước khi chọn `.cirnet/tasks/`,
xác nhận Git chưa theo dõi và đã bỏ qua đường dẫn qua quy tắc của repo được
duyệt hoặc tệp loại trừ cục bộ của checkout. Dùng `CheckpointRoot` mỗi lần gọi
nếu chọn đường dẫn bị bỏ qua khác hoặc thư mục do bạn sở hữu ngoài worktree.
Lưu ghi chú đầu vào và gói bàn giao ngoài đường dẫn Git theo dõi, chỉ cấp quyền
cho người thực hiện có thẩm quyền.

Trong `<CheckpointRoot>/<task-id>/`, `current.json` xác định snapshot đã công bố.
Thư mục `snapshots/<checkpoint-id>/` của snapshot đó chứa `task.md`,
`progress.md`, `state.json` và manifest ghi checksum `manifest.json`.

### Khởi tạo và ghi nhận công việc

Chuẩn bị ghi chú task và tiến độ bằng UTF-8 với các tiêu đề cấp hai (`##`). Chỉ
giữ thông tin về task đã được phép lưu và đã loại bỏ dữ liệu nhạy cảm. Các tiêu
đề bắt buộc gồm:

- Task: `Original Task`, `Objectives`, `Acceptance Criteria`, `Architecture & Rules`,
  `Scope & Authorization`, `Plans & Specs`.
- Tiến độ: `Completed Work`, `Pending Work`, `Decisions & Blockers`, `Next Action`,
  `Re-verification`.

JSON tham chiếu có hai mảng `documents` và `relatedRepositories`. Mỗi tài liệu
có `kind` (`plan`, `spec`, `architecture` hoặc `rules`), `repository` (`primary`
cho worktree đích) và `path` tương đối so với repo. Khai báo kiến trúc và quy tắc
áp dụng, cùng các Plan/Spec hiện có. Với repo liên quan, ghi `id`, Git `revision`
đầy đủ và quyền `access` được duyệt (`read` hoặc `edit`), rồi truyền `-RelatedRoot`
ánh xạ ID tới worktree khi dùng checkpoint.

Liệt kê từng tiêu chí bằng một dòng gạch đầu dòng hoặc đánh số dưới
`## Acceptance Criteria`. Nếu chỉ có một tiêu chí có thể dùng một đoạn văn.
JSON nghiệm thu là mảng theo cùng thứ tự. Mỗi mục có `id` ổn định và mảng
`checks` chứa các ID bằng chứng build hoặc kiểm thử bắt buộc. Ví dụ:

```json
[
  {"id": "builds", "checks": ["build"]},
  {"id": "ownership-reviewed", "checks": []}
]
```

Tiêu chí có mảng `checks` rỗng vẫn phải được người có thẩm quyền rà soát kèm
bằng chứng. Đặt các biến tệp dưới đây theo đường dẫn đã chuẩn bị. Khởi tạo
checkpoint trước khi sửa mã. Ví dụ bọc lệnh build đã được phép của dự án Rust:

```powershell
$manager = './scripts/task-continuity.ps1'
$task = @{ Repository = $repo; TaskId = 'toolkit-fix'; AgentFamily = 'codex' }
& $manager save @task -ConfirmScope -TaskFile $taskFile -ProgressFile $progressFile -ReferencesFile $referencesFile -AcceptanceFile $acceptanceFile
& $manager run @task -ConfirmScope -EvidenceId 'build' -Kind build -Executable cargo -Arguments @('build', '--offline', '--locked')
$recovery = & $manager resume @task -ConfirmScope
```

`run` lưu trạng thái trước và sau lệnh sửa, build, kiểm thử hoặc bàn giao được
kiểm soát. Khai báo tệp bị Git bỏ qua và đầu vào phụ thuộc liên quan trong
worktree bằng `-InputPath`, và biến môi trường không nhạy cảm bằng
`-EnvironmentName`. Công cụ ghi hash của giá trị môi trường đã khai báo.
Kiểm tra riêng phụ thuộc bên ngoài hoặc chưa khai báo. Với chỉnh sửa ngoài
trình bọc, xem lại thay đổi và lưu bằng `-AcceptGitDrift -ConfirmScope`.
Các điểm tích hợp áp dụng cho lệnh gọi công cụ rõ ràng.

Khi cập nhật tiến độ, quyết định hoặc trước khi bàn giao, dùng
`save -ProgressFile -ConfirmScope` kèm `-ExpectedCheckpointId` đã xem.
Thêm nội dung vào các phần việc đã làm, việc còn lại, quyết định, vướng mắc
và kiểm chứng lại. Cùng lệnh lưu đó có thể thay nội dung mục `Next Action`.
Để sửa nội dung đã ghi ở các mục còn lại, thêm
`-ConfirmProgressChange` và `-ProgressReason`. Lịch sử snapshot đã xác
nhận giữ các ghi chú trước đó. Với v1, ghi thêm lý do vào phần
Decisions & Blockers mới.

### Kiểm tra và tiếp tục

`status` hiển thị sai khác trong Git, tham chiếu đã ghim và bằng chứng.
Lệnh `verify` cục bộ trả mã khác 0 khi kết quả là `FAIL`, `NOT_RUN` hoặc `BLOCKED`.
`resume -ConfirmScope` trả ghi chú đã niêm phong để khôi phục chỉ đọc, kể cả
khi kết quả hết hiệu lực, lệnh bị gián đoạn hoặc Git đã thay đổi. Đọc
`Blockers`, `Git.Changes`, `Checks` và `Acceptance` trước khi hành động.
Mở lại tham chiếu đã ghim và xác minh quyền triển khai hiện tại. Sau đó gọi lại
`resume` cục bộ với `-ConfirmContinuation` và chỉ sửa thông thường khi
`CanContinue` cho phép. Nếu kết quả đã cũ, `CanContinue` có thể là false
trong khi `CanCorrect` cho phép sửa lỗi đã được cấp quyền bằng
`run -Kind edit -Corrective -ConfirmScope`. Cả hai luồng đều cần worktree
gốc, trạng thái Git đã đối chiếu và không còn đường dẫn chưa hợp nhất,
cùng các tham chiếu đã ghim còn hiệu lực. Nếu một mốc còn ở pha đang chạy,
xác nhận tiến trình và các tiến trình con đã dừng rồi dùng
`-ConfirmStopped` với `save` hoặc `run`. Kết quả mốc vẫn là `NOT_RUN`.

Nếu con trỏ hiện tại hoặc snapshot mà nó trỏ đến bị hỏng, dùng lệnh cục bộ
`resume -ConfirmScope -RecoverPrevious`. Mỗi lần thay con trỏ, công cụ lưu
con trỏ đã xác nhận trước đó vào `previous.json`. Khi khôi phục, công cụ kiểm
tra manifest, hash từng tệp, worktree và loại agent đã khai báo. Nếu nhận được
snapshot cũ, `RecoveredPrevious` là true và kết quả kiểm chứng là `BLOCKED`.
Trước khi ghi tiếp, bên sở hữu phải đối chiếu những gì đã xảy ra sau snapshot
đó và phục hồi con trỏ hiện tại từ bằng chứng đáng tin cậy. Sau lần lưu đầu
hoặc trong kho cũ có thể chưa có con trỏ dự phòng. Chỉ riêng thư mục snapshot
chưa đủ chứng minh snapshot đã được công bố.

`IntegrityStatus` mô tả tính toàn vẹn của checkpoint, `Ready` mô tả độ
sẵn sàng kỹ thuật và `Status` là kết quả kiểm chứng tổng thể. Việc kiểm chứng
có thể chưa hoàn tất ngay cả khi `Ready` là true. `CompletionStatus` cần hồ sơ
đã đóng và bằng chứng còn hiệu lực.
`AgentFamilyMatched` so sánh loại agent người gọi khai báo với loại đã lưu.
Công cụ không xác minh độc lập danh tính client đang chạy hay quyền của tài
khoản. Xác nhận cả hai trong phiên được cấp quyền.

### Rà soát và đóng công việc

Để ghi lượt rà soát của người có thẩm quyền, chuẩn bị `$reviewFile` là
đối tượng JSON với `criteria` và `completedWork`. Mỗi mục tiêu chí có
`id`, `status` và `evidence`. Mục việc đã làm có `status` và `evidence`,
hoặc dùng `null` nếu lượt rà soát đó giữ nguyên. Với `PASS` hoặc `FAIL`,
khai báo ít nhất một bằng chứng được phép theo dạng
`{"path":"relative/artifact.txt","sha256":"<reviewed SHA-256>"}`. Dùng
tệp thường trong worktree nằm ngoài kho checkpoint. Tệp bằng chứng tách
riêng khỏi yêu cầu rà soát. Công cụ kiểm tra lại hash để xác nhận đúng tệp và
bằng chứng còn hiệu lực, đồng thời ghi nguồn rà soát riêng với ghi chú của
agent và mã thoát tiến trình đã quan sát. Người có thẩm quyền đánh giá tệp
chứng minh được điều gì.

Thay đổi mã nguồn, tham chiếu hoặc kết quả liên kết làm lượt rà soát tương
ứng hết hiệu lực. Thay đổi phần việc đã làm cũng làm lượt rà soát phần đó
hết hiệu lực. Chỉ đóng khi phần việc đã làm và từng tiêu chí có bằng chứng
còn hiệu lực:

```powershell
& $manager review @task -ConfirmScope -ConfirmHumanReview -ExpectedCheckpointId $checkpointId -ReviewFile $reviewFile
& $manager close @task -ConfirmScope -ConfirmAcceptance
```

Checkpoint phiên bản 1 và 2 vẫn đọc được. Trước khi đóng task phiên bản 1 còn
mở, dùng `save -ConfirmScope -AcceptanceFile` để ánh xạ tiêu chí gốc và giữ
các kiểm tra bắt buộc. Thao tác ghi snapshot mới và giữ nguyên yêu cầu gốc cùng
các snapshot cũ. Các lần ghi vào checkpoint v2 còn mở áp dụng v3. Lời xác
nhận cũ và việc đóng task v1/v2 là hồ sơ lịch sử, cần kiểm chứng riêng trước
khi tuyên bố đã hoàn thành.
Mục tiêu chí cũ có cách diễn đạt chưa rõ vẫn đọc được. Cần bên sở hữu làm rõ
từng tiêu chí trước khi nâng cấp hoặc xác nhận nghiệm thu.

### Xuất và đọc gói bàn giao

Xác nhận quyền của người nhận. Rà các mục ghi chú công việc, tiến độ và đường
dẫn tài liệu tương đối để loại thông tin bí mật, dữ liệu phiên và đoạn mã nguồn.
Dùng `export -ConfirmScope -ConfirmExport -OutputFile <new-file>` với đích nằm
ngoài toàn bộ thư mục checkpoint. Gói v2 chứa ngữ cảnh được phép, dấu vân tay repo
và tóm tắt bằng chứng. Gói loại tệp mã nguồn, lệnh thô, log, đường dẫn Git cục
bộ và tên tệp đã đổi. Bộ lọc từ chối nội dung nhạy cảm nhận diện được. Người
thực hiện phải kiểm tra phần văn bản tự do còn lại.

Cùng loại agent có thể dùng `verify -HandoffFile` với `-AgentFamily` dự kiến và
`-ExpectedSha256` nhận từ lệnh export qua kênh đáng tin cậy. Để đọc ngữ cảnh rõ
ràng, dùng `resume -HandoffFile` với các tùy chọn đó và `-ConfirmScope`. Gói
có thể báo `IntegrityStatus: PASS` trong khi kiểm chứng trên trạng thái thực tế
vẫn là `NOT_RUN`. Việc kiểm tra Git hiện tại, quyền, tham chiếu và hiệu lực
bằng chứng cần worktree gốc và checkpoint cục bộ. Việc khôi phục sau gián đoạn
Native và bàn giao giữa các tài khoản được cấp quyền của cả hai loại agent được
quy định trong [tiêu chí nghiệm thu](../specs/cirnet-rules/task-continuity/spec.md#11-kiểm-chứng-và-tiêu-chí-nghiệm-thu).

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
<!-- source-sha256: 230ba8c0cd115ff478a213d5bdc4cdc645c1b5ad9a6ddbca84f2559e152319c3 -->
