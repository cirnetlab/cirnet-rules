<!-- doc-id: AUDIT-PRODUCTS -->
<!-- revision: 1 -->
<!-- source-sha256: f38552bc96084f7a90b75e1b8cb9f8c0da257b7dd184f8222cb74e7370e0c60f -->
<!-- translation-status: synchronized -->
# Kiểm toán sáu repo sản phẩm cho đợt phát hành 0.2.4

<a id="method"></a>

## Phạm vi và khả năng tái lập

Ảnh chụp bằng chứng local được thu ngày 2026-11-08 UTC trên Windows bằng Git 2.52.0.windows.1 và PowerShell 7.6.9. Sáu worktree local đều sạch và đang ở nhánh `master`. Truy vấn Git công khai độc lập xác nhận `master` là nhánh mặc định và trả về cùng commit. Truy vấn không thấy ref công khai `v0.2.4` hay `v0.2.2`. Điểm cuối phát hành công khai cho `v0.2.4` trả HTTP 404 ở cả sáu repo. Các bản nháp, phát hành riêng tư và bằng chứng CI riêng tư nằm ngoài phạm vi quan sát.

| Repository | Commit | Số đường dẫn Git theo dõi | Tag lịch sử công khai được chọn |
| --- | --- | ---: | --- |
| [cirnet-contracts](../specs/cirnet-contracts/audit.md#snapshot) | `4bfd0950f564aaf24abaf6b8b82399c2d54d0e39` | 334 | `v0.1.2` |
| [cirnet-remote](../specs/cirnet-remote/audit.md#snapshot) | `1a38c1331a07bdc3b5644c70eebb40590ae53452` | 629 | `v0.1.4` |
| [cirnet-hardware](../specs/cirnet-hardware/audit.md#snapshot) | `2139cccc8b36dc403c2f7182dfb670a97513858a` | 91 | `v0.1.2` |
| [cirnet-simulator](../specs/cirnet-simulator/audit.md#snapshot) | `c05be23bef71de44a9027319af945d3e635d0882` | 73 | `v0.1.2` |
| [cirnet-interface](../specs/cirnet-interface/audit.md#snapshot) | `f1ff70b9f22554c0680b92679e9e44a5544247ff` | 609 | `v0.1.4` |
| [cirnet-fpga](../specs/cirnet-fpga/audit.md#snapshot) | `bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0` | 56 | `v0.1.2` |

Báo cáo từng repo ghi ID cây gốc thực tế và digest danh mục thô. Sáu giá trị tra cứu ban đầu là đối tượng commit. Cây gốc ghi đường dẫn và định danh từng mục; blob định danh một đối tượng Git. SHA-256 artifact được tính trên byte artifact, độc lập với các ID Git.

Chạy các lệnh sau trong repo sản phẩm. Để tính digest danh mục, băm luồng byte nguyên gốc của lệnh Git cuối, kể cả các dấu phân cách NUL, trước khi phân tích bản ghi đường dẫn. Thay `HEAD` bằng commit đã ghim để tái lập lần kiểm toán này. Tránh băm pipeline đã được PowerShell giải mã.

```text
git --no-optional-locks status --porcelain=v1 --untracked-files=all
git rev-parse HEAD HEAD^{tree}
git --no-optional-locks -c credential.helper= ls-remote --symref origin HEAD refs/heads/master refs/tags/v0.2.4 refs/tags/v0.2.2 refs/tags/v0.1.4 refs/tags/v0.1.2
git --no-optional-locks ls-tree -r -z --full-tree HEAD
```

Đã lập danh mục đủ 1.792 đường dẫn Git theo dõi. Báo cáo từng repo ghi phần mã nguồn, tài liệu và hướng dẫn kiểm chứng đã đọc, cùng các nhóm chỉ được lập danh mục. Đã rà `README.md`, `AGENTS.md`, changelog và Git history liên quan của từng checkout. Chỉ dẫn Agent FPGA nằm trong `fpga/AGENTS.md`. Kiểm toán quản trị lần này chưa chạy build sản phẩm, máy sạch, CI đám mây hoặc phép thử vật lý. Byte artifact lưu trữ và hash biên nhận được đối chiếu riêng.

<a id="identity"></a>

## Định danh và thứ tự phụ thuộc

| Miền | Định danh quan sát được | Xử lý trong baseline |
| --- | --- | --- |
| Phát hành hệ thống | Remote sở hữu tổ hợp 0.1.4 đã phát hành. Đợt mục tiêu 0.2.4 còn bị chặn. | Chỉ owner kích hoạt tổ hợp đầy đủ đã kiểm chứng. |
| Package thành phần | Remote native và Interface là 0.2.4. Contracts, Hardware, Simulator và FPGA chuẩn bị 0.2.2. | Điều phối các bản phát hành thành phần do CIRNET sở hữu theo đợt 0.2.4 được yêu cầu. Ghi phiên bản thực được duyệt cho từng thành phần cùng commit nguồn sạch, target và biên nhận artifact. Đổi riêng số phiên bản chưa đủ đóng gate. |
| Họ tương thích CIRNET | Các biểu diễn và pin legacy vẫn tồn tại trong nguồn đã đọc. | `0.2` là họ quản trị đề xuất. Từng revision biểu diễn chính xác cần kiểm chứng tương thích. |
| Revision hợp đồng | Lock đã phát hành chọn Contracts 0.1.2. Rust Remote và Hardware ghim `=0.1.2` tại `e640f80b1199779659d3db7817fee16d13cc24c1`. Interface còn ghim archive npm Contracts 0.1.2. | Chọn revision chính xác của owner và digest artifact nguồn và sinh. Duyệt mọi thay đổi về source, wire hoặc hành vi. |
| Phiên bản quản trị | `cirnet-rules` quản lý lựa chọn và bằng chứng. Package và metadata chính sách của repo quản lý phiên bản phần cung cấp. | Ghi commit và phiên bản quản trị đã dùng cho từng quyết định. |
| Phiên bản toolchain | Rust pin 1.99.1 và MSRV 1.90 là hai miền. Simulator dùng C++20 với CMake tối thiểu 3.21; CI Interface dùng Node 26. FPGA có khóa công cụ riêng. | Ghi compiler, target và lock đã kiểm chứng cho từng artifact. |

Chốt ngữ nghĩa API Remote candidate và handoff cùng Contracts trước. Cố định và duyệt bản xuất ở revision Remote chính xác, rồi tạo tài nguyên, crate Rust và SDK Contracts được chọn. Xác minh worker C++ Simulator và nguồn FPGA theo ranh giới đã duyệt. Dùng Contracts đã chọn và worker chính xác trong Rust Hardware, rồi kiểm chứng cặp Remote native và Interface. Remote giữ quyền sở hữu release-set. Nghiệm thu CI, gate an toàn, phương án xử lý dữ liệu cũ và kích hoạt theo sau bằng chứng tích hợp.

<a id="contract-inventory"></a>

## Danh mục hợp đồng và kiểm chứng còn thiếu

**Đích của mọi dòng là họ `0.2`.** Nguồn và ảnh chụp repo bên dưới xác định revision chính xác đã quan sát. Revision cutover từng miền đang chờ owner chọn và kiểm chứng cặp. Phương ngữ schema, cú pháp protobuf và phiên bản định dạng là các định danh riêng.

| Miền và biểu diễn | Authority và bên tạo → bên dùng | Định danh quan sát, nguồn/sinh và kiểm chứng cần có |
| --- | --- | --- |
| HTTP và OpenAPI JSON | Operation và bản xuất Remote → handoff/SDK Contracts → Interface hoặc client HTTP | `contracts/api/openapi.json` của Contracts dùng OAS 3.1.0, `info.version` 0.1.2, 63 ID duy nhất. Native `native/src/openapi.rs` chiếu phiên bản runtime và giữ 46 operation. Kiểm tra sinh SDK từ đúng bản xuất native, ID giữ lại, route đã bỏ, xác thực, dạng response và ngữ nghĩa lỗi. |
| WebSocket JSON | Schema Contracts → Remote phát → event client Interface | `contracts/public/realtime.schema.json`, Remote `native/src/events.rs` và Interface `src/services/websocket/client.ts` triển khai giao thức snapshot 1.0. Kiểm tra hello, đứt chuỗi, giới hạn frame, mất kết nối và đồng bộ qua snapshot được phép. Cấm phát lại lệnh. Xem `native/tests/websocket_protocol.rs` và `tests/websocket-reconcile.test.ts`. |
| Capture nhị phân | Contracts `contracts/capture/` → writer/API Remote → bộ giải mã Interface | NCAP digital v1 và sampled v2 có bố cục nhị phân riêng; MIME HTTP chuẩn `application/vnd.cirnet.capture; version=1`. Native từ chối Accept chỉ chứa MIME legacy. Kiểm tra giới hạn parser, waveform chuẩn và thương lượng qua `native/tests/runtime_captures.rs` và `tests/capture-golden.test.ts`. |
| Hardware RPC và Preview | Proto chuẩn Contracts → Hardware/Remote; proto riêng Hardware → Remote | `contracts/hardware/v2/rpc_v2.proto` là RPC local 2.0, package `cirnet.hardware.v2`. `native/preview.proto` của Hardware định nghĩa `cirnet.hardware.internal.v1.SimulatorPreview` riêng với Describe phiên bản 1 và byte JSON. Sinh descriptor và binding từ đầu vào đã chọn. Kiểm tra giới hạn, phiên bản không hỗ trợ, số trường đã giữ chỗ, timeout và đường không khả dụng. |
| IPC worker Simulator | Codec Hardware ↔ worker C++ Simulator | `native/src/simulator/protocol.rs` của Hardware và worker native Simulator mã hóa CNP1/CNP2 qua frame độ dài little-endian, định danh request/generation và giới hạn khung. CNP2 bổ sung DC có cấp nguồn. Chạy `native/tests/simulator_parity.rs` của Hardware với đúng worker, gồm frame lỗi và cancel/timeout. |
| CircuitGraph và lưu trữ | Schema Contracts → tuần tự hóa Interface → lưu trữ Remote | `contracts/circuit-schema/` có các dạng JSON 1.0/1.1 và 1.2. Remote `native/src/db/migrations/` và `native/tests/db_migrations.rs` quản lý thứ tự migration. Kiểm tra từ chối đầu vào cũ và migration minh thị hoặc dữ liệu mới. Sao lưu, phục hồi và xác thực SQLite và PostgreSQL theo lựa chọn. |
| Thực thi, kết quả và capability | Registry và schema lab/public Contracts → Remote và Hardware → Interface | `contracts/manifest.json` ghi định danh riêng, gồm plan, quyết định, kết quả và capability 1.0. `contracts/physical-lab/lab.schema.json` và `contracts/public/api.schema.json` cấp đầu vào kiểm tra và sinh binding. Kiểm chứng đơn vị, giới hạn, thứ tự, nullability, permit, ownership và phân biệt đã triển khai, đã kiểm chứng, đang khả dụng. |
| Logic FPGA và thanh ghi | Schema logic/bản đồ thanh ghi Contracts → kiểm chứng FPGA và lựa chọn Hardware/Remote | `contracts/fpga-protocol/control.schema.json` thực thi `1.0-scaffold`; `register-map.md` mô tả `1.0-reserved`. `2.0-scaffold` là đích thiết kế. Kiểm thử chốt RTL, từ chối peer chưa hỗ trợ; khả dụng vật lý cần kiểm chứng thiết bị được phê duyệt riêng. |
| Manifest phát hành JSON | Tổ hợp Remote → kiểm chứng, bộ cài và đóng gói | `integration/release-set.json` dùng schema 1. `scripts/system_release_manifest.py` gắn với hệ thống lịch sử 0.1.4; `scripts/native-release-inputs.mjs` kiểm tra đầu vào native riêng. Xác minh nguồn, digest, nền tảng, phiên bản, capability và bằng chứng đã chọn; bảo toàn authority lịch sử. |

<a id="historical-evidence"></a>

## Bằng chứng phát hành lịch sử và ứng viên

[Tổ hợp công bố đã đối chiếu](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/integration/release-set.json) dùng `schemaVersion: 1`, `releaseVersion: 0.1.4` và kênh `github-release`. SHA-256 trên byte checkout quan sát được là `90a8ef4d30ddc19d3024dcb3790fef5d5e041f45fa47922bfd5e34c2fd19c419`. Phong bì tương thích ghi Contracts 0.1.2, Hardware RPC 2.0, FPGA `1.0-scaffold` / `1.0-reserved`, cùng origin, một API scheduler và RPC chỉ local. Băm lại 15 asset lưu trữ theo vai trò repo, tên tệp và kích thước byte đều khớp digest. Phạm vi gồm Remote wheel/sdist/deploy, Contracts wheel/sdist/npm/dev-wheel/dev-npm, Hardware và Simulator wheel/sdist, Interface source/static và FPGA source. Chưa tải lại byte asset từ nơi phát hành.

<a id="candidate-evidence"></a>

## Bằng chứng Windows theo ba định danh riêng

| Bằng chứng | Phạm vi và quyết định |
| --- | --- |
| Hệ thống 0.1.4 đã phát hành | Tổ hợp lịch sử dựa trên Python được giữ nguyên, tách biệt với đợt ứng viên. |
| Bằng chứng chức năng Windows sạch | Tổng hợp lưu trên máy host báo VERIFY và vòng đời PASS trên Windows 11 x64 VirtualBox offline, không cài Python, gồm WebSocket hello có xác thực, waveform NCAP đối chiếu, Restart và phục hồi đúng byte capture. Đã băm lại archive `b7f529820dee37680bb909881cc8de331988f893d15ddf3e250a526fef9015d4` và manifest `a40209a417d5b838b27c0686fac9c46563a5ba7b9ffef9502e76c4d0588878ea`. Manifest ghi executable 0.1.2. Hash biên nhận VERIFY và vòng đời khớp bản tổng hợp host. |
| Stop khi có tải | Cùng bản tổng hợp host ghi hai lượt đánh giá FAIL: Hardware `DRIVER_ERROR`, Remote `SAFE_STATE_FAILED` và vẫn cách ly lease. Digest hai biên nhận Stop thô khớp bản tổng hợp. Chưa đạt yêu cầu 600 mẫu. Pha dọn dẹp chưa xong được ghi là worker thoát hoặc luồng I/O kết thúc. Cần đánh giá lại theo đúng định danh nguồn và ứng viên. |
| Bộ ghép phát triển 0.2.4 | Băm lại archive `6325074ccd869a23ae1e0b6dca30e5fe00e9eda57b27a218d396fd0cfd249489`, manifest `eea887c864ecbd301c712928f8b7483a43c541d0a172bd2554098a105282f711` và đầu vào tĩnh `25b5ddec0922e076a1472d629e12a4c54dd60da2bcf9ce6a4f4240421f7bf4c9` đều khớp hồ sơ Remote. Sáu lớp thay đổi nguồn đều chưa commit. SDK gắn với bản xuất cũ `e1304e7d38581aa21455f6267e45c87f448fd136804c9b6fbf66ff758978d540`; Remote ghi bản xuất đã sửa MIME `10da9731ce85297eadb145d9910d0923e9d17cadfb82708f00eacc8cdac319dc` và việc từ chối metadata cũ của producer tĩnh. Archive phát triển khác cùng tên có hash `3af63459111bcdee501d9e1a92c559607dbfa0088a4fa473aeb6815128647b93`. Chọn theo digest và biên nhận. [Bằng chứng nguồn, target và đầu vào của owner](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/docs/vi/operations/release-version-policy.md) |

Kiểm toán đối chiếu biên nhận và byte local. Chưa chạy lại máy ảo và không chuyển kết quả PASS chức năng trước đây sang ứng viên mới.

**PRODUCT_0.2.4_CUTOVER_READINESS · BLOCKED.** Owner phát hành cần tổ hợp sáu sản phẩm sạch, kiểm chứng hợp đồng đã chọn, từ chối legacy và xử lý dữ liệu, đóng gói tái lập và provenance. Với đúng ứng viên mới, chạy lại VERIFY Windows sạch không Python gồm WebSocket, waveform, Restart và backup/restore, đánh giá Stop khi có tải theo giới hạn an toàn hiện hành và diễn tập rollback. Capability vật lý cần bằng chứng an toàn riêng.
