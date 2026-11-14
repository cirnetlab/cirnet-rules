# Chính sách tài liệu song ngữ

Tiếng Anh dưới `en/` là nguồn chuẩn. Cùng đường dẫn tương đối dưới `vi/` là
bản dịch tiếng Việt cho người đọc. Agent mặc định đọc tiếng Anh; bản Việt không
thêm yêu cầu hoặc ghi đè bản Anh. Khi dịch, cần đọc cả hai.

## Sau triển khai hoặc build

1. Xem thay đổi ảnh hưởng tài liệu nào: hành vi, API, setup, lệnh, ví dụ, giới hạn
   và tiến độ plan/spec hiện tại. Chỉ đọc context liên quan.
2. Cập nhật nguồn Anh theo triển khai và kết quả kiểm chứng thực tế.
   Dịch bản Việt bị ảnh hưởng trong cùng thay đổi.
3. Soát nghĩa, nhất là phủ định, quyền, khả năng chưa có, đường dẫn, số và
   PASS/FAIL/NOT_RUN/BLOCKED. Giữ nguyên định danh code/lệnh; dịch phần giải thích.
   Không biến bằng chứng lịch sử thành PASS mới.
4. Chỉ cập nhật metadata nguồn sau khi soát nghĩa. Cập nhật bản baseline root nếu
   `en/AGENTS.md` đổi. Không đổi adapter Claude ở root.
5. Chạy [kiểm tra tài liệu local](../hooks/README.md), xem diff cuối và báo cả hai
   đường dẫn đã cập nhật cùng kiểm chứng, giới hạn còn lại.

Nếu build không đổi nghĩa tài liệu, ghi nhận đánh giá đó cùng kết quả test thực tế
trong báo cáo task; không sửa vô nghĩa chỉ để chạm hai file.
Typo chỉ thuộc bản dịch có thể sửa riêng tiếng Việt khi nghĩa bản Anh vẫn đúng.
Review cả cặp và giữ hash khớp nguồn Anh.

Thiếu tài liệu Anh/Việt bắt buộc nghĩa là công việc chưa xong. Nếu quyền truy cập
hoặc phạm vi ngăn đồng bộ, đánh dấu bước liên quan BLOCKED, chỉ rõ bản còn thiếu
và tiếp tục phần độc lập được phép. Build pass không thay thế cập nhật tài liệu cần thiết.

## Metadata của cặp tài liệu

Mỗi Markdown tiếng Việt kết thúc bằng hai comment sau; thay đường dẫn/hash mẫu
bằng file tiếng Anh tương ứng hiện tại và digest của nó:

```text
<!-- source: en/policies/documentation.md -->
<!-- source-sha256: <64 lowercase hexadecimal characters> -->
```

Tính SHA-256 trên byte tiếng Anh UTF-8/LF chính xác. Ví dụ chạy từ root repo:

```powershell
(Get-FileHash -Algorithm SHA256 -LiteralPath .\en\policies\documentation.md).Hash.ToLowerInvariant()
```

Hook kiểm tra độ mới, không chứng minh dịch đúng. Không chỉ đổi hash để làm mất
lỗi bản dịch cũ. Không có chế độ tự stamp/dịch; sửa bản dịch, review rồi cập nhật
metadata của nó.

## Nguồn và ngoại lệ

- `en/AGENTS.md` là baseline để sửa; root `AGENTS.md` là bản phân phối giống hệt,
  tự đủ nghĩa. Root `CLAUDE.md` chứa `@AGENTS.md`.
- Root README điều hướng song ngữ. LICENSE và thông báo bên thứ ba ở root giữ vai
  trò pháp lý/attribution; không nhân bản hay dịch điều khoản như thể bản dịch
  thay thế bản gốc. File Git và mã checker dùng chung.
- Mọi Markdown dưới en/vi, gồm tài liệu skill và hồ sơ task, đều có cặp.
  Chỉ cài định nghĩa từ `en/skills/`; `vi/skills/` dành cho người đọc.
- Di chuyển hoặc ngừng dùng cả cặp cùng lúc và cập nhật các link trỏ tới nó.
  Giữ bằng chứng lịch sử; tách rõ ngày/phạm vi gốc với kiểm chứng hiện tại.
- Repo sản phẩm giữ chính sách ngôn ngữ và output riêng. Common không cấp quyền
  tạo cây song ngữ hoặc sửa repository khác.

<!-- source: en/policies/documentation.md -->
<!-- source-sha256: 4f23cc69b29107872ab4822c6f04102ab8164a1f67db21f48a86ec9afedd8def -->
