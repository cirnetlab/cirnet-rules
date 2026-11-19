# Kế hoạch tích hợp Adaptive Stable

Phạm vi là checkout CIRNET rules và gói cập nhật được đọc. [Đặc tả](../../../specs/cirnet-rules/adaptive-stable/spec.md)
quy định hành vi và giới hạn nghiệm thu.

| Bước | Phép kiểm |
| --- | --- |
| Xem gói cập nhật và checkout | Đối chiếu từng điểm chèn với source hiện tại và ghi trạng thái Git ban đầu. |
| Cập nhật Skill biên soạn, template và hướng dẫn | Rà mỗi cặp EN/VI, xác minh hash source theo byte và đếm đủ 13 Skill. |
| Nối smoke Candidate-only với runner vai trò | Chạy preview và kiểm tra luồng Author sang Work cùng xử lý lỗi bằng shim offline. |
| Rà diff và kết quả kiểm chứng | Chạy bộ kiểm tra tài liệu, Cargo và regression offline. So đầu vào và bằng chứng lịch sử với trạng thái ban đầu. |

## Kết quả kiểm chứng

Bắt đầu tích hợp trên `master` sạch tại `fdfe0949`. Đối chiếu và tích hợp gói
thủ công để cả hai hướng dẫn context đều có phần vòng đời model. Smoke offline
đã đạt preview, đối chiếu byte bàn giao, chốt bảo vệ dữ liệu giả lập, xử lý
lỗi objective và thiếu bằng chứng. Bộ kiểm tra source xác nhận 13 Skill, một
cặp mẫu Work Prompt và các source hash khớp.

Cargo format, test, Clippy, bộ chọn tài liệu thay đổi, fixture đóng gói và
cài đặt, cùng các kiểm tra hành vi offline, follow-up và shim A/B đã qua. Snapshot kiến
trúc và 5.525 file lịch sử trong `.superpowers` khớp trạng thái ban đầu.
Không thể mô phỏng RTL vì thiếu `iverilog` và `vvp`. Windows bỏ qua các ca
kiểm tra liên kết cần quyền tạo symlink hoặc reparse point.

Smoke native, rà hành vi độc lập và benchmark native đầy đủ đều `NOT_RUN`.
Chưa đo usage native. Cần quyền riêng cho đợt chạy native và phải rà độc lập
bằng chứng từ đợt chạy trước khi kết luận tương thích.

## Kiểm tra lại mã thoát CI

Trên `a7e14202`, wrapper PowerShell của GitHub Actions trên Windows in `PASS`
của bộ smoke rồi thoát với mã `2` do `$LASTEXITCODE` còn giữ mã của ca bị chặn
giả lập. Test nay thoát `0` sau khi mọi assertion đạt. Test vẫn kiểm tra
runner smoke trả đúng mã `1` khi lỗi và `2` khi bị chặn.

Các bước workflow đã chạy đạt trên Windows local và trong container Ubuntu
24.04 offline dùng bố trí checkout giống GitHub Actions. Hai bước smoke đều
thoát `0`. Bố trí checkout Ubuntu cần đúng vì runner vai trò suy ra thư mục
workspace cha từ đường dẫn checkout. CI hosted cho bản sửa đang chờ công bố.

## Kiểm chứng lại Candidate-only bằng Codex thật

Một session trên Windows yêu cầu `gpt-6.2-sol` ở mức `xhigh` và dùng ba trong
bốn lượt được phép. Dịch vụ không báo model hay effort hiệu lực. Rà độc lập
trace vai trò, byte bàn giao và các thay đổi fixture xác nhận E01 đạt Task
Success, Architecture Compliance, Verification Coverage và Privacy trong
phạm vi đã chạy. Author dùng bộ kiểm tra cấu trúc và hỏi về quyền xuất dữ
liệu còn thiếu. Work đối chiếu hash của dữ liệu nguồn, dùng lại kết quả và
giữ nguyên fixture. Bộ rà quyền riêng tư ghi nhận 0 lần phát hiện canary hay
thông tin xác thực trong các vai trò đã quan sát.

Author E06 chạm timeout 300 giây khi chưa hoàn tất bàn giao. Trace ghi nhận
bộ kiểm tra cấu trúc từ chối đọc, lỗi chế độ ngôn ngữ hạn chế của PowerShell
và nhiều lần dò chỉ đọc. Work là `NOT_RUN`, nên triển khai E06 và kiểm chứng
trước/sau đều `BLOCKED`. Bản phát hành giữ trạng thái Candidate.

| Vai trò | Token input thực tế | Thời gian chạy trên host (giây) | Lượt gọi công cụ quan sát được |
| --- | ---: | ---: | ---: |
| E01 Author | 137.671 | 119,699 | 13 |
| E01 Work | 63.854 | 37,825 | 8 |
| E06 Author | `NOT_AVAILABLE` | 300,032 | 26 |

E01 dùng 201.525 token input, gồm 161.792 token cached. Tổng input của cả
session là `NOT_AVAILABLE`. Thời gian toàn session là 766,333 giây, gồm thiết
lập và kiểm tra. Hai lần gọi bộ kiểm tra cấu trúc ở E01 trả JSON tương đương.
E06 dò thêm sau khi kiểm tra cấu trúc bị từ chối. Trace cho thấy context bị
lặp, còn tác động riêng của việc sửa source lên token và độ trễ chưa được
đo. Bản sửa CI là thay đổi mã duy nhất.

## Khôi phục luồng Author

| Bước | Phép kiểm |
| --- | --- |
| Tái hiện ranh giới trong trace | Rà lệnh đã ghi và so kết quả inspector trên source E06 và E01 gốc. |
| Bổ sung cách khôi phục vào Skill biên soạn và mẫu | Xác minh cả hai ngôn ngữ giữ mục tiêu đã nêu, dừng đọc khi đủ và giữ ranh giới quyền riêng tư. |
| Kiểm tra offline các ca smoke gốc | Xác minh E06 bàn giao đủ thao tác gốc khi inspector từ chối, và Work giữ dữ liệu ngoài phạm vi. Chạy các phép kiểm quyền riêng tư E01. |
| Chuẩn bị kiểm chứng native | Xác minh preview với model và effort đặt lúc chạy và ghi yêu cầu cấp quyền riêng. |

Trace native gốc của Author E06 ghi timeout sau 300,032 giây và 26 lượt gọi
công cụ quan sát được. Inspector từ chối nội suy template trong
`src/labels.mjs` và Author tiếp tục lặp các phép dò chỉ đọc. PowerShell từ
chối `[pscustomobject]` ở chế độ Constrained Language. Tái hiện offline cho
thấy inspector thoát mã `2` ở E06 và `0` ở E01, cùng lỗi PowerShell thoát
mã `1`. Inspector và runner giữ các kiểm soát an toàn hiện có.

Cặp hướng dẫn biên soạn nay soạn mục tiêu gốc trước khi khảo sát, dừng khi
đủ bằng chứng để bàn giao an toàn và chuyển chi tiết chưa xem sang preflight
có phạm vi của Work. Chỉ thử lại khi cần bằng chứng thiết yếu, có cách khác
đã xác minh và giữ phạm vi đọc, giới hạn output cùng chính sách shell hiện có.

Trên Windows và Ubuntu 24.04, shim Candidate-only tái hiện inspector từ chối
E06 thật, chuyển nguyên byte yêu cầu gốc sang Work kèm `NOT_INSPECTED`, chạy
kiểm thử nhãn trước và sau khi sửa và đạt các phép kiểm bảo toàn dữ liệu hiện
có. Chốt E01 phát hiện và giữ canary giả lập ngoài bằng chứng. Harness hành vi,
test shim A/B offline, test đóng gói, Cargo và kiểm tra EN/VI đều đạt. Windows
bỏ qua các ca symlink/reparse point cần quyền và mô phỏng E08 cần Icarus. Kiểm
kê lịch sử `.superpowers` khớp SHA-256 đã ghi của 5.525 file.

Smoke trên Windows mất 108,160 giây và preview riêng mất 9,793 giây. Lượt
Ubuntu gồm smoke, Cargo, kiểm tra source và quyền riêng tư mất 42,653 giây.
Preview ghi E01/E06 và giới hạn bốn vai trò với model và effort đặt lúc chạy,
gọi 0 lượt model. Theo [quy trình tương thích](../../../docs/model-compatibility.md)
khi native được cấp quyền riêng. Hành vi, token và độ trễ native của revision
này là `NOT_RUN` hoặc `NOT_AVAILABLE`. Giữ Candidate trong khi chờ rà độc lập
E01/E06.

## Kiểm tra bản bàn giao Candidate

| Bước | Phép kiểm |
| --- | --- |
| Tái hiện lỗi chuyển giao | Dùng bản bàn giao shim có byte hợp lệ nhưng mở rộng phạm vi E01 hoặc thiếu mục tiêu E06. Xác nhận Work có được gọi ở mỗi ca hay không. |
| Kiểm tra bàn giao Candidate-only | Yêu cầu giữ nhiệm vụ và phạm vi đã đóng băng, trạng thái rõ ràng và các mục để rà. Kiểm tra lại byte khi Work nhận bàn giao. |
| Xác minh cách chọn sandbox | E01 làm rõ quyền ở chế độ chỉ đọc và E06 dùng sandbox ghi hiện có. Chạy lại kiểm tra quyền riêng tư, dữ liệu được bảo vệ và harness cũ. |
| Chuẩn bị kiểm chứng native | Kiểm tra preview mới và hash source. Giữ cấu hình native đối chứng và chờ quyền riêng. |

Trên `5c705e9`, cả bàn giao E01 mở rộng phạm vi và bàn giao E06 làm mất thao
tác được yêu cầu đều đến Work với SHA-256 khớp và đạt bộ chấm shim. Điểm kiểm
tra Candidate-only nay từ chối cả hai trước khi gọi Work. Smoke offline còn
xác minh E01 chỉ làm rõ trong chế độ đọc, E06 dùng sandbox ghi hiện có, Work
từ chối hash sai và mục kiểm chứng rỗng. Ca khôi phục E06 vẫn chạy test nhãn
trước và sau phép sửa đúng phạm vi. Các kiểm tra canary và lỗi objective vẫn chạy.

Smoke Candidate-only đạt trên Windows và Ubuntu 24.04. Bộ A/B shim legacy
trên Windows, Cargo, kiểm tra tài liệu và source SHA đều đạt. Ca tiếp tục
legacy trên Ubuntu gặp cùng lỗi tìm `.gitignore` tại
`scripts/eval-codex-ab.ps1:70` trên commit gốc và bản sửa. Phép dò độc lập
xác nhận dotfile tồn tại và cần `Get-Item -Force` trên Ubuntu. Kiểm kê bằng
chứng lịch sử vẫn khớp 5.525 file.

Bằng chứng native E06 cũ ghi timeout sau 300,032 giây và 26 lượt gọi công cụ.
Quy tắc biên soạn hiện tại đã nêu khi nào dùng lại bằng chứng, giao source
chưa đọc cho Work và dừng khảo sát. Điểm kiểm tra mới giữ hướng dẫn đó.
Kết luận giảm lượt thử lại hay token cần bằng chứng native. Preview xác minh
96 mục SHA-256 với `gpt-6.2-sol` và `xhigh`, gọi 0 vai trò native. Chạy E01/E06
cùng cấu hình, tối đa bốn lượt, sau khi có quyền riêng. Giữ Candidate trong
khi chờ rà độc lập.

## Tiếp tục trên Ubuntu và chuẩn bị native

Trên `1725e8c`, A/B shim legacy trả mã 1 ở Ubuntu 24.04.5 với PowerShell
7.7.0. Lượt tiếp tục liệt kê được `.gitignore` đã đóng băng của E06, sau đó
không đọc được metadata nếu thiếu `Get-Item -Force`. Thêm `-Force` tại
lần đọc đó giúp cùng bộ shim trả mã 0. Lượt tiếp tục vẫn xác thực đường
dẫn, từ chối hard link và đối chiếu SHA-256 của bản sao.

Preview Candidate đã đối chiếu 96 mục SHA-256 của đầu vào với
`1725e8c70417f89e662cbbcb7b88570da0bef8c2`. Dùng `gpt-6.2-sol`, `xhigh`,
300 giây mỗi vai trò và tối đa bốn lượt. Sau khi có quyền riêng, đối chiếu
hash đầu vào của lượt chạy với preview và rà soát độc lập cả hai ca.
Nghiệm thu native cho revision này là `NOT_RUN`.

## Điều chỉnh trọng tâm phát triển

| Bước | Phép kiểm |
| --- | --- |
| Rà các bên dùng kết quả so sánh và checkpoint | Giữ nguồn gốc, rà chất lượng, số liệu chẩn đoán vai trò và quyền thực thi. Xác định các trường báo cáo chỉ phục vụ tối ưu. |
| Bỏ chấm điểm hiệu năng và đóng băng công cụ so sánh | Chạy test checkpoint và A/B shim cho review chất lượng, bằng chứng thiếu, quyền riêng tư và khả năng truy vết. |
| Cập nhật điểm vào và lưu hồ sơ benchmark | Kiểm tra liên kết EN/VI, SHA-256, byte nguồn được bảo vệ và bằng chứng lịch sử. Giữ kiểm tra tập trung trong luồng mặc định. |

Các phép kiểm hồi quy checkpoint và reporter đã tái hiện yêu cầu hiệu năng
cũ và đạt sau khi bỏ yêu cầu đó. Ubuntu 24.04 chạy đạt các bộ shim A/B và
Candidate-only. Windows chạy đạt kiểm tra bundle và bộ cài trong home giả,
các fixture hành vi và mô phỏng tập trung E08. Ubuntu thiếu các chương trình
mô phỏng cho fixture đó. 5.525 tệp bằng chứng lịch sử, các báo cáo hiện có,
tài liệu kiến trúc, Skills, template, fixture và rubric khớp hash ban đầu.
Kho lưu trữ giữ hồ sơ benchmark từ README và điều chỉnh liên kết local theo
vị trí mới.

## Kiểm chứng một phiên Candidate

| Thay đổi | Phép kiểm |
| --- | --- |
| Thay lịch so sánh bằng một phiên Candidate | Preview tập ca hỗn hợp và đối chiếu hash fixture, Rules, Skills và runner đã đóng băng. Kiểm tra yêu cầu cấp quyền rõ ràng cho các vai trò được chọn. |
| Hợp nhất danh mục tình huống và báo cáo | Chạy các ca quyền riêng tư E01, bảo toàn dữ liệu E06, alias Python và fixture Rust. Từ chối handoff sai hoặc bằng chứng bị sửa trước khi chấp nhận rà soát. |
| Dừng mã A/B đang hoạt động và nhóm script cùng test | Kiểm tra lệnh phân phối, đường dẫn CI và tài liệu EN/VI. Đối chiếu byte fixture đã di chuyển, rubric gốc và bằng chứng lịch sử với checkout tại `d59ac89`. |

Windows không khởi chạy được Git cho fixture Python khi ID mô tả xuất hiện
hai lần trong đường dẫn làm việc. Sau khi chỉ dùng một thư mục ca dưới
`trials/`, preview Author cho Python và Rust chuẩn bị được đầu vào đã đóng
băng. Các phép kiểm fixture chạy trên Windows và Ubuntu 24.04. Mô phỏng E08
chạy trên Windows. Ubuntu thiếu công cụ Icarus Verilog nên mô phỏng ở đó là
`NOT_RUN`.

Báo cáo độc lập bao quát các ca được chọn trong một phiên. Chạy native và
nghiệm thu phát hành cho revision này ở trạng thái `NOT_RUN`.

<!-- source: en/plans/cirnet-rules/adaptive-stable/plan.md -->
<!-- source-sha256: b225db3f67735180c835ededc77c990ddeda6c521af5c4a2a41cca96f5fdf2c0 -->
