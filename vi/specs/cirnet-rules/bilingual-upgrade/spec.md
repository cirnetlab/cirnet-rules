# Đặc tả nâng cấp tài liệu song ngữ

Ngày 2026-11-14. Tiếng Anh là nguồn chỉ dẫn chuẩn; tiếng Việt là bản dịch dành
cho người đọc. Task này thay quyết định bố cục một ngôn ngữ trước đó, vẫn giữ
phạm vi, ownership sản phẩm và kiến trúc không cần runtime.

## Yêu cầu

- Chỉ ghi worktree `rules-common`, ban đầu sạch ở
  `17f1738ccd4312f284b61372a3c507a65a26588a`. Không stage, commit hoặc sửa global.
- Dùng `en/` làm nguồn chuẩn cho Agent và tài liệu được bảo trì.
  Mỗi Markdown tiếng Anh có đường dẫn tương ứng dưới `vi/`.
- Giữ root `AGENTS.md` là bản phân phối giống hệt `en/AGENTS.md`;
  `CLAUDE.md` vẫn là `@AGENTS.md`. Root README điều hướng hai ngôn ngữ.
- Bổ sung hướng dẫn hooks, policies, plans, specs cùng rules, skills, references,
  templates, docs. Giữ đúng ba định nghĩa skill tùy chọn dành cho Agent.
- Sau triển khai, Agent cập nhật cả hai bản tài liệu bị ảnh hưởng, ghi kiểm chứng
  và giới hạn, báo trung thực bản dịch chưa xong. Không buộc sửa repo sản phẩm
  ngoài phạm vi hoặc tạo spec cho mọi typo.
- Có hook tùy chọn tại repo, chỉ đọc, kiểm tra cặp file, hash nguồn, liên kết
  Markdown, entrypoint độc lập và metadata skill. Người dùng đã đồng ý hook này.
  Không tự cài hook native/global.
- Giữ plan/spec và bằng chứng chuyển đổi cũ dưới dạng lịch sử có ngày.
  Sửa liên kết không biến kết quả cũ thành kết quả chạy mới.
- Giữ LICENSE, ignore rules và attribution bên thứ ba.

## Quyết định thiết kế

Một nguồn tiếng Anh để biên soạn, kèm bản phân phối ở root được kiểm tra.
Tài liệu tiếng Việt ghi đường dẫn nguồn và SHA-256. Hash phát hiện lệch nguồn,
không chứng minh dịch đúng: phải soát nghĩa trước khi cập nhật hash.
Agent mặc định đọc tiếng Anh; chỉ đọc tiếng Việt khi dịch hoặc review bản dịch.
Không cần registry YAML song ngữ, dịch vụ runtime, database hay approval engine.

Hook Python local là kiểm tra tùy chọn cho người bảo trì, không phải dependency
để đọc rules. Tài liệu hooks mô tả trước task, sau sửa đổi và trước hoàn tất;
kết nối vào client native là hành động cấu hình riêng của người dùng.

## Nghiệm thu

Các cặp thư mục bao phủ hooks, plans, specs, policies, references, rules, skills,
templates và docs. Facts, lệnh, nhãn trạng thái và giới hạn phải tương ứng.
Hook và các test tình huống lỗi phải pass, link nội bộ hợp lệ và
`git diff --check` sạch. Tự động thực thi hook native là `NOT_RUN`.
Không tác động repo sản phẩm; các nhánh backup giữ nguyên.

<!-- source: en/specs/cirnet-rules/bilingual-upgrade/spec.md -->
<!-- source-sha256: c846809ec758fb9c169c94580b77ca43cfec5c8f3cd891f888e6ca9dada442d9 -->
