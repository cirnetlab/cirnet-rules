# Kế hoạch khôi phục E01/E08

Phạm vi và nghiệm thu: [đặc tả](../../../specs/cirnet-rules/e01-e08-recovery/spec.md).

- [x] Xác nhận `rules-common` tại `b8981de` và lưu hash đầu vào/bằng chứng lịch sử.
- [x] Thêm hướng dẫn quyền export có điều kiện và rà soát các cặp EN/VI.
- [x] Chẩn đoán Icarus bằng profile/toolchain của runner và probe MXC không dùng model.
- [x] Đóng băng byte candidate và đánh giá E01/E08. Giữ cả hai lượt lỗi đầu; E01 đạt sau hướng dẫn đọc source cụ thể. Một lần thử lại E08 có căn cứ sau bản sửa standard handles đã chứng minh đạt mọi marker an toàn và đúng phạm vi.
- [x] Hoàn tất kiểm tra toolkit tập trung và [report follow-up](../../../docs/e01-e08-followup.md) song ngữ.
- [ ] Đạt mọi tiêu chí nghiệm thu E08 native: còn thiếu physical NOT_RUN rõ ràng và exit simulator của lượt xanh chưa được ghi riêng. Giữ behavioral FAIL cuối; không retry model thêm.

E02–E07, toàn bộ behavioral suite, cấu hình Agent thật, repo sản phẩm và thay đổi
Git của toolkit nằm ngoài follow-up này. Giữ lại cả các probe không thành công.

<!-- source: en/plans/cirnet-rules/e01-e08-recovery/plan.md -->
<!-- source-sha256: 08daaab7ef64927033f1c9e8ff52227ceec265cdd51946fcfcc9bd6015657fd1 -->
