# CIRNET Common Agent Rules & Skills

## Bắt đầu

1. Dùng [baseline độc lập](AGENTS.md); `AGENTS.md` ở root là bản phân phối tiếng Anh
   giống hệt nguồn. Adapter Claude ở root import bản phân phối đó.
2. Theo [thiết lập thủ công](docs/setup.md) để review và cài chỉ dẫn tiếng Anh hoặc
   yêu cầu đọc checkout. Giữ preferences riêng và chỉ dẫn của sản phẩm.
3. Sau triển khai/build, cập nhật cả hai bản tài liệu bị ảnh hưởng theo
   [chính sách tài liệu](policies/documentation.md), rồi chạy kiểm tra local.

## Chỉ chọn context liên quan

| Thư mục | Mục đích |
| --- | --- |
| [hooks](hooks/README.md) | Kiểm tra trước task, sau sửa đổi và trước hoàn tất; checker local tùy chọn |
| [plans](plans/README.md) | Kế hoạch thực hiện gọn và hồ sơ task trung tâm |
| [specs](specs/README.md) | Hành vi mong muốn, ranh giới và nghiệm thu |
| [chính sách tài liệu](policies/documentation.md) | Tiếng Anh chuẩn và cập nhật theo cặp |
| [chính sách quyền](policies/permissions.md) | Hành động được phép và ranh giới repository |
| [coding](rules/coding.md), [security](rules/security.md), [testing](rules/testing.md), [cross-repo](rules/cross-repo.md) | Hướng dẫn triển khai theo chủ đề |
| [implementation](skills/implementation/SKILL.md), [debugging](skills/debugging/SKILL.md), [verification](skills/verification/SKILL.md) | Bản dịch cho người đọc của ba skill tiếng Anh tùy chọn |
| [references](references/README.md) | Bằng chứng nguồn có ngày; không thay quyền quyết định của sản phẩm |
| [chỉ dẫn local](templates/AGENTS.md), [plan](templates/plan.md), [spec](templates/spec.md) | Mẫu để điều chỉnh, không ghi đè mù quáng |

Lệnh build/test, version, contracts, release được chọn, cấu hình board và quy ước
plan local vẫn thuộc repo sản phẩm. Repo này không có dịch vụ điều phối, database
task, CLI bắt buộc hay approval engine.
Các chỉ dẫn này là tài liệu Markdown độc lập. Checker tùy chọn chỉ đọc, chỉ dùng
thư viện chuẩn Rust và cần bộ công cụ Rust đã cài để biên dịch.

## Bảo trì và bằng chứng

Viết nguồn Anh, dịch Việt, soát nghĩa, cập nhật metadata bản dịch, rồi kiểm tra cả hai.
Không nạp cả hai cây ngôn ngữ làm context mặc định cho Agent.
Xem [nghiệm thu song ngữ hiện tại](docs/bilingual-verification.md) và
[bằng chứng rebuild lịch sử](docs/rebuild-verification.md).
[Plan nâng cấp song ngữ](plans/cirnet-rules/bilingual-upgrade/plan.md) và
[spec](specs/cirnet-rules/bilingual-upgrade/spec.md) ghi lại lần thay đổi này.

Ứng dụng Governance cũ được giữ ở
[governance-cli-only](https://github.com/cirnetlab/cirnet-rules/tree/governance-cli-only),
không thuộc runtime của bộ common rules.
[Giấy phép](../LICENSE) · [Thông báo bên thứ ba](../THIRD_PARTY_NOTICES.md).

<!-- source: en/README.md -->
<!-- source-sha256: b71e125be97fa6937ac8a6df59146db69f9e16ecd31fd6f254598ee61efb0990 -->
