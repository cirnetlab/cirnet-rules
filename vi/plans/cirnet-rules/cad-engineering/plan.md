# Kế hoạch triển khai kỹ thuật CAD CIRNET

Triển khai [đặc tả](../../../specs/cirnet-rules/cad-engineering/spec.md)
trong `cirnet-rules`. Giữ các checkout công cụ và sản phẩm tham chiếu ở
chế độ chỉ đọc. Cấp vị trí đầu ra tạm trong workspace được chủ sở hữu
cho phép và Git bỏ qua.

## Bước và phép kiểm

1. Ghim repo triển khai và các repo tham chiếu. Đọc lệnh công cụ, tài sản
   Interface, dependency đã chọn và hợp đồng tải.
2. Thêm quy tắc CAD song ngữ, liên kết và Skill tùy chọn. Xác minh Skill có
   trong chỉ mục và bundle, cài được khi chọn riêng và nằm ngoài mặc định.
3. Viết fixture GLB tổng hợp trước khi triển khai bộ tiếp nhận tĩnh. Kiểm tra
   cấu trúc hợp lệ, tên thành phần của cảnh kích hoạt, dùng lại mesh, pin,
   đường dẫn cũ hoặc không an toàn, đầu ra bị cắt, tính năng ngoài hỗ trợ
   và trường hợp vượt hạn mức.
4. Kiểm tra cùng mã nguồn trên Windows và Ubuntu nếu có. Chạy build Rust ngoại
   tuyến, checker tài liệu, fixture xác minh và continuity bị tác động,
   cùng kiểm thử bundle và installer trên Windows.
5. Xác định điều kiện cần để chạy CAD cục bộ. Chỉ thực hiện pilot đã duyệt
   khi có runtime phù hợp đã cài sẵn.

## Bằng chứng ban đầu

| Nguồn | Revision đã đọc |
| --- | --- |
| Rules master | `b10e75a378cca7196c3b27e346d283ddd6c6f42d` |
| text-to-cad main | `7687743d80305ba112666aee0a5e8c12af138458`, phiên bản mã nguồn 0.11.15 |
| Interface master | `82dd99a62c899aa413d03856c41e252bf289ba22`, gói 0.2.4, Three.js 0.171.0 trong lockfile |

[Quy tắc vận hành](../../../rules/cad-engineering.md) ghi hợp đồng đã kiểm tra
và những gì pilot native cần đo. Chạy CAD native và việc Interface tiếp nhận
tài sản là hai bước nghiệm thu riêng.

## Bản ghi xác minh

| Phép kiểm | Quan sát |
| --- | --- |
| Fixture cổng tĩnh | `node --test tests/verification/cad-asset.test.mjs` thoát 0. Đã kiểm tra GLB nhúng, tên trong cảnh kích hoạt, mesh dùng lại, pin, giới hạn đường dẫn và tệp, trường sai, primitive ngoài hỗ trợ và hạn mức. Các trường ghi null rõ ràng từng trả PASS nay thất bại. |
| Tài sản Interface tham chiếu | Bản sao chỉ đọc của GLB máy phát và máy hiện sóng hiện có vượt qua cổng với hash đã ghim của chủ sở hữu và tên thành phần đã chọn. Đo được 18.038/16.926 tam giác và 84/94 lần hiện diện primitive. |
| Build toolkit | Build, test, định dạng và Clippy của Cargo ngoại tuyến đều qua. Fixture liên kết trên Windows bị bỏ qua. Checker kiểm tra tám tệp Markdown mới và hash nguồn tương ứng. |
| Phân phối Skill | Kiểm tra danh mục dự phòng và metadata kích hoạt đều qua. Kiểm thử bundle và cài đặt trên Windows chạy với tập mặc định và lựa chọn CAD tùy chọn trong thư mục giả lập. Fixture reparse và junction bị bỏ qua do quyền hệ điều hành. |
| Pilot CAD native | `BLOCKED`. Máy đã kiểm tra có Python 3.14.4 và uv 0.11.9. Checkout nguồn cục bộ không cung cấp tệp thực thi CAD đã cài hoặc runtime build123d, OpenCASCADE hay Blender cần thiết. Không có thao tác cài hoặc kích hoạt plugin. |
| Nghiệm thu trực quan native và sản phẩm | `NOT_RUN`. Pilot xác định các bước kiểm hình học, chụp ảnh, xuất và rà soát native. Kiểm thử loader, trình duyệt và tương tác của Interface cần quyền tiếp nhận riêng. |

<!-- source: en/plans/cirnet-rules/cad-engineering/plan.md -->
<!-- source-sha256: e89302a9bff114083f3b9960821d8e0c07018de0185cef7283ae82380bc44234 -->
