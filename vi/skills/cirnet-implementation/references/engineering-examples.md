# Ví dụ kỹ thuật

Đây là các fixture giả lập với tên định danh và contract được đặt riêng cho
ví dụ. Các phép kiểm tra mô tả việc cần chạy và kết quả mong đợi trên fixture.
Tuyên bố về sản phẩm cần bằng chứng từ hệ CIRNET liên quan. Đọc ví dụ phù hợp
với nhiệm vụ.

## Tương thích API và SDK

**Yêu cầu** — Hiển thị nhãn nhóm ngắn trong danh sách của consumer. SDK đang
được chọn trả về mã nhóm đã được mô tả trong tài liệu. Phạm vi sửa chỉ gồm repo
consumer.

**Trước** — Thêm `groupLabel?: string` vào DTO sinh tự động và dùng nhãn rỗng
khi thiếu. Producer vẫn chỉ trả về mã nhóm. Sửa thủ công sẽ mất khi sinh lại,
và trường tùy chọn mới trong dữ liệu truyền có thể làm hỏng consumer từ chối
thuộc tính không xác định.

**Sau** — Kiểm tra phiên bản consumer đang ghim và đối chiếu với schema của bên
sở hữu producer. Nếu chấp nhận nhãn cố định, hiển thị mã đã biết bằng bộ định
dạng nhãn sẵn có của consumer. Giữ SDK, hành vi truyền dữ liệu và cách xử lý
mã chưa biết. Nếu nhãn phải đến từ máy chủ, xác định thay đổi cần thiết ở
producer cho trường mới, gồm kiểm tra tương thích với bộ giải mã nghiêm ngặt
và một bản phát hành tương thích để consumer chọn.

**Kiểm tra** — Giải mã fixture cho SDK đã ghim và hiển thị mã đã biết và chưa
biết. Xem diff để xác nhận chỉ sửa phần hiển thị nhãn và test tương ứng.

**Giới hạn** — Phép kiểm tra bao phủ hành vi consumer với SDK đã chọn. Việc
phát hành producer và tương thích với các consumer khác cần bằng chứng riêng.

## Sửa hiển thị Vue và TypeScript

**Yêu cầu** — Panel đồ thị hiển thị dấu gạch ngang dài khi số đỉnh lân cận hiện
có bằng không. Hiển thị `0` và giữ dấu gạch khi thiếu dữ liệu.

**Trước** — Viết lại kho trạng thái đồ thị và đổi tên prop chứa số đếm để xử
lý lỗi hiển thị. Biểu thức hiển thị `count || '—'` coi số không là dữ liệu thiếu.

**Sau** — Đổi biểu thức nội suy trong template thành `count ?? '—'`. Giữ kiểu
prop, sự kiện phát ra, trạng thái đồ thị và style.

**Kiểm tra** — Test tập trung của component hiển thị `0`, một số dương và
`undefined`. Assertion cho số không thất bại trước khi sửa. Chạy test sự kiện
chọn có sẵn và xem diff có giới hạn ở một biểu thức template và test tập trung.

**Giới hạn** — Phép kiểm tra bao phủ component đã mount và các sự kiện đã thử.
Hành vi end-to-end trên trình duyệt và bố cục cần phép kiểm tra riêng.

## Lấy mẫu trong testbench FPGA

**Yêu cầu** — Scoreboard của RTL giả lập báo đầu ra sai lệch ngay sau cạnh
lên. Đầu ra của thanh ghi cập nhật ở mỗi cạnh lên và ổn định giữa các lần
cập nhật đó.

**Trước** — Tắt watchdog hoặc giữ tín hiệu cho phép bảo vệ ở mức cao để test
đạt. Cả hai đều phá hỏng một tình huống an toàn và vẫn để scoreboard đọc trước
khi các phép gán không chặn cập nhật xong.

**Sau** — Xác nhận sai lệch trên giản đồ sóng. Trong fixture đồng bộ này, cho
scoreboard lấy mẫu ở cạnh xuống kế tiếp và so với kết quả mong đợi của cạnh
lên trước đó. Giữ timing của DUT, watchdog, kiểm tra tín hiệu cho phép, logic
trạng thái an toàn và các assertion an toàn đã có.

**Kiểm tra** — So giản đồ sóng của lần cập nhật bình thường với chu kỳ mong
đợi. Chạy các ca reset, mất tín hiệu cho phép và watchdog hết hạn cùng các
assertion ban đầu, rồi xác nhận đầu ra về trạng thái an toàn đã quy định.

**Giới hạn** — Mô phỏng RTL bao phủ mô hình và timing đã thử. Vận hành bo mạch,
tình trạng sẵn sàng về điện và nạp chương trình cần quyền từ bên sở hữu và
bằng chứng riêng.

## Hồi quy tại biên của khoảng

**Yêu cầu** — Báo cáo cho khoảng nửa kín `[start, end)` phải lấy mốc đầu và bỏ
mốc cuối.

**Trước** — Bộ lọc dùng `at > start && at <= end`. Assertion về số bản ghi
vẫn đạt vì cả hai cách triển khai đều trả về hai bản ghi từ đầu vào này.

**Sau** — Trước khi sửa, kiểm tra chính xác các ID từ hàm lọc công khai. Sau
đó đổi điều kiện lọc thành `at >= start && at < end`.

```typescript
const rows = [
  { id: 'at-start', at: 10 },
  { id: 'inside', at: 12 },
  { id: 'at-end', at: 15 },
]

expect(reportRows(rows, 10, 15).map(row => row.id)).toEqual(['at-start', 'inside'])
```

**Kiểm tra** — Điều kiện cũ trả về `['inside', 'at-end']` nên assertion thất
bại. Điều kiện đã sửa trả về `['at-start', 'inside']` nên cùng assertion đó
đạt. Chạy bộ test liên quan đến hàm lọc để kiểm tra các ca lân cận.

**Giới hạn** — Fixture này dùng mốc thời gian nguyên. Phân tích dấu thời gian
và chuyển đổi múi giờ cần coverage riêng.

<!-- source: en/skills/cirnet-implementation/references/engineering-examples.md -->
<!-- source-sha256: d2c51656f1244cb7a4446cc648d9101af28247fdcc1810045e20970970e88cde -->
