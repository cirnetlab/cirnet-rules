## Prompt giao nhiệm vụ mới cho AI Agent Chat

Hãy dùng Skill `cirnet-work-prompt-authoring` để soạn **một Work Prompt bằng tiếng Anh** cho Codex thực thi nhiệm vụ dưới đây và soạn Prompt Tiếng Anh.

**Yêu cầu gốc**

[DÁN NGUYÊN VĂN NHIỆM VỤ TẠI ĐÂY.]

Bổ sung repository đích, phạm vi đã cấp và đường dẫn lưu prompt nếu có.

**Cách soạn**

1. Từ yêu cầu gốc, xác định hành động, đối tượng, thay đổi cần làm, kết quả quan sát được và quyền đã cấp. Dùng checkout `cirnet-rules` được phê duyệt để đọc mẫu `en/templates/chat-to-work-prompt.md` và áp dụng bước xác thực tại `en/docs/context.md#offline-handoff-validation`. Dùng thông tin phê duyệt từ người có thẩm quyền.
2. Xác định repository chịu trách nhiệm bằng `en/policies/architecture.md` và `en/references/architecture-routing.md` khi nhiệm vụ liên quan đến kiến trúc hoặc ranh giới sở hữu. Đọc đúng phần tiếng Anh chi phối thay đổi, `AGENTS.md` có hiệu lực, Rules/Skills áp dụng, Plans/Specs liên quan, mã nguồn, hợp đồng, phiên bản đang được chọn và kiểm thử của repo sở hữu. Tra cứu theo đường dẫn và symbol, dùng lại nguồn đã đọc ở đúng revision, dừng khi đủ bằng chứng. Ghi rõ dữ kiện đã xác minh, giả thuyết và phần chưa đọc bằng locator cụ thể.
3. Viết theo độ sâu phù hợp với rủi ro. Nêu mục tiêu, phạm vi đọc/ghi và tác động được phép, nguồn có thẩm quyền, ràng buộc kỹ thuật, hành vi khi lỗi, đầu ra và tiêu chí nghiệm thu. Gắn mỗi tiêu chí với lệnh hoặc quan sát đã xác minh. Nếu lệnh chưa biết, chỉ rõ nơi Codex cần tra cứu. Dẫn tới nguồn thay vì chép lại quy tắc chung. Bảo toàn mọi yêu cầu gốc và giữ mỗi ý ở một nơi.
4. Yêu cầu Codex xác nhận repo, worktree, branch/HEAD, trạng thái Git và quyền hiện hành. Trước khi sửa, Codex phải đối chiếu các nguồn chi phối với checkout thực tế, xử lý mâu thuẫn ảnh hưởng đến cách làm và đạt bước kiểm tra trước thực thi theo hướng dẫn ở bước 1. Với tác vụ có thể kéo dài qua nhiều phiên, tạo checkpoint Task Continuity trước khi sửa và cập nhật theo `en/docs/distribution.md#task-continuity`. Báo cáo tệp đã đổi, lệnh đã chạy, kết quả thực tế và giới hạn.

**Đầu ra**

Trả `PROMPT_READY` khi handoff đủ điều kiện, lưu Work Prompt vào một tệp Markdown ở đích được phép và gửi liên kết kèm tóm tắt tiếng Việt. Trả `PROMPT_BLOCKED` khi thiếu quyết định, thẩm quyền hoặc điều kiện thiết yếu, kèm thông tin cần bổ sung. Với nguồn chưa đọc mà phạm vi an toàn đã rõ, ghi `NOT_INSPECTED` và chỉ định bước xác minh cho Codex.

## Prompt tiếp tục tác vụ khi hết quota hoặc đổi tài khoản

```text
Hãy tiếp tục tác vụ còn dang dở

Continue the task that is still in progress
```

```text
Hãy tiếp tục tác vụ đang dang dở bằng CIRNET Task Continuity. Dùng `scripts/task-continuity.ps1 bootstrap` từ checkout `cirnet-rules` đã duyệt theo `en/docs/distribution.md`. Xác minh checkpoint, đối chiếu Git và quyền hiện hành. Giữ thay đổi có sẵn và lưu checkpoint mới ở mốc tiếp theo.
```
