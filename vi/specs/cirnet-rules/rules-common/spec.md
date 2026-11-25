# Đặc tả xây dựng lại Common Rules

Hồ sơ chuyển đổi ngày 2026-11-14. Thay Governance bằng hướng dẫn Markdown dùng
độc lập trong worktree `rules-common` hiện có. Đọc sáu repo sản phẩm để lấy bằng
chứng, theo quyền sở hữu, kế hoạch và chính sách kiểm thử của từng repo.

- Cung cấp `AGENTS.md` tiếng Anh độc lập, `CLAUDE.md` chứa `@AGENTS.md`, bốn
  hướng dẫn theo chủ đề, ba skill tùy chọn, thiết lập thủ công và bản đồ nguồn
  có ngày. Bản đồ ghi nhận nguồn tại một thời điểm. Mỗi sản phẩm quản lý hợp đồng
  và bản phát hành của mình.
- Xác minh refs sao lưu trước khi xóa ứng dụng, registry, CI, tài nguyên và tài
  liệu cũ được Git theo dõi. Bảo vệ dữ liệu người dùng, Git, giấy phép, thuộc tính,
  quy tắc ignore và ghi công liên quan. Dùng Markdown không cần công cụ runtime.
- Đọc tài liệu, manifest, mã nguồn và test sản phẩm. Kiểm tra nhánh và trạng thái
  repo trước và sau, phạm vi client theo nguồn chính thức, tùy chọn người dùng,
  UTF-8, liên kết, metadata skill, adapter và khoảng trắng. Rà soát tình huống lỗi
  đánh máy, API và FPGA. Ghi riêng kết quả bộ nạp native và E2E.

Phạm vi gồm chuyển đổi Markdown và kiểm tra tĩnh tại repo. Sửa, build hoặc test
sản phẩm, stage Git hay xuất bản, cài đặt và thao tác vật lý nằm ngoài phạm vi.

[Kế hoạch](../../../plans/cirnet-rules/rules-common/plan.md) ·
[Nguồn tiếng Anh](../../../../en/specs/cirnet-rules/rules-common/spec.md)

<!-- source: en/specs/cirnet-rules/rules-common/spec.md -->
<!-- source-sha256: b6bb7be5336e3798e0d1b0d9840ac52bdcdf11c4a20422b6088636d515d9442f -->
