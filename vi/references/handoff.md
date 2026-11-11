<!-- doc-id: HANDOFF -->
<!-- revision: 3 -->
<!-- source-sha256: 9ffd1d4bd9e6fa4629d43d0d9154efaecc24cef4dd7c864d9b84e495de777150 -->
<!-- translation-status: synchronized -->
<a id="decision"></a>

# Bàn giao giữa người vận hành và coding agent

1. Xác nhận repository, branch, binding của worktree và revision của task.
   Đọc chỉ dẫn tiếng Anh được chọn và rà soát phạm vi đọc, ghi chính xác trong
   `cnr task context --task ID --session TOKEN`.
2. Mở worktree đã bind trong Codex Desktop, Claude Code hoặc Antigravity.
   Dùng [chỉ dẫn và kỹ năng của repository](../adapters/README.md) hoặc phiên
   di động đã được duyệt. Để kết nối client với kho task dùng chung, kiểm tra
   [cấu hình khởi chạy MCP](mcp.md) và chế độ truy cập đã chọn.
3. Với task phối hợp, dùng phiên đã chuẩn bị để nhận lease và gửi báo cáo
   có cấu trúc. Bàn giao ID task và các tham chiếu biên nhận liên quan theo
   [quy trình phối hợp dùng chung](shared-task-coordination.md). Giữ tệp phiên
   ở chế độ riêng tư.
4. Người kiểm chứng đối chiếu diff và bằng chứng trên binding đã chốt, rồi
   ghi kết quả theo [quy trình vận hành](cirnetrules-local.md). Agent tiếp theo
   đọc trạng thái và ngữ cảnh hiện tại trước khi nhận việc.

<a id="record-templates"></a>

## Hồ sơ bàn giao

| Hồ sơ | Trường cần ghi |
| --- | --- |
| Quyết định thiết kế | ID task, ràng buộc, quyết định, lý do, phạm vi ảnh hưởng, người phụ trách và tham chiếu phê duyệt. |
| Bàn giao task | ID và revision, repository và binding, kết quả mong đợi, ID task phụ thuộc, trạng thái, việc còn lại và bước tiếp theo. |
| Bằng chứng kiểm chứng | Revision source và governance, lệnh, phiên bản công cụ, nền tảng, thời điểm, mã thoát, digest của biên nhận và giới hạn. |
| Đề xuất đổi phạm vi | Tệp hoặc hành động cần cấp quyền, lý do, người phụ trách, thời hạn và tham chiếu phê duyệt sau khi được cấp. |

Dùng [chính sách governance](../policies/governance.md) khi thay đổi phạm vi
và [hướng dẫn kiểm chứng](verification.md) khi ghi kết quả.
