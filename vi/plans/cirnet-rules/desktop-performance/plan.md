<!-- doc-id: CIRNETRULES-DESKTOP-ASYNC-PLAN -->
<!-- revision: 1 -->
<!-- source-sha256: 76c43679bdedf31457f14e3b0c0276594798ba02ab482d3caf19b6045f8b6a01 -->
<!-- translation-status: synchronized -->
<a id="steps"></a>

# Kế hoạch triển khai Desktop bất đồng bộ

[Tiêu chí nghiệm thu](../../../specs/cirnet-rules/desktop-performance/spec.md)

**Mục tiêu** Giữ thao tác chọn và nhập liệu phản hồi trong khi bảo toàn thứ tự và thẩm quyền của thao tác lưu.

**Kiến trúc** Dùng event loop Qt hiện tại cho widget và xử lý phản hồi có giới hạn, cùng broker cho lượt đọc phục vụ view. Tái sử dụng worker lưu trữ và scheduler, tách lượt đọc SQLite tương tác khỏi dispatch chậm. Chọn thay đổi transport và transcript dựa trên thời gian parse và render đã đo.

**Nền tảng** Qt Widgets và C++20, Rust đã pin, SQLite WAL và stdio JSON-lines v3 riêng.

1. Build Release tại revision baseline. Thu mẫu paint và heartbeat native rồi so sánh độ trễ đọc đồng thời khi tái hiện runtime bị chặn. Ghi cấu trúc dữ liệu và query plan SQLite.
2. Thêm cơ chế lấy ý định mới nhất cho request chi tiết và poll trong `runtime_client.*` và `conversations.cpp`. Kiểm tra đổi nhanh và A→B→A trước/sau, gồm phản hồi sau khi đổi trang.
3. Định tuyến các lượt đọc được phân loại rõ trong `src/bin/cirnetrules-runtime.rs`. Cấp connection riêng cho worker qua storage factory. Giữ FIFO của thao tác ghi, kiểm tra hủy, giới hạn hàng đợi và đầu ra chậm.
4. Trả watermark sự kiện và giới hạn kết quả hội thoại trong `src/dashboard/sessions.rs`. Thử sự kiện sau snapshot và phân trang trên fixture SQLite. Kiểm tra preview và apply dùng cùng thẩm quyền.
5. Giới hạn định dạng transcript, bảo toàn định danh tin nhắn, bản nháp và vị trí cuộn. Thu số đo thời gian và tài nguyên local. Chạy kiểm chứng toàn bộ và ghi gate còn lại.

Rà soát tranh chấp ghi liên process, đóng view khi phản hồi muộn, retry thao tác đã commit và quay lại hội thoại có lượt đọc trước đang chạy.
