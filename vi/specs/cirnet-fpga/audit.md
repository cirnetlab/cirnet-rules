<!-- doc-id: AUDIT-FPGA -->
<!-- revision: 1 -->
<!-- source-sha256: dab4931e90a004ee5c1d7f197234311b33967a811008ce69e7a94e9a57ea61ec -->
<!-- translation-status: synchronized -->

# Rà soát sản phẩm cirnet-fpga

[English](../../../en/specs/cirnet-fpga/audit.md) · [Bằng chứng liên repo](../../references/product-audit.md)

<a id="snapshot"></a>

## Snapshot và định danh

| Trường | Bằng chứng |
| --- | --- |
| Repository | `https://github.com/cirnetlab/cirnet-fpga.git` |
| Thời điểm và phạm vi | 2026-11-08 UTC, kiểm tra chỉ đọc cục bộ trên Windows |
| Nhánh local và hosted | Local `master` khớp nhánh mặc định hosted `master`. Ref cache `origin/HEAD` trỏ tới `main`. |
| Commit SHA | `bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0` |
| Root tree SHA | `be702be824405c099d5ac80fb4f03260e366a46a` |
| Kiểm kê tracked | 56 tệp. SHA-256 của danh sách cây Git thô là `1b258fcc5af2fe13afae0ae93ab51fb8a2d8c3bbbf51fee04535f874980cc983`. Không có gitlink. |
| Cây làm việc | Sạch tại snapshot kiểm tra. |
| Tag local | v0.1.1, v0.1.2 |
| Tag lịch sử hosted đã chọn | `v0.1.2` trỏ tới `d27c818aca0eb95e9ee8d9b5ba6bda3182909914`. |
| Công bố đích | Git hosted không trả ref `v0.2.4` hoặc `v0.2.2`. Tra release công khai GitHub cho `v0.2.4` trả HTTP 404. Bản nháp riêng ngoài phạm vi quan sát. |

<a id="coverage"></a>

## Phạm vi tệp

Đã xem `VERSION`, `fpga/AGENTS.md` ở cây con, metadata đóng gói và khả năng ở `scripts/build-source.mjs`, `integration/source-provenance.json`, `integration/contracts-version.json` và `integration/toolchain-lock.json`. Đọc bước workflow đóng gói/verification, đường vào `fpga/experiment-controller/rtl/top.v` và `fpga/instrument/rtl/top.v`, mô tả bench trong `fpga/simulation/` và hướng dẫn scaffold/verification Anh–Việt. Cây đầy đủ có RTL/testbench thật và đường đối chứng lịch sử. Không tổng hợp, timing closure, tạo bitstream, nạp board, dùng giao vận thật hoặc chạy RTL. Commit này không có `AGENTS.md` tracked ở root.

<a id="findings"></a>

## Phát hiện và tác động

| ID và trạng thái | Bằng chứng tại commit này | Tác động và xử lý |
| --- | --- | --- |
| AUD-FPGA-01 · OBSERVED · BLOCKER | `VERSION` là 0.2.2. Bộ đóng gói Node tạo `cirnet-fpga-scaffold-0.2.2-native-source.tar.gz`, kèm kiểm kê nguồn và biên nhận. Khóa lịch sử Remote chọn nguồn 0.1.2. | Nếu được đưa vào đợt 0.2.4, chọn biên nhận đã kiểm chứng từ revision nguồn sạch. Archive và schema chưa phải firmware hoặc kết quả HIL. |
| AUD-FPGA-02 · OBSERVED · HIGH | `scripts/build-source.mjs` ghi `physicalAvailable: false`, envelope logic thực thi `1.0-scaffold` và bản đồ thanh ghi dự trữ `1.0-reserved`. Ví dụ `2.0-scaffold` của Contracts là đích thiết kế chưa khả dụng. | Tách các ID miền và rà soát họ 0.2 cùng Contracts/Hardware. Từ chối peer logic/vật lý không hỗ trợ trước khi bật permit. |
| AUD-FPGA-03 · OBSERVED · HIGH | Đóng gói chỉ nguồn dùng Node. Luồng verification RTL dùng tooling test Python ghim, tài nguyên Contracts đã rà soát, slang và Icarus/vvp. Gate hành vi yêu cầu `LAB_FIRST_RTL_PASS` và `PERMIT_MATRIX_RTL_PASS`. | Giữ HDL và toolchain phần cứng. Phân loại helper Python là chuẩn bị cho kiểm thử. Khả năng board thật cần rà soát permit/an toàn và bằng chứng riêng trên board đích. |

<a id="readiness"></a>

## Mức sẵn sàng và bàn giao

**BLOCKED.** Phụ thuộc envelope đã rà soát của Contracts và rà giao diện cùng Hardware. Tạo archive nguồn sạch đã kiểm chứng, gắn revision RTL/contract và thực thi cả hai gate hành vi với toolchain đã chọn. Giữ `physicalAvailable: false` đến khi có bằng chứng riêng về giao vận vật lý, nguồn, timing và board. Governance không vận hành hoặc thay đổi phần cứng.
