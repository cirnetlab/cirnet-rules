# Pilot A/B Codex — Current Work Rules, 19/11/2026

Pilot native đã hoàn tất và được rà độc lập **8/8 trial: 4/4 PASS cho A và
4/4 PASS cho Candidate B2**. Đây là kết quả cho các ca pilot, không phải
nghiệm thu Adaptive Stable hay kết luận về hiệu quả trên toàn bộ benchmark.

## Phạm vi và nguồn

- Phiên: `pilot-native-dfae559bb932480fb08e8519bf0be630`, bắt đầu lúc
  **03:15:10 ngày 19/11/2026**, giờ Việt Nam.
- Bốn ca E01, E02, E05 và E06; mỗi ca chạy một trial cho mỗi nhóm. Mỗi trial
  có hai vai trò Author và Work: tổng cộng 8 trial và 16 lượt vai trò.
- A dùng baseline Author tại revision `e2709b35f38ef1a21eda8760589b2f92f5cbc406`.
  B2 dùng snapshot Candidate từ source HEAD
  `1266f8c635b5d4f5d13413921b8ce8efe0736119`, SHA-256
  `0d0ec3319bd5c29ffd1b2d69711b1747cb64760662bebca04966617ce4e4ec3c`.
  Profile `Current` dùng Work Rules hiện tại của snapshot cho cả hai nhóm.
- Cả hai vai trò được yêu cầu dùng `gpt-6.2-sol`, effort `xhigh`, timeout
  300 giây/vai trò. Model và effort thực tế phía server chưa được xác nhận.
- Bằng chứng đã rà được lưu cục bộ trong thư mục tạm
  `%LOCALAPPDATA%/Temp/cirnet-adaptive-stable-loginfix/pilot-native-dfae559bb932480fb08e8519bf0be630`.
  Thư mục tạm có thể không còn trên máy khác hoặc sau khi dọn dữ liệu.

| Tệp evidence | SHA-256 |
| --- | --- |
| `manifest.json` | `8bba663b5bdf7989f2d0d4ac0c7d683c1412691a7e3cb0799f28a7a69a303300` |
| `ab-review.json` | `93288026ccc9d3ccad935455a05c82d15c9db481ff7157c95fe5aae09c0c1165` |
| `reviewed-409e0023030e404d8a76e792120185ea.json` | `88cc3e21e71f493637234b39b5f97f6fa369f58b6172e1f6a79be1fa5352593c` |

## Kết quả theo ca

| Ca | A | B2 |
| --- | --- | --- |
| E01 | PASS | PASS |
| E02 | PASS | PASS |
| E05 | PASS | PASS |
| E06 | PASS | PASS |
| Tổng trial | **4 PASS / 4** | **4 PASS / 4** |
| Mục kiểm tra kiến trúc | 6 PASS / 6 | 6 PASS / 6 |
| Mục kiểm chứng bắt buộc | 6 PASS / 6 | 6 PASS / 6 |

Người rà dùng artifact và trace độc lập với các vai trò Author và Work.
E01 không phát hiện giá trị synthetic được bảo vệ trong các artifact đã rà;
E05 có bằng chứng kiểm thử thất bại trước sửa và đạt sau sửa; E06 giữ phạm
vi file và dữ liệu người dùng theo review.

## Số liệu quan sát trong pilot

| Chỉ số | A | B2 |
| --- | ---: | ---: |
| Prompt trung vị, từ | 634 | 451 |
| Prompt trung vị, byte UTF-8 | 4.713 | 3.403,5 |
| Input token: Author + Work | 607.340 + 673.909 = **1.281.249** | 583.533 + 781.179 = **1.364.712** |
| Trong đó input đã cache | 1.089.280 | 1.164.032 |
| Input chưa cache | 191.969 | 200.680 |
| Output token: Author + Work | 13.845 + 13.191 = **27.036** | 15.416 + 26.918 = **42.334** |
| Lượt gọi công cụ: Author + Work | 70 + 61 = **131** | 52 + 57 = **109** |
| Thời gian Author trung vị | 90,002 giây | 94,0795 giây |
| Thời gian Work trung vị | 65,0005 giây | 200,871 giây |
| Trung vị tổng thời gian hai vai trò mỗi trial | 155,0025 giây | 294,141 giây |
| Chi phí thực tế | NOT_AVAILABLE | NOT_AVAILABLE |

Input đã cache là một phần của tổng input, không cộng thêm lần nữa. Thời gian
đo bởi máy chạy từng vai trò; không bao gồm toàn bộ pipeline hay thời gian rà
thủ công. Không có bằng chứng để kết luận tỷ lệ rework hoặc chi phí thực tế.

## Giới hạn và trạng thái

Theo báo cáo được rà của phiên, G0 là `PASS`, G1–G6 là `INCONCLUSIVE` và G7
là `NOT_ASSESSED` theo giao thức lịch sử của phiên. Bốn cặp pilot không đủ để
kết luận ưu thế thống kê, mức tiết kiệm ổn định hoặc nghiệm thu toàn bộ.

Full A/B chạy sau pilot là phiên độc lập, bị gián đoạn và không được gộp vào
số liệu trên. Bản sửa E06 tiếp theo cũng không thuộc snapshot pilot này; không
dùng kết quả pilot để khẳng định bản sửa hoặc source hiện tại đã đạt native.
