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

<!-- source: en/plans/cirnet-rules/adaptive-stable/plan.md -->
<!-- source-sha256: 9f0650451d09c68f2314625439d22f1e5606d9530608a16ca25951c3f1295963 -->
