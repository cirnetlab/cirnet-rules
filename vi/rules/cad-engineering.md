# Kỹ thuật CAD CIRNET

Dùng quy tắc này để tạo, kiểm tra, render hoặc chuẩn bị tài sản CAD thuộc
phạm vi đã duyệt. CIRNET sở hữu thiết kế, yêu cầu bằng chứng và nghiệm thu.
Dùng [giao diện workspace kỹ thuật](engineering-workspace-ui.md) cho hành vi
trình chỉnh sửa quanh tài sản. Dùng [định tuyến kiến trúc](../references/architecture-routing.md)
và kiểm tra mã nguồn hiện hành của chủ sản phẩm trước khi tiếp nhận tệp xuất.

## Đặc tả thiết kế

Dùng [đặc tả nhiệm vụ](../templates/spec.md) hiện có. Ghi các đầu vào thiết
kế sau trước khi dựng mô hình.

| Mục ghi | Đầu vào thiết kế |
| --- | --- |
| Mục đích và ràng buộc | Ca sử dụng, ảnh tham chiếu và phê duyệt, đặc trưng bắt buộc, ưu tiên, giả định và quyết định còn mở |
| Hình học và lắp ghép | Đơn vị, kích thước, tỷ lệ, dung sai, chuẩn và hướng trục, cây chi tiết, tư thế lắp, khe hở, đối xứng, biên dạng, độ cong, bo góc, lỗ thông gió và lỗ mở |
| Ngoại quan | Vật liệu và hoàn thiện theo bề mặt có tên, màu, giá trị PBR, chi tiết, decal và góc nhìn tham chiếu |
| Tương tác | Tên chi tiết ổn định, chuẩn đầu nối và lựa chọn, pivot và trục chuyển động cục bộ, ID đầu mút của chủ sở hữu nếu áp dụng |
| Bàn giao | Mã tham số chỉnh sửa được, định dạng và runtime đích, revision, ngân sách dung lượng, hình học và GPU, tiêu chí đo theo bước |

Gắn một cách quan sát với từng kích thước và đặc trưng bắt buộc. Xác định
yêu cầu bắt buộc và ưu tiên cần rà soát trực quan. Ghi giá trị tạm và xin
duyệt các quyết định còn mở ảnh hưởng đến lắp ghép, hành vi hoặc an toàn.
Đối chiếu từng revision với mục đích ban đầu.

## Dựng và xác minh tài sản

1. Hiểu và đặc tả trước khi dựng chi tiết tham số có thể chỉnh sửa trong
   workspace đã duyệt. Đặt tên bền vững cho đặc trưng, tham chiếu và chi tiết
   lắp ráp.
2. Kiểm tra kết quả thật từ kernel. Ghi lỗi dựng hoặc phép Boolean, khối rỗng,
   tính hợp lệ theo kernel, kích thước đặc trưng, khe hở hoặc giao thoa ở mỗi
   tư thế bắt buộc. Đối chiếu kích thước và đơn vị trong đặc tả. Bổ sung assertion
   native có mục tiêu khi bản tóm tắt công cụ thiếu độ bao phủ.
3. Render và so sánh. Chụp ảnh kỹ thuật trực giao từ trước, sau, hai bên,
   trên và dưới khi phù hợp. Dùng góc đẳng trục và ảnh chi tiết cho lắp ghép,
   hình dáng và vùng khuất. Giữ góc nhìn và tỷ lệ nhất quán khi so sánh.
   Xem đường nối, độ cong và chia lưới ở khoảng cách quan sát dự kiến.
4. Chỉ sửa lỗi đã quan sát. Giữ mã tham số và lần thất bại đầu, chạy lại
   assertion liên quan và làm mới bằng chứng đã cũ. Dùng
   [quy trình giới hạn lần sửa](../docs/distribution.md#giới-hạn-các-lần-sửa)
   để dừng khi thiếu bằng chứng, quyền hoặc tiến triển.
5. Xác minh từng tệp xuất rồi rà soát và bàn giao. Ghi lệnh, thư mục làm việc,
   revision và SHA-256 đầu vào, định danh và phiên bản công cụ, mã thoát và
   số assertion từng bước, vị trí báo cáo giới hạn và hash tài sản.
   Dùng [Task Continuity](../docs/distribution.md#task-continuity) để lưu
   checkpoint và [xác minh claim](../docs/distribution.md#đánh-giá-các-bước-bắt-buộc)
   cho bằng chứng hiện hành theo phạm vi. Thao tác lỗi giữ `FAIL`. Thiếu điều
   kiện tiên quyết là `BLOCKED`. Bước chưa chạy hoặc bị bỏ qua giữ trạng
   thái tương ứng và để yêu cầu bắt buộc ở trạng thái chưa đủ bằng chứng.

### Rà soát trình bày

Đối chiếu ảnh render thật với tham chiếu đã duyệt. Rà đường nét tổng thể, tỷ
lệ, độ nhất quán của bo góc và bề mặt, chi tiết, phản ứng vật liệu, ánh sáng,
bóng, phản xạ, khung hình và khả năng đọc tại tỷ lệ hiển thị dự kiến. Ghi
góc nhìn, thiết lập, lỗi, yêu cầu bị ảnh hưởng và quyết định của người rà
soát. Dùng so sánh ảnh cùng góc nhìn để tìm hồi quy. Con người quyết định
nghiệm thu thẩm mỹ. Lưu ảnh kỹ thuật và ảnh trình bày thành tài sản riêng.

## Tiếp nhận GLB tĩnh

Dùng cổng Node không cần dependency bổ sung trong checkout trước khi gửi
GLB tĩnh tự chứa. Người vận hành cung cấp kỳ vọng và pin đã rà soát độc lập.
Kỳ vọng chứa đường dẫn và SHA-256 tài sản, số byte tối đa, số tam giác tối
đa, số lần hiện diện primitive tối đa và tên nút bắt buộc nếu có. Dùng
ngân sách của chủ sở hữu. Cổng đếm riêng mỗi lần mesh xuất hiện trong cảnh
kích hoạt. Số primitive hiện diện là chỉ báo ước lượng chi phí vẽ.

```json
{
  "schemaVersion": 1,
  "path": "out/enclosure.glb",
  "sha256": "<independently reviewed artifact SHA-256>",
  "budgets": { "bytes": 1500000, "triangles": 20000, "primitiveInstances": 120 },
  "requiredNodes": ["enclosure_base", "enclosure_lid", "connector_usb"]
}
```

Các giới hạn trên là ví dụ fixture. Từ checkout Rules đã duyệt, với Node
đã cài và pin kỳ vọng chữ thường được lưu độc lập:

```powershell
node ./scripts/verify-cad-asset.mjs --root $approvedAuthoringRoot --expectation intake.json --sha256 $expectationPin
```

Cả hai đường dẫn tệp là đường dẫn nguyên văn tương đối với thư mục gốc tạo
mẫu. Dùng workspace do chủ sở hữu kiểm soát sau khi hoàn tất xuất tệp.
Cổng giới hạn kỳ vọng ở 32 KiB và GLB ở 32 MiB. Nó đọc tệp mà không chạy
công cụ tạo mẫu hay tải tệp. Phạm vi hỗ trợ là GLB 2.0 với bộ đệm nhúng,
ảnh PNG/JPEG nếu có và primitive tam giác tĩnh. Cổng chấp nhận
`KHR_materials_unlit` được khai báo đúng ở dạng tùy chọn hoặc bắt buộc.
Extension ngoài hỗ trợ và trường hợp vượt giới hạn quét trả `BLOCKED`.
Cổng kiểm tra container, cây cảnh được chọn và cấu trúc khai báo phép biến
đổi. Với primitive đang dùng, cổng kiểm tra số lượng thuộc tính, phạm vi
bộ đệm, stride và căn chỉnh, số thực hữu hạn, phạm vi chỉ số đã giải mã và
các giá trị chỉ số dành riêng. Cổng đối chiếu cận POSITION và cận chỉ số đã
khai báo với giá trị giải mã. Cổng kiểm tra liên kết giữa vật liệu và texture,
tham chiếu sampler, tập UV bắt buộc và chữ ký định dạng PNG/JPEG đã khai báo.
Buffer view chứa ảnh dành riêng cho dữ liệu ảnh. Tổng ngân sách cho các lần
quét hình học riêng là 32 MiB. Mỗi tên bắt buộc xuất hiện đúng một lần
trong cảnh đã chọn.
Báo cáo JSON dùng `PASS`/0, `FAIL`/1 hoặc `BLOCKED`/3.

Dùng phép kiểm của chủ sở hữu cho chất lượng bề mặt lưới, hướng và độ dài
pháp tuyến, kích thước thế giới, hành vi biến đổi, giải mã ảnh và ngoại quan
vật liệu. Các giá trị cực trị tùy chọn của tọa độ texture nằm ngoài
phép so sánh cận tĩnh. Đo bộ nhớ và số lệnh vẽ trong runtime của chủ sở hữu.
Giữ bằng chứng CAD native, phê duyệt trực quan và bằng chứng sản phẩm riêng.
SHA-256 đã rà soát độc lập ghim các byte. Ghi ai tạo tệp và cách chủ sở hữu
xác lập xuất xứ.

[Đặc tả kỹ thuật CAD](../specs/cirnet-rules/cad-engineering/spec.md)

## Chạy text-to-cad

Mã nguồn đã kiểm tra là `earthtojake/text-to-cad` tại `7687743d80305ba112666aee0a5e8c12af138458`, bản phát hành `0.11.15`. Manifest plugin Codex nạp các Skill và `codex.mcp.json`. Cấu hình này dùng lệnh stdio `uvx --no-config --managed-python --python 3.14 cad@0.11.15 mcp` với thời hạn khởi động 300 giây. MCP phục vụ trình xem cục bộ. Hợp đồng MCP cho Codex giao việc kiểm tra, chụp ảnh và xuất tệp cho CLI. Khởi động plugin mặc định có thể tải môi trường chạy và cài công cụ toàn cục. Dùng bản cài đã được cho phép để tương tác với trình xem và để chủ sở hữu quyết định việc kích hoạt plugin.

Chỉ áp dụng các lệnh mẫu sau khi xác minh đường dẫn tuyệt đối và đối chiếu `cad --version` với bản được duyệt. Kiểm tra trước môi trường Python tương ứng có `cad[build123d]` và nạp được build123d/OpenCASCADE. Engine có thể tự cài môi trường kernel hoặc môi trường script còn thiếu, kể cả thử lại qua mạng. Nếu ngoài phạm vi cài đặt, yêu cầu môi trường có sẵn. Ưu tiên dùng Python của môi trường đó để chạy script mô hình theo CLI reference. Chỉ dùng lệnh được tài liệu hóa `cad run <script.py>` khi môi trường tương thích đã sẵn sàng và cơ chế phân giải phụ thuộc được cho phép.

Chạy trong thư mục dựng hình cô lập đã được duyệt, với thư mục engine, trạng thái, bộ nhớ đệm và đầu ra đều nằm bên trong. Đặt `CAD_ENGINE_DIR`, `CAD_STATE_DIR`, `CAD_CACHE_DIR`, `CAD_UPDATE_CHECK=0` và `CAD_TELEMETRY=0` trong môi trường tiến trình. Daemon và Python của nó chạy với quyền truy cập của tài khoản khởi chạy. Các thư mục trên định vị dữ liệu và đầu ra. Áp dụng giới hạn ghi tệp và kết nối mạng có thể cưỡng chế cho daemon và chỉ chạy script cục bộ đã được rà soát. Giữ token và URL trình xem đã xác thực ngoài bằng chứng chia sẻ. Chỉ dừng máy chủ cô lập do phiên làm việc này khởi động.

```powershell
& $Cad --version
& $Cad server start
& $CadPython -B ./src/enclosure.py
& $Cad model inspect $Id --json
& $Cad model interference $Id --json
& $Cad model snapshot $Id ./out/iso.png --camera iso --json
& $Cad model export $Id ./out/enclosure.step --json
& $Cad model export $Id ./out/enclosure.glb --tolerance 0.05 --angle 5 --json
& $Cad glb inspect ./out/enclosure.glb --json
& $Cad server stop --json
```

Tạo `out` dưới thư mục được duyệt. `$Cad` và `$CadPython` là các đường dẫn đã xác minh. `$Id` lấy từ lần dựng hiện tại. Script gọi rõ ràng hàm `@model` không có tham số, trả về hình build123d và gắn nhãn các chi tiết trong cụm lắp. Kiểm tra kích thước bao chính xác, thể tích và các tham chiếu bắt buộc. Bổ sung kiểm tra tính hợp lệ của khối, khe hở và đặc trưng cần thiết trên hình học gốc. Kiểm tra giao thoa ở từng tư thế yêu cầu và ghi nhận các cặp chưa đo. Đọc cảnh báo ảnh chụp và tệp xuất để tìm mặt chưa được chia lưới, kể cả khi mã thoát bằng 0. Lệnh kiểm tra GLB liệt kê các node, lưới và hoạt ảnh. Xác thực tên bắt buộc, phép biến đổi và ngân sách sản phẩm trên tệp xuất. Trình xuất áp dụng `(x,z,-y) × 0.001` ở node gốc để chuyển từ milimét và trục +Z hướng lên của CAD sang mét và trục +Y hướng lên của GLB. Giữ các phép biến đổi node con và kiểm tra độc lập tỷ lệ cũng như hướng trước trong hệ tọa độ thế giới. Có thể tạo ảnh trình bày bằng `cad.blender.load` để nạp GLB khi đã có sẵn `bpy 5.2.3` trên Python 3.13. Lưu ảnh trình bày thực tế riêng với ảnh kỹ thuật OCCT.

### Chạy thử với hộp mô phỏng

Mô hình mẫu có thông số tạm thời theo milimét CAD, với gốc tọa độ ở tâm mặt đáy. Dựng hộp 80 × 50 × 21, thân cao 18 mm, nắp dày 3 mm, thành dày 2 mm, góc ngoài R3 và lỗ mặt trước 12 × 6 có tâm tại tọa độ CAD `(0,-25,10)`. Dùng các nhãn `enclosure_base`, `enclosure_lid` và `connector_usb`. Kiểm tra các khối hợp lệ, có đủ nhãn, kích thước và bán kính trong dung sai ±0,05 mm, lỗ thông, khe hở quanh đầu nối trong lỗ tối thiểu 0,5 mm và không có chồng lấn ngoài ý định ở tư thế lắp. Chụp mặt trước, sau, trái, phải, trên, dưới, hình trục đo và chi tiết bên trong khi ẩn nắp. Kiểm tra tên node trong tệp xuất, kích thước bao 0,080 × 0,021 × 0,050 m trong hệ tọa độ thế giới, mặt trước GLB hướng +Z, và mã SHA-256 của nguồn và hiện vật. Đây là mẫu dự kiến chạy bằng công cụ thực. Khi thiếu môi trường chạy, ghi rõ thao tác nào bị chặn hoặc bỏ qua cùng lý do.

Trong một lần chạy fixture riêng, bỏ chuẩn đầu nối và xác nhận yêu cầu về
tên node thất bại. Kiểm tra node trực tiếp trên GLB. Ở revision đã đọc,
trình xuất GLB có thể dùng chung node giữa các phần tử cùng cấp có cùng
nhãn và bỏ đường dẫn rỗng. Trình xuất ghi vị trí đỉnh, pháp tuyến và hỗ trợ
tham số vật liệu. Sản phẩm cần texture phải có quy trình UV và ảnh được chủ
sở hữu rà soát. Xem `packages/cad/src/cad/engine/export/glb.py` của công cụ.

## Bàn giao tài sản cho Interface

Ghim revision Interface và xác định hồ sơ yêu cầu của tài sản đích trước khi
xuất. Hợp đồng preview đã rà soát nằm tại revision Interface
`82dd99a62c899aa413d03856c41e252bf289ba22` (Three.js 0.171.0). Workbench dựng
cả hai thiết bị bằng factory thủ tục. Các trang preview độc lập nạp GLB do
Interface quản lý và hiển thị trạng thái khi dùng mô hình thủ tục dự phòng.
Giữ cả hai đường chạy cùng hành vi đặt, chọn, biến đổi, lưu và Undo/Redo.
(`docs/en/guides/instrument-models.md:5–29`,
`src/three/ComponentModel.ts:346–349`, `src/preview/instrumentPreview.ts:57–75`.)

Preview thiết bị để bàn có trục Y hướng lên, mặt trước hướng +Z, gốc ở tâm
đáy theo dấu chân thiết bị và nút gốc có biến đổi đồng nhất. Các chân đỡ và
mép sau vỏ tiếp xúc mặt phẳng Y=0. Kích thước hiển thị trong catalog của máy
phát và máy hiện sóng là `[6,3.6,2.8]` và `[6,3.8,2.8]` theo đơn vị thế giới
của trình biên tập. `visual_chassis` nghiêng −10° quanh X cục bộ, còn nút gốc
giữ góc xoay của trình biên tập quanh Y. Với vỏ linh kiện vật lý, dùng hệ tọa
độ footprint chuẩn có gốc trên mặt phẳng tiếp xúc chân và tỉ lệ
`s = 0.13 / 2.54` đơn vị thế giới trên mỗi milimét, nên bước chân 2.54 mm
tương ứng 0.13 đơn vị thế giới. Trừ tọa độ gốc nguồn đã quy định trước khi áp
dụng hướng trục đã đo và tỉ lệ. Ví dụ, nếu hệ CAD đo được có +X hướng phải,
+Z hướng lên và −Y hướng trước, điểm tương đối với gốc nguồn được đổi thành
`s × (x, z, −y)` khi dùng tỉ lệ vật lý đó. Chỉ đổi trục một lần. Với GLB
từ text-to-cad đã dùng hệ Y hướng lên được duyệt và đơn vị mét, dùng chuẩn
được duyệt và `1000 × s` đơn vị thế giới trên mỗi mét cho vỏ linh kiện vật lý
đó. Sau khi nạp, kiểm tra mốc trục, kích thước và vị trí điểm neo. Với thiết bị để bàn,
xác định tỉ lệ theo bố cục đích và ghi rõ phép đổi tọa độ.
(`src/data/partCatalog.ts:7–8`, `src/three/PackageModel.ts:6–17`,
`src/electrical/modulePose.ts:3–19`, `src/three/InstrumentChassis.ts:5–31`.)

Giữ ổn định tên nút và điểm xoay cục bộ. `screen` độc lập và dùng vật liệu
unlit. Các núm quay quanh Z cục bộ qua tâm, hướng nhấn nút cũng theo Z cục bộ.
Các cụm đầu nối gồm `ch1_output`, `ch2_output`, `sync_counter` cho máy phát và
`ch1_bnc`, `ch2_bnc`, `trig_out_bnc` cho máy hiện sóng. Giữ tên nút gốc lịch sử
`net_circuit_*` đã duyệt cho tệp đang phân phối. Dùng vật liệu PBR chuẩn và các
extension được bên sở hữu hỗ trợ. Cả hai GLB được theo dõi đều nhúng ba PNG
và dùng `KHR_materials_unlit`. CanvasTexture của bản nạp dùng sRGB và
`flipY = false`. Ánh xạ mỗi điểm neo chức năng vào ID đầu nối logic chuẩn có
sẵn. Các đầu BNC của hai thiết bị chỉ phục vụ hiển thị. Contracts sở hữu các
định danh trong catalog. Lệnh thao tác đồ thị của Interface quản lý kết nối
điện và lịch sử, còn scene chiếu các đầu nối đã được phân giải để đi dây và
chọn đối tượng. (`docs/en/architecture/namespace.md:62`,
`src/preview/instrumentPreview.ts:35–47`, `src/electrical/endpoints.ts:15–38`,
`src/three/SceneManager.ts:78–90`, `docs/en/architecture/workbench.md:5–17`.)

Ghi ID tài sản, revision nơi tạo, nguồn gốc của mã nguồn và công cụ, SHA-256
và các chi phí đã đo. Provider preview kiểm tra mã băm trong descriptor và
giới hạn 2.000.000 byte cho mỗi GLB do Interface quản lý. Test tài sản phân
phối yêu cầu dưới 1.500.000 byte. Test factory thủ tục yêu cầu mỗi thiết bị
dưới 100 mesh, máy phát dưới 20.000 tam giác và máy hiện sóng dưới 25.000 tam
giác. Tệp phân phối thực tế có 84 mesh, 18.038 tam giác và 1.460.816 byte cho
máy phát, 94 mesh, 16.926 tam giác và 1.219.028 byte cho máy hiện sóng. Dùng giới
hạn được bên sở hữu duyệt cho các hồ sơ yêu cầu khác và mọi giới hạn về lượt
vẽ hoặc bộ nhớ. (`src/data/instrumentAssets.ts:3–12`,
`src/services/assets/gltfAssets.ts:47–104`, `tests/function-generator.test.ts:75–96`,
`tests/oscilloscope.test.ts:54–60,128–133`.)

Chuyển kết quả kiểm tra tĩnh ứng viên và bằng chứng đo khi bàn giao cho
Interface. Nghiệm thu của bên sở hữu gồm thử nạp lại qua `GLTFLoader`, hành vi
preview và mô hình dự phòng, kết xuất trên trình duyệt và các kiểm thử hồi quy
workbench, đồ thị chịu ảnh hưởng. (`tests/gltf-assets.test.ts`,
`tests/instrument-chassis.test.ts`, `tests/function-generator.test.ts:91–136`,
`tests/oscilloscope.test.ts:128–173`, `e2e/assets.spec.ts`.)

<!-- source: en/rules/cad-engineering.md -->
<!-- source-sha256: 443bd24c5f2059fe9a946c935a80afa40142bee115c1f65f1ab2956bf86717bf -->
