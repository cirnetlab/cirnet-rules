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

ERROR làm kiểm tra thất bại. WARN báo ngân sách context và dấu hiệu trùng lặp.
INFO báo số đo văn bản. Hash bản dịch và heuristic văn bản cần review nghĩa riêng.
Mã thoát 0 là PASS, 1 là kiểm tra thất bại và 2 là cách gọi không hợp lệ.
Thiếu bộ công cụ Rust thì kiểm tra thực thi ở trạng thái BLOCKED. Ghi kết
quả review thủ công riêng.

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
<!-- source-sha256: 47919860950df227232e7984863bf111eb1b3db0a2bf98d70994697bc9b0b2e7 -->
