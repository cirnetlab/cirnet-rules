<!-- doc-id: CIRNETRULES-IDENTITY-RECOVERY-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: d8f51bcbab1fe25641c769b24bd33246201a30b195249b022a620fa7d6e54add -->
<!-- translation-status: synchronized -->
<a id="recovery-acceptance"></a>

# Tiêu chí nghiệm thu khôi phục định danh

- [x] CR-23 khôi phục task cũ đã rà soát với cùng ID và biên nhận nguồn lưu bền vững. Task không kế thừa quyền hay trạng thái thực thi. Giữ khả năng đọc byte và bằng chứng gốc. Chặn đầu vào đã đổi hoặc xung đột. Binding và cache mới dùng đường dẫn hiện hành.
- [x] CR-26 kiểm chứng lỗi ACL Windows thực khi di trú SQLite. Runtime báo lỗi lưu trữ, giữ nguồn, bản sao lưu và journal, rồi tiếp tục sau khi khôi phục quyền truy cập.
- [x] Kiểm chứng bộ Rust, Desktop, đối chiếu tài liệu và phạm vi thay đổi. Chỉ dùng dữ liệu fixture.
