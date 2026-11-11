<!-- doc-id: CIRNETRULES-DESKTOP-ASYNC-VERIFICATION -->
<!-- revision: 1 -->
<!-- source-sha256: aedb39e07cf124523eff55a7742b721c14d4b17861ab546673ba9fefed59c7c4 -->
<!-- translation-status: synchronized -->
<a id="evidence"></a>

# Kiểm chứng Desktop bất đồng bộ

Source baseline là `ddc6892`. Dùng cùng toolchain Release và màn hình cho mỗi cặp đo. Máy Windows 11 dùng MSVC 2022, Qt 6.11.2, Rust và Cargo 1.99.1, i5-12500H ở chế độ Balanced, cửa sổ logic 1200 × 700 và DPI 125% ở 144 Hz. Fixture Qt có 24 hội thoại, trả dòng dài, hơn 400 dòng Unicode và marker paint riêng. Đo warm bỏ mười lượt chọn đầu. Lượt chọn đầu chạy ở tiến trình mới. Fixture runtime riêng giữ lệnh Git đầu một giây rồi chạy Git thật trên checkout dùng một lần.

| Chỉ số | Số mẫu | Trước | Sau |
| --- | ---: | ---: | ---: |
| Lượt chọn đầu với dòng 64 KiB, ms | 1 mỗi bản | 146.37 | 24.84 |
| Paint warm với dòng 64 KiB, p50/p95/p99/max ms | 50 mỗi bản | 148.78 / 187.67 / 206.86 / 206.86 | 39.28 / 61.66 / 69.74 / 69.74 |
| Paint warm với dòng 8 KiB, p50/p95/p99/max ms | 50 mỗi bản | 33.86 / 43.96 / 54.48 / 54.48 | 41.01 / 57.63 / 74.83 / 74.83 |
| Heartbeat tuần tự 64 KiB, p50/p95/p99/max ms | 317 / 203 | 0.78 / 122.95 / 140.78 / 163.69 | 0 / 14.36 / 18.22 / 29.43 |
| Heartbeat hỗn hợp 64 KiB, p95/p99/max ms | 459 / 396 | 6.09 / 6.45 / 8.77 | 17.03 / 24.57 / 34.94 |
| Request chi tiết gửi trong 100 lượt chọn | 100 mỗi bản | 31 | 3 |
| Round trip đọc khi Git chờ, p50/p95 ms | 10 mỗi bản | 1111.85 / 1122.13 | 0.69 / 0.81 |
| Chờ hàng đợi runtime / handler, p50 ms | 10 mỗi bản | 1111.09 / 0.50 | 0.08 / 0.41 |
| Truy vấn hiển thị SQLite có 100.001 blocker, p50/p95/p99/max ms | 36 mỗi truy vấn | 121.87 / 312.23 / 386.26 / 386.26 | 26.53 / 29.59 / 30.29 / 30.29 |

Fixture Qt xử lý tuần tự có lệnh chậm và lượt chọn cuối chưa paint khi hết thời gian chờ ở baseline. Sau refactor lượt cuối paint sau 1,76 giây, trong khi lệnh chậm chủ động vẫn mất 2,3 giây. Fixture này đo phản hồi GUI và gộp request. Bài runtime thật đo lane Git riêng. Ca warm 8 KiB tăng 13,67 ms ở p95. Heartbeat hỗn hợp 64 KiB cao hơn mục tiêu p95 16 ms khoảng 1,03 ms. Các ngưỡng gốc được giữ nguyên.

Chuỗi đo xen kẽ bản sau / baseline / bản sau với dòng 64 KiB có 50 mẫu warm mỗi lượt. P95 của paint đầu lần lượt là 54,46 / 514,55 / 55,80 ms. Cả hai lượt bản sau gửi ba request chi tiết và paint được lựa chọn cuối. P95 heartbeat hỗn hợp lần lượt là 17,46 và 18,11 ms. Một lượt riêng trên bản sau cuối cùng có p95 paint đầu 473,78 ms và cực đại 914,68 ms. Lượt đó gửi 69 request chi tiết và p95 heartbeat hỗn hợp đạt 64,21 ms. Thời gian CPU GUI trong phần warm gần với các lượt bản sau khác. Nguyên nhân của đuôi trễ này chưa được xác định. Lượt lặp cuối của ca 8 KiB đạt p95 50,17 ms, vẫn cao hơn baseline 43,96 ms.

Lượt có trace cho dòng 64 KiB đo 60 phản hồi chi tiết đầu của mỗi bản. Parse p95 là 1,24 ms trước và 1,42 ms sau. Callback p95 từ 469,37 xuống 12,91 ms và còn cao hơn mục tiêu apply 4 ms khi tính cả callback. Callback gồm cập nhật model và tài liệu đồng bộ. Probe đo riêng paint thật của viewport transcript. Fixture SQLite cô lập dùng SQLite 3.50.4, 10.000 hội thoại, 50.005 tin nhắn và đề xuất, 50.001 lượt chạy. Chat có sáu blocker vẫn gần 0,03 ms p95 khi nhiều lượt chạy thuộc chat khác. Query plan baseline dùng các nhánh có index và quét CTE thuộc hội thoại đã chọn. Ca nặng phải đếm chính xác blocker của chính chat đó. Writer đã commit giữa hai lượt đọc metadata và watermark trong test snapshot. Poll tiếp theo trả sự kiện mới.

Ở soak 1.000 lượt, private bytes GUI là 48332800 sau warm-up và 48275456 ở cả mốc 500 và 1.000. Số QObject, QWidget và timer con giữ ở 1096, 632 và 8. Mẫu tiến trình giữ 12 thread và một child. Handles từ 1044 lên 1046 rồi ổn định. Soak kết nối lại của GUI hoàn thành 100 chu kỳ với 200 callback dự kiến và còn 0 child sau khi thu hồi. Soak Rust riêng giữ 84 handles và 13 threads trong 1.000 cặp đọc và hủy, đồng thời chạy 100 chu kỳ kết nối lại và hủy. Private bytes từ 2887680 lên 2920448 và ổn định ở mẫu 500. Đây là số đo tài nguyên tiến trình. Chưa đo heap live, hiệu năng trên máy sạch và benchmark GUI native đầu cuối trên database thật lớn.

## Cổng kiểm chứng

- Định dạng Rust, Clippy với cảnh báo bị từ chối và build Release đầy đủ đạt.
- Bộ Rust đạt 152 test. Fixture `uv` offline được bật riêng và đã đạt.
- Preset Qt CTest đạt. Bộ native báo 53 đạt, không lỗi và hai ca bật riêng.
  Workload hiệu năng và kết nối lại có log native riêng. Ca toast kiểm tra
  focus trong cửa sổ chủ qua thời điểm hết hạn.
- Governance kiểm tra đạt 106 cặp tài liệu song ngữ. Checksum runtime đã stage
  khớp với binary Release và pin. Lệnh trợ giúp và phiên bản của `cnr` chạy được.
- Chủ động dừng đọc stdout và stderr tái hiện runtime kẹt lúc đóng trước khi
  sửa chẩn đoán. Runtime Release cuối thoát sau 2,04 giây trong ca tái hiện
  trực tiếp. Kiểm tra biên nhận, hủy và quá tải đạt.

## Tái hiện

Build trong môi trường compiler local với Qt đã cài và Rust đã pin.

```powershell
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
$qtKit = Read-Host 'Qt MSVC kit directory'
$runtime = (Resolve-Path .\target\release\cirnetrules-runtime.exe).Path
cmake --preset desktop "-DCMAKE_PREFIX_PATH=$qtKit" "-DCIRNETRULES_RUNTIME_EXECUTABLE=$runtime"
cmake --build --preset desktop
ctest --preset desktop
.\target\release\cirnet-governance.exe validate
.\target\release\cnr.exe --help
.\target\release\cnr.exe --version
```

Thêm thư mục `bin` của kit vào path của tiến trình hiện tại trước khi mở
`build/desktop/desktop/cirnetrules-desktop.exe`. Dùng `--data-dir` tuyệt đối cho
một phiên Dashboard dùng một lần.

Chạy `asyncConversationPerformance` từ Qt Test đã build, đặt
`QT_QPA_PLATFORM=windows`, `CIRNETRULES_TEST_PRIMARY_SCREEN=1` và
`CIRNETRULES_ASYNC_BENCH=1`. Đặt `CIRNETRULES_ASYNC_BENCH_TYPICAL=1` cho biến thể
8 KiB và `CIRNETRULES_ASYNC_BENCH_SOAK=1` cho 1.000 lượt chọn.
Chạy `asyncRuntimeReconnectSoak` với `CIRNETRULES_ASYNC_BENCH_RECONNECT=1` để đo vòng đời tiến trình.
Log Qt Test dạng văn bản ghi số đo. Chạy `python src/tests/runtime_performance.py`
với đường dẫn runtime Release, `--iterations 10` và `--trace` cùng tệp đầu ra.
Lặp lại lệnh với binary baseline đã lưu. Bản candidate hỗ trợ `--exercise` để
kiểm tra client đọc chậm, hủy và biên nhận, cùng `--soak` để đo tài nguyên.
Dùng thư mục dữ liệu dùng một lần cho mọi test tích hợp.

Đặt `CIRNETRULES_TRACE_IPC=1` cho tiến trình để ghi thời gian theo request ID
vào đầu ra chẩn đoán Qt hoặc stderr của runtime. Mỗi client có ngân sách trace
256 KiB. Runtime nhận chẩn đoán vào 64 vị trí, mỗi vị trí tối đa 512 byte, và
giới hạn trace request ở 2.048 bản ghi. Runtime có thể đóng stdout bị nghẽn
khi sink chẩn đoán đang chờ stderr. Đầu ra gồm thời gian, tên method đã biết,
độ sâu hàng đợi và kích thước frame. Trace loại payload, đường dẫn và giá trị
môi trường. Bỏ biến để tắt trace. So sánh thời lượng trong từng tiến trình và
đối chiếu ID giữa các tiến trình.

## Quyền sở hữu và rollback

`runtime_client.cpp` quản lý frame IPC, expiry, slot ý định và thu hồi tiến trình.
`conversations.cpp` quản lý thế hệ lượt mở, giới hạn transcript và cập nhật widget.
`src/bin/runtime/dispatcher.rs` quản lý tiếp nhận theo lane, hủy và chuyển phản hồi.
`src/dashboard/storage.rs` quản lý tạo connection và gỡ progress hook.
`src/dashboard/sessions.rs` và `src/dashboard/sessions/sqlite.rs` quản lý snapshot và
truy vấn phân trang có giới hạn. Tiến trình Git dùng runner có containment hiện có
và chia sẻ deadline qua các lệnh.

Giới hạn frame là 256 KiB. UI giữ tối đa 32 callback hoặc suất gửi, hai ý định
đang chờ và cửa sổ transcript tối đa 48 tin nhắn, 3.500 block, 768 KiB chuỗi UTF-16.
Preview giới hạn mỗi dòng ở 1.024 đơn vị UTF-16. Trình xem đầy đủ cho phép chọn và
sao chép nội dung đã lưu. Bản nháp có giới hạn riêng. Mỗi lane runtime có một yêu cầu đang chạy và tám
yêu cầu đang chờ. Đường phản hồi giữ 32 vị trí thông thường, bốn vị trí báo từ
chối và tám vị trí điều khiển. Writer giữ biên nhận khi mất phản hồi, pipe nghẽn
hết hạn sau hai giây. Tác vụ file thiếu hook cộng tác có thể chạy đến lúc
containment tiến trình can thiệp hoặc hết deadline riêng.

Để rollback source, đóng Desktop, khôi phục revision source trước đó rồi build
và stage runtime tương ứng. Giữ thư mục dữ liệu hiện có và bản sao lưu đã rà
soát. Thay đổi này không cần migration database. Runtime thiếu
`read_cancellation` vẫn nhận envelope request v3 hiện có.
