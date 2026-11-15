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

<!-- source: en/specs/cirnet-rules/toolkit-upgrade/spec.md -->
<!-- source-sha256: cfe944f25558772afb5cb37e8c7fc6ad54f0bbd8595ab17a90b1cfb26c5bdd5f -->
