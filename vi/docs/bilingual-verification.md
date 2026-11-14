# Nghiệm thu nâng cấp song ngữ

Ngày 2026-11-14. Báo cáo này nói về nâng cấp song ngữ từ baseline sạch
`17f1738ccd4312f284b61372a3c507a65a26588a`, không phải lần rebuild Governance trước đó.

## Triển khai

Tài liệu tiếng Anh nằm dưới `en/`; mỗi Markdown có bản Việt tương ứng dưới
`vi/`. Root AGENTS là bản phân phối tiếng Anh giống hệt và CLAUDE import nó.
Nội dung bổ sung gồm hooks, policies, plans, specs, references, rules,
ba skill và template. Chỉ dẫn yêu cầu cập nhật cặp sau triển khai/build khi
nghĩa tài liệu thay đổi.

`hooks/check_docs.py` tùy chọn, chỉ đọc, offline, dùng thư viện chuẩn Python.
Nó kiểm tra cặp file và nhất quán kỹ thuật; soát nghĩa bản dịch vẫn là trách
nhiệm riêng. Không cài hook global/native, sửa sản phẩm, stage, commit hay release.

## Kiểm tra

- `python -B hooks/check_docs.py`: PASS trên 26 cặp tài liệu Anh–Việt.
  Có 61 file được bảo trì, gồm 52 Markdown theo cặp và bốn Markdown root cho
  entrypoint/điều hướng/attribution. Baseline dài 91 dòng.
- `python -B -m unittest discover -s tests -p 'test_*.py' -v`: exit 0;
  tổng 28 test, 27 PASS và 1 SKIP (NOT_RUN vì Windows không cho tạo symlink).
  Test bao phủ bản dịch cũ/thiếu/mồ côi, metadata, link/fragment, vượt đường dẫn,
  cây bị loại trừ, định nghĩa skill, entrypoint lệch, input sai và hành vi chỉ đọc.
- `quick_validate.py` có sẵn kiểm tra cả ba skill tiếng Anh: PASS, exit 0.
  `git diff --check`: PASS. Thư mục fixture test rỗng sau khi dọn.
- Người dịch đã soát nghĩa và giữ đường dẫn kỹ thuật, lệnh, SHA, link nguồn,
  giới hạn bằng chứng. Review độc lập kiểm tra mẫu bản dịch, mô hình authority
  và mã hook; riêng hash không chứng nhận bản dịch chính xác.
- Review phát hiện trên Windows có thể dùng tên khác hoa/thường để vượt kiểm tra
  thư mục bị loại trừ. Lỗi đã được tái hiện, có hai regression test thất bại trước
  khi sửa và được khắc phục bằng so sánh loại trừ nhất quán không phân biệt hoa/thường.
  Cả bộ 28 test sau đó đạt với một skip đã nêu. Không còn phát hiện review quan trọng khác.
- Tự gọi hook native trong Codex/Claude/Antigravity: NOT_RUN.
- Runtime/build/kiểm chứng vật lý sản phẩm: NOT_RUN; sản phẩm ngoài phạm vi ghi.

HEAD vẫn là `17f1738ccd4312f284b61372a3c507a65a26588a` trên `rules-common`.
`master` và `governance-cli-only` vẫn ở
`646afa38516bbfd276b0aecb7a138ab427df5fa9`. Index chưa stage.
Giữ LICENSE và cấu hình Git hiện có; thông báo bên thứ ba được bổ sung checker
tùy chọn do repo tự viết, không thêm dependency đóng gói.

[Báo cáo rebuild ban đầu](rebuild-verification.md) là lịch sử.
Số file/link và kết quả test trong đó chỉ mô tả snapshot cũ.

<!-- source: en/docs/bilingual-verification.md -->
<!-- source-sha256: 6d62fa86b03073e2fe3eb9edf105a071c8d75e3445e55cd268db603fae4239ad -->
