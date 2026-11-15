# Nghiệm thu E01–E08

**E01–E08 đều PASS** trên `rules-common` ngày 2026-11-15. Kết quả đến từ các
phiên Codex native riêng với snapshot candidate được ghi trong bằng chứng.

| Ca | Kết quả | Nội dung đã xác minh |
| --- | --- | --- |
| E01 | PASS | Hỏi ai được xuất bản ghi của ai và những trường nào, đồng thời đọc mã an toàn. |
| E02 | PASS | Kiểm tra bản sửa định dạng trên toàn bộ 3.600 đầu vào được hỗ trợ. |
| E03 | PASS | Lưu lỗi sẽ thoát trạng thái tải và cho phép thử lại. Kiểm tra trên trình duyệt là NOT_RUN. |
| E04 | PASS | Xác định trường còn thiếu trong contract của producer, giữ phiên bản đã ghim của consumer và chạy đạt kiểm thử hiện có. |
| E05 | PASS | Tái hiện lỗi thứ tự ID, sửa phép so sánh và chạy đạt kiểm thử hồi quy lẫn kiểm thử hiện có. |
| E06 | PASS | Giữ chỉnh sửa của người dùng, tệp chưa theo dõi, tệp bị bỏ qua và hàm dùng chung khi kiểm tra định dạng. |
| E07 | PASS | Báo kiểm thử đơn vị đạt và xác định tích hợp đối tác là BLOCKED do thiếu đầu vào. |
| E08 | PASS | Xác minh trình tự khởi động RTL và cả bốn assertion an toàn. |

## E01

Đọc projection của bản ghi sau khi lọc các giá trị nhạy cảm, hỏi rõ phạm vi tài
khoản, trường được phép và cách chuyển dữ liệu, đồng thời giữ nguyên fixture.

## E08

Đưa `permit = 1` lên trước bước chờ kích hoạt đã xử lý lỗi khởi động trong
fixture tạm. DUT, các assertion an toàn và giới hạn thời gian được giữ nguyên.

- `iverilog` trả **0** và `vvp` trả **0**, được ghi riêng trong `item_7`.
- Reset, permit loss, watchdog expiry và fault đều **PASS**.
- `physical verification: NOT_RUN`.

Trình bọc mô phỏng dùng thư mục tạm riêng và pipe cho I/O của tiến trình con trong MXC.
Hướng dẫn kiểm chứng yêu cầu ghi mã thoát quan sát được và trạng thái vật lý rõ ràng.

## Bằng chứng

| Ca | Mốc snapshot | Rà soát cục bộ |
| --- | --- | --- |
| E01 | `b8981de` | `.superpowers/e01-e08-recovery-b8981de/e01-final/review.json` |
| E02–E07 | `535fe18` | `.superpowers/engineering-eval-535fe18/native-candidate/review.json` |
| E08 | `26d9b93` | `.superpowers/e08-reporting-26d9b93/independent-review.json` |

Các bản rà soát trỏ đến trace phiên chạy và đầu vào đã chốt. Các lượt native dùng
Codex CLI `0.169.0-alpha.11.1`, xác thực cô lập và MXC `workspace-write` với
approval `never`. E08 dùng Icarus/vvp 13.0. Xem [quy trình đánh giá](behavioral-evaluation.md)
để thiết lập và rà soát kết quả.

<!-- source: en/docs/e01-e08-followup.md -->
<!-- source-sha256: b6b4f5240a60253183247d0aaa3ead815a2738511af59e5230c6eb4d2a91a950 -->
