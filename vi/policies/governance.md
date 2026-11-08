<!-- doc-id: POLICY -->
<!-- revision: 2 -->
<!-- source-sha256: 25dc1a7952d24ac92906cb43f60089bf4cce0cea07998dd97578ae11bdb6265f -->
<!-- translation-status: synchronized -->

<a id="release"></a>

# Phát hành, phê duyệt và chuyển trạng thái

## REL-01 — Chọn đúng tổ hợp

Release train là `0.2.4`. Họ tương thích là chuỗi `"0.2"`.
Revision contract như `0.2.0` gắn với commit và digest chính xác.
Governance có phiên bản riêng trong `VERSION`. Toolchain và định dạng bên ngoài
có khóa độc lập. Xem [thuật ngữ](../references/glossary.md).

Remote sở hữu duy nhất `integration/release-set.json`. Giữ nguyên tag và
bytes đã phát hành. Kiểm tra đủ sáu định danh, digest ma trận đã duyệt,
các miền tương thích, artifact và bằng chứng trước khi kích hoạt. Nhãn khớp
hoặc phiên bản từ baseline trở lên chưa cấp quyền tương thích. [Schema phát hành](../schemas/README.md)
mô tả đầu vào đề xuất cho repo sở hữu. Registry hiện ở `NOT_ACTIVATED`.

## Cutover và khôi phục

Đợt đặt lại tương thích này là một thay đổi phá vỡ tương thích. Sau khi
kích hoạt rõ ràng, từ chối profile trước baseline và tổ hợp ngoài danh sách
trước tác dụng phụ. Gỡ parser, alias và đường fallback cũ bằng thay đổi được
rà soát. Giữ tag lịch sử và fixture từ chối ngoài discovery hiện hành.
Xác thực và phân quyền độc lập với phiên bản, digest. Contract nhị phân phải
định nghĩa encoding wire rõ ràng và có giới hạn.

Sau khi kích hoạt, sửa lỗi tương thích dùng bản patch. Thay đổi phá vỡ source,
wire hoặc hành vi cần ADR, rà soát consumer và family hoặc revision phù hợp.
Giữ dialect OpenAPI và JSON Schema, cú pháp protobuf, thứ tự migration cơ sở
dữ liệu, chuẩn trình biên dịch và định danh board là các miền riêng.

Chỉ khôi phục bộ hoàn chỉnh đã được duyệt riêng với dữ liệu khớp. Nếu thiếu,
dừng an toàn rồi sửa tiến hoặc phục hồi đã duyệt. Dữ liệu cũ cần công cụ Rust
chạy offline tường minh, sao lưu, dry-run và đầu ra kiểm chứng, hoặc bộ dữ
liệu mới được cấp phép.

## Nguồn chuẩn và phê duyệt

Tác giả đưa thiết kế ở `DRAFT`. Maintainer sở hữu ghi tham chiếu phê duyệt
thật trước `APPROVED`. Thiết kế thay thế hoặc nghỉ dùng ghi `SUPERSEDED`
hoặc `RETIRED` và rời danh sách chỉ dẫn hiện hành. Hồ sơ cutover lịch sử
giữ `PLANNED` và `RELEASED`. Task local mới đi từ `PROPOSED` → `READY` →
`IN_PROGRESS` → `IMPLEMENTED` → `VERIFIED` → `ACCEPTED` → `DOCS_SYNCED`
→ `CLOSED`. Thiếu điều kiện cho `BLOCKED`. Quyết định miễn tài liệu tường
minh có thể đóng task đã nghiệm thu. Người quản lý release ghi bằng chứng
phát hành riêng. Kiểm chứng gắn đầu vào đã kiểm với revision governance và task.
Xem [quy trình CGACP](../references/cgacp-local.md).

Danh sách ngoại lệ ban đầu đóng. Ngoại lệ tương lai cần ID quy tắc, owner,
lý do, hạn và phê duyệt. Agent không tự miễn gate của mình. Quy tắc và hook
bổ trợ quyền filesystem, CI và interlock.
