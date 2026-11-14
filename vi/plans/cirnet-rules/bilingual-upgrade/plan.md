# Kế hoạch nâng cấp tài liệu song ngữ

Ngày 2026-11-14.
Đặc tả: [nâng cấp song ngữ](../../../specs/cirnet-rules/bilingual-upgrade/spec.md).

- [x] Xác nhận đúng worktree, baseline sạch và refs được bảo vệ.
- [x] Chọn tiếng Anh chuẩn, bản dịch Việt cho người đọc và hook local tùy chọn.
- [x] Chuyển nội dung sang các cặp en/vi, giữ nguyên bằng chứng nguồn.
- [x] Thêm hướng dẫn hooks/policies/plan/spec và template tái sử dụng.
- [x] Yêu cầu cập nhật cặp tài liệu trong baseline và skill triển khai/kiểm chứng.
- [x] Viết checker chỉ đọc cùng regression test tình huống lỗi trước triển khai.
- [x] Review bản dịch rồi ghi hash chính xác của nguồn tiếng Anh.
- [x] Chạy hook, unit test, kiểm tra skill, link/whitespace và review độc lập.
- [x] Ghi kết quả hiện tại tách biệt với bằng chứng chuyển đổi lịch sử.

Quyết định: yêu cầu mới thay bố cục một ngôn ngữ trước đó.
Giữ bản phân phối root tự đủ nghĩa để dùng global không phụ thuộc liên kết.
Không kích hoạt hook native, sửa global, sản phẩm, stage, commit hay release.

<!-- source: en/plans/cirnet-rules/bilingual-upgrade/plan.md -->
<!-- source-sha256: b19af8bf2555bcbce7aa133d6a816a39f5896cd94cb07315610b748e6a1b58c9 -->
