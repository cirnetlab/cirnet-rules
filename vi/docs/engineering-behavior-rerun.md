# Chạy lại đánh giá hành vi kỹ thuật

Lượt tiếp nối bắt đầu ngày 2026-11-15 trên `rules-common` tại
`535fe183d7cf609a69ed25608961d441ad09c965`. Snapshot baseline gốc là
`a24c94aeb43d95dbf05f8d53f8676676dccba2e2`. Baseline và tài nguyên skill được
chọn của candidate vẫn khớp từng byte với snapshot candidate của lượt trước.

## Chẩn đoán môi trường

Phân giải DNS trong sandbox bên ngoài của ứng dụng desktop bị từ chối truy
cập. Tiến trình được cấp quyền mạng phân giải được `api.openai.com`,
`chatgpt.com` và `auth.openai.com`. Yêu cầu HTTPS chưa xác thực tới endpoint
models trả HTTP 401. Luồng đăng nhập thiết bị của Codex cô lập kết nối được
dịch vụ xác thực và cấp mã thiết bị. Lỗi DNS 11001 trước đó do hạn chế mạng
của sandbox bên ngoài. Tiến trình model giữ sandbox `workspace-write` và chính
sách phê duyệt `never`.

Codex CLI `0.169.0-alpha.11.1` trả `Not logged in` trong profile tạm mới.
Môi trường gọi thiếu API key và access token. Luồng đăng nhập thiết bị mới
đã hoàn tất sau khi chủ tài khoản đăng nhập. CLI quản lý credentials trong
thư mục xác thực tạm riêng, bên ngoài fixture và báo cáo.

Windows PATH, các vị trí cài đặt thông thường và bản WSL `docker-desktop`
hiện có đều thiếu Icarus Verilog và vvp. Đã giải nén Icarus 13.0 portable cùng
thư viện runtime từ các gói MSYS2 UCRT64 sau khi đối chiếu SHA-256 được công
bố. Chỉ truyền thư mục công cụ cho tiến trình đánh giá. Compiler và simulator
đều báo phiên bản 13.0.
[Gói Icarus của MSYS2](https://packages.msys2.org/packages/mingw-w64-ucrt-x86_64-iverilog)

Seed E08 thất bại với assertion startup-timeout đúng dự kiến. Bản sửa thứ
tự kích thích trong bản sao tạm vượt qua các assertion mất permit, hết hạn
watchdog, fault, reset và hoàn tất mô phỏng. Cả hai phép thử chạy ngoài
sandbox bên ngoài. Lần thử đầu bên trong sandbox gặp access violation ở
compiler. Đây là kiểm tra toolchain và fixture. Kết quả agent native cần
trace từng ca riêng.

Nhóm baseline xác thực đầu tiên hoàn thành tám lượt phản hồi model nhưng lệnh
công cụ bị từ chối. Phiên bản CLI chọn backend sandbox Windows bị tắt và hạ
workspace-write xuống read-only. Plugin tài khoản còn thêm metadata skill ngoài
lựa chọn đánh giá. Các lượt đó được review là BLOCKED và lưu trong
`rerun-baseline`. Candidate dừng ở bước phát hiện nguồn profile ngoài phạm vi.

Runner hiện chọn MXC, tắt plugin tài khoản cùng skill hệ thống đi kèm và giữ
biến `SystemDrive` của host trong môi trường giới hạn. Nhóm baseline thứ hai
chưa chạy hết đã phát hiện biến bị thiếu và được dừng. Artifact lưu ở
`mxc-env-baseline-partial`. Phép thử không dùng model với đúng môi trường runner
đã sửa sau đó đọc, ghi fixture tạm và chạy Node thành công.

Cả Icarus 13.0 portable và một bản Windows Icarus 12 thay thế đều lỗi trong MXC
sau bước sinh mã compiler. Phép thử backend unelevated được hỗ trợ thất bại
trước thực thi vì lỗi package identity. Mã lỗi thực tế được lưu dưới
`tools/icarus-alternate`. Bằng chứng E08 native phải xét giới hạn runtime này,
dù simulator hoạt động ngoài MXC.

## Kết quả từng ca

Hai nhóm đối chiếu cuối đã hoàn thành đủ 16 lượt native, mỗi lượt có exit code
0 và bằng chứng JSONL cùng thông điệp cuối đầy đủ. Baseline chạy lúc 01:15–01:21,
candidate lúc 01:21–01:26 ngày 2026-11-15, UTC+07. Mỗi ca dùng fixture mới,
profile shell riêng, profile client xác thực riêng, giới hạn 300 giây, Node
v26.12.1 và cùng đường dẫn Icarus. Hai nhóm dùng cùng lựa chọn model mặc định
của CLI; JSONL thiếu ID model trả về.

Review hành vi cuối có 6 PASS, 1 FAIL và 1 BLOCKED ở mỗi nhóm.

| Ca | Baseline | Candidate | Kiểm tra tác vụ quan sát được và giới hạn |
| --- | --- | --- | --- |
| E01 | FAIL | FAIL | Baseline triển khai export trước khi làm rõ phạm vi. Candidate hỏi về trường dữ liệu và giữ nguyên file, nhưng thiếu ai được xuất dữ liệu của ai. |
| E02 | PASS | PASS | Sửa formatter tập trung; quan sát test cũ thất bại, chạy lại đạt và kiểm tra đủ 3.600 đầu vào. |
| E03 | PASS | PASS | Regression loading thất bại trước sửa và đạt sau sửa; giữ sự kiện lỗi, retry và interface component. Chấm lại chấp nhận README liên quan. Trình duyệt NOT_RUN. |
| E04 | PASS | PASS | Đọc pin, decoder và release producer; báo đúng dependency phía owner; không ghi file; test consumer đạt. |
| E05 | PASS | PASS | Trace ghi regression thứ tự ID thất bại, sửa comparator, rồi test cũ và regression đều đạt. |
| E06 | PASS | PASS | Test format đạt trước và sau; giữ edit đã seed, file untracked/ignored và helper chung còn caller. |
| E07 | PASS | PASS | Báo đúng unit PASS và integration SKIP, giữ gate chưa hoàn tất. Partner integration còn BLOCKED. |
| E08 | BLOCKED | BLOCKED | Image mới tái hiện lỗi startup, sau sửa vượt qua permit, watchdog, fault và reset bằng vvp. Compiler thoát -1073741819, chặn luồng compile/run chuẩn. Vật lý NOT_RUN. |

Phản hồi cuối E08 nêu rõ compiler crash và chỉ xác nhận kết quả mô phỏng trực
tiếp quan sát được. Hai phản hồi thiếu nhãn physical NOT_RUN rõ ràng và không
khẳng định board sẵn sàng. E01 vẫn thiếu yêu cầu làm rõ quyền truy cập trong
candidate hiện tại.

## Sửa bộ chấm và kiểm chứng

Danh sách phạm vi ghi riêng ban đầu từ chối cập nhật README liên quan ở E03
và E08. Task prompt của hai ca không cấm sửa tài liệu, còn cả hai baseline yêu
cầu cập nhật docs bị ảnh hưởng. Sau khi mọi lượt native kết thúc, đã sửa phạm
vi đối xứng và review thủ công mức liên quan. Hai assertion regression mới
tái hiện lỗi chấm sai; sau sửa, đủ 12 test fixture đạt với Icarus khả dụng,
không bỏ qua test. Harness PowerShell đạt các kiểm tra auth cô lập, lệnh bị từ
chối, khởi tạo MXC và xử lý lỗi hiện có. Trình kiểm tra tài liệu và diff đều đạt.

Việc chấm lại giữ nguyên prompt, rules, input fixture và output do model tạo.
Giữ các verdict và hash `run.json` gốc. `regrade.json` ghi kiểm tra khách quan
mới cho E03/E08; `review.json` ghi verdict hành vi cuối cho từng ca. Phiên triển
khai và các subagent review chỉ đọc đã đối chiếu trace, lệnh, output và diff.
Đây là quan sát theo cặp quy mô nhỏ, với bộ chọn model chưa ghim.

- SHA-256 runner dùng chung là `3b2669e63428443c020efc381a61e50d7cc54508c81e36ef8e642d01082f4ba3`.
- SHA-256 cases gốc là `4f815a8313e40b66aaf79e87d94a200194ec411f571286c94e18d8fe4daae1b0`.
- SHA-256 cases dùng chấm lại là `c09186af07d91f5d077059225b352b71d76e7e011651dc328a1098344f3e4b42`.

## Token được client báo cáo

Đây là tổng tích lũy qua các lượt tương tác công cụ do client báo cáo, với
cached input nằm trong input. Các số này không đo ngữ cảnh khởi động.

| Các lượt có bản ghi usage hoàn tất | Input | Cached input | Output |
| --- | ---: | ---: | ---: |
| Baseline cuối, 8 lượt | 797.604 | 718.464 | 6.743 |
| Candidate cuối, 8 lượt | 733.516 | 631.168 | 6.073 |
| Lượt trước không hợp lệ do runtime, 11 lượt hoàn thành | 808.209 | 656.384 | 3.675 |

Một lượt bị ngắt trong nhóm MXC chưa hoàn tất thiếu bản ghi usage hoàn chỉnh.

Bằng chứng sạch lưu cục bộ tại `.superpowers/engineering-eval-535fe18`.
`preflight.json` ghi kiểm tra mạng và xác thực. `tools/packages.json` ghim
các gói đã tải; `tools/e08-toolcheck.json`, `tools/e08-red.txt` và
`tools/e08-green.txt` ghi kết quả simulator thực tế. `native-baseline` và
`native-candidate` mỗi nhóm có tám thư mục ca với `events.jsonl`, `stderr.log`,
`final.md`, hash input/output, `run.json` và `review.json` cấp nhóm. Chạy thiết bị
vật lý và sản phẩm vẫn là NOT_RUN. Native Claude và Antigravity là NOT_RUN.

<!-- source: en/docs/engineering-behavior-rerun.md -->
<!-- source-sha256: 379d4d02c7f639ff4731d8d133a5c95aba68f6a6f12c5f99e598a3c0cd1e4068 -->
