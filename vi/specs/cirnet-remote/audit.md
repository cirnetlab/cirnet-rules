<!-- doc-id: AUDIT-REMOTE -->
<!-- revision: 1 -->
<!-- source-sha256: 6a0d8d2b5a84f940cf02160d39bd09b93472d299ae64cb265b8c4b3eb18b4ee1 -->
<!-- translation-status: synchronized -->

# Rà soát sản phẩm cirnet-remote

[English](../../../en/specs/cirnet-remote/audit.md) · [Bằng chứng liên repo](../../references/product-audit.md)

<a id="snapshot"></a>

## Snapshot và định danh

| Trường | Bằng chứng |
| --- | --- |
| Repository | `https://github.com/cirnetlab/cirnet-remote.git` |
| Thời điểm và phạm vi | 2026-11-08 UTC, kiểm tra chỉ đọc cục bộ trên Windows |
| Nhánh local và hosted | Local `master` khớp nhánh mặc định hosted `master`. Ref cache `origin/HEAD` trỏ tới `master`. |
| Commit SHA | `1a38c1331a07bdc3b5644c70eebb40590ae53452` |
| Root tree SHA | `fbd18a76493861a713e7fdad88fb4628cb9c18bc` |
| Kiểm kê tracked | 629 tệp. SHA-256 của danh sách cây Git thô là `cbb87cf38302f9ba9ff6114f9c616516294aba4bef7e0f3f617c53d80bea79ac`. Không có gitlink. |
| Cây làm việc | Sạch tại snapshot kiểm tra. |
| Tag local | v0.1.1, v0.1.2, v0.1.3, v0.1.4 |
| Tag lịch sử hosted đã chọn | `v0.1.4` trỏ tới `38aea6120920f48cea817827ac2277cce4bef25e`. |
| Công bố đích | Git hosted không trả ref `v0.2.4` hoặc `v0.2.2`. Tra release công khai GitHub cho `v0.2.4` trả HTTP 404. Bản nháp riêng ngoài phạm vi quan sát. |

<a id="coverage"></a>

## Phạm vi tệp

Đã phân tích `integration/release-set.json` và xem các đoạn native, pin phiên bản trong `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `integration/contracts-version.json`, `integration/ci-source-dependencies.json`, `native/src/main.rs`, `native/src/openapi.rs`, `native/src/http_experiments.rs`, `native/src/websocket.rs` và `native/src/hardware.rs`. Đã xem `scripts/package-native-windows.mjs`, `scripts/native-release-inputs.mjs`, `scripts/install_release.py`, `scripts/system_release_manifest.py`, `run-cirnet.bat`, `deployment/windows-native/cirnet-native.ps1`, `services/api/pyproject.toml` và các bước CI liên quan. Đối chiếu tài liệu kiến trúc, Lab-First, an toàn, phiên bản release, profile HTTP và verification ở `docs/en/` và `docs/vi/`.

Kiểm kê tracked đầy đủ gồm mô-đun dịch vụ Rust/Python, launcher, mẫu triển khai, ảnh, lịch sử và test. Các dòng mã, fixture, phụ thuộc generated và asset khác chỉ được kiểm kê. Lượt này không chạy cổng trình duyệt, dịch vụ, cơ sở dữ liệu, đóng gói, benchmark, Stop hoặc phần cứng.

<a id="findings"></a>

## Phát hiện và tác động

| ID và trạng thái | Bằng chứng tại commit này | Tác động và xử lý |
| --- | --- | --- |
| AUD-REM-01 · VERIFIED · BLOCKER | `integration/release-set.json` vẫn là lựa chọn chuẩn của hệ thống 0.1.4 đã phát hành. 15 tệp lịch sử lưu cục bộ đều khớp kích thước và SHA-256. `Cargo.toml` là 0.2.4 nhưng ghim crate `=0.1.2` tại `e640f80b1199779659d3db7817fee16d13cc24c1`. | Bảo toàn bytes lịch sử, chỉ chọn manifest mới khi có bằng chứng chính xác về họ 0.2. Phiên bản executable native chưa kích hoạt train. |
| AUD-REM-02 · VERIFIED · HIGH | So sánh tĩnh tài liệu đóng băng với `native/src/openapi.rs` thấy 63 operation ID duy nhất, 17 đã bỏ và 46 giữ lại. Capture native dùng `application/vnd.cirnet.capture; version=1` và trả 406 nếu Accept chỉ yêu cầu legacy. `native/src/websocket.rs` và `native/src/hardware.rs` có ranh giới WebSocket và RPC cục bộ riêng. | Đối chiếu metadata operation xuất thật với SDK sinh ra. Kiểm chứng tương thích thân gói, method/path và định danh operation, MIME, session/CSRF. Phân tích tĩnh chưa kiểm tra API đang chạy. [Bảng profile của owner](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/docs/vi/api/http-profiles.md) ghi từng method, route thay thế và thay đổi dữ liệu. |
| AUD-REM-03 · OBSERVED · BLOCKER | `services/api/` còn nguồn sản phẩm FastAPI và đóng gói của nó. `scripts/install_release.py` và `scripts/system_release_manifest.py` thực thi cài đặt/xác minh profile đã công bố. Ghép bộ Windows native dùng Node tại `scripts/package-native-windows.mjs`. | Giữ profile đã phát hành làm bằng chứng lịch sử. Chuỗi release/bootstrap mới dùng authority Rust, với launcher shell mỏng nếu cần. Giữ Python trong test/benchmark cô lập. |
| AUD-REM-04 · OBSERVED · HIGH | `docs/en/architecture/physical-lab-safety-contract.md` và `native/src/main.rs` tách an toàn, khả năng thực thi và vòng đời. `scripts/native-release-inputs.mjs` kiểm digest đầu vào và provenance chính xác khi ghép bộ. | Bảo toàn một authority lập lịch, không phát lại, cho phép có bảo vệ, xác nhận safe state và cách ly. Kiểm tra định danh và smoke phần mềm chưa xác nhận an toàn điện. |

<a id="readiness"></a>

## Mức sẵn sàng và bàn giao

**BLOCKED.** Phối hợp nguồn Contracts và SDK xuất chính xác với Interface. Chuẩn bị biên nhận nguồn sạch và tổ hợp C++/Hardware đã rà soát. Xác minh HTTP/WebSocket có xác thực, NCAP, migration dữ liệu lưu dưới dạng thao tác ngoại tuyến tường minh, Stop dưới tải và rollback trên cùng bộ artifact. Rollback hợp lệ phải là bộ đầy đủ sau cutover được phép. Nếu thiếu, dừng an toàn và sửa tiếp về phía trước. Khóa hiện hành và các đường runtime được giữ nguyên.
