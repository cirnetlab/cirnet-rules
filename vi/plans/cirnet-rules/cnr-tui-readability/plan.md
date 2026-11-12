<!-- doc-id: CNR-TUI-READABILITY-PLAN -->
<!-- revision: 1 -->
<!-- source-sha256: 7157f3ae1e5d17327c4a6270816b509c135cc603d868a5350d24cc944dc99e98 -->
<!-- translation-status: synchronized -->
<a id="implementation"></a>

# Kế hoạch cải tiến Check và editor CNR

1. CP1 ghi nhận checkout và tái hiện lỗi wrap ở terminal hẹp cùng những phím còn thiếu.
2. CP2 hiển thị finding có kiểu, ưu tiên mục cần xử lý và dùng phần chi tiết thích ứng. Kiểm thử trạng thái, bộ lọc, tìm kiếm, che dữ liệu khi sao chép và resize.
3. CP3 dùng chung editor theo cụm ký tự cho lệnh, tìm kiếm và các trường thiết lập. Kiểm thử chọn vùng, undo, redo và giới hạn dán.
4. CP4 ánh xạ vùng chọn nhập và kết quả theo ô terminal, bổ sung menu ngữ cảnh và clipboard Windows. Kiểm thử phương án thay thế và hủy tác vụ đang chạy.
5. CP5 xác minh Windows Terminal, build bằng toolchain đã ghim, chạy toàn bộ test và đồng bộ tài liệu vận hành.

[Phạm vi và quyết định giao diện](../../../specs/cirnet-rules/cnr-tui-readability/spec.md)
