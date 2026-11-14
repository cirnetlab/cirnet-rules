# Thay đổi qua ranh giới sở hữu

Dùng khi nhiệm vụ thay đổi API, mô hình dùng chung, SDK sinh tự động hoặc đầu vào
tích hợp. Bắt đầu từ [bộ quy tắc nền tảng](../AGENTS.md);
[bản đồ repository](../references/repository-map.md) là công cụ điều hướng có ghi thời điểm.

1. Xác định bên cung cấp và các bên sử dụng trực tiếp từ mã nguồn và manifest hiện
   tại của họ. So sánh phụ thuộc mà bên sử dụng đã chọn với revision bên cung cấp đang đọc.
2. Phân biệt chi tiết triển khai với thay đổi hợp đồng chuẩn. Lần theo schema, cơ
   chế tuần tự hóa và fixture tương thích trước khi chọn nơi sửa.
3. Nếu nhiệm vụ cần thay đổi ở bên cung cấp ngoài repo được phép, mô tả chính xác
   thay đổi phía chủ sở hữu và phần việc phía sử dụng đang bị chặn. Tiếp tục phần
   việc độc lập trong phạm vi; cần được cho phép rõ phạm vi trước khi sửa repository khác.
4. Nếu nhiều repo được cho phép, xác minh độc lập từng bản checkout và các chỉ dẫn
   cục bộ của nó. Giữ từng diff và kết quả xác minh gắn rõ với chủ sở hữu tương ứng.
5. Báo khả năng tương thích và tính sẵn có riêng với việc triển khai hoặc lựa chọn
   bản phát hành. Phiên bản mã nguồn mới hơn không cập nhật sản phẩm đầu ra mà bên
   sử dụng đã chọn.

Với CIRNET, Contracts sở hữu schema chuẩn và SDK; Remote sở hữu việc lựa chọn bản
phát hành hệ thống. Xác nhận quyền sở hữu hiện tại trong các repository đó. Không
sao chép phiên bản, quy tắc an toàn sản phẩm hoặc điều kiện chấp thuận phát hành của
họ vào bộ quy tắc nền tảng chung.

<!-- source: en/rules/cross-repo.md -->
<!-- source-sha256: af9975c57d50be3308228d93631e0996d9374400803d8dfb4d1c37b26d1544cb -->
