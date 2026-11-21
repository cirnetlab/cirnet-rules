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
Dùng [bằng chứng kiểm chứng](../docs/distribution.md#bằng-chứng-kiểm-chứng)
khi xem trước phép kiểm của chủ sở hữu, đánh giá bước bắt buộc hoặc giới hạn lần thử lại.

Ghi lại lệnh, thư mục làm việc, mã thoát, kết quả và phạm vi bằng chứng. Nêu điều
kiện tiên quyết còn thiếu. Trường hợp bị bỏ qua không được xem là đã kiểm tra chỉ
vì mã thoát tổng thể thành công. Phân biệt lỗi có sẵn với hồi quy mới mà không che
giấu bên nào. Mở rộng kiểm thử cho phụ thuộc bị ảnh hưởng hoặc rủi ro chưa giải quyết;
không lặp lại các kiểm tra đã đạt nếu không có lý do. Rà soát mã nguồn, mock, fixture
tổng hợp và kiểm thử môi trường thực thi đã cài đặt xác lập những kết luận khác nhau.

Với kiểm tra compile/simulation hoặc giao tiếp thiết bị, giữ mã thoát từng tiến
trình trước khi lệnh shell khác ghi đè. Đọc wrapper chỉ cung cấp kết quả tổng hợp;
ghi kết quả từng bước thực tế mà không đổi đầu vào, môi trường, cách xử lý tiến
trình hoặc safety gate. Chỉ file output hay thông báo thành công không chứng minh
PASS. Giữ trạng thái NOT_RUN cho bước phụ thuộc chưa thực thi. Báo riêng trạng
thái compile và simulation cùng mã thoát quan sát được, nêu các assertion thực sự
được thử và ghi `physical verification: NOT_RUN` khi chưa kiểm tra vật lý. Thiếu
capture vẫn là khoảng trống bằng chứng; không bịa mã hoặc đánh đồng mô phỏng phần
mềm với kiểm chứng vật lý. [Skill kiểm chứng](../skills/cirnet-verification/SKILL.md)
định nghĩa các trường báo cáo FPGA/RTL và giữ sự phân biệt này khi được cài riêng.

<!-- source: en/rules/testing.md -->
<!-- source-sha256: 438569e7b7d294c7e48019a975f7b3d3c281939ee36303cd253cfac0f1acedb0 -->
