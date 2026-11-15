# Chính sách tài liệu song ngữ

Tiếng Anh dưới `en/` là nguồn chuẩn. Cùng đường dẫn tương đối dưới `vi/` là
bản dịch tiếng Việt cho người đọc. Agent mặc định đọc tiếng Anh; bản Việt không
thêm yêu cầu hoặc ghi đè bản Anh. Khi dịch, cần đọc cả hai.

## Vòng đời tài liệu

1. **Tìm trước khi tạo.** Xác định mục đích và chủ sở hữu. Tìm trong repository
   được phép theo tên tệp, tiêu đề, heading, chủ đề hoặc issue, rồi theo liên
   kết từ chỉ mục và nguồn sở hữu nội dung. Phân biệt hướng dẫn hiện hành, kế
   hoạch, đặc tả, changelog, kiểm chứng, bằng chứng có ngày, ví dụ và fixture.
2. **Cập nhật tài liệu chuẩn.** Dùng lại tài liệu cùng mục đích và liên kết đến
   tài liệu đó từ các trang khác. Cập nhật tại chỗ hành vi hiện tại, cách tái
   hiện, kết quả quan sát được và giới hạn. Liên kết bằng chứng có ngày và giữ
   kết quả gốc. Chỉ tạo tệp cho mục đích, đối tượng đọc hoặc tính năng riêng,
   yêu cầu của repository hoặc hồ sơ bằng chứng độc lập. Báo lý do và liên kết
   trỏ tới từng tệp mới.
3. **Đề xuất hợp nhất.** Nêu các ứng viên, phần trùng và nội dung riêng. Giải
   thích nguồn nào nên quản lý chủ đề. Bảo toàn yêu cầu, quyết định, ví dụ,
   lệnh, phiên bản, schema, số đo, trạng thái kiểm thử, giới hạn an toàn và nguồn gốc.
   Dự kiến cập nhật liên kết, chỉ mục, cặp EN/VI và cách truy cập lịch sử.
   Chỉ hợp nhất trong phạm vi ghi của nhiệm vụ khi có thể bảo toàn nội dung.
   Dùng cảnh báo tương đồng để xác định phần cần xem xét.
4. **Kiểm chứng.** Xem lại hướng dẫn, ví dụ API và tiến độ kế hoạch/đặc tả bị
   ảnh hưởng. Cập nhật bản Anh theo kết quả đã quan sát, rồi sửa bản Việt
   tương ứng trong cùng thay đổi. Soát nghĩa, nhất là phủ định, quyền, khả
   năng chưa có, đường dẫn, số và PASS/FAIL/NOT_RUN/BLOCKED. Giữ nguyên
   định danh mã và lệnh rồi cập nhật hash nguồn sau khi soát. Đồng bộ bản
   baseline ở root nếu có sửa baseline. Chạy [kiểm tra tài liệu cục bộ](../hooks/README.md)
   và `git diff --check`. Xử lý lỗi, rà soát cảnh báo và báo đường dẫn đã
   thay đổi, quyết định cùng giới hạn.

Trình bày lần lượt mục đích, đầu vào, các bước, kiểm chứng và giới hạn khi cần.
Dùng liên kết nếu tài liệu khác quản lý chi tiết đó. Bỏ câu lặp và giữ lệnh,
tiêu chí nghiệm thu, trường hợp biên và ràng buộc an toàn.

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
<!-- source-sha256: eba656693e5a29a825012effb82f7b26c4436d6913b36472831c0ee25722282f -->
