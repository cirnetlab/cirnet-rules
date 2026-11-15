# Quyền sở hữu repository và định tuyến kiến trúc

Bảng tra cứu dựa trên [Kiến trúc Ecosystem §4](../../docs/tai-lieu-kien-tuc-he-thong/cirnetlab-architecture-vi.md).
Dùng bảng để tìm repo sở hữu, rồi xác minh hợp đồng đang dùng, artifact được
chọn và khả năng sẵn có từ checkout đó. Bảng ghi quyền sở hữu và luồng tích hợp
mục tiêu. Kiểm tra phần đã triển khai trong
[bằng chứng sản phẩm](../docs/toolkit-verification.md#tuân-thủ-kiến-trúc).

Mọi dòng dùng `ecosystem`. Sáu repository cốt lõi của Lab-First dùng thêm
`lab-first`. `fpga` áp dụng cho triển khai bo Lab. Chọn thêm kiến trúc liên quan
khi tác vụ vượt một trong các ranh giới đó. Mũi tên chỉ chiều gọi API hoặc trao
đổi dữ liệu, artifact giữa các bên trực tiếp. Xem phạm vi thẩm quyền trong
[chính sách kiến trúc](../policies/architecture.md).

## Ứng dụng

| Repository | Kiến trúc | Phạm vi trách nhiệm | Giao diện và artifact sở hữu | Trao đổi trực tiếp | Ranh giới cần bảo vệ |
| --- | --- | --- | --- | --- | --- |
| `cirnet_web` | `ecosystem` | Trang công khai | Site tĩnh, nội dung đã duyệt, demo có nhãn | Bên xuất bản được duyệt → người đọc công khai | Giữ dữ liệu học viên riêng tư, đáp án giảng viên, bí mật và điều khiển Lab ngoài site. |
| `cirnet-interface` | `ecosystem`, `lab-first` | Bàn làm việc Lab | Trình sửa CircuitGraph, breadboard Three.js, bảng thiết bị đo, client Lab có kiểu | Contracts và Remote → Interface → Studio và người dùng Lab | Kết nối theo ngữ nghĩa đồ thị. Remote và Hardware cấp quyền thao tác và truy cập FPGA. |
| `cirnet-studio` | `ecosystem` | Tổ hợp không gian làm việc kỹ thuật | Shell, điều hướng, duyệt dự án, bố trí công cụ | Project, Interface, IDE và Analyze → Studio | Tích hợp công cụ của bên sở hữu. Interface, IDE, Numerics và LMS giữ engine và logic miền của mình. |
| `cirnet-ide` | `ecosystem` | UI sửa mã và làm việc với thiết bị | Trình sửa, UI LSP/debug, terminal, serial monitor, lệnh có kiểu | Blocks hoặc Project → IDE → Toolchains và Devices | Gọi môi trường build đã ghim và adapter thiết bị được phép. Toolchains sở hữu thiết lập compiler. |
| `cirnet-blocks` | `ecosystem` | Lập trình trực quan | AST/IR khối, kiểm tra kiểu, xuất mã, ánh xạ lỗi | Blocks → IDE, Project và Toolchains | Xuất mã không cấp quyền nạp firmware hay thực thi đặc quyền. |
| `cirnet-analyze` | `ecosystem` | Trải nghiệm phân tích | Notebook, đồ thị, job tái lập và phép đo dẫn xuất | Remote, Simulator hoặc IoT cùng Numerics → Analyze → Project | Giữ nguyên bằng chứng thô, đơn vị và nguồn gốc. Numerics sở hữu nhân tính toán dùng chung. |
| `cirnet-learn` | `ecosystem` | Trải nghiệm học tập | UI bài học, phản hồi và tiến độ | LMS → Learn, cùng liên kết đến Project và Lab | LMS sở hữu sổ điểm, rubric và quyết định học vụ. Giới hạn nội dung được phép tiết lộ. |
| `cirnet-desktop` | `ecosystem` | Host native | Shell đã ký, bridge tệp/USB/serial theo phạm vi, cập nhật và offline | Interface, Studio hoặc IDE ↔ bridge Desktop có giới hạn | Giới hạn quyền native theo thao tác. Ứng dụng sở hữu giữ logic nghiệp vụ. |
| `cirnet-app` | `ecosystem` | Trải nghiệm di động và máy tính bảng | Client di động, thông báo, cache offline giới hạn | LMS, Project và IoT → App | Xác minh khả năng di động. Thao tác thiết bị và an toàn thuộc bên sở hữu. |

## Nền tảng và giáo dục

| Repository | Kiến trúc | Phạm vi trách nhiệm | Giao diện và artifact sở hữu | Trao đổi trực tiếp | Ranh giới cần bảo vệ |
| --- | --- | --- | --- | --- | --- |
| `cirnet-platform` | `ecosystem` | Định danh dùng chung và tenant | API IAM và tenant, phiên OIDC, claim theo phạm vi, ngữ cảnh audit | Platform → Project, LMS, Remote và các miền khác sau chuyển giao đã kiểm thử | Miền sở hữu cấp quyền cho tài nguyên, lease, điểm và thao tác vật lý. |
| `cirnet-contracts` | `ecosystem`, `lab-first` | Hợp đồng liên repo chuẩn | Schema có phiên bản, DTO, mô tả API/RPC/NCAP, SDK Rust/TS, fixture tuân thủ | Bên sở hữu miền → Contracts → Interface, Remote, Hardware và bên tiêu thụ đã ghim | Ngữ nghĩa nghiệp vụ thuộc chủ miền. Sinh SDK từ schema chuẩn. |
| `cirnet-project` | `ecosystem` | Vòng đời EngineeringProject và cộng tác | API Project, ID revision bất biến, ACL, tham chiếu artifact | Project → Studio, Lab, LMS và Format-Project | Project cấp quyền cho tham chiếu. Bên tạo sở hữu artifact thô và quyền truy cập. Format-Project sở hữu codec. |
| `cirnet-format-project` | `ecosystem` | Tuần tự hóa dự án để trao đổi | Định dạng gói, manifest, reader/writer, fixture toàn vẹn và di trú | Project và artifact miền → Format-Project → Studio và CLI | Kiểm tra giới hạn đường dẫn, kích thước và toàn vẹn. Bên sở hữu tài nguyên cấp quyền export và các hiệu lực import. |
| `cirnet-database` | `ecosystem` | Vận hành hạ tầng dữ liệu | Cấp phát, trình chạy di trú, lưu giữ, quy trình khôi phục đã kiểm thử | Database → Remote, LMS, Project và các miền dữ liệu khác | Miền giữ schema và migration nghiệp vụ. Bảo vệ bản sao lưu và khôi phục. |
| `cirnet-lms` | `ecosystem` | Vòng đời học vụ có thẩm quyền | Khóa học, ghi danh, bài tập, bài nộp, rubric, API và sự kiện sổ điểm | Project và Remote cung cấp tham chiếu → LMS → Learn và giảng viên | Giới hạn bản ghi và tiết lộ theo lớp và tenant. Remote giữ điều khiển Lab và bản ghi đo. |

## Lab, mô phỏng và thiết bị đo

| Repository | Kiến trúc | Phạm vi trách nhiệm | Giao diện và artifact sở hữu | Trao đổi trực tiếp | Ranh giới cần bảo vệ |
| --- | --- | --- | --- | --- | --- |
| `cirnet-remote` | `ecosystem`, `lab-first` | Điều phối Lab | API/WS có phân quyền, thí nghiệm, kế hoạch, lease, sự kiện thao tác, metadata bản ghi đo, bộ phát hành Lab | Interface → Remote → Hardware. Remote → client được cấp quyền. | Kiểm quyền thao tác Lab, lease và nội dung tiết lộ. Hardware sở hữu driver, LMS sở hữu điểm, Platform sở hữu định danh chung sau chuyển giao. |
| `cirnet-hardware` | `ecosystem`, `lab-first` | HardwareStation ảo và vật lý | RPC Rust riêng, giám sát worker, khả năng, Stop an toàn, bảo vệ trạm và truyền tải | Remote → Hardware → Simulator hoặc FPGA. Hardware → Remote trạng thái và bản ghi đo. | Giữ RPC trạm ở vùng riêng và cưỡng chế bảo vệ tại chỗ. Xác nhận trạng thái an toàn hoặc cách ly. |
| `cirnet-simulator` | `ecosystem`, `lab-first` | Mô phỏng điện tử | Worker C++ mặc định cho số và DC có giới hạn, giao thức riêng, adapter SPICE và kết quả mô phỏng | Hardware → worker Simulator đã ghim → Hardware. Gói SPICE native tùy chọn → Simulator. | Giới hạn thực thi và gắn nhãn mô phỏng. Thao tác FPGA vật lý và quyền người dùng thuộc bên khác. |
| `cirnet-fpga` | `ecosystem`, `lab-first`, `fpga` | RTL và vận hành bo trạm Lab | Định tuyến bảo vệ, timing, trigger/capture, SDRAM/CDC, ICD, bitstream và artifact timing/HIL | Driver Hardware ↔ FPGA. FPGA cung cấp artifact bo đã xác minh. | Giữ điều khiển Lab tin cậy, bảo vệ tại chỗ và chỉ định tuyến hạ tầng. API Web, IAM và lịch thuộc bên khác. |
| `cirnet-nhspice` | `ecosystem` | Đóng gói SPICE native có điều kiện | Build/patch và gói ghim độc lập, nguồn gốc model/giấy phép và bằng chứng tương thích | Gói tùy chọn → Simulator | Simulator sở hữu giao thức worker, adapter và việc thực thi. Xác nhận quyết định đóng gói và đặt tên trước khi sửa. |

## Công cụ phát triển, tính toán và mở rộng

| Repository | Kiến trúc | Phạm vi trách nhiệm | Giao diện và artifact sở hữu | Trao đổi trực tiếp | Ranh giới cần bảo vệ |
| --- | --- | --- | --- | --- | --- |
| `cirnet-cli` | `ecosystem` | Trải nghiệm dòng lệnh | Lệnh và binary có phiên bản | CLI → API Project, Toolchains, Devices và Lab tùy tác vụ | Gọi API của miền. Gọi lệnh không làm phát sinh quyền thiết bị hoặc Lab. |
| `cirnet-toolchains` | `ecosystem` | Môi trường build đã ghim | Registry compiler/SDK, adapter/spec build, manifest, đầu ra và log | IDE và CLI với nguồn và profile Devices → Toolchains → artifact build | Giới hạn và cô lập build nguồn chưa tin cậy. Nạp firmware cần quyền thiết bị riêng. |
| `cirnet-devices` | `ecosystem` | Loại thiết bị đích, profile bo và adapter | Danh mục bo, khả năng chân/nguồn, adapter nạp/debug | Profile Devices → Toolchains. IDE và CLI → thao tác Devices. | Kiểm quyền theo thiết bị đích và thao tác. IoT sở hữu các thiết bị trong fleet. Remote sở hữu lease Lab và Hardware sở hữu bảo vệ trạm. |
| `cirnet-numerics` | `ecosystem` | Nhân tính toán khoa học dùng chung | API thư viện, đại số tuyến tính, FFT, thống kê, ODE, độ chính xác và benchmark | Numerics → Analyze, Robotics và job tương thích | Analyze sở hữu trải nghiệm. Simulator sở hữu ngữ nghĩa mạch điện tử. Bảo toàn đơn vị và độ chính xác. |
| `cirnet-integrations` | `ecosystem` | Adapter nhà cung cấp ngoài | Connector LMS, Git, EDA, dữ liệu có phiên bản và fixture tương thích | Nhà cung cấp ngoài ↔ Integrations ↔ miền CIRNET tương ứng | Giới hạn thông tin xác thực connector và đồng bộ. Miền nội bộ giữ thẩm quyền. |
| `cirnet-extensions` | `ecosystem` | Hợp đồng host và vòng đời tiện ích mở rộng | SDK/ABI, manifest, kiểm gói, chính sách tin cậy, khả năng và sandbox | Người phát hành tiện ích → Extensions → ứng dụng host hỗ trợ | Cưỡng chế quyền tường minh và cách ly. Dịch vụ được bảo vệ giữ thẩm quyền phê duyệt. |

## Miền kỹ thuật nâng cao

| Repository | Kiến trúc | Phạm vi trách nhiệm | Giao diện và artifact sở hữu | Trao đổi trực tiếp | Ranh giới cần bảo vệ |
| --- | --- | --- | --- | --- | --- |
| `cirnet-robotics` | `ecosystem` | Mô hình hóa và điều khiển robot | Model, động học, hợp đồng chuyển động/điều khiển và thí nghiệm, adapter mô phỏng | Numerics và Devices → Robotics → Studio hoặc thiết bị đích được phép | Yêu cầu an toàn chuyển động độc lập. Numerics sở hữu nhân dùng chung và Toolchains sở hữu build. |
| `cirnet-iot` | `ecosystem` | Instance thiết bị và đội thiết bị được quản lý | Cấp phát, vòng đời thông tin xác thực, twin, telemetry, API lệnh và cảnh báo | Thiết bị đã ghi danh ↔ IoT → client App và Analyze được phép | Kiểm lệnh theo tenant và thông tin xác thực. Devices sở hữu danh mục và Hardware sở hữu trạm Lab được bảo vệ. |
| `cirnet-silicon` | `ecosystem` | Luồng ASIC giáo dục | RTL sang ASIC, luồng tổng hợp/STA/EDA/PDK và artifact thiết kế đã xác minh | Project và Toolchains đã ghim → Silicon → tham chiếu của Analyze hoặc LMS | Bảo vệ giấy phép PDK và bằng chứng sign-off. FPGA sở hữu RTL điều khiển Lab. |
| `cirnet-ai` | `ecosystem` | Trợ giảng và hỗ trợ kỹ thuật mang tính tư vấn | Truy xuất theo quyền, đề xuất, giải thích và dữ liệu audit | Nguồn miền được cấp quyền → AI → người hoặc client rà soát | Quyền chấm điểm, nạp firmware, cấp nguồn và định tuyến thuộc bên sở hữu. Giới hạn truy xuất theo chủ thể và tài nguyên. |

Ecosystem §3 còn giao nhiệm vụ tổ chức cho `cirnet-rules` về chỉ dẫn và phân
phối agent, `about-cirnetlab` về hồ sơ, thương hiệu và tài liệu công khai đã
duyệt, và `.github` về cấu hình tổ chức GitHub và metadata tự động hóa. Chủ sở
hữu sản phẩm giữ API sản phẩm và quyền phê duyệt thao tác vật lý.

<!-- source: en/references/architecture-routing.md -->
<!-- source-sha256: eb5b10ac42541219cfcc4d5cac77ab900f3f1e2d1fbf38d2b938c4c377a80cf5 -->
