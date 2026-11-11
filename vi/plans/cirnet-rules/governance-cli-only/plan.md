<!-- doc-id: CNR-CLI-ONLY-PLAN -->
<!-- revision: 2 -->
<!-- source-sha256: 3a8d810e329b5f0389de378399db112b12d54db0f46129b31ccbbee81a67f6fd -->
<!-- translation-status: synchronized -->
<a id="implementation"></a>

# Kế hoạch triển khai governance CLI-only

1. Duy trì lõi governance Rust, kho task, binding Git worktree và các cổng
   kiểm chứng qua CNR và CLI tương thích.
2. Kiểm tra registry tiếng Anh có version. Phân giải chỉ dẫn, quy tắc và
   kỹ năng theo phạm vi task, rồi gắn digest nguồn với biên nhận và độ mới.
3. Xuất chỉ dẫn repository và kỹ năng cho phiên độc lập trong Codex Desktop,
   Claude Code và Antigravity. Kiểm thử sync theo phạm vi và rollback trong
   Git fixture đã bind với quyền theo từng tệp.
4. Bảo toàn hồ sơ governance hiện có bằng archive SQLite đã kiểm chứng và
   đề xuất được rà soát theo digest. Kiểm thử migration và khôi phục trong
   fixture dùng một lần.
5. Kiểm chứng build Rust, tài liệu song ngữ, kết quả CLI và biên nhận audit.
   Ghi kết quả kiểm tra trên client riêng với kết quả kiểm tra tĩnh gói xuất.

[Phạm vi](../../../specs/cirnet-rules/governance-cli-only/spec.md) ·
[Hồ sơ kiểm chứng](../../../specs/cirnet-rules/governance-cli-only/verification.md)
