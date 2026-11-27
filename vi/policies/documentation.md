# Chính sách tài liệu song ngữ

Tiếng Anh dưới `en/` là nguồn chuẩn. Cùng đường dẫn tương đối dưới `vi/` là
bản dịch tiếng Việt cho người đọc. Agent mặc định đọc tiếng Anh; bản Việt không
thêm yêu cầu hoặc ghi đè bản Anh. Khi dịch, cần đọc cả hai.

## Quyền sở hữu tài liệu

Giữ chính sách kỹ thuật và các bản chụp kiến trúc chuẩn trong CIRNET Rules.
Dùng [bảng định tuyến kiến trúc](../references/architecture-routing.md)
để tìm chủ sở hữu từng sản phẩm. Giữ mô tả triển khai hiện hành, vận hành
và lịch sử phát hành tại repo sở hữu. Chỉ mục của tổ chức và website liên
kết tới các nguồn đó. Ghi revision, mã băm và mục đích của bản sao cần giữ
để dùng ngoại tuyến hoặc bảo đảm tương thích.

Dùng [Task Continuity](../docs/distribution.md#task-continuity) cho kế hoạch
tạm thời, tiến độ và tiêu chí chấp nhận của tác vụ. Giữ đặc tả thiết kế có
giá trị riêng và kế hoạch dài hạn hiện có tại repo sở hữu, rồi ghim chúng
từ checkpoint. Phiên artifact được cấp quyền lưu đầu ra build và test.

## Vòng đời tài liệu

1. **Tìm trước khi tạo.** Xác định mục đích và chủ sở hữu. Tìm trong repository
   được phép theo tên tệp, tiêu đề, heading, chủ đề, mã issue và nội dung liên
   quan. Lần theo liên kết trỏ tới tài liệu và chỉ mục để xác định chủ sở hữu.
   Phân biệt hướng dẫn hiện hành, kế hoạch, đặc tả, changelog, kiểm chứng,
   bằng chứng có ngày, ví dụ và fixture.
2. **Cập nhật tài liệu chuẩn.** Dùng lại tài liệu cùng mục đích và liên kết đến
   tài liệu đó từ các trang khác. Với bản sửa, ghi hành vi hiện tại, cách tái
   hiện, kết quả quan sát và giới hạn trong hướng dẫn hoặc báo cáo hiện có.
   Liên kết bằng chứng có ngày và giữ kết quả gốc. Chỉ tạo tệp cho mục đích,
   đối tượng đọc hoặc tính năng riêng, yêu cầu của repository hoặc hồ sơ bằng
   chứng độc lập. Báo lý do và liên kết trỏ tới tệp mới.
3. **Đề xuất hợp nhất.** Nêu các ứng viên, phần trùng và nội dung riêng. Giải
   thích nguồn nào nên quản lý chủ đề. Bảo toàn yêu cầu, quyết định, ví dụ,
   lệnh, tham số, phiên bản, khả năng tương thích, schema, số đo, kết quả kiểm
   thử, giới hạn an toàn và nguồn gốc. Giữ kế hoạch và đặc tả có mục đích
   riêng, bằng chứng có ngày, các phiên bản phục vụ tương thích, cặp EN/VI,
   fixture, mẫu và tài liệu khác được bảo vệ. Cập nhật liên kết, chỉ mục và
   giữ đường dẫn tới lịch sử. Hợp nhất trong phạm vi ghi của nhiệm vụ
   khi có thể bảo toàn nội dung. Đối chiếu cảnh báo tương đồng với mục đích
   của từng tài liệu.
4. **Kiểm chứng.** Xem lại hướng dẫn, ví dụ API và tiến độ kế hoạch/đặc tả bị
   ảnh hưởng. Cập nhật bản Anh theo kết quả đã quan sát, rồi sửa bản Việt
   tương ứng trong cùng thay đổi. Soát nghĩa, nhất là phủ định, quyền, khả
   năng chưa có, đường dẫn, số và PASS/FAIL/NOT_RUN/BLOCKED. Giữ nguyên
   định danh mã và lệnh rồi cập nhật hash nguồn sau khi soát. Đồng bộ bản
   baseline ở root nếu có sửa baseline. Chạy [kiểm tra tài liệu cục bộ](../hooks/README.md)
   với `--new-file PATH` cho mỗi tệp Markdown mới mà checker hỗ trợ, rồi chạy
   `git diff --check`. Khi không có checker phù hợp, kiểm tra thủ công các đường
   dẫn bị ảnh hưởng về mục đích, nội dung trùng, liên kết, fragment và độ mới
   bản dịch khi áp dụng. Xử lý lỗi, rà soát cảnh báo và báo đường dẫn thay đổi,
   quyết định, phần chưa kiểm tra cùng giới hạn.

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

## Liên kết và changelog

Dùng đường dẫn Markdown tương đối cho tài liệu hiện hành trong cùng repo.
Dùng liên kết GitHub HTTPS chính thức cho hướng dẫn giữa các repo và permalink
ghim commit cho bằng chứng lịch sử hoặc bản chụp cần đúng revision. Xác minh
đường dẫn, chữ hoa thường, revision và fragment. Đọc ngữ cảnh bắt buộc khi
ngoại tuyến từ checkout đã duyệt hoặc artifact đã kiểm chứng.

Duy trì mỗi changelog tiếng Anh theo một phiên bản chính trong `docs/changelogs/`.
Dùng `CHANGELOG.md` ở thư mục gốc và chỉ mục của thư mục changelog để điều hướng.
Đặt Unreleased đầu tệp major mới nhất, sau đó xếp các mục Semantic Versioning
đầy đủ từ mới tới cũ. Ghi ngày công bố đã kiểm chứng cho bản phát hành và nêu
trạng thái của mốc phát triển, bản tiền phát hành và ứng viên. Giữ dữ kiện riêng
khi dịch lại mục cũ. Changelog và chỉ mục dùng chung tiếng Anh, không có bản dịch
tiếng Việt tương ứng.

## Metadata của cặp tài liệu

Kết thúc mỗi tài liệu Markdown dưới `vi/` bằng hai comment sau. Dùng đường dẫn
và digest của tệp tiếng Anh tương ứng hiện tại:

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
- Giữ nguyên các [bản lưu kiến trúc](architecture.md) được cung cấp dưới
  `docs/`. Ghi đường dẫn và digest của cặp EN/VI trong
  `docs/architecture-sources.tsv`.
- Di chuyển hoặc ngừng dùng cả cặp cùng lúc và cập nhật các link trỏ tới nó.
  Giữ bằng chứng lịch sử; tách rõ ngày/phạm vi gốc với kiểm chứng hiện tại.
- Repo sản phẩm giữ chính sách ngôn ngữ và output riêng. Common không cấp quyền
  tạo cây song ngữ hoặc sửa repository khác.

<!-- source: en/policies/documentation.md -->
<!-- source-sha256: bf2c193a0263d1c0036174fc892edee25d3c12a04c37a1e563930e4e9501d27d -->
