<!-- doc-id: DASHBOARD-RESTORE-CI-PLAN -->
<!-- revision: 1 -->
<!-- source-sha256: a7fcd192831e3d3c35b538aad36c61705dc287660babcc0a33f98c8ba7758752 -->
<!-- translation-status: synchronized -->
<a id="steps"></a>

# Kế hoạch sửa CI khôi phục Dashboard

[Phạm vi và nghiệm thu](../../../specs/cirnet-rules/dashboard-restore-ci/spec.md)

1. Truy nguyên lỗi khôi phục đến thư mục do bài kiểm thử tích hợp tạo.
2. Tạo thư mục khôi phục với quyền chỉ dành cho chủ sở hữu trên Unix trước khi sao chép bản chụp SQLite.
3. Chạy kiểm chứng governance bằng toolchain đã ghim và ghi nhận kết quả trên các nền tảng có thể kiểm tra.
