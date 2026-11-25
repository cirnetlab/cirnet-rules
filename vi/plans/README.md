# Kế hoạch

Plan mô tả cách thực hiện một thay đổi có phạm vi. Với task common-rules không
đơn giản, lưu plan gọn tại `en/plans/cirnet-rules/<task>/plan.md` và bản Việt
cùng đường dẫn dưới vi. Đặt đặc tả ở cây specs tương ứng.

Dùng [mẫu plan](../templates/plan.md) khi hữu ích. Ghi phạm vi, thứ tự bước,
kiểm tra có ý nghĩa và tiến độ; liên kết spec thay vì lặp nội dung.
Sau triển khai/build, cập nhật bước hoàn thành và kết quả quan sát ở cả hai
ngôn ngữ. Không đánh dấu xong chỉ theo dự định hoặc một test không liên quan pass.

Typo không cần plan mới, trừ yêu cầu local. Task sản phẩm dùng vị trí plan
của owner, kể cả quy ước `.superpowers` hiện có; thư mục trung tâm này
không thay thế quy tắc đó.

Hồ sơ task: [rebuild ban đầu](cirnet-rules/rules-common/plan.md) (lịch sử),
[nâng cấp song ngữ](cirnet-rules/bilingual-upgrade/plan.md),
[nâng cấp toolkit](cirnet-rules/toolkit-upgrade/plan.md),
[hành vi kỹ thuật](cirnet-rules/engineering-behavior/plan.md),
[tích hợp Adaptive Stable](cirnet-rules/adaptive-stable/plan.md) (lịch sử),
[Task Continuity](cirnet-rules/task-continuity/plan.md),
[Bằng chứng độ tin cậy](cirnet-rules/reliability-evidence/plan.md),
[Kỹ thuật CAD](cirnet-rules/cad-engineering/plan.md).

<!-- source: en/plans/README.md -->
<!-- source-sha256: 04ba888b412018786c338ca2cfb74cbd64564b434b5fe20033071004b57829e2 -->
