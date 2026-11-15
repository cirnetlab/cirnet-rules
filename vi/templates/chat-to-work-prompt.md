# Mẫu Prompt từ Chat sang Work

Thay các placeholder bằng dữ kiện của nhiệm vụ và chỉ dẫn cho Work Agent.
Giữ đủ 12 mục. Với nhiệm vụ nhỏ, rút gọn từng mục và ghi rõ phần không áp dụng.
Dùng cùng cấu trúc cho Feature, Bugfix, Refactor, Integration, FPGA, Performance,
Documentation, Contract và Verification. Mặc định trả một Prompt trong chat.

## 01. Định danh nhiệm vụ

`{{TASK_NAME}}` · `{{TASK_ID_IF_NEEDED}}` · `{{TASK_TYPE}}` · Chủ sở hữu `{{TARGET_REPOSITORY}}`

## 02. Mục tiêu

`{{USER_REQUEST}}` · `{{CURRENT_BEHAVIOR}}` · `{{EXPECTED_BEHAVIOR}}` · `{{EXPECTED_OUTPUT}}`

## 03. Phạm vi repository

`{{WRITABLE_REPOSITORY_AND_PATHS}}` · `{{READ_ONLY_REPOSITORIES}}` · `{{AUTHORIZED_ACTIONS}}`

Ghi các đường dẫn được sửa hoặc vùng có ranh giới để Work xác nhận khi kiểm
tra trước. Ghi quyền cho từng repo và thao tác bị ảnh hưởng.

## 04. Thẩm quyền kiến trúc

`{{ARCHITECTURE_REFERENCES}}` · `{{APPLICABLE_DECISIONS}}` · `{{RULES_AND_SKILLS}}`

Dẫn các mục áp dụng, chủ sở hữu và chỉ dẫn local. Chỉ đưa vào những quyết
định ràng buộc nhiệm vụ này.

## 05. Bằng chứng mã nguồn và kiểm tra trước

`{{SOURCE_EVIDENCE}}` · `{{WORK_PREFLIGHT}}`

Gắn nhãn bằng chứng cho từng nhận định. Dùng `CONFIRMED` cho source hoặc artifact
đã quan sát, `INFERRED` cho giả thuyết kèm phép kiểm tra để xác định, `NOT_INSPECTED`
cho đầu vào chưa xem và `BLOCKED` cho điều kiện tiên quyết chưa thỏa. Dẫn chính xác
đường dẫn, ký hiệu, API, manifest, test và revision khi đã xác minh. Yêu cầu Work
kiểm tra lại checkout, phạm vi và bằng chứng liên quan trước khi chọn cách sửa.

## 06. Yêu cầu chức năng

`{{REQUIREMENTS_AND_NEGATIVE_CASES}}`

Nêu hành vi cần có, các đường lỗi và hành vi cần giữ nguyên.

## 07. Ràng buộc kỹ thuật

`{{TECHNICAL_CONSTRAINTS}}`

Ghi các ràng buộc runtime, bảo mật, an toàn, tương thích và hiệu năng áp dụng.
Xác định contract của chủ sở hữu, pin chính xác và artifact đã đóng băng.

## 08. Quy trình triển khai

Khảo sát → Tái hiện/xác nhận → Lập kế hoạch → Triển khai → Kiểm thử → Rà soát.

`{{TASK_STEPS}}`

## 09. Tiêu chí nghiệm thu

`{{ACCEPTANCE_CRITERIA_AND_CHECKS}}`

Gắn mỗi kết quả quan sát được với phép kiểm tra và bằng chứng cần thu.

## 10. Build và kiểm chứng

`{{VERIFICATION_COMMANDS_AND_PREREQUISITES}}`

Nêu lệnh và môi trường đã xác minh hoặc giao Work tìm chúng trong chỉ dẫn
của chủ sở hữu. Kết luận E2E thật cần chuỗi thực thi thật và pin chính xác.
Báo phạm vi của mock hoặc mô phỏng. HIL cần quyền riêng và bằng chứng an toàn
của chủ sở hữu.

## 11. Tài liệu và đầu ra

`{{DOCUMENTATION_AND_DELIVERABLES}}`

Xác định các cặp EN/VI bị ảnh hưởng, artifact phải bàn giao và nơi ghi báo cáo
trong phạm vi được phép.

## 12. Báo cáo cuối và quyền thao tác

`{{REPORT_REQUIREMENTS}}` · `{{SEPARATE_AUTHORIZATIONS}}`

Yêu cầu báo đường dẫn đã sửa, lệnh đã chạy, mã thoát, `PASS`, `FAIL`, `NOT_RUN`
hoặc `BLOCKED` cho từng phép kiểm tra, ảnh hưởng tương thích và giới hạn. Ghi quyền
tường minh hoặc điều kiện tiên quyết cho thay đổi Git, công bố, phát hành,
triển khai và vận hành vật lý. Bản bàn giao tự nó không cấp quyền.

<!-- source: en/templates/chat-to-work-prompt.md -->
<!-- source-sha256: f0cf740789f86dc58e32b1c7e1e28604f5a3e7c0ae2919a50326aad314fdacf4 -->
