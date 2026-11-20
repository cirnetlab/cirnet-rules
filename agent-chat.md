## Prompt giao nhiệm vụ mới cho AI Agent Chat

Hãy đọc mã nguồn `cirnet-rules` và sử dụng Skill `cirnet-work-prompt-authoring` để biên soạn **Work Prompt** cho nhiệm vụ sau:

**Nhiệm vụ:** [MÔ TẢ CHỨC NĂNG CẦN PHÁT TRIỂN HOẶC CHỈNH SỬA]

Yêu cầu:

- Chủ động đọc và đối chiếu kiến trúc CIRNETLAB, repository liên quan, Rules, Skills, Plans/Specs và mã nguồn hiện có.
- Xác định chính xác mục tiêu, phạm vi được phép chỉnh sửa, yêu cầu kỹ thuật, ràng buộc và tiêu chí nghiệm thu.
- Bảo toàn yêu cầu gốc, không tự ý mở rộng phạm vi hoặc suy đoán những thông tin chưa được xác minh.
- Soạn Work Prompt bằng tiếng Anh, rõ ràng, chính xác, tối ưu hiệu quả thực thi và tiết kiệm token.
- Yêu cầu Agent Work sử dụng Task Continuity để bảo toàn tiến độ khi nhiệm vụ có thể kéo dài qua nhiều phiên.
- Chỉ biên soạn Prompt, không trực tiếp triển khai mã nguồn.

**Đầu ra:** Một file Markdown chứa Work Prompt hoàn chỉnh, kèm phần tóm tắt ngắn gọn bằng tiếng Việt trong cuộc trò chuyện để tôi giao cho AI Agent Work.

## Prompt tiếp tục tác vụ khi hết quota hoặc đổi tài khoản

```text
Hãy tiếp tục tác vụ còn dang dở

Continue the task that is still in progress
```

- AI Chat sẽ tự xác định kiến trúc, Repository, Rules/Skills, phạm vi chỉnh sửa, yêu cầu kỹ thuật và kiểm thử; sau đó tạo Prompt để giao cho AI Work.
