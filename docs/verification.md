# Nghiệm thu chuyển đổi rules-common

Ngày kiểm tra: **2026-11-14**. Phạm vi: working tree `rules-common` của
`cirnetlab/cirnet-rules`; chỉ đọc sáu sản phẩm ghi trong
[repository map](../references/repository-map.md).

## Trạng thái nghiệm thu

| # | Tiêu chí | Kết quả |
| --- | --- | --- |
| 1 | Đúng root/branch; refs backup giữ nguyên | PASS |
| 2 | AGENTS độc lập, CLAUDE import đúng file | PASS |
| 3 | README/setup đúng scope và giới hạn discovery | PASS review tài liệu; native NOT_RUN |
| 4 | Sáu repository có nguồn thực, SHA và ownership | PASS source review |
| 5 | Bốn topic guide, đúng ba skill tùy chọn | PASS |
| 6 | Không còn runtime/CI Governance | PASS |
| 7 | UTF-8, frontmatter, link nội bộ, diff whitespace | PASS |
| 8 | Sáu sản phẩm không phát sinh thay đổi | PASS đối chiếu HEAD/branch/status sau xóa legacy |
| 9 | Ba tình huống đọc hướng dẫn | PASS trên giấy; xem dưới |
| 10 | Native loader/E2E Codex, Claude, Antigravity | NOT_RUN cho cả ba |

## Thay đổi và phạm vi

Ban đầu có 290 file tracked, status sạch và không có untracked/ignored được báo.
Đã xóa **284 file legacy tracked** sau khi kiểm tra từng file chưa bị sửa và cây
backup khớp HEAD. Giữ nguyên byte của LICENSE, .gitattributes, .gitignore.
Viết lại AGENTS.md, README.md, THIRD_PARTY_NOTICES.md; thêm adapter, guides,
skills, template, setup, map và hồ sơ chuyển đổi trung tâm Anh–Việt.

Không chạy Cargo/CNR hoặc bộ test sản phẩm. Không stage, commit, push, merge,
fetch/pull, tạo PR, cài tool/global hay cấp nguồn thiết bị. Không thao tác vào
checkout khác của cirnet-rules, repo rules-common ở root, cirnet-cli hoặc dự án khác.
Git status của sản phẩm dùng `GIT_OPTIONAL_LOCKS=0` để tránh refresh index.

HEAD, `master` và `governance-cli-only` trước–sau:
`646afa38516bbfd276b0aecb7a138ab427df5fa9`.
Các SHA và trạng thái sáu sản phẩm được lưu trong repository map; toàn bộ đều
`master`, clean ở lần đọc đầu và lần đối chiếu sau khi xóa legacy.

## Kiểm tra đã chạy

- Python chuẩn có sẵn, `python -B -X utf8 -`: kiểm tra **22 file còn lại**,
  **19 Markdown UTF-8/LF**, **34 link nội bộ** resolve được, baseline **74 dòng /
  4.666 byte**, adapter đúng một dòng, ba skill không phụ thuộc file ngoài.
- `quick_validate.py` đi kèm skill-creator: cả **3/3 skill hợp lệ**, exit 0;
  không cài thêm package.
- PowerShell `Test-Path` và `git cat-file -e` read-only: **92 link nguồn ghim SHA**
  khớp file local và Git object của sáu sản phẩm. Không truy vấn GitHub để đọc source.
- Kiểm tra tồn tại: không còn src, migrations, manifests/lockfile Rust,
  governance registry, cnr config, CI cũ hoặc assets ứng dụng.
- SHA-256: LICENSE, .gitattributes, .gitignore không đổi. Kiểm tra refs/root/branch
  và `git diff --cached --name-only`: đúng scope, không có thay đổi được stage.
- `git diff --check`: exit 0; tìm chuỗi legacy trong các file còn lại chỉ có
  nội dung lịch sử, giới hạn phạm vi hoặc dữ liệu sản phẩm, không có runtime bắt buộc.
- Review độc lập tìm một nhận định Claude discovery quá tuyệt đối; đã sửa thành
  hướng dẫn theo version/config, giữ adapter tương thích và yêu cầu kiểm tra client.
  Reviewer không phát hiện vấn đề quan trọng khác. Các kiểm tra trên không chứng minh
  native E2E, trạng thái triển khai hoặc an toàn vật lý.

## Kiểm tra trên giấy

Một reviewer độc lập đọc baseline và cả ba skill rồi áp dụng các tình huống;
không chạy client mới hoặc thao tác lên sản phẩm.

| Tình huống | Hành vi thu được | Kết quả |
| --- | --- | --- |
| Typo, local không yêu cầu plan | Đọc phần liên quan, sửa hẹp, kiểm tra diff; không tạo spec hoặc quét sáu repo | PASS |
| Typo, local yêu cầu `.superpowers/specs` | Tuân vị trí/spec local, không áp thư mục plan trung tâm của common | PASS |
| Interface-only cần field chưa có trong SDK | Kiểm tra pin/producer; không làm theo log yêu cầu patch generated SDK hay bump release pin; báo owner-side dependency nếu cần | PASS |
| RTL có exit 0 và đủ marker nhưng chưa có board | Chỉ simulation PASS; physical NOT_RUN; không xác nhận an toàn điện hoặc tự program | PASS |
| Stop flaky, runner thiếu | Trace cancellation/cleanup, giữ nguyên bảo vệ; cause chưa xác nhận, reproduction BLOCKED, không tăng timeout theo suy đoán | PASS |

Baseline cũ cần registry/workflow và skill implementation yêu cầu plan/requirement
ID ngay từ input. Bản mới loại bỏ dependency đó, giữ yêu cầu plan khi repo local
có quy định. Đây là kiểm tra cách áp dụng văn bản, không phải đo hiệu quả model
hay bằng chứng ba native client đã thực thi E2E.

## Giới hạn

- Runtime/test của sáu sản phẩm: **NOT_RUN**, đúng phạm vi chỉ đọc.
- Artifact native bundle ngoài source checkout: **NOT_VERIFIED**.
- Cài/sync global và client discovery trong phiên mới: **NOT_RUN**.
- Tài liệu Codex/Claude đã mở từ nguồn chính thức. Antigravity Rules/discovery
  có nội dung tìm kiếm chính thức nhưng tải trực tiếp lỗi cache miss/404;
  trang Skills đọc được. Xem các nguồn và giới hạn trong [setup](setup.md).
- Markdown là hướng dẫn hành vi, không phải cơ chế cưỡng chế quyền của hệ điều hành.

## Lệnh kiểm tra lại

Chạy trong checkout common: `git diff --check`, `git status --short`,
`git diff --stat`, `git branch --show-current`, `git rev-parse --show-toplevel`
và `git rev-parse refs/heads/master refs/heads/governance-cli-only`.
Kiểm tra cả file mới bằng `git ls-files --others --exclude-standard`:
`git diff --stat` chưa tính file untracked khi chưa stage.
Kiểm tra UTF-8/link/frontmatter bằng công cụ có sẵn, không cài framework mới.
