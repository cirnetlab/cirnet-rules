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

Quyền sửa mã không cấp quyền đọc, xuất hay chia sẻ dữ liệu. Chỉ yêu cầu thêm tính
năng xuất không cho phép truy cập mọi tài khoản hoặc trường. Xác lập quyền của
người thực hiện đối với các bản ghi và trường dự kiến từ yêu cầu và contract đáng
tin cậy; hỏi về quyết định chưa rõ trước khi triển khai quyền truy cập đó. Áp dụng
[hướng dẫn xuất và tiết lộ dữ liệu](../rules/security.md#xuất-và-tiết-lộ-dữ-liệu)
khi liên quan, và dùng lại quyền đã được xác lập cho nhiệm vụ.

Hook tài liệu chỉ đọc và kiểm tra checkout common-rules này.
PASS của nó không chứng minh nghiệm thu release sản phẩm, cưỡng chế quyền truy cập
hay an toàn vật lý. Cài hook vào client native là hành động cấu hình riêng.

<!-- source: en/policies/permissions.md -->
<!-- source-sha256: 3f625947de7bdbd042166ad77ff73f2640f6b71de20f45e0610bb5505ad6bf7d -->
