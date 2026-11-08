<!-- doc-id: TASK-LOCK-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: 2c60df4834213c9b36f53ccffb75a7a88e9566fd2021d1a77dddea47d2b6212a -->
<!-- translation-status: synchronized -->
<a id="acceptance"></a>

# Nghiệm thu hồi quy khóa tác vụ

- [x] Tái hiện khóa còn giữ bằng bản sao handle của tệp.
- [x] Giải phóng khóa tác vụ đã kết thúc ngay cả khi bản sao vẫn mở.
- [x] Duy trì từ chối ghi đồng thời và xung đột revision.
- [x] Kiểm tra workflow trên Windows và đồng bộ hướng dẫn.
- [ ] Kiểm tra workflow đã sửa trên runner Linux.
