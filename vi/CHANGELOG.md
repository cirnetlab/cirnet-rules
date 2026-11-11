<!-- doc-id: CIRNETRULES-CHANGELOG -->
<!-- revision: 6 -->
<!-- source-sha256: 28d8dfe028952da605f3e05f2031b145c61e8191e92fe51b774a4d1775837e1b -->
<!-- translation-status: synchronized -->
<a id="unreleased"></a>

# Nhật ký thay đổi

## Chưa phát hành

- CNR cung cấp một CLI governance và giữ quy trình môi trường dự án cùng thực thi native.
- Registry tiếng Anh có version phân giải ngữ cảnh task và xuất chỉ dẫn repository cùng skill cho Codex, Claude Code và Antigravity.
- Quy trình SQLite đời trước tạo archive local đã kiểm tra và đưa đề xuất vào staging để review.
- Qt, runtime riêng, chat, lịch chạy và probe engine đã ngừng. Lịch sử release trước đó nằm trên [nhánh lưu trữ](https://github.com/cirnetlab/cirnet-rules/tree/52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c).

- Bổ sung tool MCP STDIO và quy trình phối hợp task dùng chung với worktree
  có tên, lease có fencing, báo cáo có cấu trúc và biên nhận dependency.
- Bổ sung gate CI có kiểm soát cho bằng chứng candidate có thể di chuyển đã review.
