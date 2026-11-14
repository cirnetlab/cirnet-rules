<!-- doc-id: CNR-CODEX-NATIVE-SETUP-PLAN -->
<!-- revision: 1 -->
<!-- source-sha256: a3f95a5c5b449ef3d59c9c2b93ffa392b8e98b7d96d4caa2022f339d9921dd59 -->
<!-- translation-status: synchronized -->
<a id="implementation"></a>

# Kế hoạch cấu hình Codex cấp dự án

1. Xác định `codex-e2e-20261112-01` và xác nhận commit gốc của worktree Interface.
2. Nhận phê duyệt phạm vi setup chính xác. Bắt đầu task riêng và đánh giá từng đường dẫn theo revision hiện hành.
3. Review preview khối quản lý và áp dụng bằng digest hiện hành. Đối chiếu tệp sau thao tác và bảo toàn chỉ dẫn sẵn có.
4. Xác nhận trust của dự án Desktop qua người vận hành. Review và chỉ áp dụng mục MCP chỉ đọc `cirnetrules` cấp dự án.
5. Xác minh receipt đăng ký và sync, chạy `interface-diff` và giữ kết quả cho review độc lập.

[Phạm vi setup chính xác](../../../specs/cirnet-interface/codex-native-setup-20261112-01/spec.md)
