# Thay đổi qua ranh giới sở hữu

Dùng khi thay đổi quyền sở hữu repository, API, mô hình dùng chung, SDK sinh
tự động hoặc đầu vào tích hợp. Bắt đầu từ [chọn kiến trúc](../policies/architecture.md)
và [định tuyến quyền sở hữu](../references/architecture-routing.md).

1. Chọn kiến trúc cho ranh giới bị ảnh hưởng. Xác nhận chủ sở hữu, bên cung cấp
   và các bên sử dụng trực tiếp từ mã nguồn và manifest hiện tại. Dùng chỉ dẫn
   của repository đích cho việc triển khai, lệnh và kiểm thử.
2. So sánh phụ thuộc được chọn chính xác với revision bên cung cấp đang đọc.
   Lần theo schema, cơ chế tuần tự hóa và fixture để phân biệt chi tiết triển
   khai với thay đổi hợp đồng chuẩn. Gửi các xung đột kiến trúc đáng kể cho
   các chủ sở hữu liên quan xem xét trước khi thay đổi ranh giới đó.
3. Nếu nhiệm vụ cần thay đổi ở bên cung cấp ngoài repo được phép, mô tả chính xác
   thay đổi phía chủ sở hữu và phần việc phía sử dụng đang bị chặn. Tiếp tục phần
   việc độc lập trong phạm vi; cần được cho phép rõ phạm vi trước khi sửa repository khác.
4. Nếu nhiều repo được cho phép, xác minh độc lập từng bản checkout và các chỉ dẫn
   cục bộ của nó. Giữ từng diff và kết quả xác minh gắn rõ với chủ sở hữu tương ứng.
5. Báo khả năng tương thích và tính sẵn có riêng với việc triển khai hoặc lựa chọn
   bản phát hành. Phiên bản mã nguồn mới hơn không cập nhật sản phẩm đầu ra mà bên
   sử dụng đã chọn.

Dùng [bản đồ repository có ghi thời điểm](../references/repository-map.md) để
xem quan sát trước đây. Giữ phiên bản sản phẩm, quy tắc an toàn và điều kiện
chấp thuận phát hành tại nơi sở hữu.

<!-- source: en/rules/cross-repo.md -->
<!-- source-sha256: a025231ae9b89a1f7f59a46e174e352de01b8bd3a62740b84d9a3ab3db1e5a18 -->
