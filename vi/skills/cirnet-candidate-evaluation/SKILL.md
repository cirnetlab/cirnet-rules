---
name: cirnet-candidate-evaluation
description: Dùng khi lập kế hoạch, chạy, rà soát hoặc báo cáo đánh giá hành vi CIRNET theo quy trình Single Candidate.
---

# Đánh giá một Candidate CIRNET

Dùng quy trình Single Candidate Evaluation hiện có cho chuỗi Author → Work.
Dùng phép kiểm của chủ sở hữu để nghiệm thu sản phẩm.

1. **Chọn theo rủi ro.** Xác định hành vi, công cụ hoặc ranh giới tin cậy bị
   ảnh hưởng. Chọn tập ca nhỏ nhất trong danh mục để kiểm tra phần đó. Ghi lại
   các ca đã chọn và lý do. Nếu chạy lại toàn bộ E01–E08, phải có lý do riêng
   giải thích độ bao phủ cần thiết trên cả tám ca.
2. **Đóng băng đầu vào và xem trước.** Dùng benchmark runner hiện có và một
   phiên mới trong phạm vi sở hữu. Xác minh revision source, hash của Rules,
   Skills, bộ đánh giá và fixture gốc trong bản đóng băng. Bảo toàn báo cáo
   lịch sử, bằng chứng gốc E01–E08 và tài liệu lưu trữ về A/B và Adaptive
   Stable Orchestrator. Trước khi gọi native, lấy quyền riêng và giới hạn
   chính xác số lần gọi theo vai trò. Áp dụng quy trình rà soát bàn giao
   độc lập hiện có trước khi chạy Work native.
3. **Chấm theo bằng chứng.** Kiểm tra thực thi, kết quả khách quan, kiểm tra
   phạm vi, quyền riêng tư và bằng chứng đã niêm phong. Kết quả shim chỉ bao
   phủ phần bộ chạy kiểm thử đã thực thi. `OBJECTIVE_PASS_REVIEW_REQUIRED`
   nghĩa là vẫn đang chờ rà soát hành vi. Nộp bản rà soát độc lập dựa trên
   tiêu chí đã đóng băng và các artifact trong chỉ mục bằng chứng. Chỉ kết
   luận hành vi native đạt PASS khi có bằng chứng chạy native thực tế, rà
   soát độc lập và phán quyết chất lượng cuối cùng cho các tiêu chí được nêu.
4. **Báo cáo phạm vi.** Ghi các ca, revision, hash, cấu hình runtime và vị trí
   bằng chứng. Dùng `PASS` cho tiêu chí có bằng chứng đạt, `FAIL` cho lỗi
   quan sát được, `BLOCKED` khi thiếu điều kiện cần và `NOT_RUN` cho phép
   kiểm chưa chạy. Nêu kết quả nào bao phủ bộ chạy kiểm thử, phép kiểm khách
   quan hoặc hành vi native, đồng thời chỉ rõ tiêu chí chưa được kiểm chứng.

Dùng Single Candidate cho đánh giá hiện hành. Khi đã chứng minh lỗi tương
thích của runner, xin quyền riêng trước khi sửa runner.

Dùng revision áp dụng trong checkout đã được duyệt.
Quy trình Single Candidate (`vi/docs/behavioral-evaluation.md`) ·
Tương thích theo rủi ro (`vi/docs/model-compatibility.md`)

<!-- source: en/skills/cirnet-candidate-evaluation/SKILL.md -->
<!-- source-sha256: c6091feebfdcc4e4a89a10f67f31f822b3111713dd2561999990c3fdb10c39f2 -->
