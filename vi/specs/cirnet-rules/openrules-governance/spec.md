<!-- doc-id: OPENRULES-GOVERNANCE-SPEC -->
<!-- revision: 2 -->
<!-- source-sha256: 2ef7ef8a7f381d898b51d7564f8ca5b48317412d54816379f0a772fc967214fb -->
<!-- translation-status: synchronized -->
<a id="acceptance"></a>

# Nghiệm thu governance OPENRULES

Chỉ chỉnh sửa checkout cirnet-rules. Các thao tác ghi để kiểm thử dùng workspace và database tạm. Đặc tả OPENRULES đính kèm định nghĩa đầy đủ các trường hợp OR-01 đến OR-64. Ghi bằng chứng riêng và để mở các gate chưa đạt.

- [x] Kiểm chứng định danh OPENRULES, di trú và xử lý xung đột kho mà không làm lộ dữ liệu người dùng.
- [x] Kiểm chứng chu trình soạn thảo native offline tiếng Anh và tiếng Việt, bao gồm duyệt, áp dụng và rollback.
- [x] Kiểm chứng import, tệp đính kèm, xung đột revision và giới hạn phụ thuộc.
- [ ] Kiểm chứng gán phạm vi và ranh giới context hiệu lực, phân biệt xem trước với thực thi.
- [ ] Kiểm chứng contract adapter đã pin và thu bằng chứng thực thi live riêng cho hai engine.
- [ ] Kiểm chứng dòng đời task chung, thực thi có kiểm soát, review độc lập và snapshot automation.
- [ ] Kiểm chứng khôi phục, hồi quy giao diện, workload hiệu năng và ma trận khả năng truy cập được yêu cầu.
- [ ] Ghi kết quả nghiệm thu, bằng chứng capability và release readiness.

Phạm vi đã kiểm chứng dùng fixture cô lập. QTest native kiểm editor song ngữ, tệp đính kèm, duyệt, áp dụng, lịch sử và khôi phục. Fixture Rust kiểm di trú SQLite cục bộ, gói JSON, revision lỗi thời và kiểm tra phụ thuộc. Ma trận nghiệm thu OR đầy đủ được ghi riêng.
