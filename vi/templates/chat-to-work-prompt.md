# Mẫu Prompt từ Chat sang Work

## Hướng dẫn biên soạn dành cho Chat

Dùng bước chuẩn bị tự động trong `en/docs/context.md#offline-handoff-validation`
trước khi giao. Suy ra đầu vào kỹ thuật từ các lần đọc được phép và giao
chi tiết chưa biết cho Work kiểm tra trước. Tạo lại sau khi sửa lỗi cơ học.
Nêu rõ owner, tác động trong phạm vi, nghiệm thu, báo cáo và phê duyệt còn
chờ. Work phải kiểm tra bản bàn giao đã duyệt độc lập và qua bước tiếp nhận
thực thi. Với Work có thể ghi, yêu cầu baseline đã xác minh và đối chiếu
sau khi chạy.

Dùng một mẫu này cho nhiệm vụ Work có phạm vi rõ. Chat khảo sát chỉ đọc và biên
soạn. Work phụ trách kiểm tra local trước, triển khai, build/test được phép và
báo cáo. Trình bày một bản bàn giao hoàn chỉnh bằng Markdown thông thường theo
định dạng người dùng yêu cầu.

Soạn mục tiêu và phạm vi được cấp quyền từ nhiệm vụ đã nêu trước khi khảo sát,
kể cả yêu cầu gốc được lồng hoặc trích dẫn. Giữ rõ thao tác, đích và phép đổi
được yêu cầu trong bàn giao. Đọc lại nhiệm vụ trước khi kết luận thiếu mục tiêu.

Xét đủ 12 trường ngữ nghĩa bên dưới. Gom thông tin áp dụng vào các heading nhiệm
vụ cần. Bỏ phần không áp dụng và placeholder thừa. Giữ chỉ dẫn dùng mẫu cùng
checklist trường thông tin ở ngoài bản bàn giao cuối. Mỗi dữ kiện, ràng buộc và
tiêu chí có một vị trí.

Đối chiếu mục tiêu, phạm vi và nghiệm thu của bản nháp với thao tác, đích,
phép đổi, kết quả, ràng buộc, ca âm tính và quyền người dùng yêu cầu. Khôi
phục yêu cầu còn thiếu trước khi rút gọn. Giữ phép đối chiếu này ở nội bộ.

### Chọn mức chi tiết theo nhiệm vụ

Giữ bản bàn giao độc lập model. Nêu kết quả quan sát được, quyền, khả năng
công cụ bắt buộc và lệnh đã xác minh. Ghi model và effort được yêu cầu rõ ràng
cho đợt đánh giá dưới dạng metadata runtime. Work ánh xạ các khả năng sang
công cụ sẵn có và đánh dấu bước `BLOCKED` khi thiếu khả năng bắt buộc. Giữ các
kiểm tra an toàn áp dụng và theo quy trình tương thích model trong `en/docs/model-compatibility.md`
khi thay đổi runtime.

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
Dùng lại context đã nạp khi xác minh được định danh. Mở lại source khi revision
đổi hoặc có thiếu sót ảnh hưởng tới quyết định. Dừng khảo sát khi mục tiêu,
thẩm quyền, phạm vi và yêu cầu kiểm chứng đủ để bàn giao an toàn.

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

Khi đọc thất bại, ưu tiên dùng lại bằng chứng đã xác nhận. Chỉ thử lại bằng
cách đã xác minh nếu dữ kiện thiết yếu cho việc bàn giao an toàn và cách đó
giữ nguyên phạm vi đọc và giới hạn output. Tuân thủ từ chối truy cập và chính
sách shell. Khi thao tác đã nêu, owner và phạm vi an toàn đã rõ, ghi
`NOT_INSPECTED` cho source chưa đọc hoặc lệnh chưa xác minh. Giao Work preflight
có phạm vi, vị trí nguồn, phép kiểm được phép và điều kiện dừng. Chỉ chặn bàn
giao khi quyết định, thẩm quyền, xung đột hoặc điều kiện an toàn thiết yếu cản
trở bàn giao an toàn. Ghi rõ lần đọc bị chặn và phép kiểm chưa chạy.

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

Khi có inspector đã xác minh và có giới hạn, đưa cho Work đúng đường dẫn tương đối,
lệnh gọi metadata/projection, digest source và giới hạn output. Ưu tiên định danh
trong allowlist thay cho tự viết tìm kiếm chứa giá trị. Shell không bị giới hạn vẫn
có thể làm lộ dữ liệu; sanitizer trace hoặc audit về sau không thể hoàn tác phơi lộ
cho mô hình. Giữ rõ câu hỏi về quyền và giới hạn này.

Giao Work đối chiếu tham chiếu với source hiện tại được phép và dùng lại bằng
chứng khớp. Với tham chiếu Work không mở lại được, đưa trực tiếp dữ kiện đáng
tin cậy thiết yếu, định danh/phiên bản nguồn, ràng buộc và nghiệm thu. Dùng cách
này khi bằng chứng đáng tin cậy xác lập phạm vi an toàn. Đánh dấu `BLOCKED` cho
bước thiếu thẩm quyền hoặc bằng chứng thiết yếu. Chat phải đọc được chính mẫu
tiếng Anh được phép theo Skill biên soạn.

Đưa bằng chứng cần thiết cho lỗi tạm thời, ca khó tái hiện, security, safety
hoặc compatibility. Giữ thẩm quyền kiến trúc,
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
  Gom tiêu chí cùng một phép kiểm bao phủ, giữ nguyên mọi assertion ban đầu.
  Không yêu cầu chạy hai lần cùng test chưa đổi dưới nhãn focused/full-suite;
  chỉ lặp khi source, đầu vào, phạm vi hoặc bằng chứng còn thiếu yêu cầu.
  Khi lọc output kiểm chứng, giữ các dữ kiện đã xác minh an toàn: marker stage
  máy đọc được, mã thoát tiến trình con dạng số và định danh assertion. Loại
  payload nhạy cảm mà không đổi nhãn trong giao thức output.
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
<!-- source-sha256: 89cbf9b5164443b93a43abd9369c8aee6f245c8ab91893ed7b2fb5a40db24bf3 -->
