# CIRNET Common Agent Rules & Skills

Bộ hướng dẫn chung, gọn và độc lập với sản phẩm. Chọn **Codex hoặc Claude Code
hoặc Antigravity** để làm việc tại một repository. Không cần mở các agent khác,
đồng bộ task hay chạy bộ điều phối trung gian.

**[AGENTS.md](AGENTS.md) là nguồn quy tắc chung duy nhất**, bằng tiếng Anh và
dùng được khi copy riêng lẻ. Mỗi repository vẫn sở hữu kiến trúc, contracts,
build/test, tài liệu, môi trường, CI và quyết định release của mình.

## Dùng nhanh

1. Đọc [hướng dẫn thiết lập](docs/setup.md), chọn đúng client/scope.
2. Xem trước và backup cấu hình hiện có, rồi tự hợp nhất bản common vào user scope;
   hoặc yêu cầu agent đọc file common từ checkout local được phép truy cập.
3. Mở repository cần làm việc và giữ nguyên chỉ dẫn local. Chỉ đọc thêm tài liệu
   liên quan tới thay đổi. Không cần mạng để tải lại rules cho mỗi task.

Repo này không tự được nạp khi agent đang mở một repo khác, kể cả khi có trên
GitHub. `skills/` cũng không tự xuất hiện trong mọi workspace. Phiên rebuild này
chỉ tạo tài liệu, **không cài global, không sửa repository sản phẩm**.

## Nội dung

| File/thư mục | Khi dùng |
| --- | --- |
| [AGENTS.md](AGENTS.md) | Baseline độc lập: scope, ownership, bảo vệ dữ liệu, kiểm chứng |
| [CLAUDE.md](CLAUDE.md) | Adapter `@AGENTS.md` cho Claude khi mở checkout này |
| [security](rules/security.md), [coding](rules/coding.md), [testing](rules/testing.md), [cross-repo](rules/cross-repo.md) | Hướng dẫn bổ sung theo tình huống |
| [implementation](skills/implementation/SKILL.md), [debugging](skills/debugging/SKILL.md), [verification](skills/verification/SKILL.md) | Ba workflow tùy chọn, đọc/cài khi cần |
| [templates/AGENTS.md](templates/AGENTS.md) | Mẫu điền quy tắc local; không ghi đè file của repo |
| [repository-map](references/repository-map.md) | Bản đồ sáu nguồn đã đọc, có SHA và giới hạn bằng chứng |
| [verification](docs/verification.md) | Kết quả nghiệm thu lần chuyển đổi |

Common chỉ định cách làm việc. Framework, toolchain/version, schema/ABI, board,
release pin, CI gate và thư mục plan/spec do repo đích quy định.
Quy trình `.superpowers` của sản phẩm vẫn có hiệu lực tại sản phẩm đó.

## Bảo trì

Sửa baseline tại root; topic guide không sao chép toàn bộ baseline. Chỉ thêm skill
khi có workflow tái sử dụng rõ ràng. Kiểm tra UTF-8, liên kết nội bộ, metadata skill,
`git diff --check` và ba tình huống trong tài liệu verification khi đổi hướng dẫn.
Không cần compiler, package manager hay framework kiểm tra để đọc bộ rules.

Lần tái cấu trúc ngày **2026-11-14** thay ứng dụng Governance bằng Markdown.
Rust/CNR, MCP server, database, task registry/grants/leases, CLI/TUI và CI cũ
không còn là tính năng của nhánh này. Bản cũ được giữ ở
[governance-cli-only](https://github.com/cirnetlab/cirnet-rules/tree/governance-cli-only).
Quản lý runtime/SDK/toolchain là ngoài phạm vi.

[Plan chuyển đổi](en/plans/cirnet-rules/rules-common/plan.md) và
[spec](en/specs/cirnet-rules/rules-common/spec.md) có bản đối chiếu tiếng Việt;
đó là hồ sơ một lần, không phải cây rules song ngữ hoặc bootstrap bắt buộc.

Giấy phép được giữ nguyên trong [LICENSE](LICENSE).
[Thông báo bên thứ ba](THIRD_PARTY_NOTICES.md) phân biệt phần hiện tại với runtime cũ.

English: Portable common instructions and three optional skills. Product
repositories retain their own authority; no governance runtime is required.
