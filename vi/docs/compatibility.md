# Bằng chứng tương thích

Chọn miền trước khi đề xuất phiên bản. Toolkit, package/library, API công khai,
schema, protocol, SDK sinh tự động, firmware/board profile và system release
selection có owner và phiên bản riêng. Đọc selector thật của consumer và nguồn
chuẩn producer. Một số phép kiểm tra cần cả hai chiều writer/reader hoặc sender/receiver.

| Ranh giới | Đọc skill | Xem xét |
| --- | --- | --- |
| HTTP client thấy hoặc SDK công khai | [Tương thích API](../skills/cirnet-api-compatibility/SKILL.md) | Request, response, default, lỗi, auth và hành vi SDK |
| Dữ liệu chuẩn, wire format hoặc ABI | [Thay đổi contract](../skills/cirnet-contract-change/SKILL.md) | Encoding, định danh, độ dài, reader cũ, binding sinh tự động |
| Artifact producer và pin consumer | [Tương thích liên repo](../skills/cirnet-cross-repo-compatibility/SKILL.md) | Revision chính xác, input được chọn và test lane cần thiết |
| Đề xuất version và migration người dùng | [Release và changelog](../skills/cirnet-release-changelog/SKILL.md) | Policy version của owner, tác động breaking và giới hạn khôi phục |

Phân loại riêng source, wire, dữ liệu lưu và hành vi. Ghi bằng chứng trong
[ma trận](../templates/compatibility-matrix.md) tại vị trí owner cho phép.
Field mới tùy chọn vẫn có thể làm validator nghiêm ngặt lỗi. Ghi migration dữ liệu
hoặc bước không thể đảo ngược trước khi đề nghị rollback. Trạng thái release đến
từ system owner và artifact đang được chọn.

<!-- source: en/docs/compatibility.md -->
<!-- source-sha256: 92bc746056d8e14fb9c8e6024043413340025d36376c1b017a0316291cce12b0 -->
