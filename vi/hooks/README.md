# Hooks tại repository

Các điểm kiểm tra này hỗ trợ một coding agent, không điều phối nhiều Agent hay
đăng ký trạng thái task. Đọc chỉ dẫn tiếng Anh. Bản Việt để người dùng review,
không phải lớp context bổ sung.

| Điểm kiểm tra | Hành động |
| --- | --- |
| Trước task | Xác nhận root/branch/status và phạm vi ghi; đọc quy tắc local, context Anh liên quan |
| Sau triển khai/build | Xác định ảnh hưởng tài liệu; cập nhật Anh–Việt cùng nhau; khớp tiến độ plan/spec và kết quả thực tế |
| Trước hoàn tất | Soát nghĩa, cập nhật metadata bản dịch đã review, chạy checker, xem diff và báo giới hạn |

## Kiểm tra thực thi local tùy chọn

Chạy các lệnh sau từ root repository common-rules với Cargo và Rust 1.88 trở lên.

```powershell
cargo run --offline --locked --bin check-docs
cargo test --offline --locked
git diff --check
```

Checker mặc định kiểm tra checkout đã dùng để biên dịch. Để kiểm tra
fixture hoặc một checkout common-rules cụ thể, truyền `--root` sau `--`.

```powershell
cargo run --offline --locked --bin check-docs -- --root PATH
```

Checker trong `hooks/check_docs.rs` chạy offline và chỉ đọc. Checker và các bài
kiểm tra trong `tests/check_docs.rs` chỉ dùng thư viện chuẩn Rust. Theo mặc định,
Cargo ghi đầu ra build vào `target/` đã được ignore. Các bài kiểm tra chỉ ghi
fixture dưới `.superpowers/bilingual-tests/` đã được ignore của repo này.

Kiểm tra bao gồm cặp đường dẫn, nguồn/hash bản dịch còn khớp, UTF-8/LF,
whitespace, link/fragment Markdown local, bản baseline root, adapter Claude,
sự thống nhất phiên bản toolkit và skill tiếng Anh được khám phá tự động.
Subset frontmatter portable có fence `---` chính xác và một dòng `name`,
một dòng `description` không thụt lề. Dùng chuỗi thuần một dòng hoặc chuỗi
nháy đơn không escape. Dẫn file bắt buộc bằng link Markdown trong cùng thư mục
skill. Path phải đúng chữ hoa thường và dùng dấu gạch chéo xuôi. Dẫn nguồn HTTP,
HTTPS và mailto được phép. URI file và path ra khỏi skill làm kiểm tra thất bại.

Mã thoát 0 là PASS, 1 khi có ERROR và 2 khi cách gọi không hợp lệ. WARN gồm
ngân sách context và phát hiện tương đồng. INFO gồm số đo và hướng dẫn rà
soát tệp. Thiếu bộ công cụ Rust thì kiểm tra thực thi ở trạng thái BLOCKED.
Ghi kết quả rà soát thủ công riêng.

## Rà soát Markdown mới

Sau khi tìm tài liệu sở hữu nội dung, liệt kê rõ từng đường dẫn Markdown mới
để đối chiếu với các tệp checker tìm được. Đường dẫn tương đối so với root đã
chọn và giữ nguyên cách viết cùng chữ hoa, chữ thường mà checker thấy.

```powershell
cargo run --offline --locked --bin check-docs -- --new-file 'en/docs/New guide.md' --new-file 'vi/docs/New guide.md'
```

Có thể lặp lại `--new-file` hoặc dùng dạng `--new-file=PATH`. Phạm vi gồm
Markdown hiện có ở root và hai cây `en/`, `vi/`, kể cả tệp Git chưa theo dõi
khi được liệt kê. Người chạy cung cấp đường dẫn. Checker dùng cùng danh sách
tệp chỉ đọc và trả mã 2 nếu đường dẫn không hợp lệ hoặc không được tìm thấy.

Lượt kiểm tra mặc định cũng báo tương đồng. Checker đối chiếu theo vai trò
riêng trong từng nhóm root, tiếng Anh và tiếng Việt. Nó bỏ qua các điểm vào
có chủ đích, mẫu, ví dụ, fixture kiểm thử, tệp giấy phép và ghi công chuẩn,
tệp rỗng hoặc chỉ có tiêu đề, README hoặc chỉ mục dưới 40 từ. Trùng chính xác
được xác định theo byte. Trùng sau chuẩn hóa xử lý CRLF, dòng trống cuối tệp
và metadata nguồn hợp lệ cuối bản Việt có cặp. Mã, dấu câu, số và trạng thái
được giữ nguyên. Kiểm tra LF vẫn áp dụng. Gần trùng dùng độ tương đồng
Jaccard trên các chuỗi ba từ phân biệt. Mỗi tài liệu cần ít nhất 40 từ và
24 chuỗi phân biệt. Ngưỡng là
60%, hoặc 40% khi chung tiêu đề, mã issue hay họ tên tệp trong cùng thư mục
sau khi bỏ hậu tố phiên bản.

Checker báo đường dẫn, cách đo, điểm, mức tin cậy, số tiêu đề và chuỗi từ
không trùng, cùng các liên kết Markdown trỏ đến tệp mà nó phát hiện. Rà soát
đề xuất `REVIEW_MERGE` và `UPDATE_EXISTING` theo mục đích của chủ sở hữu, nội dung
riêng và đích liên kết. `KEEP_SEPARATE` đánh dấu các đường dẫn được nhận diện
là bằng chứng lịch sử, báo cáo kiểm chứng hay ghi chú phát hành cần bảo toàn.
`NEW_DOCUMENT_REVIEW` yêu cầu người bảo trì quyết định nếu chưa thấy ứng viên
hiện có phù hợp. Xác nhận mục đích riêng ngay cả khi checker không đề xuất
ứng viên. Nếu tệp được liệt kê trùng chính xác một tài liệu
không được liệt kê và không thuộc diện bảo vệ, checker báo
`DOC_NEW_DUPLICATE` (mã thoát 1). Cặp có cả hai đường dẫn được liệt kê vẫn ở mức
tham khảo. Checker giữ nguyên các tệp. Người rà soát quyết định theo ý nghĩa,
mục đích và phạm vi hợp nhất.

## Kết nối với Agent sau này

Codex, Claude Code hoặc Antigravity có thể chạy lệnh rõ ràng khi task bảo trì
repo này. Để tự chạy theo lifecycle native, kiểm tra hỗ trợ hooks của client
đang cài, preview cấu hình cụ thể và giữ settings hiện hữu trước khi người dùng
cho phép cài. Thay đổi này không cài hook native/global; tự gọi hook trên mỗi
client là NOT_RUN.

Không áp checker này lên repo sản phẩm bất kỳ: nó yêu cầu layout en/vi và các
entrypoint của common-rules. Kiểm tra sản phẩm và bộ cấp phát output vẫn theo local.
Xem [chính sách tài liệu](../policies/documentation.md) về nghĩa vụ cập nhật.

<!-- source: en/hooks/README.md -->
<!-- source-sha256: 03541531d57691918e7b458162235b38c20180adbf778149201084dd83ff0043 -->
