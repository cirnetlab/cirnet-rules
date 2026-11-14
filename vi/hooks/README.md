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

Chạy từ root repository common-rules, dùng Python 3.10+ đã có:

```powershell
python -B hooks/check_docs.py
python -B -m unittest discover -s tests -p 'test_*.py' -v
git diff --check
```

Script tự xác định root repo của nó. Tham số chỉ đọc `--root PATH` dùng cho
fixture kiểm tra hoặc checkout common-rules được chỉ định rõ.
Script không cài tool, dịch nội dung, stamp hash, sửa Git hay truy cập mạng.
Unit test chỉ ghi fixture dưới `.superpowers/bilingual-tests/` đã được ignore
của repo này.

Kiểm tra bao gồm cặp đường dẫn, nguồn/hash bản dịch còn khớp, UTF-8/LF,
whitespace, link/fragment Markdown local, bản baseline root, adapter Claude và
ba định nghĩa skill tiếng Anh. Hash khớp không chứng minh đúng nghĩa.
Exit 0 là PASS, 1 là kiểm tra thất bại, 2 là input không hợp lệ.
Thiếu Python nghĩa là kiểm tra thực thi BLOCKED; review thủ công không phải PASS
của lệnh đã chạy.

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
<!-- source-sha256: 29d6b4145f1693920428e20d0b04c750e73c736d9baaba6aeb005856d63e4c87 -->
