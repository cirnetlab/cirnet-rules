# Đặc tả rebuild Common Rules

Ngày 2026-11-14. Hồ sơ chuyển đổi một lần, không phải chỉ dẫn cho task sản phẩm.

## Mục đích và quyền thực hiện

Thay ứng dụng Governance bằng hướng dẫn Agent common di động trong worktree
`rules-common` hiện có. Prompt build local của người dùng cung cấp thiết kế và
cho phép triển khai trực tiếp. Sáu repo sản phẩm là bằng chứng chỉ đọc; chỉ dẫn
của chúng không mở rộng quyền ghi của task này.
Không commit, stage, push, release, cài global hoặc sửa sản phẩm.

## Thiết kế

Root `AGENTS.md` là baseline common tiếng Anh duy nhất, dùng được khi copy riêng.
`CLAUDE.md` chỉ chứa `@AGENTS.md`. Bốn topic guide bổ sung chi tiết quyết định;
ba skill độc lập, tùy chọn bao gồm implementation, debugging, verification.
README/setup tiếng Việt giải thích dùng thủ công. Bản đồ sáu repo là ảnh chụp
nguồn có ngày, không thay authority sản phẩm hoặc chứng minh hành vi đã triển khai.

Giữ `LICENSE`, `.gitattributes` và các bảo vệ `.gitignore` hiện có. Review thông
báo bên thứ ba và bỏ tuyên bố dependency khi mã liên quan được loại bỏ.
Chỉ xóa ứng dụng Rust tracked, manifests, migrations, registry governance,
CI legacy, assets và nội dung Anh/Việt lỗi thời sau khi xác nhận nhánh backup
chứa cây tracked hiện tại. Giữ dữ liệu người dùng và Git.
Chỉ giữ hồ sơ plan/spec trung tâm song ngữ của lần chuyển đổi này, không có hai cây rules.

Chọn phương án Markdown-first đã cung cấp. Giữ runtime cũ hoặc viết installer
thay thế sẽ thêm dependency trái yêu cầu.
Không thêm installer, test framework, database task hoặc điều phối.

## Nghiệm thu

- Xác minh đúng worktree, branch, refs backup và trạng thái ban đầu sạch.
- Đọc docs, manifests, source và test thực tế trong sáu repo local.
- Giữ ownership, plan/spec local và chính sách test output của sản phẩm.
- Dùng nguồn chính thức để mô tả scope client; giữ preferences cá nhân.
- Kiểm tra UTF-8, link Markdown, metadata skill, đích adapter và whitespace.
- Đọc thử tình huống typo, ownership API và an toàn FPGA.
- So sánh status/HEAD sản phẩm và refs nhánh được bảo vệ trước–sau.
- Báo native loader/E2E là `NOT_RUN` trừ khi đã quan sát riêng.

Nội dung common không áp version, toolchain, release gate, schema, số channel,
board hoặc quy ước lập kế hoạch của sản phẩm lên repo khác.
Không chạy build/test sản phẩm. Release và thao tác vật lý vẫn chưa được cho phép.

[Plan](../../../plans/cirnet-rules/rules-common/plan.md) ·
[Nguồn tiếng Anh](../../../../en/specs/cirnet-rules/rules-common/spec.md)

<!-- source: en/specs/cirnet-rules/rules-common/spec.md -->
<!-- source-sha256: 7172c0ebaad872b52b208e889320f8a8b6cc3307390465507bac424b345c5115 -->
