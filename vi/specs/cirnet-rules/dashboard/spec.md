<!-- doc-id: CGACP-DESKTOP-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: d46de70a6f89a672701104185d0b7971f808bb52268de00450579c993791ba66 -->
<!-- translation-status: synchronized -->
<a id="scope"></a>

# Phạm vi Dashboard CGACP

CGACP v1 có một runtime Rust và giao diện Qt Widgets cho người vận hành. Runtime quản lý định danh repository local, đường dẫn đã chọn và hoạt động qua SQLite. Chọn repo không cấp quyền ghi. Nội dung plan và spec vẫn nằm trong Git. Gói ứng dụng đích là 0.1.0.

Kiểm chứng Desktop, catalog, hội thoại và scheduler dùng workspace tạm. Phát hành, thực thi qua provider native và phê duyệt quyền nâng cao cần bằng chứng riêng.
