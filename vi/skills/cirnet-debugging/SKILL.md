---
name: cirnet-debugging
description: Dùng khi lỗi, regression hoặc kết quả chập chờn cần xác định nguyên nhân trước khi sửa.
---

# Chẩn đoán lỗi

Dùng khi triệu chứng chưa rõ nguyên nhân. Dùng implementation khi thay đổi và nguyên nhân đã rõ.

Tối thiểu cần: Kết quả mong đợi và thực tế, revision, input gây lỗi, môi trường và thông báo lỗi đã che dữ liệu nhạy cảm.

1. Tái hiện theo quy tắc test local và so sánh với ca chạy được.
2. Tìm ranh giới lỗi đầu tiên, lập một giả thuyết kiểm chứng được và chạy thử nghiệm nhỏ nhất giúp phân biệt nguyên nhân.
3. Sửa hẹp sau khi có bằng chứng. Giữ cleanup, cancellation và tương thích, rồi chạy lại cách tái hiện và kiểm tra liên quan.

Báo bằng chứng, nguyên nhân đã hoặc chưa xác nhận, bản sửa và kết quả chạy lại. Nêu điều cản tái hiện nếu bị chặn.

Ví dụ: Yêu cầu Stop timeout sau khi hủy. Theo dõi xác nhận cleanup trước khi đổi timeout. Một lần retry pass chưa xác minh được nguyên nhân.

<!-- source: en/skills/cirnet-debugging/SKILL.md -->
<!-- source-sha256: 6707357b63e90b5c1a009e2ada5fdc93d3bef5ae4e41a8c94907cc6ff44c2073 -->
