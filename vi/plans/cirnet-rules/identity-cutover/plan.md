<!-- doc-id: CIRNETRULES-REBRAND-PLAN -->
<!-- revision: 1 -->
<!-- source-sha256: 68ef0d46a52bebf4d624a841137047575068331e147a0377458ed5a10ed8fd9a -->
<!-- translation-status: synchronized -->
<a id="steps"></a>

# Kế hoạch chuyển định danh CIRNET RULES

[Tiêu chí nghiệm thu](../../../specs/cirnet-rules/identity-cutover/spec.md)

1. Kiểm kê source, build, giao thức, dữ liệu và tham chiếu hiện hành. Ghi baseline và fixture bảo toàn.
2. Dùng metadata đã kiểm tra cho Rust và Qt. Đổi tên binary, tài nguyên và tham chiếu. Kiểm tra định danh và giao thức mới ở IPC riêng và khi khởi động.
3. Tách phát hiện kho cũ khỏi journal chuyển đổi mới. Kiểm chứng sao lưu online, dữ liệu staging, xung đột và tiếp tục trên kho tạm. Giữ riêng dữ liệu lịch sử và quyền đã cấp.
4. Kiểm tra luồng soạn thảo, lưu trữ và repository hiện hành. Kiểm tra staging Windows, UI native, hash tài liệu, ngoại lệ tên cũ và ma trận nghiệm thu CR.
