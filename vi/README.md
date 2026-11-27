# CIRNET Common Rules & Agent Skills Toolkit

Bắt đầu với [baseline độc lập](AGENTS.md) và chỉ dẫn áp dụng của repo đích.
Dùng tiếng Anh cho Agent và tiếng Việt để rà soát. Đọc source của bên sở hữu,
rule và [Skill](skills/README.md) liên quan đến tác vụ.

| Nhu cầu | Đọc |
| --- | --- |
| Chọn context cho tác vụ | [Nạp context](docs/context.md) |
| Chọn kiến trúc, tìm chủ sở hữu và rà soát ưu tiên Lab-First | [Chính sách kiến trúc](policies/architecture.md) · [Định tuyến repository](references/architecture-routing.md) |
| Triển khai, gỡ lỗi, refactor và kiểm chứng trong phạm vi | [Skills thực thi](skills/README.md) · [Kiểm thử](rules/testing.md) |
| Kiểm tra quyền và ràng buộc cụ thể | [Quyền](policies/permissions.md) · [Coding](rules/coding.md) · [Bảo mật](rules/security.md) · [Liên repo](rules/cross-repo.md) |
| So phiên bản, contract và artifact được chọn | [Bằng chứng tương thích](docs/compatibility.md) |
| Xác định ownership file và kiểm tra di chuyển | [Đề xuất tiêu chuẩn tổ chức](rules/repository-organization.md) · [Mẫu bản đồ repository](templates/repository-map.md) |
| Thiết lập client, đóng gói hoặc restore bundle local | [Thiết lập](docs/setup.md) · [Phân phối](docs/distribution.md) |
| Bảo trì tài liệu song ngữ và chạy checker | [Policy tài liệu](policies/documentation.md) · [Kiểm tra](hooks/README.md) |
| Soạn chỉ dẫn local hoặc ghi hồ sơ task | [AGENTS local](templates/AGENTS.md) · [Plan](plans/README.md) · [Spec](specs/README.md) |

Thay đổi candidate nằm trong [changelog](../CHANGELOG.md).
Bằng chứng sản phẩm có ngày trong [reference](references/README.md).
Xem [kiểm chứng toolkit](docs/toolkit-verification.md) để biết kết quả chạy
và [kiểm chứng song ngữ trước đây](docs/bilingual-verification.md) về lịch sử.
Dùng [kiểm tra tương thích tập trung](docs/model-compatibility.md) và
[benchmark Candidate một phiên](docs/behavioral-evaluation.md) khi thay đổi model
hoặc ranh giới công cụ. [Kho lưu trữ benchmark](../reports/work-prompt-benchmark-history.md)
ghi lại các lượt chạy lịch sử.

[Giấy phép](../LICENSE) · [Thông báo bên thứ ba](../THIRD_PARTY_NOTICES.md)

<!-- source: en/README.md -->
<!-- source-sha256: 7d3da4eee82a7d61b086906e665d90b2ecbfeb92606dab09a703bb70094d73a3 -->
