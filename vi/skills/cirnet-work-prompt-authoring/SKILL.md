---
name: cirnet-work-prompt-authoring
description: Dùng khi được yêu cầu viết Work Prompt hoặc chỉ dẫn bàn giao để Codex, Claude Code, Antigravity hay Agent khác thực thi. Yêu cầu triển khai trực tiếp dùng Skill thực thi phù hợp.
---

# Biên soạn Work Prompt

Chat khảo sát chỉ đọc trong phạm vi được phép và soạn một bản bàn giao rõ phạm vi.
Yêu cầu sửa source trực tiếp dùng Skill thực thi phù hợp. Work kiểm tra lại
checkout local, nhánh, chỉ dẫn, quyền và bằng chứng trước khi triển khai.

1. **Interpret.** Soạn mục tiêu và phạm vi được cấp quyền trước khi khảo sát.
   Dùng nhiệm vụ đã nêu, kể cả yêu cầu gốc được lồng. Theo dõi thao tác, owner,
   tệp và symbol, phép đổi, kết quả, phạm vi, ca âm tính và quyền. Đọc lại yêu
   cầu đã cung cấp trước khi kết luận thiếu mục tiêu. Truy cập repo chỉ xác lập
   quyền đọc.
2. **Locate Owner.** Dùng tên tường minh và
   `en/references/architecture-routing.md` đã duyệt. Định vị checkout bằng
   registry cục bộ theo `en/docs/context.md`. Xác minh quyền sở hữu chưa rõ
   bằng bằng chứng được phép. Nếu thiếu truy cập cục bộ, chuyển owner đã xác
   minh sang preflight có giới hạn. Giải quyết mơ hồ trước khi giao.
3. **Select Authority/Skills.** Đọc các mục kiến trúc tiếng Anh và Rules liên
   quan. Xem metadata của các Skill hiện có, dùng `en/skills/README.md` nếu client
   không cung cấp metadata. Đọc Work Skills áp dụng. Liệt kê tên chính xác
   và trách nhiệm. Giữ invariant mang tính quyết định.
4. **Gather Minimum Evidence.** Xem source, contract, manifest và output lỗi của
   owner khi truy cập được. Ghi nhận định, trạng thái, vị trí nguồn và tác động
   hoặc phép kiểm tiếp theo. Dùng `CONFIRMED` cho quan sát, `INFERRED` cho giả
   thuyết kèm phép kiểm để xác định, `NOT_INSPECTED` cho đầu vào chưa đọc,
   `BLOCKED` cho điều kiện còn thiếu. Dẫn repo/path/symbol/revision đã xác minh.
   Với CI, ghi run URL/ID, commit SHA, job/step và lỗi đầu tiên có thể xử lý.
   Giữ kết quả lịch sử gắn với đầu vào và ngày ban đầu.
   Chọn cách đọc an toàn trước khi gọi công cụ. Tìm đường dẫn, cấu trúc,
   định danh và tên trường được phép. Lọc giá trị trước khi chúng vào output.
   Tránh in nguyên kho bản ghi, literal nhạy cảm, thông tin xác thực hoặc nội
   dung môi trường. Giữ các giá trị ngoài bàn giao và bằng chứng.
   Tìm tên trường có thể in cả dòng source chứa giá trị. Chỉ xuất định danh
   hoặc trích đoạn đã lọc giá trị.
   Với dữ liệu bản ghi, ưu tiên inspector đã xác minh, có giới hạn, trả metadata
   cấu trúc mà không thực thi source. Đưa đúng lệnh gọi và giới hạn cho Work.
   Dừng khảo sát khi có thể bàn giao an toàn. Chuyển quy định riêng tư vào preflight.
5. **Draft Adaptive Prompt.** Mở `references/chat-to-work-prompt.md` cạnh
   Skill sau khi xác minh bundle và bản cài, hoặc dùng
   `en/templates/chat-to-work-prompt.md` từ checkout được phép. Thiếu nguồn
   mẫu đã xác minh thì trả `PROMPT_BLOCKED`.
   Chọn `LIGHT`, `STANDARD` hoặc `DEEP` theo độ phức tạp, rủi ro, phụ thuộc,
   bất định và khả năng truy cập nguồn. Xét đủ 12 trường ngữ nghĩa, gom thông
   tin áp dụng vào các heading Markdown thông thường.
6. **Deduplicate.** Dùng context Required, Conditional và Already-loaded trong
   `en/docs/context.md`. Dùng lại bằng chứng khớp. Đặt mỗi dữ kiện, yêu cầu và
   phép kiểm tại một vị trí, kèm vị trí nguồn chính xác, điều kiện an toàn và
   quyền áp dụng.
7. **Editorial Gate.** Rà mục tiêu, owner, phạm vi, bằng chứng, phụ thuộc, ca từ
   chối, invariant và nghiệm thu để Work có thể bắt đầu. Gắn mỗi tiêu chí với
   lệnh/quan sát đã xác minh hoặc giao Work tìm lệnh trong chỉ dẫn owner.
   Đối chiếu mục tiêu, phạm vi, ràng buộc và nghiệm thu với nhiệm vụ đã nêu.
   Giữ rõ thao tác, đích và phép đổi đã được cung cấp; mục tiêu chung chung
   không đủ nếu làm mất thao tác được yêu cầu, kể cả ở LIGHT. Khôi phục
   thao tác hay ràng buộc bị mất trước khi rút gọn. Khi yêu cầu rõ và có quyền,
   ghi `NOT_INSPECTED` và giao Work kiểm tra trước đối với source chưa đọc,
   chi tiết triển khai và lệnh kiểm thử chưa biết. Chỉ dùng `PROMPT_BLOCKED`
   khi thiếu quyết định hoặc thẩm quyền thiết yếu, thao tác không an toàn hay
   xung đột chưa giải quyết cản trở bàn giao an toàn. Dùng bước chuẩn bị
   tự động trong `en/docs/context.md`. Rà ý định theo mệnh đề và các loại trừ.
   Để riêng tác động hạn chế. Chỉ dùng lại phê duyệt có nguồn gốc tin cậy.
   Tạo lại một lần để sửa lỗi cơ học.
8. **Hand Off.** Trả `PROMPT_READY` và một Work Prompt hoàn chỉnh có thể sao
   chép theo định dạng được yêu cầu. Nêu riêng quyền thực thi đang chờ.
   Chạy phép kiểm offline của bản đã duyệt khi Work tiếp nhận.
   Báo `PROMPT_BLOCKED` cùng đầu vào, quyền hoặc quyết định còn thiếu nếu chưa
   thể bàn giao an toàn. Nhiệm vụ tách riêng cần được cấp quyền riêng.
   Giữ bước kiểm tra trước được phép của Work cùng giới hạn quyền riêng tư,
   an toàn và quyền đọc theo nhiệm vụ ngay cả trong bản bàn giao bị chặn,
   theo quy trình đọc an toàn của mẫu. Chặn triển khai không bảo vệ output
   công cụ về sau.

Với công việc Codex hoặc Claude Code đang tiếp tục, ghi Task ID, vị trí
checkpoint và loại agent được phép. Yêu cầu Work khôi phục ngữ cảnh và kiểm
tra lại Git, tham chiếu và phạm vi trước khi sửa. Yêu cầu Work lưu checkpoint
trước khi bàn giao. Cần quyền riêng để cung cấp gói bàn giao.

## Truy cập và thẩm quyền

Bàn giao độc lập model: nêu khả năng bắt buộc. Ghi model và effort đánh giá
dưới dạng metadata runtime (`vi/docs/model-compatibility.md`).

Giao Work xác lập chỉ dẫn local/lồng nhau, Rules, Skills và nguồn owner áp dụng
một lần trong vai trò của mình, dùng lại revision khớp còn sẵn và mở lại bằng chứng
đã đổi hoặc còn thiếu trước khi thao tác. Nếu Work không
đọc được nguồn tham chiếu, đưa trực tiếp dữ kiện đáng tin cậy thiết yếu, định
danh/phiên bản nguồn, ràng buộc và nghiệm thu vào bản bàn giao. Chỉ dùng fallback
khi bằng chứng đáng tin cậy xác lập phạm vi an toàn. Đánh dấu phần chưa giải
quyết là `BLOCKED`, hoặc cả bản bàn giao là `PROMPT_BLOCKED` nếu thiếu thẩm
quyền hay quyết định thiết yếu. Chat thiếu quyền đọc source sản phẩm có thể ghi
`NOT_INSPECTED` và giao Work kiểm tra trước khi đã rõ owner, phạm vi an toàn.
Phân biệt quyền đọc mẫu với quyền Work đọc source.

Giữ lựa chọn công nghệ áp dụng, `LAB_FIRST_PRIMARY`, pin chính xác, artifact
bất biến và provenance. `PARALLEL_ALLOWED` cần bằng chứng theo chính sách kiến
trúc. Công việc Ecosystem độc lập nằm trong ranh giới owner của mình. Ở Lab vật
lý, phân biệt Safety, Topology và Correctness, gắn với topology quan sát được,
giữ routing chỉ cho hạ tầng và dây sinh viên. Dùng khả năng có bằng chứng.
Kết luận E2E thật cần chuỗi thực thi thật đã chọn. Vận hành vật lý cần ủy quyền
rõ ràng và bằng chứng an toàn của owner.

Bản bàn giao chỉ chuyển quyền đã cấp, gồm ghi liên repo, commit/push/merge,
công bố và triển khai. `PROMPT_READY` mô tả mức sẵn sàng của bản bàn giao.
`IMPLEMENTATION_VERIFIED` cần bằng chứng mới từ Work. Phép kiểm người dùng
hoãn giữ `NOT_RUN` cùng lý do, gồm `DEFERRED_BY_USER_PHASE_1` khi áp dụng.

<!-- source: en/skills/cirnet-work-prompt-authoring/SKILL.md -->
<!-- source-sha256: d97324fb7912fec505de1ec36b5d6e8a7deebf3e430d2e99f9842d671be706db -->
