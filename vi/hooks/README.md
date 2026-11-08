<!-- doc-id: HOOKS -->
<!-- revision: 1 -->
<!-- source-sha256: 6625065e7a4fdc6a1ef9bc11032dd5f40085b6bd0bbdc4f8858aaeb2be540d80 -->
<!-- translation-status: synchronized -->

<a id="manual"></a>

# Hook kiểm tra

Dùng cùng CLI ở preflight và trước bàn giao. Workflow CI gọi validation và
kiểm tra sẵn sàng release độc lập. Thư mục này ghi quy trình di động. Adapter
ngoài checkout phải kiểm tra phạm vi repo trước khi gọi governance và giữ
nguyên dự án không liên quan. Dùng dry-run trước thay đổi cài đặt.
[Giới hạn adapter](../adapters/README.md)
