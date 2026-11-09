<!-- doc-id: OPENRULES-IDENTITY -->
<!-- revision: 1 -->
<!-- source-sha256: fa5024854114e113a6bd5d8504aca8535ab41e53a0cc1db6f0cfe496c8b2f9f8 -->
<!-- translation-status: synchronized -->
<a id="identity-cutover"></a>

# Định danh OPENRULES và nâng cấp dữ liệu cục bộ

Gói ứng dụng là OPENRULES 0.1.0. Kho mã và bản phát hành dự kiến giữ định
danh cirnet-rules và cirnet-rules-0.1.0. Desktop khởi chạy runtime đã kiểm
tra mã băm trong cùng thư mục cài đặt. Kênh stdio riêng dùng giao thức
phiên bản 2 và yêu cầu mọi phương thức mang tiền tố openrules. Runtime
kiểm tra phiên bản và namespace trước khi xử lý yêu cầu. Gói là openrules
và hai tệp thực thi của ứng dụng là openrules-desktop và openrules-runtime.
Tệp thực thi cirnet-governance hiện có tiếp tục phục vụ bảo trì.

## Nâng cấp cơ sở dữ liệu

Desktop kiểm tra vị trí dữ liệu cũ và hiện tại trước khi mở kho. Nếu chỉ có
cơ sở dữ liệu cũ, hãy đóng ứng dụng và runtime trước đây, kiểm tra hai đường
dẫn rồi chọn **Sao lưu và nâng cấp**. Runtime yêu cầu kết nối SQLite độc
quyền, kiểm tra tính toàn vẹn và lịch sử migration, tạo bản sao lưu đã kiểm
tra rồi chuẩn bị bản sao. Runtime ghi mã băm tệp đã xác minh trước khi kích
hoạt tên tệp mới. Các migration schema tiếp theo kiểm tra mã băm lịch sử và
bảo toàn ID, dấu thời gian, tùy chọn, bằng chứng và khóa chống lặp thao tác.

Nếu quá trình kích hoạt bị gián đoạn, hãy mở lại Desktop. Runtime có thể
tạo lại bản sao chưa hoàn chỉnh từ nguồn cũ. Nếu bản sao đã được đặt vào vị
trí đích, runtime kiểm tra biên nhận đã lưu và bản sao lưu trước khi mở. Khi
cả hai cơ sở dữ liệu độc lập cùng tồn tại, Desktop chờ lựa chọn rõ ràng.
**Dùng dữ liệu hiện tại** giữ hai tệp và chọn cơ sở dữ liệu hiện tại. Để kiểm
tra hoặc khôi phục dữ liệu cũ, hãy hủy khởi động và giữ cả hai bản. Cơ sở dữ
liệu cũ và các bản sao lưu trước đây giữ nguyên vị trí để khôi phục offline.

Sau chuyển đổi, runtime hiện tại chỉ ghi vào cơ sở dữ liệu OPENRULES. Giữ
ứng dụng cũ ở trạng thái đóng để tránh ghi vào kho đã lưu trữ. Các bản ghi
tác vụ dạng tệp trong checkout cũ vẫn tách riêng. Công cụ bảo trì chặn các
thao tác tác vụ cục bộ khi kho cũ còn tồn tại. Cổng lưu trữ giữ các bản ghi
dưới dạng bằng chứng mà không kích hoạt tác vụ hoặc chuyển quyền.

<a id="identity-namespaces"></a>

## Định danh cũ

Thương hiệu cũ chỉ còn trong mã phát hiện và lưu trữ dữ liệu, cơ chế chặn
truy cập thư mục trạng thái riêng trước đây, fixture migration và các tham
chiếu tác vụ hoặc bằng chứng lịch sử. Các tệp migration SQLite lịch sử và
nội dung do người dùng viết giữ nguyên.
