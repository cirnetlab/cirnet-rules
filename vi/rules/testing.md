# Chọn và diễn giải các bước kiểm tra

Dùng hướng dẫn này để quyết định một thay đổi cần bằng chứng gì. Nhãn kết quả và
yêu cầu báo cáo được định nghĩa trong [bộ quy tắc nền tảng](../AGENTS.md) độc lập.

| Thay đổi | Bằng chứng hữu ích, tùy theo yêu cầu cục bộ |
| --- | --- |
| Văn bản hoặc liên kết | Đọc ngữ cảnh hiển thị/mã nguồn, kiểm tra đích liên kết, xem xét diff |
| Logic hoặc lỗi | Kiểm tra hồi quy hành vi tập trung cùng bộ kiểm thử cục bộ bị ảnh hưởng |
| API/tuần tự hóa | Hợp đồng của bên cung cấp, fixture tương thích và kiểm tra bên sử dụng trực tiếp |
| Phân quyền hoặc dọn dẹp | Hành vi từ chối, hủy, hết thời gian chờ và thất bại |
| Mã giao tiếp với thiết bị | Kiểm thử phần mềm cùng bằng chứng vật lý do chủ sở hữu quy định và được cho phép riêng |

Đọc script kiểm thử trước khi chạy nếu chưa rõ tác dụng phụ. Dùng cơ chế cấp phát/
thư mục đầu ra của repo; xác định lệnh có tải công cụ, thay đổi tệp khóa, khởi động
dịch vụ hoặc cần fixture hay không trước khi chạy.

Ghi lại lệnh, thư mục làm việc, mã thoát, kết quả và phạm vi bằng chứng. Nêu điều
kiện tiên quyết còn thiếu. Trường hợp bị bỏ qua không được xem là đã kiểm tra chỉ
vì mã thoát tổng thể thành công. Phân biệt lỗi có sẵn với hồi quy mới mà không che
giấu bên nào. Mở rộng kiểm thử cho phụ thuộc bị ảnh hưởng hoặc rủi ro chưa giải quyết;
không lặp lại các kiểm tra đã đạt nếu không có lý do. Rà soát mã nguồn, mock, fixture
tổng hợp và kiểm thử môi trường thực thi đã cài đặt xác lập những kết luận khác nhau.

<!-- source: en/rules/testing.md -->
<!-- source-sha256: 8660f9b2863d976093883502d4ee4eeddb5a1b9fa3746edbe60e7e0655a11d3c -->
