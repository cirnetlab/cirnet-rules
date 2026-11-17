# Mẫu Prompt từ Chat sang Work

## Hướng dẫn biên soạn dành cho Chat

Dùng một mẫu này cho nhiệm vụ Work có phạm vi rõ. Chat khảo sát chỉ đọc và biên
soạn. Work phụ trách kiểm tra local trước, triển khai, build/test được phép và
báo cáo. Trình bày một bản bàn giao hoàn chỉnh bằng Markdown thông thường theo
định dạng người dùng yêu cầu.

Xét đủ 12 trường ngữ nghĩa bên dưới. Gom thông tin áp dụng vào các heading nhiệm
vụ cần. Bỏ phần không áp dụng và placeholder thừa. Giữ chỉ dẫn dùng mẫu cùng
checklist trường thông tin ở ngoài bản bàn giao cuối. Mỗi dữ kiện, ràng buộc và
tiêu chí có một vị trí.

Đối chiếu mục tiêu, phạm vi và nghiệm thu của bản nháp với thao tác, đích,
phép đổi, kết quả, ràng buộc, ca âm tính và quyền người dùng yêu cầu. Khôi
phục yêu cầu còn thiếu trước khi rút gọn. Giữ phép đối chiếu này ở nội bộ.
Khi thao tác, owner và phạm vi an toàn đã rõ, ghi `NOT_INSPECTED` cho source
chưa đọc hay lệnh chưa biết, rồi giao Work kiểm tra trước. Chỉ chặn khi quyết
định, thẩm quyền, xung đột hoặc điều kiện an toàn thiết yếu cản trở bàn giao.

### Chọn mức chi tiết theo nhiệm vụ

| Mode | Cách chọn | Cách trình bày gợi ý |
| --- | --- | --- |
| `LIGHT` | Nhiệm vụ nhỏ, phạm vi rõ, rủi ro thấp và ít phụ thuộc | Khoảng 5–6 nhóm heading ngắn |
| `STANDARD` | Feature, bugfix hoặc refactor một repo với phụ thuộc đơn giản | Khoảng 6–8 nhóm, làm rõ bằng chứng, nghiệm thu và kiểm chứng |
| `DEEP` | Liên repo hoặc contract, CI phức tạp, FPGA/safety, security, dữ liệu người dùng, thao tác khó đảo ngược hoặc hạn chế đọc source | Có thể tách đủ 12 trường thành mục riêng, thêm mục khi giúp giải quyết mơ hồ |

Chọn theo độ phức tạp, rủi ro, phụ thuộc, bất định và khả năng truy cập nguồn.
Độ dài yêu cầu người dùng không quyết định mode. Khoảng heading là gợi ý biên
tập. Giữ đủ bằng chứng, ca từ chối và điều kiện an toàn dù cần thêm chi tiết.
Không đặt hard token cap hoặc word cap.

### Checklist đầy đủ ngữ nghĩa

| Trường | Thông tin chuyên biệt cần xét |
| --- | --- |
| Task Identity | `{{TASK_NAME}}`, `{{TASK_ID_IF_NEEDED}}`, `{{TASK_TYPE}}`, owner `{{TARGET_REPOSITORY}}` |
| Objective | `{{USER_REQUEST}}`, `{{CURRENT_BEHAVIOR}}`, `{{EXPECTED_BEHAVIOR}}`, `{{EXPECTED_OUTPUT}}` |
| Repository Scope | `{{WRITABLE_REPOSITORY_AND_PATHS}}`, `{{READ_ONLY_REPOSITORIES}}`, `{{AUTHORIZED_ACTIONS}}` |
| Architecture Authority | `{{ARCHITECTURE_REFERENCES}}`, `{{APPLICABLE_DECISIONS}}`, `{{RULES_AND_SKILLS}}` |
| Source Evidence & Preflight | `{{SOURCE_EVIDENCE}}`, `{{WORK_PREFLIGHT}}` |
| Functional Requirements | `{{REQUIREMENTS_AND_NEGATIVE_CASES}}` |
| Technical Constraints | `{{TECHNICAL_CONSTRAINTS}}` |
| Implementation Workflow | `{{TASK_STEPS}}`, chỉ thứ tự, phụ thuộc và điểm quyết định riêng của nhiệm vụ |
| Acceptance Criteria | `{{ACCEPTANCE_CRITERIA_AND_CHECKS}}` |
| Build & Verification | `{{VERIFICATION_COMMANDS_AND_PREREQUISITES}}` |
| Documentation & Deliverables | `{{DOCUMENTATION_AND_DELIVERABLES}}` |
| Final Report & Authorization | `{{REPORT_REQUIREMENTS}}`, `{{SEPARATE_AUTHORIZATIONS}}` |

### Tham chiếu, bằng chứng và truy cập

Ghi checkout được phép hoặc artifact đáng tin cậy và đường dẫn tương đối chính
xác tới chỉ dẫn local, mục kiến trúc, Rules và Skills áp dụng. Kèm revision đã
xác minh nếu biết. Trong Applicable Work Skills, chỉ liệt kê đúng tên và trách
nhiệm liên quan. Dùng metadata để chọn Skill rồi đọc thân áp dụng. Tham chiếu
workflow chung, ghi trực tiếp quyết định và invariant then chốt của nhiệm vụ.
Dùng lại context đã nạp khi xác minh được định danh. Ghi trực tiếp dữ kiện
Work cần nhưng khó mở lại. Mở thêm source khi phụ thuộc, rủi ro hoặc lỗi đầu
tiên đòi hỏi.

Nén bằng chứng theo Claim / Status / Locator / Impact or Next check.
Dùng `CONFIRMED`, `INFERRED`, `NOT_INSPECTED`, `BLOCKED` cho từng nhận định.
Với CI, ghi run URL/ID, commit SHA, job/step và lỗi đầu tiên có thể xử lý.
Với source, ghi repo/path/symbol/revision thực sự đã xem. Giao Work tìm vị trí
hoặc lệnh còn chưa biết. Giữ kết quả lịch sử gắn với đầu vào ban đầu.
Dẫn log và source truy cập được thay vì chép cả tệp. Chỉ kèm trích đoạn ngắn
khi cần để khôi phục bằng chứng.
Chọn đoạn source an toàn trước khi gọi công cụ. Xem cấu trúc và tên trường
được phép trước. Lọc literal nhạy cảm trước output, giữ chúng ngoài prompt
và bằng chứng. Đánh dấu lần đọc `BLOCKED` khi thiếu cách truy cập an toàn.

Đưa giới hạn đọc an toàn theo nhiệm vụ vào nội dung cuối dành cho Work,
kể cả `PROMPT_BLOCKED`. Khi thiếu thẩm quyền xuất dữ liệu, Work có thể xác nhận
chỉ dẫn local, revision và phạm vi rồi hỏi các quyết định còn thiếu. Dùng lại
dữ kiện projection và tên trường đã xem cùng vị trí nguồn; không yêu cầu đọc
lại kho bản ghi chỉ để xác nhận lại trạng thái bị chặn. Nếu có quyền thực hiện
một lần đọc độc lập cần thiết, chỉ xuất định danh hoặc mã projection đã lọc
giá trị. Loại literal nhạy cảm trong bộ nhớ trước khi trích source, dòng tìm
kiếm khớp, lỗi hay log được đưa ra output. Xuất tệp chưa lọc hay dòng tìm kiếm
khớp chứa giá trị đều không an toàn. Nếu không thể lọc an toàn, giữ lần đọc đó
ở trạng thái bị chặn và yêu cầu đường truy cập an toàn hơn.

Giao Work mở lại tham chiếu trong checkout được phép và đối chiếu source local
hiện tại trước khi thao tác. Nếu Work không đọc được tham chiếu, đưa trực tiếp
dữ kiện đáng tin cậy thiết yếu, định danh/phiên bản nguồn, ràng buộc và nghiệm
thu. Nếu thiếu thẩm quyền hay thông tin quyết định, đánh dấu phần phụ thuộc là
`BLOCKED` hoặc trả `PROMPT_BLOCKED`. Khi Chat chưa xem được source sản phẩm
nhưng rõ owner và phạm vi an toàn, ghi `NOT_INSPECTED` và yêu cầu Work kiểm tra
trước. Chat phải đọc được chính mẫu tiếng Anh được phép theo
[Skill biên soạn](../skills/cirnet-work-prompt-authoring/SKILL.md).

Mở rộng bằng chứng cho lỗi tạm thời hoặc khó tái hiện, nguồn không truy cập được,
security, safety hoặc compatibility. Giữ thẩm quyền kiến trúc,
`LAB_FIRST_PRIMARY` và quyết định ưu tiên có bằng chứng, ownership từng repo,
tương thích, pin chính xác và artifact bất biến. Ở Lab vật lý, giữ Safety,
Topology và Correctness riêng biệt, binding quan sát được và routing chỉ cho
hạ tầng. Vận hành vật lý cần ủy quyền rõ ràng và bằng chứng an toàn của owner.

## Tổ chức bản bàn giao được tạo

Với LIGHT hoặc STANDARD, có thể đưa các trường áp dụng vào các nhóm sau.

- **Goal & Expected Behavior** gồm định danh, hành vi hiện tại và mong muốn.
- **Owner & Permissions** gồm từng vùng được ghi, phụ thuộc chỉ đọc và thao tác
  đã cấp quyền. Đọc repo không xác lập quyền ghi.
- **References & Evidence** gồm thẩm quyền, Applicable Work Skills, vị trí
  nguồn, nhãn bằng chứng và kiểm tra trước của Work.
- **Requirements & Constraints** gồm hành vi chức năng và khi lỗi, invariant
  tương thích, bảo mật, dữ liệu và an toàn, cùng thứ tự riêng của nhiệm vụ.
- **Acceptance & Verification** gắn mỗi kết quả với lệnh, test hoặc quan sát
  đã xác minh, hoặc giao Work tìm lệnh trong chỉ dẫn owner. Nêu môi trường,
  điều kiện tiên quyết và bằng chứng cần thu.
- **Deliverables & Reporting** gồm tài liệu, cặp EN/VI khi bắt buộc, artifact,
  nơi lưu, đường dẫn đã sửa, lệnh/mã thoát thực tế và trạng thái từng phép kiểm
  `PASS` / `FAIL` / `NOT_RUN` / `BLOCKED`, ảnh hưởng tương thích và giới hạn.

Với DEEP, tách Root Cause, Repository-by-Repository Plan, Contract Compatibility,
Safety Gates hoặc Build/Test Matrix khi cần. Mục root cause giữ nhãn giả thuyết
tới khi bằng chứng xác lập nguyên nhân. Công việc liên repo ghi riêng owner,
ranh giới ghi/đọc và nghiệm thu từng repo.

Tham chiếu Skills/chỉ dẫn local áp dụng để Work thực hiện workflow chung.
Giữ ca từ chối, kiểm tra quyền, cleanup và Stop. Kết luận E2E thật cần chuỗi
được chọn thực sự chạy, kèm phạm vi mock/mô phỏng. Ghi rõ phép kiểm người dùng
hoãn và lý do.

Nêu quyền đã cấp và quyền riêng còn cần cho thay đổi Git, công bố, phát hành,
triển khai và vận hành vật lý. Trả `PROMPT_READY` khi bàn giao có thể thực thi
hoặc `PROMPT_BLOCKED` khi thiếu thẩm quyền thiết yếu. Bản bàn giao chuyển phạm
vi người dùng đã cấp. Work cung cấp bằng chứng mới cho
`IMPLEMENTATION_VERIFIED`.

Trước khi bàn giao, tự rà tính đầy đủ, khả năng truy cập và phần lặp. Bổ sung
chi tiết quyết định còn thiếu hoặc chặn phần việc phụ thuộc. Chỉ trả bản bàn
giao chuyên biệt hoàn chỉnh và trạng thái sẵn sàng, bỏ chỉ dẫn dùng mẫu và chuỗi
Inspect → Plan → Implement → Test → Report chung bị lặp.

<!-- source: en/templates/chat-to-work-prompt.md -->
<!-- source-sha256: 7b705db86919bbebc1a9254ce7fc76449b667da7cdf5e5d484fd3b56b879b39e -->
