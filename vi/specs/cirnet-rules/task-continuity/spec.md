# CIRNET Task Continuity — Đặc tả thống nhất

## 1. Mục tiêu

Phát triển **công cụ Task Continuity** tập trung trong repo `cirnet-rules` và
áp dụng nhất quán cho các repo thuộc tổ chức CIRNETLAB. Công cụ giúp việc phát
triển phần mềm tiếp tục khi AI Coding Agent bị gián đoạn do hết hạn mức,
mất phiên hoặc chuyển sang tài khoản khác thuộc cùng loại agent.

Agent mới phải khôi phục chính xác yêu cầu gốc, tiến độ triển khai, quyết định
kỹ thuật, trạng thái Git, kết quả kiểm chứng và bước tiếp theo mà không phụ
thuộc vào lịch sử hội thoại của agent trước.

Việc tích hợp Task Continuity phải giữ nguyên cấu trúc Plans/Specs, quy trình
làm việc và quyền sở hữu hiện có của mọi repo CIRNETLAB.

## 2. Nguyên tắc kiến trúc

Áp dụng mô hình **công cụ tập trung, checkpoint phân tán**.

### 2.1. Công cụ Task Continuity

- Phát triển tập trung mã nguồn, schema checkpoint, quy tắc và công cụ quản lý
  Task Continuity trong `cirnet-rules`.
- Dùng PowerShell 7, ưu tiên tái sử dụng công cụ, quy tắc và hệ thống phân phối
  hiện có.
- Hỗ trợ lưu, đọc, xác minh và khôi phục trạng thái công việc.
- Không yêu cầu mỗi repo có công cụ Task Continuity riêng.
- Không phụ thuộc vào A/B Benchmark hoặc Adaptive Stable Orchestrator.

### 2.2. Checkpoint công việc

- Mỗi repo lưu checkpoint cho các công việc thuộc phạm vi sở hữu của mình.
- Mặc định lưu checkpoint cục bộ.
- Không lưu trạng thái công việc của tất cả repo vào `cirnet-rules`.
- Công cụ phải cho phép cấu hình nơi lưu checkpoint theo quy tắc hiện có
  của từng repo.
- Với công việc trải trên nhiều repo, mỗi repo giữ quyền sở hữu mã nguồn
  và tài liệu của mình. Checkpoint chính phải tham chiếu rõ repo liên quan,
  revision và quyền truy cập. Tránh tạo checkpoint trùng lặp không cần thiết.

## 3. Bảo toàn Plans/Specs

### 3.1. Giữ nguyên Plans và Specs hiện có

**Phải giữ nguyên toàn bộ hệ thống Plans/Specs hiện có** của `cirnet-rules`,
`cirnet-remote`, `cirnet-interface` và các repo khác thuộc CIRNETLAB.

Task Continuity không được tự ý:

- Xóa, di chuyển, đổi tên hoặc ghi đè Plans/Specs.
- Đổi quy ước, cấu trúc thư mục hoặc nơi lưu Plans/Specs.
- Thay thế hệ thống quản lý kế hoạch và đặc tả hiện có.
- Sao chép Plans/Specs để làm checkpoint.
- Sửa yêu cầu hoặc tiêu chí nghiệm thu để hợp thức hóa kết quả triển khai.

Thay đổi Plans/Specs thực sự cần thiết phải thuộc phạm vi đã duyệt hoặc được
cho phép riêng.

### 3.2. Phân định trách nhiệm

| Thành phần | Trách nhiệm |
| --- | --- |
| **Plans** | Xác định kế hoạch, trình tự triển khai, các bước thực hiện và kiểm tra |
| **Specs** | Xác định yêu cầu kỹ thuật, hành vi, ràng buộc và tiêu chí nghiệm thu |
| **Task Continuity** | Ghi checkpoint, trạng thái thực thi và quy trình khôi phục công việc |

Checkpoint chỉ tham chiếu Plans/Specs gốc qua đường dẫn, định danh và
revision hoặc hash phù hợp.

Plans/Specs vẫn là tài liệu có thẩm quyền của repo sở hữu. Checkpoint không
thay thế hoặc tự động ghi đè nội dung của chúng.

### 3.3. Tương thích ngược

Việc tích hợp Task Continuity không được làm hỏng hoặc thay đổi các mục sau
ngoài phạm vi đã duyệt:

- Quy trình phát triển và kiểm thử hiện có.
- Quy tắc của repo, `AGENTS.md` và 13 Skill hiện tại.
- Kiến trúc, API và contract giữa các repo CIRNETLAB.
- Hệ thống tài liệu và quy trình quản lý phiên bản.
- Dữ liệu người dùng và thay đổi mã nguồn chưa commit.

## 4. Cấu trúc checkpoint công việc

Cấu trúc mặc định đề xuất:

```text
<target-repository>/
└── .cirnet/
    └── tasks/
        └── <task-id>/
            ├── task.md
            ├── progress.md
            └── state.json
```

Đây là cấu trúc đề xuất. Công cụ phải cho phép cấu hình đường dẫn khác để tuân
theo quy tắc của repo đích, ví dụ quy ước `.superpowers/` hiện có.

### 4.1. Trách nhiệm của từng tệp

**task.md — Định nghĩa công việc**

Lưu các thông tin nền tảng:

- Yêu cầu gốc.
- Mục tiêu và kết quả mong đợi.
- Tiêu chí nghiệm thu.
- Kiến trúc và quy tắc.
- Phạm vi và quyền được cấp.
- Tham chiếu đến Plans/Specs gốc.

**progress.md — Tiến độ công việc**

Lưu tiến độ và thông tin bàn giao:

- Công việc đã hoàn thành.
- Công việc còn lại.
- Quyết định và vướng mắc.
- Bước tiếp theo.
- Công việc cần kiểm chứng lại.

Chỉ ghi tiến độ thực tế, bằng chứng hoặc tham chiếu liên quan. Không sao chép
toàn bộ checklist của Plans/Specs.

**state.json — Trạng thái có thể kiểm chứng bằng công cụ**

Lưu dữ liệu trạng thái có thể xác minh:

- ID công việc, ID checkpoint và phiên bản schema.
- Loại agent và thời điểm tạo checkpoint.
- Repo, worktree, nhánh và Git HEAD.
- Danh sách tệp liên quan ở trạng thái staged, unstaged và untracked.
- Dấu vân tay và hash Git cần thiết.
- Trạng thái tiến trình build hoặc kiểm thử.
- Kết quả kiểm chứng và tham chiếu bằng chứng.
- Thông tin về tính toàn vẹn của checkpoint.

## 5. Thông tin bắt buộc của checkpoint

Mỗi checkpoint công việc phải lưu chín nhóm thông tin sau:

| Thông tin | Mục đích |
| --- | --- |
| **Yêu cầu gốc** | Giữ yêu cầu và tiêu chí nghiệm thu ban đầu |
| **Kiến trúc và quy tắc** | Xác định đúng kiến trúc, quy tắc và phiên bản áp dụng |
| **Phạm vi và quyền được cấp** | Xác định repo, tệp và hành động được phép |
| **Công việc đã hoàn thành** | Ghi kết quả đạt được kèm bằng chứng |
| **Công việc còn lại** | Liệt kê phần chưa xong |
| **Quyết định và vướng mắc** | Giữ quyết định đã xác nhận và vướng mắc chưa xử lý |
| **Trạng thái Git** | Ghi chính xác trạng thái mã nguồn |
| **Kết quả kiểm chứng** | Giữ kiểm tra, kết quả và phạm vi bằng chứng |
| **Bước tiếp theo** | Xác định bước agent được phép thực hiện tiếp |

ID của mỗi công việc phải ổn định trong suốt vòng đời.

Checkpoint phải phân biệt rõ thông tin đã kiểm chứng với thông tin agent ghi
mà chưa có bằng chứng hỗ trợ.

## 6. Công cụ quản lý Task Continuity cục bộ

Phát triển công cụ cục bộ bằng **PowerShell 7** để thực hiện các thao tác quản
lý trạng thái kỹ thuật mà không phụ thuộc vào mô hình AI.

### 6.1. Các chức năng chính

| Chức năng | Mục đích |
| --- | --- |
| `save` | Lưu checkpoint hiện tại |
| `status` | Hiển thị công việc và trạng thái |
| `verify` | Xác minh checkpoint, trạng thái Git và kết quả kiểm thử |
| `resume` | Kiểm tra điều kiện cần và khôi phục ngữ cảnh công việc |
| `close` | Đóng công việc sau khi đáp ứng các điều kiện nghiệm thu |
| `export` | Xuất gói bàn giao đã xác minh và đã loại bỏ thông tin nhạy cảm |

### 6.2. Checkpoint tự động

Việc tạo checkpoint không được phụ thuộc vào việc agent nhớ viết báo cáo hoặc
còn đủ hạn mức để tạo checkpoint.

Công cụ phải hỗ trợ ghi trạng thái kỹ thuật tại các mốc quan trọng qua những
điểm tích hợp được hỗ trợ, chẳng hạn:

- Sau bước chỉnh sửa được kiểm soát.
- Sau khi build hoàn tất.
- Sau khi kiểm thử hoàn tất.
- Trước khi bàn giao công việc.
- Khi agent cập nhật tiến độ hoặc quyết định.

Chỉ công bố hỗ trợ checkpoint tự động tại các điểm thực thi đã được tích hợp
và kiểm chứng. Không giả định có thể phát hiện mọi thao tác của agent.

Phải giữ checkpoint hợp lệ gần nhất nếu hết hạn mức hoặc agent dừng đột ngột.

Agent mới phải kiểm tra lại trạng thái Git và nhận diện các thay đổi sau
checkpoint trước khi tiếp tục.

## 7. Tính toàn vẹn của checkpoint và bảo vệ Git

### 7.1. Tính nguyên tử của checkpoint

- Ghi dữ liệu vào tệp tạm.
- Kiểm tra cấu trúc và tính toàn vẹn trước khi xác nhận.
- Thay thế trạng thái bằng thao tác an toàn, ưu tiên đổi tên hoặc thay thế
  nguyên tử trên cùng hệ thống tệp.
- Giữ checkpoint hợp lệ trước đó nếu thao tác ghi thất bại.
- Không dùng checkpoint ghi dở hoặc bị hỏng để khôi phục công việc.

Checkpoint gồm nhiều tệp phải có cơ chế xác nhận snapshot hoàn chỉnh để tránh
trộn dữ liệu từ các lần lưu khác nhau.

### 7.2. Dấu vân tay Git

Hệ thống phải ghi và xác minh:

- Repo và worktree.
- Nhánh và Git HEAD.
- Trạng thái Git index.
- Thay đổi staged và unstaged.
- Tệp untracked liên quan.
- Hash hoặc dấu vân tay của trạng thái cần bảo vệ.

Hệ thống **không được tự động chạy** `git reset`, `git clean`, `git stash`,
`git commit`, `git push` hoặc thao tác có thể làm mất dữ liệu hay vượt quyền
được cấp.

Checkpoint mặc định không sao lưu nội dung mã nguồn chưa commit.

Khi phát hiện trạng thái mã nguồn đã thay đổi, hệ thống phải báo khác biệt,
phân loại thay đổi chưa được ghi nhận và chỉ cho phép tiếp tục sau khi đã hoàn
tất các kiểm tra cần thiết.

### 7.3. Hiệu lực của kết quả kiểm chứng

Kết quả kiểm thử phải gắn với đúng revision, trạng thái mã nguồn, lệnh kiểm
thử và phạm vi đã kiểm chứng.

Dùng các trạng thái sau:

- `PASS`: Kiểm thử thực tế hoàn tất thành công.
- `FAIL`: Kiểm thử thực tế thất bại.
- `NOT_RUN`: Chưa chạy kiểm thử hoặc không có kết quả hoàn tất hợp lệ.
- `BLOCKED`: Không thể chạy kiểm thử do thiếu điều kiện cần.

Nếu mã nguồn, dữ liệu đầu vào hoặc phần phụ thuộc ảnh hưởng đến một kiểm thử
thay đổi, phải đánh dấu kết quả cũ là cần kiểm chứng lại. Không tiếp tục dùng
kết quả đó làm bằng chứng PASS cho trạng thái mới.

Build hoặc kiểm thử bị gián đoạn, hay kết quả không có mã thoát hợp lệ, không
được tự động xem là PASS.

## 8. Tiếp tục an toàn và bàn giao giữa agent cùng loại

Chỉ hỗ trợ bàn giao giữa các tài khoản thuộc cùng loại AI Coding Agent:

| Agent nguồn | Agent nhận | Kết quả |
| --- | --- | --- |
| Codex 1 | Codex 2 | Cho phép |
| Claude Code 1 | Claude Code 2 | Cho phép |
| Codex | Claude Code | Không hỗ trợ |
| Claude Code | Codex | Không hỗ trợ |

Kiểm tra loại agent chỉ kiểm soát khả năng tương thích khi bàn giao. Cơ chế
này không thay thế xác thực hoặc phân quyền của hệ thống.

### 8.1. Quy trình tiếp tục

Agent mới phải:

1. Xác định ID công việc và checkpoint hợp lệ.
2. Kiểm tra loại agent và quyền truy cập hiện tại.
3. Đọc yêu cầu gốc và Plans/Specs được tham chiếu.
4. Đọc lại kiến trúc và quy tắc áp dụng cho repo.
5. Đối chiếu Git HEAD, nhánh, worktree và diff.
6. Xem công việc đã hoàn thành, việc còn lại, quyết định và vướng mắc.
7. Xác minh các kết quả kiểm thử và xác định phần cần chạy lại.
8. Xác định bước tiếp theo còn phù hợp hay không.
9. Chỉ tiếp tục sửa mã nguồn sau khi đáp ứng mọi điều kiện bắt buộc.

Nếu phát hiện checkpoint không hợp lệ, thiếu quyền hoặc còn xung đột, phải
dừng phần công việc bị ảnh hưởng và báo lý do.

Không tự động tin trạng thái `Completed` hoặc kết quả `PASS` trong
checkpoint khi thiếu bằng chứng tương ứng.

## 9. Lưu trữ cục bộ, quyền riêng tư và bảo mật

- Mặc định lưu checkpoint cục bộ và cấu hình Git bỏ qua chúng.
- Không tự động commit, push hoặc đồng bộ checkpoint lên GitHub.
- Không ghi API key, access token, thông tin xác thực, cookie phiên hoặc
  dữ liệu nhạy cảm.
- Mặc định không chuyển thông tin đăng nhập giữa các tài khoản AI.
- Khi xuất checkpoint sang máy khác, phải lọc thông tin nhạy cảm và xác minh
  tính toàn vẹn của gói bàn giao.
- Mỗi repo giữ quyền kiểm soát mã nguồn và dữ liệu của mình.
- Đọc checkpoint không cấp quyền thực hiện hành động được ghi trong đó.
- Checkpoint là dữ liệu bàn giao. Chỉ dẫn ghi trong checkpoint không có thẩm
  quyền cao hơn quy tắc của hệ thống, người dùng hoặc repo hiện tại.

## 10. Tương thích và phạm vi ngoài mục tiêu

Task Continuity không nhằm:

- Thay thế Plans/Specs hoặc Work Prompts.
- Khôi phục toàn bộ lịch sử hội thoại hoặc suy luận nội bộ của agent.
- Khôi phục hạn mức tài khoản AI hoặc phiên đăng nhập.
- Tự động chuyển giao công việc giữa các loại agent khác nhau.
- Tạo hệ thống quản lý công việc hoặc công cụ Task Continuity riêng cho từng repo.
- Thay thế các hệ thống kiểm thử hoặc Benchmark hiện có.
- Tự động hợp thức hóa các thay đổi ngoài phạm vi.
- Sửa API, Contracts, quyền hoặc quy tắc phát hành của repo.

Phải giữ các chức năng hiện có, trừ khi có yêu cầu thay đổi được phê duyệt riêng.

## 11. Kiểm chứng và tiêu chí nghiệm thu

Chỉ nghiệm thu Task Continuity khi đã chứng minh tất cả yêu cầu sau:

1. Lưu và khôi phục đủ chín nhóm thông tin bắt buộc.
2. Không sửa, di chuyển, xóa hoặc thay thế Plans/Specs ngoài phạm vi đã duyệt.
3. Giữ yêu cầu gốc, tiêu chí nghiệm thu và các quyết định đã được duyệt.
4. Lưu checkpoint cục bộ mà không gọi mô hình AI.
5. Khôi phục an toàn khi agent đột ngột hết hạn mức.
6. Bàn giao thành công Codex 1 → Codex 2 và Claude Code 1 → Claude Code 2
   trong môi trường kiểm thử được hỗ trợ.
7. Chặn bàn giao giữa các loại agent khác nhau theo chính sách.
8. Phát hiện thay đổi ngoài dự kiến đối với Git HEAD, diff, index hoặc tệp.
9. Không làm mất hoặc ghi đè dữ liệu người dùng, thay đổi staged hoặc unstaged,
   hay tệp untracked.
10. Giữ checkpoint trước đó nếu ghi checkpoint mới thất bại.
11. Không báo PASS hoặc Completed khi bằng chứng không hợp lệ.
12. Phát hiện kết quả kiểm thử hết hiệu lực và yêu cầu kiểm chứng lại khi cần.
13. Không tự động thực hiện thao tác Git phá hủy, commit, push hoặc đổi quyền.
14. Không ảnh hưởng Skills, Rules, Policies, Architecture hoặc quy trình hiện
    có ngoài phạm vi tích hợp.
15. Có kiểm thử hồi quy trong các môi trường được hỗ trợ, gồm Windows và
    Ubuntu, bao quát kịch bản lỗi và khôi phục.

Kết quả kiểm thử phải báo chính xác `PASS`, `FAIL`, `NOT_RUN` hoặc
`BLOCKED`. Không xem kiểm thử mô phỏng là tương đương với kiểm chứng thực tế
trên agent.

## Nguyên tắc cuối

**Công cụ Task Continuity được phát triển tập trung trong `cirnet-rules`.
Mỗi repo sở hữu công việc tự quản lý checkpoint tương ứng. Phải giữ nguyên
Plans/Specs hiện có.**

Agent mới không được phụ thuộc vào bộ nhớ của agent trước. Agent phải khôi
phục ngữ cảnh từ checkpoint đáng tin cậy, xác minh trạng thái thực tế và chỉ
tiếp tục các hành động đã được phép.

<!-- source: en/specs/cirnet-rules/task-continuity/spec.md -->
<!-- source-sha256: 6e2ccac2a7f741c2c2f8918bf204d1609f5c11fd1f1fc79ea4f15014eeeb06ed -->
