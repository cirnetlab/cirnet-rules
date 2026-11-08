<!-- doc-id: STANDARDS -->
<!-- revision: 1 -->
<!-- source-sha256: 8f6c5ee5363b6f5513d45ba3de0285206e426de31dec3a639a6eebc40606305a -->
<!-- translation-status: synchronized -->

<a id="applicability"></a>

# Phạm vi tiêu chuẩn

Kiểm tra ngày 2026-11-08 trên danh mục ISO chính thức. Cả bảy edition đã công
bố. Bản thay thế 29148 ở mức [DIS](https://www.iso.org/standard/94091.html), nên
edition 2018 vẫn là bản tham chiếu đã công bố. Đã truy cập metadata và mô tả
phạm vi công khai. Mỗi ánh xạ là **tham chiếu định hướng**. Đối chiếu điều
khoản và chứng nhận ngoài phạm vi rà soát.

| Edition và nguồn chính thức | Phạm vi | Quy tắc CIRNET | Artifact hoặc phép kiểm | Vai trò sở hữu | Bằng chứng còn thiếu |
| --- | --- | --- | --- | --- | --- |
| [ISO/IEC/IEEE 12207:2026](https://www.iso.org/standard/90219.html) | Vòng đời phần mềm | REL-01 | Chuyển trạng thái, bằng chứng gate và chính sách hỗ trợ | người quản lý release | Nghiệm thu vận hành |
| [ISO/IEC/IEEE 15288:2023](https://www.iso.org/standard/81702.html) | Tích hợp hệ thống | ARCH-01 | Hồ sơ sản phẩm và tổ hợp hệ thống | kiến trúc sư | Bằng chứng board và triển khai |
| [ISO/IEC/IEEE 42010:2022](https://www.iso.org/standard/74393.html) | Mô tả kiến trúc | ARCH-01 | Vai trò, mối quan tâm các bên và hồ sơ quyết định | kiến trúc sư | Owner rà soát góc nhìn sản phẩm |
| [ISO/IEC/IEEE 29148:2018](https://www.iso.org/standard/72089.html) | Yêu cầu và truy vết | QA-01 | Công việc từng repo và liên kết bằng chứng release | người kiểm chứng | Phê duyệt từng việc sản phẩm |
| [ISO/IEC 25010:2023](https://www.iso.org/standard/78176.html) | Mô hình chất lượng sản phẩm | QA-01 | Đích chất lượng kiểm chứng được trong quy tắc | người kiểm chứng | Tải và ngưỡng mục tiêu đã duyệt |
| [ISO/IEC/IEEE 29119-2:2021](https://www.iso.org/standard/79428.html) | Quy trình kiểm thử | QA-01 | Lệnh test, định danh kết quả và giới hạn | người kiểm chứng | Lượt kiểm thử sản phẩm đầy đủ |
| [ISO 10007:2017](https://www.iso.org/standard/70400.html) | Hướng dẫn quản lý cấu hình | REL-01 | Snapshot đã duyệt, manifest bất biến và kiểm tra drift | người quản lý release | Receipt producer sạch đã duyệt |

Registry Rust kiểm tra định danh và đường truy vết. Nghiệm thu sản phẩm và
rà soát ngữ nghĩa do người thực hiện khi máy chưa kiểm được.
[Quy tắc](../README.md) · [SemVer](https://semver.org/) · [OpenAPI](https://spec.openapis.org/oas/v3.1.0.html) · [JSON Schema](https://json-schema.org/) · [Protobuf](https://protobuf.dev/)
