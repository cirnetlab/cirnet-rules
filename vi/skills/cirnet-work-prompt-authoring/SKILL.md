---
name: cirnet-work-prompt-authoring
description: Dùng khi được yêu cầu viết Work Prompt hoặc chỉ dẫn bàn giao để Codex, Claude Code, Antigravity hay Agent khác thực thi. Yêu cầu triển khai trực tiếp dùng Skill thực thi phù hợp.
---

# Biên soạn Work Prompt

Trong vai trò Chat, khảo sát chỉ đọc trong phạm vi được phép và soạn nội dung
bàn giao. Khi được yêu cầu trực tiếp thực hiện nhiệm vụ, dùng Skill triển khai,
gỡ lỗi hoặc kiểm chứng phù hợp. Work Agent kiểm tra lại checkout local, nhánh,
quyền và bằng chứng, sau đó lập kế hoạch, sửa, kiểm thử và báo cáo đúng phạm vi.

1. Đọc yêu cầu và xác định kết quả cần đạt, đường dẫn repo được phép sửa,
   phụ thuộc chỉ đọc và thao tác đã được cấp quyền.
2. Mở `en/policies/architecture.md` và `en/references/architecture-routing.md`
   từ toolkit được phép. Xác định chủ sở hữu, chọn các mục tiếng Anh Ecosystem,
   Lab-First hoặc FPGA áp dụng cùng Rules và Skills liên quan. Giữ các ràng
   buộc công nghệ áp dụng và `LAB_FIRST_PRIMARY`. Công việc Ecosystem độc lập
   có thể tiếp tục trong ranh giới của mình. Chuyển sang `PARALLEL_ALLOWED`
   cần bằng chứng theo chính sách.
3. Khảo sát source, contract, test và manifest của chủ sở hữu khi truy cập
   được. Ghi dữ kiện quan sát cùng phạm vi và đánh dấu nhận định chưa xác minh.
   Nếu Chat không truy cập source nhưng biết chủ sở hữu và phạm vi an toàn,
   dùng `NOT_INSPECTED` và giao Work xác nhận khi kiểm tra trước. Kết quả test
   trước đây chỉ là bằng chứng cho đầu vào ban đầu của chúng.
4. Đọc `en/templates/chat-to-work-prompt.md` từ checkout `cirnet-rules` đã
   được phép và điền chỉ dẫn chuyên biệt. Bundle đã cài cung cấp Skill này và
   cần quyền truy cập checkout đó để đọc mẫu. Nếu không truy cập được checkout
   hoặc mẫu, báo `PROMPT_BLOCKED` cùng quyền truy cập cần có.

Trước khi trả Prompt, kiểm tra chủ sở hữu, kiến trúc, ranh giới với repo khác,
nhãn bằng chứng và tiêu chí nghiệm thu quan sát được. Giữ tính tương thích của
contract, pin chính xác và artifact đóng băng. Tại ranh giới Lab vật lý, tách
Safety khỏi Correctness, gắn kết theo topology đã quan sát và loại bỏ việc tự
sửa dây sinh viên. Dùng khả năng và kết quả có bằng chứng. Gắn kết luận E2E thật
với các ranh giới thực sự được chạy. Chỉ chuyển quyền đã được cấp, nhất là khi sửa nhiều repo, công bố,
triển khai hoặc vận hành vật lý.

Trả `PROMPT_READY` và một Work Prompt có thể sao chép trong chat khi đủ điều
kiện bàn giao đúng phạm vi. Work vẫn phải cung cấp bằng chứng để xác nhận
`IMPLEMENTATION_VERIFIED`. Trả `PROMPT_BLOCKED` cùng đầu vào, quyền còn thiếu
hoặc quyết định xung đột nếu điểm đó cản trở việc bàn giao an toàn. Chỉ kèm
Work Prompt cho một nhiệm vụ tách được nếu nhiệm vụ đó đã được cấp quyền riêng.

<!-- source: en/skills/cirnet-work-prompt-authoring/SKILL.md -->
<!-- source-sha256: a2d00eb57892c7d9dcf9bcf11f9feeae2a80dc656e8a1c3e22e994bac083e7c7 -->
