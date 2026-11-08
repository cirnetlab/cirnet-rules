<!-- doc-id: TESTS -->
<!-- revision: 1 -->
<!-- source-sha256: e2de0dc00e6c4368607a4a8eac39b512017bad504d303e46cfe9294a1fe13168 -->
<!-- translation-status: synchronized -->

<a id="verification"></a>

# Kiểm thử governance

Chạy cargo test với lockfile. Test Rust bao phủ digest bản dịch, định danh
máy, đầu vào sai, gate release ứng viên và thao tác adapter cô lập. CI cũng
chạy validation CLI và kiểm tra workspace sản phẩm bị thiếu cho trạng thái
bị chặn. Test sản phẩm chạy tại repo sở hữu.
[Quy tắc bằng chứng](../rules/testing.md)
