<!-- doc-id: CNR-NATIVE-SMOKE-20261112-01-PLAN -->
<!-- revision: 1 -->
<!-- source-sha256: 15521dfc1ae3e1018b4497047276197cd1f4726b2bcc0cf785d7842c4424b78d -->
<!-- translation-status: synchronized -->
<a id="implementation"></a>

# Kế hoạch kiểm thử chỉnh tài liệu bằng Codex Desktop

Task `codex-native-smoke-20261112-01` đang chờ người vận hành phê duyệt và thực thi trong phiên native. [Đặc tả và các thay đổi chính xác](../../../specs/cirnet-interface/codex-native-smoke-20261112-01/spec.md)

- [ ] Rà soát binding hiện hành `codex-e2e-20261112-01` và các biên nhận cấu hình đã hoàn tất. Chỉ phê duyệt hai đường dẫn mục lục và lệnh kiểm chứng `interface-diff`. Bắt đầu task với baseline mới sau task cấu hình.
- [ ] Trong phiên Codex Desktop mở tại worktree đã binding, gọi `cirnet_repositories_discover` và `cirnet_policy_lookup`. Đọc `cirnet_task_status` và dùng revision vừa nhận để gọi `cirnet_task_context`. Đối chiếu context và binding của task với CNR.
- [ ] Đánh giá quyền ghi cho `docs/en/index.md` và `docs/vi/index.md`. Đánh giá một đường dẫn thứ ba mà không sửa tệp. Khi cả hai quyền ghi đạt, dùng editor trong phiên Desktop đó để thay một dòng trên mỗi tệp theo đặc tả.
- [ ] So sánh hai tệp đã sửa với diff đề xuất và chạy `cnr task diff-check`. Dùng digest của snapshot trong phạm vi hiện hành để gọi `cirnet_diff_check`. Ghi các lời gọi native và digest trước, sau mà không lưu thông tin xác thực.
- [ ] Ghi nhận đã thực hiện và chạy `cnr task verify` với bước `interface-diff` đã duyệt. Đọc audit của task và giữ patch để review độc lập.
