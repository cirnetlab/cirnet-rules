# Nâng cấp hành vi kỹ thuật

Bắt đầu ngày 2026-11-15 tại `a24c94aeb43d95dbf05f8d53f8676676dccba2e2`.

- Bổ sung nguyên tắc làm rõ, đơn giản, sửa đúng phạm vi và kiểm chứng kết quả
  vào baseline, quy tắc lập trình và sáu skill. Đóng gói bốn ví dụ Before/After
  giả lập tự viết trong skill triển khai. Giữ 12 ID skill, tài nguyên dùng
  độc lập, EN/VI và yêu cầu an toàn.
- Đánh giá tám ca giả lập theo rubric và phạm vi ghi của từng ca. Chốt đầu
  vào, dùng fixture mới và profile cách ly. Dùng cùng thiết lập khi đối
  chiếu các lượt chạy. Khi chỉ chạy lại một số ca trong candidate, dùng
  Attempt 2 và chỉ rõ ID ca. Ghi riêng bằng chứng native và kiểm tra offline,
  đồng thời ghi kích thước ngữ cảnh.
- **E01.** Nếu quyền chưa rõ, xác nhận ai được xuất những bản ghi và trường
  nào. Quyền sửa mã không cấp quyền xuất dữ liệu. Chỉ đọc mã nguồn liên quan,
  lọc giá trị nhạy cảm trước khi in ra và giữ nguyên fixture.
- **E08.** Xác minh bộ công cụ portable trong MXC với workspace-write và
  approval never. Dùng TEMP/TMP/TMPDIR riêng và pipe cho I/O của tiến trình con.
  Chỉ mô phỏng sau khi biên dịch thành công. Giữ nguyên DUT, assertion và thời hạn.
  Bằng chứng native cần mã thoát thực tế của cả hai bước, cả bốn assertion
  an toàn và dòng `physical verification: NOT_RUN`.

Chỉ ghi vào `rules-common` và output kiểm thử riêng. Bảo vệ bằng chứng đã
lưu, repo sản phẩm, cấu hình client thật và Git refs.

[Kế hoạch](../../../plans/cirnet-rules/engineering-behavior/plan.md) ·
[Báo cáo nghiệm thu](../../../docs/e01-e08-followup.md)

<!-- source: en/specs/cirnet-rules/engineering-behavior/spec.md -->
<!-- source-sha256: 479f04cc217611332bc77f1bc005350e244c865b1792e244d229168c3f657a51 -->
