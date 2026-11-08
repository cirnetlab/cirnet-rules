<!-- doc-id: CGACP-CONVERSATION-AUTOMATION-DELETION-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: dcc13002cbea353e7020a3e0244f24b282c458a8da02149bf5cab9395e1dda34 -->
<!-- translation-status: synchronized -->
<a id="acceptance"></a>

# Xóa Hội thoại và Automation

- [x] Khi xóa cả Hội thoại do một lượt chạy sinh ra, Automation gốc biến mất khỏi danh sách và dừng lịch. Các Hội thoại khác và metadata lượt chạy đã kết thúc vẫn còn.
- [x] Bản xem trước gắn với đúng ID và revision của Automation. Lượt chạy còn hoạt động hoặc còn lease chặn xóa.
- [x] Xóa một tin nhắn, lưu trữ hoặc bản ghi trùng tên nhưng không liên kết không làm đổi trạng thái Automation.
- [x] Có thể xóa riêng Automation đã tách liên kết bằng ID. Thử lại và khởi động lại giữ kết quả xóa. Qt cập nhật danh sách hiển thị.
