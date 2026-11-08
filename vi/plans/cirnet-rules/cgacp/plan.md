<!-- doc-id: CGACP-PLAN -->
<!-- revision: 2 -->
<!-- source-sha256: dc69c345aeb26b710d412ce77f0702e9417b132c66ce478f88f27e2fdcfe5108 -->
<!-- translation-status: synchronized -->
<a id="work"></a>

# Kế hoạch triển khai CGACP

[Đặc tả](../../../specs/cirnet-rules/cgacp/spec.md)

1. Chuyển toàn bộ Rust vào src và xác lập bộ kiểm tra hồi quy.
2. Thêm liên kết worktree local và định tuyến hẹp. Kiểm tra đầu vào dirty,
   delta cache và ngân sách phạm vi.
3. Thêm chuyển trạng thái, ghi lạc quan và hành động gắn bằng chứng.
   Kiểm tra lệnh có kiểm soát và phạm vi diff bằng fixture dùng một lần.
4. Thêm kiểm chứng, review, nghiệm thu và đồng bộ tài liệu có thể tiếp tục.
   Tách kiểm tra release tường minh khỏi WORK và giữ bằng chứng lịch sử.
5. Kiểm chứng CLI, cập nhật hướng dẫn song ngữ và báo giới hạn nền tảng.
