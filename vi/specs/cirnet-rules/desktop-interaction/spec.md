<!-- doc-id: CGACP-DESKTOP-INTERACTION-SPEC -->
<!-- revision: 1 -->
<!-- source-sha256: 66c69dc85e45642fdc17a8abec5d9ac968bd6fb618836482ba770f63457c392a -->
<!-- translation-status: synchronized -->
<a id="acceptance"></a>

# Kiểm chứng tương tác và Cài đặt Desktop

- [x] Tái hiện các triệu chứng đã báo trên Dashboard Windows cô lập.
- [x] Kiểm chứng điều hướng, Sidebar, control, Cài đặt và thông báo.
- [x] Ghi bằng chứng và giới hạn kiểm chứng từ UX-01 đến UX-40.

## Baseline và tái hiện

Bản Windows ở Git revision 37808be mở CGACP v1 và khởi tạo fixture SQLite riêng. Qt 6.11.2 dùng plugin Windows. Ảnh native baseline đo được DPR 1.25. Video được cung cấp giải mã ở 1918×1018 và 30 khung hình mỗi giây. Các quan sát dưới đây dùng khung hình đã xem và ảnh native tạm để đối chiếu độc lập.

| Phạm vi | Quan sát và tái hiện | Triển khai và kiểm chứng |
| --- | --- | --- |
| Route và trạng thái nút | Ở 00:05 cả Automations và hội thoại có nền chọn. Regression native tái hiện viền chuột còn lại, hàng được chọn sau khi chuyển và refresh đến muộn. | Route đang mở quyết định tô nền. Focus bàn phím và lúc nhấn có kiểu riêng. |
| Sidebar | Ảnh thu gọn được gửi có điều khiển phân tán. Baseline native ở cửa sổ cao có nhóm từ y 160–741 logical px. | Vùng giãn ở giữa giữ điều khiển ở đầu và footer ở đáy. |
| Cài đặt và mũi tên | Ở 00:20 Cài đặt dùng dropdown đổi section. Test native tái hiện thiếu danh mục và vùng bấm mũi tên quá nhỏ. | Cài đặt dùng danh mục trái và bản xem trước. Style Qt chung vẽ và kiểm tra glyph của control ghép. |
| Repository và automation | Các khung 00:42, 01:04 và 02:03 cho thấy điều khiển repository và form automation. | Fixture thật kiểm tra thao tác repository, rà soát và lịch sử chạy một lần. Form và stepper được chụp riêng. |
| Toast | Ở 01:22 và 02:28 toast nằm góc trên phải. Trên baseline native, thẻ mới nhất cách đáy 206 logical px. | Toast mới nhất neo dưới phải cách mép 18 logical px, tăng khoảng chừa nếu có vùng nhập cần tránh. |

## UX-01 đến UX-40

PASS ghi bằng chứng cho kiểm tra được nêu. NOT_RUN ghi phần chưa thao tác và không tính đạt. Ảnh dùng dữ liệu tạm. Xem [hướng dẫn trạng thái](../../../references/desktop-interaction.md) và [test Qt](../../../../desktop/tests/desktop_test.cpp).

| ID | Kết quả | Bằng chứng |
| --- | --- | --- |
| UX-01 | PASS | Escape hủy hộp thoại, Automations vẫn hoạt động và fixture không ghi lệnh tạo hội thoại. |
| UX-02 | PASS | Action dùng chung chỉ phát một click sau nhấn, nhả và rời chuột, không giữ viền accent. Đã thử Refresh, Copy và Reset repository. |
| UX-03 | PASS | Nền chính đạt kiểm tra pixel. Đã thử Add Automation, Save và lượt chạy một lần có xác nhận trên fixture. |
| UX-04 | PASS | Tab và Shift+Tab chuyển focus hai chiều và khôi phục chỉ báo accent của bàn phím. |
| UX-05 | PASS | Ảnh Chat và automation cho thấy vùng nhập tách rõ. Bản nháp và trường lịch ngày vẫn thao tác được. |
| UX-06 | PASS | Mở chat đã lưu rồi chuyển Automations xóa tô nền chat và giữ bản nháp. |
| UX-07 | PASS | Chọn hội thoại khác chỉ tô nền hàng tương ứng. Automations bỏ checked. |
| UX-08 | PASS | Phản hồi danh sách hội thoại được giữ lại rồi gửi muộn. Automations vẫn hoạt động và lịch sử không được tô nền. |
| UX-09 | PASS | Runtime giả gửi phản hồi chi tiết chat đảo thứ tự. Chat và route do người dùng chọn sau cùng được giữ. |
| UX-10 | PASS | Quay lại khôi phục luồng đang làm, bản nháp và trạng thái bảng. Escape đóng popup trước khi thoát Cài đặt. |
| UX-11 | PASS | Nhóm icon gọn giữ vị trí ở cả cửa sổ thấp và cao. Hai điều khiển footer nằm trọn trong vùng cha. |
| UX-12 | PASS | Đã rà icon panel và nút không khung trong ảnh native. Qt kiểm tra vùng bấm và thao tác thu gọn. |
| UX-13 | PASS | Kéo và bật tắt liên tiếp giữ độ rộng mở rộng. Runtime thật khôi phục tùy chọn sidebar sau khi mở lại. |
| UX-14 | PASS | Test rail gọn kích hoạt New Chat, Automations và Cài đặt bằng bàn phím. |
| UX-15 | PASS | Raster và vùng bấm mũi tên đạt ở Dark và Light. Ảnh native Cài đặt và repository cho thấy dropdown thật. |
| UX-16 | PASS | Chuột và bàn phím đổi spinbox đúng bước. Đã kiểm tra giới hạn, hậu tố và cuộn khi chưa focus. |
| UX-17 | PASS | Fixture control thử hai mũi tên và trạng thái vô hiệu hóa. Hộp thoại automation tăng phút đang chọn bằng bàn phím. |
| UX-18 | PASS | Menu con Relink trên native mở bằng bàn phím, căn theo menu cha và trở lại menu cha khi bấm Escape. Binding được chọn vẫn giữ nguyên. |
| UX-19 | PASS | Checkbox Dark và Light giữ pixel dấu tick. Thẻ Appearance có ngữ nghĩa radio và giữ trạng thái được chọn. |
| UX-20 | PASS | Ảnh cắt glyph native và kiểm tra hình học bao phủ các tỷ lệ hiệu dụng đã báo. DPR đo được nằm trong từng log. |
| UX-21 | PASS | Test native xác nhận danh mục bên trái nội dung và không còn combobox điều hướng section. |
| UX-22 | PASS | Tìm kiếm local đưa focus tới Giảm chuyển động. Liên kết repository từ Chat và Automation quay lại form đang mở. |
| UX-23 | PASS | Thẻ Dark và Light áp dụng ngay, System giữ sau khi mở lại. Đổi chế độ màu Qt trong tiến trình cập nhật cả cửa sổ chính và hộp thoại automation đang mở, đồng thời giữ nội dung đã nhập. |
| UX-24 | PASS | Lưu thất bại trên fixture hiện trạng thái chưa lưu và nút thử lại. Database thật mở lại với chủ đề và tùy chọn giảm chuyển động đã lưu. |
| UX-25 | PASS | Ảnh toast native và kiểm tra Qt xác nhận thẻ mới nhất neo góc dưới phải vùng ứng dụng. |
| UX-26 | PASS | Qt kiểm tra xếp lên trên, đóng, giới hạn hàng đợi, gộp trùng và không tràn khi resize. |
| UX-27 | PASS | Qt kiểm tra giữ focus, chừa vùng soạn tin và hộp xác nhận. Fixture Windows đo thời gian tạm dừng hết hạn khi Qt nhận trạng thái hover hoặc focus bàn phím, rồi đếm tiếp khi cả hai kết thúc. |
| UX-28 | PASS | Đã thử chế độ ít thông báo và sự kiện trùng. Fixture khởi động thành công, tải dữ liệu mà không phát toast thành công thường lệ. |
| UX-29 | PASS | Lỗi giữ nguyên nội dung trong lịch sử có giới hạn. Lượt chạy một lần thật vẫn đi qua xác nhận và luồng phê duyệt. |
| UX-30 | PASS | Qt bỏ fade đang chạy và tạo toast sau ở trạng thái tức thời khi bật Giảm chuyển động. Mở lại giữ được tùy chọn. |
| UX-31 | PASS | Bảng repository native giữ kéo cột, sắp xếp, lựa chọn và khôi phục header. Ảnh không còn viền sáng quanh bảng. |
| UX-32 | PASS | Repo Git tạm thật đạt xem trước, liên kết, mở lại và liên kết lại với đường dẫn Unicode. File URL của Qt dùng thư mục đã kiểm tra. |
| UX-33 | PASS | Runtime thật tạo định nghĩa fixture. Qt sửa, xác nhận lượt chạy và bật tắt tạm dừng, giữ ID lượt chạy và nguồn gốc. |
| UX-34 | PASS | Ảnh UI và danh sách request fixture không có workflow gallery. |
| UX-35 | PASS | Giới thiệu và trang runtime hiển thị CGACP v1 cùng trạng thái bàn giao và bị chặn từ runtime. |
| UX-36 | PASS | Đã kiểm tra chữ Qt tiếng Việt, danh mục và Appearance ở cỡ chữ ứng dụng lớn trên native. |
| UX-37 | PASS | Số request fixture ghi nhận khởi động và resize thuần UI. Đổi chủ đề, thu gọn và điều hướng không gọi release hoặc quét repo. |
| UX-38 | PASS | Diff chỉ bao phủ Desktop, tùy chọn trình bày, test trực tiếp và tài liệu song ngữ trong checkout được phép. |
| UX-39 | PASS | Smoke binary, harness native và test Rust dùng database fixture riêng cùng repository tạm. |
| UX-40 | PASS | Thư mục bằng chứng có ảnh native trước và sau, kết quả regression và giới hạn môi trường. |

## SOURCE_CHANGES

Thay đổi Qt gồm khung ứng dụng, điều hướng hội thoại, style chung, Cài đặt và NotificationCenter. Rust và schema IPC thêm trường giảm chuyển động tùy chọn vào record trình bày hiện có. Hàng cũ dùng mặc định và không cần migration database.

## GUI_VERIFICATION

Harness native Windows chạy widget Dashboard với fixture IPC xác định hoặc runtime Rust thật và workspace Git tạm. Log ghi DPR hiệu dụng 100%, 125%, 150% và 200% cùng hình học cửa sổ logical. Một test riêng chuyển cửa sổ giữa hai màn hình native 125% và 100%. CTest offscreen cũng kiểm tra runtime và đối chiếu clipboard. Binary phát triển độc lập mở được với SQLite riêng sau 1.454 ms trên máy này.

## RUNTIME_REGRESSION

Bộ Rust/Cargo 1.99.1 đã ghim đạt kiểm tra định dạng, Clippy nghiêm ngặt, build khóa dependency và 69 bài test. Validator kiểm tra 72 cặp EN/VI. Test tùy chọn tải hàng cũ, từ chối giá trị không phải boolean và mở lại giá trị đã lưu.

## UNVERIFIED_ENVIRONMENTS

Chưa kiểm chứng GUI Linux, đổi chủ đề qua Windows Settings và trình đọc màn hình. Test thời gian hết hạn cấp trực tiếp trạng thái hover của Qt vì sandbox phát chuyển động con trỏ thiếu ổn định. So sánh clipboard native được bỏ qua vì sandbox chặn COM. Kiểm tra clipboard offscreen và URL thư mục qua handler tách biệt với Explorer và clipboard native.

## RELEASE_ACTIONS

CGACP v1 giữ phiên bản ứng dụng 0.1.0. Thay đổi nằm trong working tree local. Chưa thực hiện thao tác phát hành.
