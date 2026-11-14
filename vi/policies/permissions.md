# Chính sách phạm vi và quyền

Dùng [baseline](../AGENTS.md) và quy tắc ownership local của repo đích.
Chính sách này mô tả ranh giới hành động, không phải database cấp quyền hay task engine.

| Hành động | Căn cứ cần có |
| --- | --- |
| Đọc source/chỉ dẫn liên quan | Liên quan task và được phép truy cập |
| Sửa code và tài liệu theo cặp | Phạm vi repo/file người dùng đã cho phép |
| Chạy kiểm tra/build | Prerequisites local và side effect/vị trí output được phép |
| Sửa repo khác hoặc cấu hình global | Phạm vi rõ ràng bao phủ nơi đó |
| Commit, push, merge, publish, deploy hoặc vận hành thiết bị thật | Cho phép hành động đó; yêu cầu safety/release của owner |

Tiếp tục công việc thông thường đã được cho phép. Trước hành động vượt phạm vi,
làm rõ tác động đề xuất, giữ công việc hiện hữu và hỏi phần quyền còn thiếu.
Plan, tài liệu tham khảo hoặc bản dịch không thể cấp quyền đó.

Hook tài liệu chỉ đọc và kiểm tra checkout common-rules này.
PASS của nó không chứng minh nghiệm thu release sản phẩm, cưỡng chế quyền truy cập
hay an toàn vật lý. Cài hook vào client native là hành động cấu hình riêng.

<!-- source: en/policies/permissions.md -->
<!-- source-sha256: 9b27c0e57dc902a728beb5c63bd5eec3f3835b3982a2097ef0e982a66857c1fb -->
