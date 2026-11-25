---
name: cirnet-repository-structure
description: Dùng khi rà soát cách tổ chức repository hoặc module, quyền sở hữu tệp, đầu ra sinh tự động hoặc việc di chuyển tệp có phạm vi.
---

# Cấu trúc repository

Dùng để xác định hoặc chỉnh ranh giới thực tế. Dùng refactoring khi chỉ đổi
cấu trúc code. Giữ layout theo ngôn ngữ nếu đang phù hợp.
Chọn AUDIT khi được yêu cầu rà soát hoặc khi phạm vi ghi chưa rõ.
Chọn MODIFY khi thay đổi được yêu cầu đã có quyền rõ ràng.

Dùng chỉ dẫn hiện hành của chủ sở hữu và kiến trúc áp dụng. Tham khảo đề xuất
tại `en/rules/repository-organization.md` và mẫu bản đồ tùy chọn tại
`en/templates/repository-map.md` khi hữu ích. Đọc các tài liệu này từ checkout
`cirnet-rules` đã duyệt. Báo tài liệu nào không truy cập được và tiếp tục với
bằng chứng của chủ sở hữu trong phạm vi.

## AUDIT

Chỉ đọc khi rà soát. Xác định repository, miền, tính năng và module chịu trách
nhiệm cho các đường dẫn, đầu ra và bằng chứng liên quan. Phân biệt kiến trúc
đích với thẩm quyền hiện tại. Tìm đường dẫn, symbol, tài liệu và test được phép
đọc, rồi xem nơi gọi trực tiếp trước khi đề xuất dùng lại hoặc bổ sung. Báo
phát hiện trong câu trả lời, kèm đường dẫn và revision làm bằng chứng, chủ
sở hữu đã xác nhận hoặc còn bỏ ngỏ, phần trùng lặp hoặc rủi ro phụ thuộc,
và hành động đề xuất. Nêu bằng chứng hoặc quyết định cần có cho từng chỗ
còn thiếu. Ghi đề xuất dọn dẹp, xóa hoặc tái cấu trúc trong phần phát hiện.

## MODIFY

Chỉ triển khai khi có quyền rõ ràng cho các đường dẫn và thao tác liên quan.
Nếu còn vướng quyền sở hữu hoặc quyền thao tác, nêu điều kiện còn thiếu và
tạm dừng phần thay đổi phụ thuộc vào đó. Trước khi thêm đường dẫn, kiểm tra
chủ sở hữu và tìm phần triển khai hiện có theo AUDIT. Rà soát migration và lần theo nơi
gọi, import, export, tệp build, script, test, CI, đóng gói, tài liệu và bên
dùng phân biệt chữ hoa/thường. Áp dụng thay đổi nhỏ nhất được phép và cập
nhật tham chiếu cùng lúc. Giữ chỉnh sửa của người dùng, dữ liệu chưa theo
dõi, nguồn của bộ sinh, fixture, mẫu, giấy phép, phiên bản tương thích và
bằng chứng lịch sử cần bảo vệ. Giữ kế hoạch và đặc tả ở vị trí của repo sở hữu.

Chạy phép kiểm tập trung cho phụ thuộc, điểm vào, test và vị trí đầu ra.
Theo quy trình đồng bộ tài liệu khi sửa nội dung song ngữ. Báo đường dẫn đã
sửa, artifact được bảo vệ, bước migration, kết quả kiểm chứng và giới hạn.

Ví dụ: Chỉ di chuyển nguồn của binding sinh tự động sau khi tìm generator và mọi consumer. Sinh lại output bằng lệnh của owner.

<!-- source: en/skills/cirnet-repository-structure/SKILL.md -->
<!-- source-sha256: 8691b4c5c99faf90c92238f6ac384425aa9820478a382b95789921ecf742de0e -->
