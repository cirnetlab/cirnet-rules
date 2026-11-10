<!-- doc-id: CIRNETRULES-IDENTITY-RECOVERY-PLAN -->
<!-- revision: 1 -->
<!-- source-sha256: ac69bac32f89258988b2e8e587faa4539b078eda7fa660481d3df8fb1a3857fc -->
<!-- translation-status: synchronized -->
<a id="recovery-steps"></a>

# Kế hoạch hoàn tất kiểm tra khôi phục định danh

[Tiêu chí nghiệm thu](../../../specs/cirnet-rules/identity-recovery/spec.md)

1. Tái hiện task bảo trì bị chặn và lỗi ACL Windows bằng fixture riêng.
2. Bổ sung khôi phục từng task qua công cụ bảo trì hiện có. Giữ bằng chứng lưu trữ và yêu cầu cấp quyền mới trước khi thực thi.
3. Kiểm tra nguồn và thẩm quyền thay đổi, xung đột, mở lại, binding mới và dựng lại cache. Kiểm tra journal SQLite tiếp tục sau khi khôi phục ACL.
4. Chạy các bước kiểm chứng hiện hành, đồng bộ tài liệu vận hành và ghi kết quả nghiệm thu.
