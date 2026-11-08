<!-- doc-id: AUDIT-INTERFACE -->
<!-- revision: 1 -->
<!-- source-sha256: 9128a0a9c6de06a604b18e6c132079f55f7511d36fb60d97391ed8c0e7f5983e -->
<!-- translation-status: synchronized -->

# Rà soát sản phẩm cirnet-interface

[English](../../../en/specs/cirnet-interface/audit.md) · [Bằng chứng liên repo](../../references/product-audit.md)

<a id="snapshot"></a>

## Snapshot và định danh

| Trường | Bằng chứng |
| --- | --- |
| Repository | `https://github.com/cirnetlab/cirnet-interface.git` |
| Thời điểm và phạm vi | 2026-11-08 UTC, kiểm tra chỉ đọc cục bộ trên Windows |
| Nhánh local và hosted | Local `master` khớp nhánh mặc định hosted `master`. Ref cache `origin/HEAD` trỏ tới `master`. |
| Commit SHA | `f1ff70b9f22554c0680b92679e9e44a5544247ff` |
| Root tree SHA | `1f7e59f4b3e219fea6fa27b0183337c6d63a15c6` |
| Kiểm kê tracked | 609 tệp. SHA-256 của danh sách cây Git thô là `f2180cae375c887b79817ca25fc3c270ca8bf044afd9cf8677f3e68bb3bdb9e0`. Không có gitlink. |
| Cây làm việc | Sạch tại snapshot kiểm tra. |
| Tag local | v0.1.1, v0.1.2, v0.1.3, v0.1.4 |
| Tag lịch sử hosted đã chọn | `v0.1.4` trỏ tới `f1fe0c4738e105727aff1a331327caf7172cdf5f`. |
| Công bố đích | Git hosted không trả ref `v0.2.4` hoặc `v0.2.2`. Tra release công khai GitHub cho `v0.2.4` trả HTTP 404. Bản nháp riêng ngoài phạm vi quan sát. |

<a id="coverage"></a>

## Phạm vi tệp

Đã phân tích `package.json`, các mục phụ thuộc liên quan trong `package-lock.json` và hai mô tả `integration/*.json`. Xem `src/main.ts`, `src/services/api/client.ts`, `src/services/api/captures.ts`, các đoạn nguồn WebSocket/NCAP, `scripts/bootstrap-contracts.mjs`, `scripts/build-release.mjs`, `.github/workflows/frontend.yml` và tài liệu triển khai, namespace, verification Anh–Việt. Cây tracked có bộ test browser/e2e, asset, tệp mô hình nhị phân, hồ sơ migration và khai báo generated. Phần lớn chỉ kiểm kê. Không chạy bundle, browser, ứng dụng desktop/mobile, build hoặc endpoint tích hợp.

<a id="findings"></a>

## Phát hiện và tác động

| ID và trạng thái | Bằng chứng tại commit này | Tác động và xử lý |
| --- | --- | --- |
| AUD-INT-01 · VERIFIED · BLOCKER | JSON xác định `@cirnet/web` 0.2.4 với tệp SDK `file:.artifacts/cirnet-contracts-0.1.2.tgz`. `integration/compatibility.json` ghi `UNRELEASED_NATIVE_HANDOFF`, SDK đích 0.2.2 và thực tế 0.1.2. SHA-256 khóa là `b963cf4990fee8b5a9294a6aaf85cfd803ed7724efc9ad2b3ee1ec302646b3c8`. | Bản gói đi trước handoff đã ghim. Giữ đầu vào đã phát hành đã rà soát. Sinh lại SDK từ bản xuất Remote được chấp nhận và kiểm chứng build Interface sạch mới với artifact đó. |
| AUD-INT-02 · OBSERVED · HIGH | `scripts/build-release.mjs` tách artifact kiểm chứng/ứng viên khỏi đầu vào phát hành, yêu cầu nguồn commit sạch ngoài chế độ ứng viên và kiểm tính toàn vẹn SDK/metadata producer. `.github/workflows/frontend.yml` xuất asset kiểm chứng bằng Node 26. | Tên workflow hay asset chưa chứng minh đủ điều kiện phát hành. Cần biên nhận producer, SDK khớp đích, gate browser và nghiệm thu consumer Remote. Giữ Vue/TypeScript/Vite/Three.js/Pinia hiện tại. |
| AUD-INT-03 · OBSERVED · HIGH | Nguồn cấu hình HTTP mặc định same-origin `/api` và mô-đun WebSocket/NCAP riêng. Request ghi xử lý redirect bằng lỗi. Mô tả tương thích liệt kê graph 1.0, 1.1 và 1.2, Web đã triển khai, desktop/mobile đề xuất. | Giữ định danh thao tác, xác thực, đối soát reconnect không phát lại và ngữ nghĩa khả năng chưa có. Chốt xử lý dữ liệu lưu trước khi loại đường legacy. Bằng chứng trình duyệt không thay khả năng phần cứng. |

<a id="readiness"></a>

## Mức sẵn sàng và bàn giao

**BLOCKED.** Phụ thuộc SDK Contracts và handoff Remote native đã rà soát. Cập nhật đúng pin SDK và lock ở repo sản phẩm khi được phê duyệt. Kiểm tra từ chối profile cũ, không phát lại lệnh ghi, giới hạn WebSocket/NCAP và browser/authority trên cùng bundle. Giữ stack giao diện và dùng migration ngoại tuyến đã duyệt hoặc dữ liệu mới.
