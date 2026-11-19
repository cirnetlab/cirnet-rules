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

Chạy công cụ PowerShell 7 từ checkout `cirnet-rules` đã được review. Đặt
`Repository` là thư mục gốc của Git worktree đích và dùng cùng `TaskId` viết
thường và `AgentFamily` (`codex` hoặc `claude-code`) trong suốt task. Repo đích
sở hữu kho `.cirnet/tasks/`. Git phải được cấu hình để bỏ qua đường dẫn này
trước khi chạy. Dùng `CheckpointRoot` trong mỗi lần gọi để chọn đường dẫn khác
được Git bỏ qua và chưa được theo dõi, hoặc thư mục ngoài worktree do bạn sở hữu.

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

Từ checkout toolkit, đặt `$repo`, `$taskFile`, `$progressFile` và `$referencesFile`
trỏ đến các đường dẫn đã chọn. Ví dụ sau dùng một dự án Rust:

```powershell
$manager = './scripts/task-continuity.ps1'
$task = @{ Repository = $repo; TaskId = 'toolkit-fix'; AgentFamily = 'codex' }
& $manager save @task -TaskFile $taskFile -ProgressFile $progressFile -ReferencesFile $referencesFile -RequiredCheck 'build'
& $manager run @task -EvidenceId 'build' -Kind build -Executable cargo -Arguments @('build', '--offline', '--locked')
& $manager verify @task
& $manager resume @task -ConfirmScope
& $manager close @task -ConfirmScope -ConfirmAcceptance
```

`run` tạo checkpoint trước và sau lệnh sửa, build, kiểm thử hoặc bàn giao được
kiểm soát. Truyền `-InputPath` cho đầu vào liên quan bị Git bỏ qua hoặc tệp phụ
thuộc trong worktree để thay đổi làm mất hiệu lực bằng chứng cũ. Dùng `status` để
xem trạng thái đã lưu. Nếu trạng thái Git thay đổi, xem khác biệt trước khi lưu
bằng `-AcceptGitDrift -ConfirmScope` rồi chạy lại các kiểm tra bị ảnh hưởng. Đọc
các tham chiếu đã ghim và xác nhận quyền hiện tại trước khi tiếp tục. Chỉ xác nhận
nghiệm thu sau khi xem tiêu chí của task và bằng chứng build hoặc kiểm thử còn hiệu lực.

`export -ConfirmScope -OutputFile <new-file>` tạo biên nhận bàn giao chỉ gồm
metadata. Trên máy nhận, dùng `verify -HandoffFile` với `-AgentFamily` dự kiến
và `-ExpectedSha256` đáng tin cậy do lệnh export trả về. Để tiếp tục cần có
quyền truy cập checkpoint cục bộ và worktree liên quan. Xem
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
<!-- source-sha256: 3b89fb3ee6f89e10ff00756537ac92baf4b157e162e7765d41539ac63819735d -->
