<!-- doc-id: CNR-MCP-COORDINATION-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: 497446b562e2f81b7288c31c4329f5444d8d46daacbe834c84121491c7cce1ee -->
<!-- translation-status: synchronized -->
<a id="scope"></a>

# Phối hợp task dùng chung và MCP

Một service Rust đọc governance Git và kho task local hiện có. CLI và MCP dùng chung thao tác cập nhật task. Task phối hợp cố định worktree đã chọn và dành nó cho một writer. Generation của lease chặn worker cũ. Hồ sơ task đã ghi liên kết biên nhận và sự kiện audit.

Báo cáo giữ khai báo có giới hạn của worker cùng digest source do core quan sát độc lập. Kiểm tra dependency và CI gắn bằng chứng với đúng producer đã được chấp nhận và candidate. Quyền operator và chấp nhận độc lập vẫn theo lifecycle hiện hành.

MCP khởi chạy với authority và workspace cố định. Profile chỉ đọc cung cấp tool governance có giới hạn. Truy cập phối hợp cần một phiên local được ủy quyền riêng. Cấu hình client được xuất để review. Ghi vào checkout sản phẩm, cài đặt native và kích hoạt release là các thao tác riêng.
