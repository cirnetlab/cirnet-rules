<!-- doc-id: CNR-MCP-COORDINATION-PLAN -->
<!-- revision: 1 -->
<!-- source-sha256: d43ac4e798a6945795105cfe23c61d337772684fcf1e7d4fa41e1ab0e395610e -->
<!-- translation-status: synchronized -->
<a id="implementation"></a>

# Kế hoạch triển khai phối hợp chung

1. Kiểm tra baseline của linked worktree, vị trí dữ liệu đời trước và digest SQL lịch sử. Giữ CLI Rust, hồ sơ task và authority Git hiện hành.
2. Mở rộng chọn binding và giao dịch task. Kiểm thử worktree có tên, quyền ghi dùng chung, fencing, báo cáo và freshness dependency bằng fixture local.
3. Bổ sung dịch vụ STDIO với context cố định và cấu hình client để review. Kiểm thử giao thức thật qua fixture tiến trình.
4. Bổ sung bằng chứng candidate có thể di chuyển và gate dựa vào đầu vào tin cậy riêng. Chạy kiểm tra hồi quy và ghi giới hạn ở cả hai ngôn ngữ.
