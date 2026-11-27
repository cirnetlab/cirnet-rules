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

## Task Continuity

Chạy `scripts/task-continuity.ps1` cùng các module bằng PowerShell 7 từ checkout
`cirnet-rules` được phép ở revision đã duyệt. Bundle đã cài cung cấp baseline và
Skills. Cấp riêng quyền đọc checkout. Đặt `Repository` là thư mục gốc của worktree
sở hữu và dùng `AgentFamily` hiện tại (`codex` hoặc `claude-code`). Với thao tác
trên task đã xác định, dùng `TaskId` viết thường ban đầu.

Đặt biến cấp tiến trình `CIRNET_LOCAL_STORAGE_ROOT` thành thư mục cục bộ
hiện hữu ngoài checkout, worktree và thư mục quản trị Git. Với origin
CIRNETLAB chính tắc, lệnh checkpoint mặc định dùng
`<root>/.cirnet/tasks/<repository-id>`. Benchmark cục bộ và đầu ra phát triển do
Contracts cấp phát dùng `<root>/.cirnet-build-output/<repository-id>`.
Đặt TEMP, TMP và TMPDIR vào thư mục tạm sở hữu trong tiến trình chạy test.

`-CheckpointRoot` chỉ định kho khác hoặc kho lịch sử đã được phép. Khi không
cấu hình cả hai, mặc định cũ là `.cirnet/tasks` đã được Git bỏ qua. Worktree liên
kết dùng namespace của repo. Dùng Task ID khác nhau giữa các worktree. Cơ chế
định danh hiện có chặn khôi phục hoặc sửa checkpoint của worktree khác. Ghi
claim nghiêm ngặt truyền đường dẫn checkpoint đã phân giải vào trình xác minh
cô lập. Giữ hồ sơ claim đã review trong worktree sở hữu theo quy ước hiện có.

Khi khôi phục dữ liệu cũ, chạy verify và resume trên worktree ban đầu với
`-CheckpointRoot` cũ. Trước khi sao chép, kiểm tra con trỏ, băm snapshot, worktree,
Task Key và xung đột ở đích task lẫn binding. Xác minh lại bản sao từ worktree
ban đầu với thư mục gốc mới. Giữ bản gốc và đường dẫn tuyệt đối của artifact
session đã ghi.

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

### Dùng lại phiên artifact của task

Khôi phục cùng Task ID trên worktree gốc. Sau khi review checkpoint và xác nhận
phạm vi hiện tại, dùng `TaskKey` của task làm `CIRNET_ARTIFACTS_TASK_KEY`
khi cấp phát hoặc chạy lại. Ghi đầu ra `node $allocator --session-info` và
tham số build trong tiến độ hiện có. Lưu khi không đổi sẽ giữ snapshot hiện tại.

Khi phục hồi, dùng cùng Task Key, khôi phục `CIRNET_ARTIFACTS_SESSION` đã ghi
và chạy `--session-info` từ checkout sở hữu. So sánh SHA-256 của marker và
ngữ cảnh với checkpoint. Kiểm tra toolchain, profile, bộ sinh và cấu hình phụ
thuộc trước khi dùng cache. Kiểm tra riêng bản build và bằng chứng. Nếu thiếu
phiên hoặc thông tin không khớp, giữ bằng chứng và ghi phiên mới đã được phép.

Chạy `--session-info` trước khi review dọn dẹp. Kiểm tra con trỏ candidate và
profile, ghi chú checkpoint, bằng chứng, đầu vào offline và tiến trình đang chạy.
Chỉ khi được phép dọn dẹp riêng, chạy `node $allocator --cleanup $session` với
ngữ cảnh sở hữu và Task Key đã xác minh. Giữ phiên dở dang và lịch sử khi còn
tham chiếu chưa rõ.

### Tìm và khôi phục task chưa hoàn tất

Với yêu cầu ngắn như “Resume unfinished task”, dùng worktree sở hữu và kho
checkpoint đã duyệt. Khi có quyền đọc, chạy lệnh từ checkout toolkit đã duyệt:

```powershell
$manager = './scripts/task-continuity.ps1'
$scope = @{ Repository = $repo; AgentFamily = 'codex'; ConfirmScope = $true }
$recovery = & $manager bootstrap @scope -Compact
```

Dùng `claude-code` cho Claude Code. Thêm `CheckpointRoot` vào `$scope` nếu
dùng kho khác mặc định đã duyệt. Đường dẫn tương đối tính từ `Repository`.
Discovery chỉ xét thư mục task ngay dưới kho đó. Công cụ kiểm tra snapshot
đã công bố, repo và worktree sở hữu, cùng loại agent. `discover @scope` trả
ID và trạng thái chọn task, không trả ghi chú hay toàn bộ dữ liệu Git.
`bootstrap` chọn và khôi phục task trong một lần gọi. Khi không truyền nhãn
phiên, chỉ tự chọn nếu quét đủ kho và có đúng một task hợp lệ trong worktree
và loại agent đã duyệt. Nếu có binding worktree, binding phải khớp task đó.
Khi truyền nhãn phiên, chỉ tự chọn nếu có binding chính xác cho nhãn, kể cả
khi chỉ có một ứng viên. Với `AMBIGUOUS`, yêu cầu người vận hành chọn trong
`Candidates` rồi gọi `bootstrap @scope -TaskId <id>`. Với `NONE`, kiểm tra
worktree, loại agent và vị trí kho.

Mỗi lượt quét tự động xét tối đa 128 mục trực tiếp và 16 MiB dữ liệu snapshot.
Ứng viên hỏng, không đọc được hoặc vượt giới hạn dẫn đến `BLOCKED`. Task ID
tường minh đã xác minh hoặc binding phiên chính xác cho phép khôi phục một
task mà không quét các ứng viên khác.

Để tạo binding phiên ổn định đã được cấp quyền, thêm `-SessionKey <local-label>`
vào lần `save` đầu tiên hoặc dùng `bind @scope -TaskId <id> -SessionKey <local-label>`
cho task đã tồn tại. Dùng cùng nhãn không chứa bí mật đã được người vận hành
duyệt khi gọi `discover` và `bootstrap`. Bỏ tham số nếu phiên chưa có nhãn đáng
tin cậy. Lệnh `bind @scope -TaskId <id>` ghi binding worktree cho luồng không
có nhãn phiên. Binding được chọn phải trỏ đến task đang mở có cùng khóa task
ổn định. Xử lý binding cũ hoặc hỏng bằng Task ID đã duyệt và lệnh
`bind -ReplaceBinding` đã được cấp quyền. Nếu đã lưu checkpoint khi khởi tạo
nhưng công bố binding lỗi, khôi phục theo Task ID rồi tạo binding riêng.
Binding nằm trong `<CheckpointRoot>/.bindings/`, lưu cục bộ và giới hạn truy
cập. Phiên dùng quy trình này cần nạp baseline và có quyền vào toolkit đã duyệt.

Dùng `Task`, `Progress`, `References`, `Git.Changes` và tóm tắt kiểm tra để
chỉ đọc ngữ cảnh cần cho bước tiếp theo. Tóm tắt Git gồm worktree, nhánh và
HEAD đã ghi nhận và hiện tại. Rà Rules cục bộ hiện tại, Plans/Specs
đã ghim có liên quan và kết quả đối chiếu. `-Compact` giữ yêu cầu gốc, tiến độ,
tham chiếu, tiêu chí nghiệm thu và kết quả kiểm tra còn hiệu lực. Dùng
`bootstrap` thông thường để xem đầy đủ bằng chứng tiến trình và rà soát.
Sau khi xử lý vướng mắc và xác lập
độc lập quyền hiện tại, gọi lại `bootstrap` với `-ConfirmContinuation
-ExpectedCheckpointId $recovery.CheckpointId`, cùng phạm vi và Task ID tường
minh nếu đã chọn. Chỉ sửa thông thường khi `CanContinue` cho phép. Bootstrap
kiểm tra lại lựa chọn ngầm định sau khi đối chiếu Git và bằng chứng. Nếu tập
ứng viên, checkpoint hoặc binding đổi, đọc lại. Có thể dùng lại nhãn phiên cục
bộ khi đổi tài khoản nếu truy cập được kho đã duyệt. Nhãn xác định binding
cục bộ. Phiên đang hoạt động xác lập danh tính runtime và quyền của agent.

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

## Bằng chứng kiểm chứng

Dùng chỉ dẫn của chủ sở hữu và worktree đã duyệt để xác định phép kiểm bắt
buộc. Xem phụ thuộc, tác dụng phụ, định danh lệnh, thư mục làm việc và nơi
cấp phát đầu ra trước khi chạy. Các công cụ dưới đây chạy từ checkout
`cirnet-rules` đã duyệt và tạo bản tóm tắt cục bộ. Lệnh của chủ sở hữu vẫn
chạy theo quy trình của họ hoặc qua runner Task Continuity hiện có.

### Xem trước phép kiểm của chủ sở hữu

```powershell
node scripts/preview-owner-checks.mjs --root $approvedRoot --manifest package.json --select npm:test
```

Truyền đường dẫn tuyệt đối tới thư mục gốc của chủ sở hữu và đường dẫn manifest
tương đối chính xác. Đầu vào hỗ trợ gồm script npm trong `package.json`,
target Cargo có giá trị `name` khai báo trực tiếp trong `Cargo.toml` dùng
chuỗi một dòng, các preset configure, build, test, package và workflow hiển thị
trong `CMakePresets.json`, và script RTL do chủ sở hữu chỉ định qua
`--rtl scripts/check-rtl.ps1`. Bản xem trước ghi hash nguồn, ID ứng viên, lựa
chọn và tác dụng có thể có. Trạng thái thực thi là `NOT_RUN`, điều kiện tiên
quyết là `NOT_CHECKED` và quyền là `NOT_GRANTED_BY_PREVIEW`. Cùng chủ sở hữu
kiểm tra lifecycle hook, include, thiết lập kế thừa, các bước trong workflow,
phụ thuộc và tác dụng thực tế. Dùng hướng dẫn của chủ sở hữu để tìm target
Cargo ngầm định, target trong manifest có chuỗi nhiều dòng và quy trình ngoài
các định dạng được hỗ trợ. Chọn test dựa trên yêu cầu hoặc phụ thuộc bị thay đổi.

#### Từ lựa chọn đến kết luận đã rà soát

1. Ghi revision của worktree phía chủ sở hữu, đường dẫn tương đối và SHA-256
   của nguồn đã chỉ định, ID ứng viên đã chọn và tiêu chí được kiểm tra. Đọc
   hướng dẫn của chủ sở hữu để xác định chính xác tệp thực thi, đối số, thư
   mục làm việc và các pin đầu vào. Dùng trực tiếp hướng dẫn đó nếu phép kiểm
   nằm ngoài phạm vi xem trước.
2. Kiểm tra công cụ, phụ thuộc đã ghim, fixture và dịch vụ. Rà lifecycle hook,
   tiến trình con, truy cập mạng và dữ liệu, thao tác ghi và nơi lưu log, tạo
   artifact. Chỉ chạy lệnh khi chủ sở hữu đã phê duyệt đúng phạm vi. Kiểm tra
   lại phạm vi khi lệnh hoặc pin thay đổi.
3. Chạy theo quy trình hoặc CI đã được chủ sở hữu phê duyệt. Dùng lệnh `run`
   hiện có của Task Continuity cho lệnh trực tiếp tương thích khi được cấp
   quyền. Ghi định danh lệnh hoặc CI job, revision, đầu vào, mã thoát của tiến
   trình trực tiếp và tình trạng hết thời gian. Dùng bộ kiểm tra của chủ sở
   hữu để thu mã thoát, số assertion và phạm vi kiểm tra thực tế của từng bước.
4. Giữ đầu ra thô ở nơi chủ sở hữu cấp phát. Khi có tham chiếu được phép cung
   cấp cho người rà soát, ghi thông tin có giới hạn, đã lọc dữ liệu nhạy cảm,
   chẳng hạn CI job và tên tương đối của artifact. Đối chiếu các quan sát từng
   bước do chủ sở hữu tạo với nguồn đó. Chỉ chuyển bản ghi có cấu trúc được
   phép, nằm ở đường dẫn tương đối trong worktree và có pin đã duyệt, cho bộ
   xác minh nghiêm ngặt bên dưới.
5. Báo cáo riêng phạm vi đã kiểm tra, kết quả tiến trình trực tiếp và trạng
   thái kết luận nghiêm ngặt. Dùng `NOT_RUN` khi phép kiểm chưa chạy và
   `BLOCKED` khi thiếu điều kiện tiên quyết, quyền hoặc bằng chứng đáng tin cậy
   bắt buộc. Trước khi đóng task bằng Task Continuity, trình bằng chứng còn
   hiệu lực của từng tiêu chí cho người có thẩm quyền rà soát.

Các ví dụ dưới đây chỉ đến hướng dẫn trong từng checkout đã duyệt của chủ sở hữu.

| Chủ sở hữu | Nguồn chỉ định và hướng dẫn kiểm chứng của chủ sở hữu |
| --- | --- |
| `cirnet-interface` | Trong `package.json`, chọn `npm:test`, `npm:check:boundaries` hoặc `npm:test:e2e:platform` theo yêu cầu. `docs/en/contributing/verification.md` quy định phiên bản Node dùng trong CI, SDK và Remote OpenAPI đã rà soát, cùng quy trình Chromium đã ghim. Kiểm chứng trình duyệt trên sản phẩm native đã cài cần host và fixture đã ghim riêng. |
| `cirnet-remote` | `Cargo.toml` khai báo các target như `cargo:test:cli` và `cargo:test:db_readonly`. Dùng `docs/en/contributing/rust-verification.md` để xác định chính xác lệnh Cargo, toolchain và Contracts đã ghim, đường dẫn đầu ra ngoài repo do chủ sở hữu cấp phát, cùng fixture biệt lập. Dùng cùng hướng dẫn cho target ngầm định và test đánh dấu bỏ qua. |
| `cirnet-simulator` | `CMakeLists.txt` thiếu `CMakePresets.json` nằm ngoài phạm vi xem trước. Chọn quy trình C++ worker trong `docs/en/contributing/verification.md` và kiểm tra công cụ MSVC tương ứng, đầu ra đã cấp phát và phép kiểm artifact đã cài theo hướng dẫn đó. |
| `cirnet-embedded-hardware` | Phần mở rộng của `scripts/check_rtl.py` nằm ngoài phạm vi xem trước. Mục kiểm chứng RTL biệt lập trong `docs/en/verification.md` quy định toolchain và đầu vào bắt buộc, đồng thời yêu cầu mô phỏng tạo cả `LAB_FIRST_RTL_PASS` và `PERMIT_MATRIX_RTL_PASS`. |

Kết quả tương tác UI, hiệu năng, E2E trên đúng chuỗi artifact đã chọn và
kiểm chứng vật lý/HIL đều cần quy trình và bằng chứng riêng của chủ sở hữu.

### Đánh giá các bước bắt buộc

Dùng PowerShell 7.2 trở lên với bản kỳ vọng được ghim độc lập và tài liệu
quan sát do chủ sở hữu cung cấp. Đường dẫn phải tương đối so với worktree
Git được chỉ định.
Cả hai tệp JSON dùng schema phiên bản 1. Bản kỳ vọng khai báo revision đã
duyệt, hash đầu vào liên quan và các bước bắt buộc hoặc tùy chọn cùng số
assertion tối thiểu. Ví dụ dưới đây dùng hash giữ chỗ.

```json
{
  "schemaVersion": 1,
  "revision": "<Git HEAD>",
  "inputs": {"tests/example.test.mjs": "<reviewed SHA-256>"},
  "stages": [{"id": "unit", "required": true, "minAssertions": 2}]
}
```

```json
{
  "schemaVersion": 1,
  "expectationSha256": "<expectation SHA-256>",
  "stages": [
    {"id": "unit", "status": "PASS", "exitCode": 0, "assertions": 2, "failedAssertions": 0, "reason": ""}
  ]
}
```

```powershell
./scripts/verify-claim.ps1 -Repository $approvedRoot `
    -ExpectationPath $plan -ExpectationSha256 $planPin `
    -ObservationPath $observation -ObservationSha256 $observationPin
```

Chế độ mặc định `record-consistency` kiểm tra hai pin, HEAD hiện tại, byte
đầu vào đã khai báo, ID bước duy nhất, mã thoát và số assertion trong bản
ghi. `PASS` ở chế độ này xác nhận bản ghi nhất quán. `executionVerified`
có giá trị false. Cả hai chế độ xuất JSON và thoát 0 cho `PASS`, 1 cho
`FAIL`, 2 cho `NOT_RUN` và 3 cho `BLOCKED`.
Bước bắt buộc vắng mặt hoặc bị bỏ qua giữ kết luận ở `NOT_RUN` trừ khi có
lỗi hoặc điều kiện chặn ưu tiên hơn ở bước bắt buộc. Kết quả tùy chọn vẫn
hiện trong bản tóm tắt. Với quan sát `BLOCKED`, `NOT_RUN` hoặc `SKIPPED`,
dùng mã thoát null, số assertion bằng 0 và mã lý do như `tool-unavailable`,
`authorization-required`, `owner-policy`, `timeout` hoặc `not-executed`.

#### Xác minh thực thi đã rà soát

Sau lần chạy của chủ sở hữu, người rà soát độc lập truy vết từng bước đến
một lần build hoặc test đã hoàn tất trong Task Continuity, với lệnh, worktree,
tập đầu vào và revision đã duyệt. Người rà soát đối chiếu đầu ra của chủ sở
hữu để xác định mã thoát và độ bao phủ thực tế, kể cả mã thoát tiến trình con
khi có wrapper. Giữ đầu ra thô ở vị trí chủ sở hữu đã duyệt. Với mỗi bước đã
rà soát, dùng một tệp JSON riêng, có giới hạn dung lượng và đã lược dữ liệu
nhạy cảm tại đường dẫn tương đối được duyệt trong worktree. Điền
`schemaVersion: 1`, `stageId`, `exitCode`, `assertions` và
`failedAssertions` từ quan sát của chủ sở hữu.

Tệp JSON đã rà soát gắn các pin này với hai bản ghi kết luận và checkpoint
hiện tại. Người rà soát đọc `Seal` chuẩn từ `Read-ContinuityCheckpoint`
trong `scripts/continuity/Checkpoint.psm1`. Dùng đúng ID kiểm tra của chủ
sở hữu cho từng bước.

```json
{
  "schemaVersion": 1,
  "expectationSha256": "<expectation SHA-256>",
  "observationSha256": "<observation SHA-256>",
  "taskId": "owner-checks",
  "checkpointSeal": "<current reviewed checkpoint seal>",
  "stages": [
    {"id": "unit", "checkId": "owner-unit", "artifactPath": "target/owner-unit.json", "artifactSha256": "<artifact SHA-256>"}
  ]
}
```

```powershell
./scripts/verify-claim.ps1 -Repository $approvedRoot `
    -ExpectationPath $plan -ExpectationSha256 $planPin `
    -ObservationPath $observation -ObservationSha256 $observationPin `
    -Strict -TrustedReviewPath $review -TrustedReviewSha256 $reviewPin
```

Nhận pin rà soát qua người rà soát độc lập hoặc môi trường thực thi đáng
tin cậy của chủ sở hữu. Bộ xác minh đối chiếu pin, seal, worktree hiện tại,
các lần đo đã hoàn tất, độ bao phủ đầu vào và giá trị theo từng bước. Hash
xác định các byte khớp nhau. Rà soát độc lập xác lập nguồn gốc, kiểm tra liên
kết giữa lần chạy và từng bước, đồng thời xác định ý nghĩa các assertion.
Khi wrapper thoát 0 và bản ghi tiến trình con đã rà soát cho thấy lỗi, kết
luận trả `FAIL` và giữ cả hai mã thoát. Bước đã hoàn tất bị chặn khi thiếu
bằng chứng hoặc dữ liệu không khớp. Bước bắt buộc bị bỏ qua hoặc vắng mặt
giữ trạng thái bao phủ tương ứng. `PASS` ở chế độ strict đặt
`executionVerified` thành true trong phạm vi bắt buộc đã khai báo. Đối
chiếu bằng chứng với tiêu chí gốc trước khi đóng nhiệm vụ.

Với tài liệu tham chiếu repo phụ, thêm đối tượng `relatedRoots` tùy chọn vào
bản rà soát được ghim độc lập. Ánh xạ đúng các ID repo đã khai báo tới thư
mục gốc chuẩn của từng Git worktree cục bộ. Sau khi xác nhận quyền đọc hiện
tại, thêm `-ConfirmRelatedRead` vào tham số verifier. Verifier dùng Task
Continuity để đối chiếu HEAD đã chọn và hash tài liệu của từng repo. Ánh
xạ xác định phạm vi đọc để kiểm tra mã nguồn hoặc hợp đồng.

### Báo cáo claim đã đo

Chạy `verify-claim.ps1` dưới dạng check build hoặc test trực tiếp của Task
Continuity với `pwsh -NoProfile -NonInteractive -File <đường-dẫn-tuyệt-đối>`
và các tham số có tên của verifier. Truyền đường dẫn tuyệt đối cho
`-Repository`. Ghim kỳ vọng, quan sát, các đầu vào đã khai báo và bản rà soát
được tin cậy nếu có trong `-InputPath` của check. Ghim script verifier tại đó
nếu script nằm trong cây làm việc, hoặc ghim nó dưới dạng tài liệu tham chiếu
trong checkout bộ công cụ đã được rà soát. Với chế độ strict, giữ checkpoint
nguồn đã rà soát trong task của chủ sở hữu và đo chạy verifier trong một task
báo cáo riêng. Check báo cáo có thể tạo checkpoint riêng mà vẫn giữ được seal
của checkpoint nguồn cho verifier.

Bật thu nhận bằng `-ClaimOutputPath $resultRelativePath` khi gọi
`task-continuity.ps1 run`. Chọn tệp mới trong thư mục có sẵn, được chủ sở
hữu duyệt, thuộc diện Git bỏ qua trong worktree và nằm ngoài kho task.
Runner đọc tối đa 1 MiB stdout hoàn chỉnh của verifier trong thời hạn của
tiến trình. Runner kiểm tra cấu trúc JSON, sàng lọc thông tin xác thực,
ghi đúng các byte một lần và lưu digest của chúng trong quan sát tiến trình.
Chế độ thu nhận dùng tập biến môi trường hệ điều hành tối thiểu và giữ
stderr ngoài tệp kết quả. Chủ sở hữu quản lý quyền truy cập và thời hạn
lưu. Checkpoint giữ digest. Giữ checkpoint trong quy trình tin cậy của chủ
sở hữu và nhận pin kết quả đã rà soát qua kênh độc lập.

```powershell
./scripts/report-claim.ps1 -Repository $approvedRoot -TaskId $reportingTask `
    -EvidenceId $verifierCheck -ExpectedCheckpointId $reportedCheckpoint `
    -ResultPath $resultRelativePath -ResultSha256 $reviewedResultPin
```

Truyền cùng ánh xạ `-RelatedRoot` đã được duyệt cho tài liệu tham chiếu repo
phụ trong checkpoint báo cáo hoặc checkpoint nguồn strict. Với checkpoint
nguồn, ánh xạ phải khớp bản rà soát đã ghim và lệnh verifier đã đo phải có
`-ConfirmRelatedRead`.

Báo cáo chỉ đọc trình bày `process.recordedStatus`,
`process.effectiveStatus` và `process.exitCode` cùng với `claim.status`,
`claim.stages`, `task.acceptanceReady` và `task.completionStatus` hiện tại
của Task Continuity. Cặp kết quả hợp lệ giữ cách ánh xạ sau.

| Mã thoát tiến trình | Trạng thái tiến trình đã ghi | Trạng thái claim |
| --- | --- | --- |
| 0 | `PASS` | `PASS` trong chế độ và phạm vi đã khai báo |
| 1 | `FAIL` | `FAIL` |
| 2 | `FAIL` | `NOT_RUN` |
| 3 | `FAIL` | `BLOCKED` |

`SKIPPED` là trạng thái cấp giai đoạn. Giai đoạn bắt buộc bị thiếu hoặc bỏ
qua làm claim không thể đạt `PASS`. Báo cáo hợp lệ dùng mã thoát của claim.
`reportStatus: BLOCKED` với claim rỗng và mã thoát 3 báo hiệu cặp kết quả
không khả dụng, đã cũ hoặc không khớp. `claim.mode: record-consistency` biểu
thị đánh giá tính nhất quán của bản ghi. Dùng cơ chế rà soát độc lập và cổng
đóng task của Task Continuity để nghiệm thu tổng thể.
Báo cáo đánh dấu kết quả đã gắn với tiến trình bằng
`outputOrigin: captured-stdout`. Với lần chạy cũ thiếu digest quan sát,
cần một lần chạy mới có bật thu nhận để xác lập nguồn gốc đầu ra. Rà soát
độc lập vẫn xác định ý nghĩa assertion và thẩm quyền của người rà soát.

### Giới hạn các lần sửa

Dùng `assessCorrection` từ `scripts/verification/retry-policy.mjs` với
nhóm lỗi đã duyệt, các lần chạy theo thứ tự và hành động đề xuất. Mỗi lần
ghi `id`, `clusterId`, `kind` (`check` hoặc `repair`), `identity`
(SHA-256 của lệnh, công cụ, thư mục làm việc và đầu vào áp dụng),
`launched`, `status`, `exitCode` và giả thuyết nếu là lần sửa. Đặt
`kind` của đề xuất thành `repair` hoặc `retry` và cung cấp
`checkDigest` hiện tại, các quyết định `approved`, `safe`, `progress`,
cùng `hypothesis` khi sửa hoặc `transientReason` đã ghi khi chạy lại
lệnh giữ nguyên.

Cơ chế tư vấn cho phép tối đa hai lần sửa đã khởi chạy mỗi nhóm lỗi và
một lần chạy lại lệnh giữ nguyên khi có lý do nhất thời đã ghi. Dừng khi
thiếu bằng chứng, phạm vi hoặc an toàn chưa rõ, hết ngân sách hoặc không
có tiến triển. Ghi lý do và giữ quan sát thất bại. Dùng ID riêng cho mỗi
lần. Module trả quyết định và số lần dùng, với thao tác chạy và cấp quyền
do chủ sở hữu quyết định.

Với quy trình sửa kéo dài qua nhiều phiên, dùng
`readCorrectionHistory({ repository, taskId, clusterId, agentFamily,
confirmScope: true })` và `assessCheckpointCorrection` trong cùng module.
Dùng kho Task Continuity cục bộ gốc và dành các ID bằng chứng liên tiếp
`<cluster>-a001`, `<cluster>-a002`, v.v. Dùng ID mới cho mỗi lần kiểm tra
hoặc sửa được quan sát. Với lần sửa có kiểm soát, thêm dòng
`- cirnet-correction <evidence-id> <digest>` vào **Decisions & Blockers**
trong bản cập nhật tiến độ được xác nhận cùng lần sửa. Tính digest bằng
`correctionHypothesisDigest(hypothesis)`.

Chủ sở hữu hoặc người rà soát độc lập xác nhận và lưu `anchor` được trả
về bên ngoài kho checkpoint có thể chỉnh sửa. Họ rà soát `checkpointId`
hiện tại để dùng làm `expectedCheckpointId`. Cập nhật anchor đã lưu sau
mỗi lần quan sát, trước lần tư vấn tiếp theo hoặc bàn giao phiên. Truyền
cùng tùy chọn đọc, anchor đã lưu, ID checkpoint đã rà soát và yêu cầu
vào `assessCheckpointCorrection`. Đặt `request.checkDigest` theo định danh
lần kiểm tra thất bại đã rà soát và xác nhận rõ `timeAvailable`,
`resourcesAvailable` và `prerequisitesUnchanged`. Chính sách đối chiếu
snapshot hiện tại và trước đó với lịch sử đã lưu, định nghĩa kiểm tra và
fingerprint hiện tại. Lần sửa đã khởi chạy rồi gián đoạn vẫn tính vào ngân
sách sửa. Giới hạn một lần chạy lại lệnh giữ nguyên áp dụng cho nhóm lỗi
đã rà soát, kể cả sau lần sửa thay đổi đầu vào. Tư vấn dừng khi tiến trình
có thể còn chạy. Hai phiên có thể nhận tư vấn trên cùng checkpoint. Khi
thực thi theo quyền đã duyệt riêng, truyền `-ExpectedCheckpointId` đã rà soát
cho Task Continuity. Khóa theo task và kiểm tra ID chỉ cho tối đa một lệnh
cạnh tranh được chạy với checkpoint đó. Dùng nhóm lỗi và lịch sử gốc đã
rà soát độc lập khi đối chiếu yêu cầu còn lại.

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

Đọc [changelog](../../CHANGELOG.md) để xem thay đổi và [thiết lập](setup.md) để xem
đích client. Chọn revision ổn định đã review để dùng thường xuyên và giữ bundle
đã thử trước đó để phục hồi.

<!-- source: en/docs/distribution.md -->
<!-- source-sha256: 75738fb151cba151c91b18b259cd1d5a7172ec2728f8374931a91cc332aaf471 -->
