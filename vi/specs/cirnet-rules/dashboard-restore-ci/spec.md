<!-- doc-id: DASHBOARD-RESTORE-CI-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: 3f255d2e431d11cfe93ba360d87f6e6ffc0c2c2df7dbb8155ad47011604d619d -->
<!-- translation-status: synchronized -->
<a id="acceptance"></a>

# Tiêu chí nghiệm thu CI khôi phục Dashboard

Fixture khôi phục phải chuẩn bị thư mục đích riêng tư và mở lại bản chụp SQLite đã sao chép thông qua runtime. Chính sách bảo mật thư mục của runtime tiếp tục được áp dụng.

- [x] Xác định lỗi kiểm thử Ubuntu và kết quả job Windows từ log Actions.
- [x] Kiểm chứng khôi phục và các kiểm tra governance bằng Rust 1.99.1 trên Windows.
- [x] Kiểm chứng khôi phục và các kiểm tra governance bằng Rust 1.99.1 trên Linux.

Job Ubuntu báo `PERMISSION_DENIED` khi mở lại bản chụp đã khôi phục. Job Windows trong matrix bị hủy giữa bước build. Kiểm chứng local trên Windows đạt kiểm tra định dạng, Clippy, build, toàn bộ 67 bài test và kiểm tra song ngữ. [Log Ubuntu](https://github.com/cirnetlab/cirnet-rules/actions/runs/42495833286/job/128777867294), [Log Windows](https://github.com/cirnetlab/cirnet-rules/actions/runs/42495833286/job/128777867386).

Container Linux cô lập tái hiện mã thoát 101 tại revision đã chạy trên Actions với Git 2.56.0. Sau khi áp dụng bản sửa bài kiểm thử, kiểm tra định dạng, Clippy, build, toàn bộ 68 bài test và validator đều đạt. Revision đã sửa vẫn chờ kiểm chứng trên runner của GitHub.
