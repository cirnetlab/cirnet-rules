<!-- doc-id: TASK-LOCK-PLAN -->
<!-- revision: 1 -->
<!-- source-sha256: a0af7d190e5e75be49b096ab98e2378f8d590dc981ac5d636e89f32aa55fa97e -->
<!-- translation-status: synchronized -->
<a id="work"></a>

# Kế hoạch kiểm thử hồi quy khóa tác vụ

1. Tái hiện khóa còn giữ bằng bản sao handle và xác nhận từ chối tranh chấp.
2. Giải phóng khóa tác vụ tường minh khi guard kết thúc phạm vi.
3. Kiểm tra các lượt ghi đồng thời, workflow và hướng dẫn song ngữ.
