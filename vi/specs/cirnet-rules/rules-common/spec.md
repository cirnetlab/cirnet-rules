# Đặc tả chuyển đổi Common Rules

Ngày 2026-11-14. Bản đối chiếu của
[đặc tả tiếng Anh](../../../../en/specs/cirnet-rules/rules-common/spec.md),
chỉ ghi nhận lần chuyển đổi này, không áp dụng cho task sản phẩm.

Người dùng yêu cầu triển khai trực tiếp bộ hướng dẫn Markdown trong worktree
`rules-common`. Sáu repository sản phẩm chỉ được đọc; không build/test, cài tool,
ghi file hay thay đổi Git tại đó. Không stage, commit, push, release hoặc cài global.

Thiết kế gồm `AGENTS.md` tiếng Anh độc lập, `CLAUDE.md` import một dòng,
bốn hướng dẫn bổ sung, ba skill tùy chọn, mẫu local, hướng dẫn dùng và bản đồ
nguồn đã kiểm tra. Mỗi sản phẩm giữ ownership, quy tắc local, build/test và release.
Không áp đặt version, toolchain, schema, board hay quy trình plan của sản phẩm
thành quy tắc chung. Bản đồ phân biệt source, candidate, release và UNAVAILABLE.

Giữ LICENSE, .gitattributes, .gitignore và attribution còn áp dụng. Chỉ xóa legacy
tracked sau khi xác minh nhánh backup. Bỏ runtime Rust, registry, migrations,
CI cũ và tài liệu không còn dùng; không thay bằng nền tảng mới.
Chỉ giữ bản plan/spec Anh–Việt của lần chuyển đổi, không duy trì hai cây rules.

Nghiệm thu bằng kiểm tra UTF-8, liên kết Markdown, metadata skill, adapter,
diff, ba tình huống đọc hướng dẫn và đối chiếu Git trước–sau. Không coi tài liệu
hay RTL test là bằng chứng native client E2E hoặc an toàn điện.
Các native loader/E2E chưa chạy phải ghi `NOT_RUN`.
