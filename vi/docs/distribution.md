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

Bundle có `cirnet-work-prompt-authoring`. Chọn qua
`-Skills 'cirnet-work-prompt-authoring'` khi cài và xác minh, cùng các Skill
khác bạn muốn chọn. Khi được yêu cầu bàn giao, Agent dùng checkout sẽ mở
trực tiếp [mẫu tiếng Anh](../../en/templates/chat-to-work-prompt.md). Agent
dùng Skill đã cài cần quyền đọc riêng đối với cùng mẫu trong checkout được
phép. Mẫu nằm ngoài bundle. Nếu thiếu quyền truy cập, Skill báo
`PROMPT_BLOCKED`.

## Task Continuity

Chạy công cụ PowerShell 7 cùng các module từ checkout `cirnet-rules` được phép
ở revision đã duyệt. Bundle đã cài cung cấp baseline và Skills. Cấp cho agent
quyền riêng vào checkout chứa công cụ. Đặt `Repository` là thư mục gốc của Git
worktree sở hữu và giữ cùng `TaskId` viết thường và `AgentFamily` (`codex`
hoặc `claude-code`).

Dùng quy ước lưu trữ cục bộ của bên sở hữu. Trước khi chọn `.cirnet/tasks/`,
xác nhận Git chưa theo dõi và đã bỏ qua đường dẫn qua quy tắc của repo được
duyệt hoặc tệp loại trừ cục bộ của checkout. Dùng `CheckpointRoot` mỗi lần gọi
nếu chọn đường dẫn bị bỏ qua khác hoặc thư mục do bạn sở hữu ngoài worktree.
Lưu ghi chú đầu vào và gói bàn giao ngoài đường dẫn Git theo dõi, chỉ cấp quyền
cho người thực hiện có thẩm quyền.

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
worktree bằng `-InputPath`. Với chỉnh sửa của agent ngoài trình bọc, xem lại
thay đổi và lưu bằng `-AcceptGitDrift -ConfirmScope`. Dùng `save -ProgressFile`
khi tiến độ hoặc quyết định thay đổi và trước khi bàn giao. Các điểm tích hợp
áp dụng cho lệnh gọi công cụ rõ ràng. Xem riêng thay đổi đầu vào bên ngoài và
môi trường.

`resume -ConfirmScope` trả ghi chú đã niêm phong để khôi phục chỉ đọc, kể cả
khi kết quả hết hiệu lực, lệnh bị gián đoạn hoặc Git đã thay đổi. Đọc
`Blockers`, `Git.Changes`, `Checks` và `Acceptance` trước khi hành động.
Mở lại tham chiếu đã ghim và xác minh quyền triển khai hiện tại. Sau đó gọi lại
`resume` cục bộ với `-ConfirmContinuation` và chỉ sửa khi `CanContinue` cho phép.
Đối chiếu sai khác Git hoặc
tài liệu với sự chấp thuận của bên sở hữu và chạy lại các kiểm tra hết hiệu lực.
Nếu `run` còn ở pha đang chạy, xác nhận tiến trình và tiến trình con đã dừng
trước khi dùng `-ConfirmStopped` để chạy lại mốc hoặc chấp nhận thay đổi Git.

Sau khi xem bằng chứng của từng tiêu chí, chuẩn bị `$reviewFile` là mảng JSON
gồm các mục có `id`, `status` và tham chiếu `evidence` đã loại dữ liệu nhạy cảm.
Trạng thái rà soát là `PASS`, `FAIL`, `NOT_RUN` hoặc `BLOCKED`. Công cụ
đánh dấu lượt rà soát hết hiệu lực khi mã nguồn đã ghi, tham chiếu hoặc kết quả
liên kết thay đổi. Chỉ đóng sau khi kiểm tra quyền và bằng chứng hiện tại:

```powershell
& $manager review @task -ConfirmScope -ReviewFile $reviewFile
& $manager close @task -ConfirmScope -ConfirmAcceptance
```

Checkpoint phiên bản 1 vẫn đọc được. Trước khi đóng công việc phiên bản 1 còn
mở, dùng `save -ConfirmScope -AcceptanceFile` để ánh xạ tiêu chí gốc và giữ
các kiểm tra bắt buộc. Thao tác ghi snapshot mới và giữ nguyên yêu cầu gốc cùng
các snapshot cũ. Trạng thái đóng của phiên bản 1 là thông tin lịch sử.
Mục tiêu chí cũ có cách diễn đạt chưa rõ vẫn đọc được. Cần bên sở hữu làm rõ
từng tiêu chí trước khi nâng cấp hoặc xác nhận nghiệm thu.

Trước khi bàn giao dạng tệp, rà quyền của người nhận, các phần được phép trong
ghi chú công việc, tiến độ và tham chiếu tài liệu tương đối. Dùng
`export -ConfirmScope -ConfirmExport -OutputFile <new-file>`. Gói chứa ngữ cảnh
được phép, dấu vân tay repo và tóm tắt bằng chứng. Giữ bí mật, dữ liệu phiên và
đoạn mã nguồn ngoài ghi chú chia sẻ. Công cụ từ chối nội dung nhạy cảm nhận diện
được và loại tệp mã nguồn, lệnh thô, log, đường dẫn Git cục bộ và tên tệp đã đổi
khỏi gói. Người thực hiện phải xem văn bản tự do vì bộ lọc không nhận ra mọi giá
trị nhạy cảm.

Cùng loại agent có thể dùng `verify -HandoffFile` với `-AgentFamily` dự kiến và
`-ExpectedSha256` nhận từ lệnh export qua kênh đáng tin cậy. Để đọc ngữ cảnh rõ
ràng, dùng `resume -HandoffFile` với các tùy chọn đó và `-ConfirmScope`. Xác
minh gói chỉ kiểm tra tính toàn vẹn, không cài checkpoint hoặc khôi phục tệp.
Việc kiểm tra Git hiện tại, quyền, tham chiếu và hiệu lực bằng chứng cần worktree
gốc và checkpoint cục bộ. Xem
[đặc tả Task Continuity](../specs/cirnet-rules/task-continuity/spec.md).

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
<!-- source-sha256: 2f8d3c5de501a34db318fd4c4bcbc7f4bb26eac3f47b83c25b251706c5476cf5 -->
