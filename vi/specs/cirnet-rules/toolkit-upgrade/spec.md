# Nâng cấp Common Rules Toolkit

Baseline ngày 2026-11-14 là `35e677b1264958839a7e0cb883c4477eb24da6df`.

- Cung cấp baseline EN độc lập, ngữ cảnh theo nhu cầu, 12 skill có namespace
  và bản VI, mẫu tương thích/repo, gói offline có phiên bản. Giữ kiểm tra Rust
  cho cặp ngôn ngữ, liên kết và điểm vào.
- PowerShell 7 mặc định xem trước ở phạm vi người dùng. Chỉ áp dụng gói local
  đã duyệt khi được chỉ định, giữ nguyên byte cá nhân, sao lưu đường dẫn bị
  ảnh hưởng và kiểm tra xung đột khi khôi phục.
- Thử lỗi trong home giả và gói đã giải nén. Rà soát cách nạp ngữ cảnh, đo
  văn bản, chạy test Rust và CI toolkit. Ghi riêng kết quả nạp native.

Chỉ ghi vào `rules-common`. Thao tác trên sản phẩm, phát hành, cài client thật
và thay đổi Git nằm ngoài phạm vi.

## Rà soát Markdown dư thừa

Mở rộng checker Rust chỉ đọc, chạy offline, dựa trên tập Markdown đang quét.
Phân biệt trùng byte, trùng sau chuẩn hóa thận trọng và tương đồng có điểm số
trong nhóm tài liệu cùng vai trò và ngôn ngữ. Gợi ý nguồn chuẩn, đánh dấu nội
dung riêng và liên kết trỏ tới để rà soát. Bảo vệ nội dung lặp có chủ đích và
bằng chứng. Rà soát tệp mới được chỉ định, không cần Git hay mạng.

Dùng rules và skills hiện có để tìm, cập nhật, hợp nhất và kiểm chứng.
Nghiệm thu khi các test tập trung của checker, tài liệu EN/VI và checkout đều đạt.
Các ca đánh giá hành vi và bằng chứng của chúng nằm ngoài phạm vi kiểm chứng này.

Đợt hoàn thiện tại `66d2c8a` thêm lời nhắc tự đủ trong baseline và yêu cầu
rà soát từng đường dẫn Markdown mới khi có checker. Cặp trùng byte hoặc trùng
sau chuẩn hóa thận trọng gây lỗi nếu ít nhất một tệp được khai báo mới và cặp
không thuộc ngoại lệ. Tương đồng chỉ phát cảnh báo. CI chuyển các tệp Markdown
đã thêm trong commit cho checker offline bằng Git base đã xác minh cục bộ và
tham số nguyên văn. Khi thiếu base hợp lệ, báo giới hạn quét. Kiểm thử luồng
này bằng fixture Git cô lập và bảo toàn bằng chứng native.

## Tuân thủ kiến trúc

Giữ nguyên các cặp nguồn Ecosystem, Lab-First và FPGA dưới `docs/`, kèm
revision và hash. Dùng [chính sách](../../../policies/architecture.md) và
[định tuyến chủ sở hữu](../../../references/architecture-routing.md) khi cần.
Mặc định `LAB_FIRST_PRIMARY`; chỉ ưu tiên song hành khi có bằng chứng đã ghi
cho lát cắt xuyên suốt với artifact được ghim. Ghi kết quả VD-01–VD-10 riêng.

Mở rộng checker offline và CI hiện có để báo lỗi khi thiếu hoặc sai bản chụp,
liên kết kiến trúc đã lỗi thời hay bảng định tuyến không hợp lệ. Cảnh báo bản
khác biệt mà không sửa tệp. Xác nhận gói và bộ cài bằng fixture cô lập.
Hành vi sản phẩm, vận hành phần cứng, cấu hình client thật và bằng chứng lịch
sử E01–E08 nằm ngoài thay đổi này.

## Biên soạn Chat-to-Work

Một [mẫu](../../../templates/chat-to-work-prompt.md) quy định bản bàn giao
12 mục và một [Skill](../../../skills/cirnet-work-prompt-authoring/SKILL.md)
áp dụng mẫu khi người dùng yêu cầu Work Prompt. Dùng mẫu tiếng Anh từ checkout
được phép, tách bằng chứng quan sát khỏi giả định và giữ các ràng buộc CIRNET
áp dụng về chủ sở hữu, kiến trúc và quyền. Trả bản bàn giao hoặc điểm chặn
cụ thể trong chat. Work kiểm tra trước, triển khai và kiểm chứng trong phạm
vi đã được cấp.

Nghiệm thu bằng tài liệu theo cặp EN/VI, chỉ dẫn kích hoạt ngắn trong baseline,
kết quả checker local, chọn Skill tường minh khi cài bundle và các Prompt
giả lập cho bàn giao thông thường, thiếu bằng chứng và yêu cầu bị chặn.
Bundle chuyển Skill, checkout đã được phép cung cấp mẫu.

<!-- source: en/specs/cirnet-rules/toolkit-upgrade/spec.md -->
<!-- source-sha256: 841ef2d4ef840930be66d7d04334d462bc41fdd832caeb73467c1f56e9acd49b -->
