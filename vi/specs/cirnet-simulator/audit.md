<!-- doc-id: AUDIT-SIMULATOR -->
<!-- revision: 1 -->
<!-- source-sha256: 505632940edbffa2bbc7f9ef4cf1cb2e930131f96c9cbdd18c6a6f96156e9ef8 -->
<!-- translation-status: synchronized -->

# Rà soát sản phẩm cirnet-simulator

[English](../../../en/specs/cirnet-simulator/audit.md) · [Bằng chứng liên repo](../../references/product-audit.md)

<a id="snapshot"></a>

## Snapshot và định danh

| Trường | Bằng chứng |
| --- | --- |
| Repository | `https://github.com/cirnetlab/cirnet-simulator.git` |
| Thời điểm và phạm vi | 2026-11-08 UTC, kiểm tra chỉ đọc cục bộ trên Windows |
| Nhánh local và hosted | Local `master` khớp nhánh mặc định hosted `master`. Ref cache `origin/HEAD` trỏ tới `main`. |
| Commit SHA | `c05be23bef71de44a9027319af945d3e635d0882` |
| Root tree SHA | `3c6dac7c4163a36b26df9cc29de31310f3945021` |
| Kiểm kê tracked | 73 tệp. SHA-256 của danh sách cây Git thô là `5557996d208554b6cd16e9884718ed807b104563994687c145a555594f136e83`. Không có gitlink. |
| Cây làm việc | Sạch tại snapshot kiểm tra. |
| Tag local | v0.1.1, v0.1.2 |
| Tag lịch sử hosted đã chọn | `v0.1.2` trỏ tới `9ec36fa2772ba8227e15ae33fec9710943a9a775`. |
| Công bố đích | Git hosted không trả ref `v0.2.4` hoặc `v0.2.2`. Tra release công khai GitHub cho `v0.2.4` trả HTTP 404. Bản nháp riêng ngoài phạm vi quan sát. |

<a id="coverage"></a>

## Phạm vi tệp

Đã đọc đầu vào sản phẩm/phiên bản ở `CMakeLists.txt`, `pyproject.toml` và `integration/contracts-version.json`. Đã xem các đoạn khung và worker trong `native/worker.cpp` và codec của Hardware, bộ chọn Python đối chứng trong `cirnet_sim/backends.py`, kiểm tra định danh cài đặt trong `scripts/check_installed.py`, cả hai workflow CI và tài liệu backend/verification Anh–Việt. Cây đầy đủ còn có mô-đun biên dịch/thực thi graph Python, alias đối chứng, header native, test giao thức và fixture so sánh. Dòng mã và payload còn lại chỉ được kiểm kê. Không build, trao đổi khung, benchmark CPU hoặc chạy mô hình.

<a id="findings"></a>

## Phát hiện và tác động

| ID và trạng thái | Bằng chứng tại commit này | Tác động và xử lý |
| --- | --- | --- |
| AUD-SIM-01 · OBSERVED · BLOCKER | `CMakeLists.txt` dựng worker 0.2.2 bằng C++20 và MSVC runtime tĩnh trên Windows. `pyproject.toml` dựng bản đối chứng Python cô lập `0.2.2.dev0` với Contracts 0.1.2. Profile 0.1.4 đã phát hành chọn wheel Python 0.1.2. | Đợt 0.2.4 cần byte worker build từ nguồn sạch và ghim hash. Đường sản phẩm trước baseline trở thành lịch sử sau kích hoạt. Chỉ giữ test/benchmark so sánh có giới hạn. |
| AUD-SIM-02 · OBSERVED · HIGH | Worker đọc độ dài 32 bit little-endian và payload có giới hạn, magic CNP1/CNP2 cùng ID request và generation. Nguồn giới hạn khung 16 MiB và bốn probe. CNP2 mở rộng mô hình cho DC giới hạn/cổng có nguồn. Workflow native Windows có smoke cho hai tag. | Kiểm chứng biểu diễn chính xác trong họ 0.2 cùng Hardware. Chứng minh trường độ dài, dữ liệu sai, sai tag/ID, timeout, vòng đời worker và parity capture. IPC cung cấp cách ly tiến trình. Chưa có bằng chứng cần đổi sang FFI. |
| AUD-SIM-03 · OBSERVED · HIGH | `cirnet_sim/backends.py` giữ bộ chọn đối chứng và `netcircuit_sim/` giữ alias import đối chứng. Tài liệu phân biệt `ngspice` chưa khả dụng với phần mô phỏng giới hạn đã triển khai. | Loại bộ chọn và alias trước baseline khỏi closure phụ thuộc sản phẩm đích. Giữ kho nguồn hoặc mã chỉ dành test một cách tường minh. Ghi nhận analog chưa hỗ trợ, không suy hỗ trợ vật lý. |
| AUD-SIM-04 · VERIFIED · HIGH | `git diff --shortstat v0.1.2 HEAD -- cirnet_sim pyproject.toml setup.py MANIFEST.in scripts/check_installed.py` trả sáu đường dẫn thay đổi, gồm `cpp_backend.py` và `cpp_dc.py` đóng gói mới. ZIP `METADATA` của wheel được chọn ghi `cirnet-simulator` 0.1.2. `.github/workflows/simulator.yml` tại HEAD chỉ có quyền đọc và build wheel `0.2.2.dev0` chưa phát hành để kiểm chứng. | Đường build HEAD đã đọc tách nguồn thay đổi khỏi định danh 0.1.2 đã phát hành. Không thấy bước công bố wheel trong workflow đã rà. Giữ nguyên định danh lịch sử và yêu cầu định danh producer mới được duyệt nếu sau này cho phép artifact khác. |

<a id="readiness"></a>

## Mức sẵn sàng và bàn giao

**BLOCKED.** Nhận profile contract/khung đã rà soát, build worker chính xác, ghi đích/toolchain/hash executable và chạy test khung dương/âm, thời gian và vòng đời với Hardware. Xác định điều kiện nghiệm thu hiệu năng/an toàn được phê duyệt trước khi công bố số đo. Lượt này chưa đóng cổng runtime hoặc benchmark.
