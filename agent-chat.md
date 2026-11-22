## Prompt giao nhiệm vụ mới cho AI Agent Chat

Dùng Skill `cirnet-work-prompt-authoring` để khảo sát trong phạm vi đọc và soạn một Work Prompt cho Codex. Viết hướng dẫn bằng tiếng Anh, giữ nguyên văn và thứ tự các dòng yêu cầu đã duyệt.

**Yêu cầu gốc**

[DÁN NGUYÊN VĂN NHIỆM VỤ TẠI ĐÂY.]

Ghi riêng repo đích, quyền đã cấp và đích lưu nếu có.

1. **Xác nhận đầu vào.** Đọc mẫu và quy trình xác thực từ `cirnet-rules` đã duyệt. Trước khi khảo sát, xác nhận yêu cầu gốc, checkout đích và hồ sơ cấp quyền. Kiểm tra SHA-256 của hồ sơ với giá trị nhận riêng từ người duyệt. Chỉ dùng bản trích xuất đã duyệt cho nguồn nhạy cảm. [Mẫu tiếng Anh](en/templates/chat-to-work-prompt.md) · [Xác thực](en/docs/context.md#offline-handoff-validation)
2. **Đối chiếu nguồn.** Xác định repo sở hữu. Đọc `AGENTS.md` có hiệu lực, Rules/Skills, Plans/Specs và phần mã, hợp đồng, artifact đã chọn, kiểm thử liên quan. Khi ảnh hưởng kiến trúc hoặc quyền sở hữu, đọc phần kiến trúc tiếng Anh có thẩm quyền. Tra theo đường dẫn/symbol và dùng lại bằng chứng đúng revision. Tách dữ kiện khỏi giả thuyết. Ghi `NOT_INSPECTED` kèm bước xác minh nếu nguồn chưa đọc và phạm vi đã rõ. [Chính sách kiến trúc](en/policies/architecture.md) · [Bảng chủ sở hữu](en/references/architecture-routing.md)
3. **Soạn handoff.** Chọn LIGHT/STANDARD/DEEP theo rủi ro và độ phức tạp. Nêu thay đổi cụ thể, kết quả quan sát được, quyền đọc/ghi/thực thi, ràng buộc và trường hợp lỗi. Gắn mỗi tiêu chí nghiệm thu với lệnh hoặc quan sát đã xác minh. Chỉ nơi tra cứu khi chưa biết lệnh. Dẫn đúng mục và revision, giữ mỗi ý một lần.
4. **Kiểm tra và giao việc.** Trước khi giao, chạy `scripts/validate-work-prompt.ps1` và yêu cầu `handoffStatus = PASS`. Codex phải xác nhận repo/worktree, branch/HEAD, trạng thái Git và quyền, rồi đọc và đối chiếu nguồn chi phối tại checkout thực tế. Chỉ thực thi khi `-ExecutionPreflight` cho `executionStatus = PASS`. Khi ghi hoặc chạy build/test có thể ghi, dùng baseline phạm vi đã duyệt và đối soát sau chạy. Tác vụ nhiều phiên cần checkpoint trước khi sửa và cập nhật tiến độ. Báo cáo đầu ra, tệp đổi, kết quả kiểm tra thực tế và giới hạn. [Task Continuity](en/docs/distribution.md#task-continuity)

**Đầu ra**

Khi đủ điều kiện, lưu một Work Prompt Markdown tại đích được phép. Trả `PROMPT_READY`, liên kết và tóm tắt tiếng Việt. Nếu thiếu điều kiện thiết yếu, trả `PROMPT_BLOCKED` và nêu phần cần bổ sung.


## Prompt phụ giao nhiệm vụ cho AI Agent Chat

Hãy biên soạn Prompt Tiếng Anh build mã nguồn cirnet-rules sao cho tối ưu nhất. Tôi muốn Prompt được biên soạn theo hướng tối ưu hiệu quả thực thi, rõ ràng, chính xác và tiết kiệm token: xác định mục tiêu cụ thể, đặt ra các ràng buộc nghiêm ngặt, yêu cầu Codex chủ động đọc và đối chiếu tài liệu có thẩm quyền trước khi triển khai. Prompt cần đủ thông tin để AI hiểu đúng yêu cầu và thực hiện chính xác ngay từ đầu, tránh viết quá dài, lặp ý, dư thừa nội dung gây lãng phí token, nhưng cũng không quá ngắn đến mức thiếu bối cảnh, dữ kiện hoặc ràng buộc quan trọng dẫn đến hiểu sai và phải thực hiện lại.

## Prompt tiếp tục tác vụ khi hết quota hoặc đổi tài khoản

```text
Hãy tiếp tục tác vụ còn dang dở

Continue the task that is still in progress
```

```text
Hãy tiếp tục tác vụ đang dang dở bằng CIRNET Task Continuity. Dùng `scripts/task-continuity.ps1 bootstrap` từ checkout `cirnet-rules` đã duyệt theo `en/docs/distribution.md`. Xác minh checkpoint, đối chiếu Git và quyền hiện hành. Giữ thay đổi có sẵn và lưu checkpoint mới ở mốc tiếp theo.
```
