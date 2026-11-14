# Kế hoạch triển khai rebuild Common Rules

Ngày 2026-11-14. Chỉ là hồ sơ chuyển đổi; không phải quy trình task tái sử dụng.

**Mục tiêu:** Repo common rules nhỏ, di động, không có dependency runtime.
**Kiến trúc:** Một baseline độc lập, topic guide và skill tùy chọn,
thiết lập thủ công, tham khảo sản phẩm có ngày.
**Công cụ:** Markdown, PowerShell/Git hiện có và validator local sẵn dùng.
**Đặc tả:** [Đặc tả rebuild](../../../specs/cirnet-rules/rules-common/spec.md).

## Ràng buộc và trọng tâm review

Chỉ ghi worktree `rules-common` được phép; giữ thay đổi chưa stage.
Không cài tool, chạy lệnh runtime cũ, sửa repo khác hoặc settings global.
Kiểm tra copy độc lập, preferences hiện có, chỉ dẫn FPGA lồng nhau,
candidate so với release được chọn và các tuyên bố an toàn vật lý.

## Các bước

- [x] Xác minh worktree, branch và backup; ghi status/refs ban đầu.
- [x] Đọc sáu sản phẩm local ở chế độ chỉ đọc và ghi bằng chứng nguồn.
- [x] Thay chỉ dẫn root, thêm bốn topic guide, ba skill và template local.
- [x] Viết setup/map nguồn; giữ license và attribution phù hợp.
- [x] Chỉ xóa file legacy tracked đã có trong commit backup được xác minh.
- [x] Kiểm tra mọi file còn lại, tình huống trên giấy và review độc lập.
- [x] Đối chiếu refs/status sản phẩm, ghi kết quả cho người dùng review.

## Baseline và quyết định

Ban đầu: worktree sạch, 290 file tracked, không có untracked/ignored được báo.
HEAD, `master`, `governance-cli-only`: `646afa38516bbfd276b0aecb7a138ab427df5fa9`.
Kiểm tra baseline: adapter/setup/map/skills mới chưa tồn tại; bootstrap root
phụ thuộc registry governance và mọi task cần grant trung tâm. Điều này
không đáp ứng yêu cầu hoạt động độc lập và nhẹ.

Quyết định: dùng worktree đã tách và thiết kế được cung cấp; không tạo worktree
mới, vòng xin phép hoặc commit. Phạm vi và yêu cầu thực thi trực tiếp của người dùng
là căn cứ.
Quyết định: giữ hồ sơ chuyển đổi theo cặp trong các thư mục task trung tâm hiện có;
đó là bằng chứng lịch sử, không thuộc chỉ dẫn global.
Quyết định: kiểm tra tĩnh và tình huống trên giấy phù hợp đầu ra chỉ có Markdown;
thực thi native client vẫn là kiểm tra riêng, được ghi rõ chưa chạy.

[Nguồn tiếng Anh](../../../../en/plans/cirnet-rules/rules-common/plan.md)

<!-- source: en/plans/cirnet-rules/rules-common/plan.md -->
<!-- source-sha256: b3b19c82c0c2187929c33fb22b3aa8e2ad839da533701d2fc77ce44d6290c7bc -->
