# Mẫu ma trận tương thích

Ghi owner, miền phiên bản, bề mặt công khai, policy liên quan và phạm vi được phép.
Đánh dấu input chưa biết. Tách source hiện tại, candidate và artifact được chọn.

| Revision producer / hash artifact | Revision consumer / pin được chọn | Thay đổi và chiều | Source / wire / dữ liệu / hành vi | Test và kết quả | Migration / giới hạn rollback |
| --- | --- | --- | --- | --- | --- |
| Định danh chính xác | Định danh chính xác hoặc unknown | Writer tới reader, cũ tới mới hoặc operation API | Miền bị ảnh hưởng | Lệnh, môi trường, PASS / FAIL / NOT_RUN / BLOCKED | Việc phải làm hoặc unknown |

Liệt kê thời hạn deprecation và chiều chưa thử. Dẫn bằng chứng tại repo sở hữu.
Lựa chọn release phải được xác minh bằng dữ liệu release của owner.

<!-- source: en/templates/compatibility-matrix.md -->
<!-- source-sha256: e088c6dcaedd9343f2a0b262d3bfbd524e3db30bea94b77c3f5c6d7742bb2492 -->
