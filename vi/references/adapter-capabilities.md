<!-- doc-id: OPENRULES-ADAPTER-CAPABILITIES -->
<!-- revision: 1 -->
<!-- source-sha256: 81fee0e639a1506a7702487406f1350facf7329dd20c580b4871f1831bdbe4f8 -->
<!-- translation-status: synchronized -->
<a id="adapter-capabilities"></a>

# Khả năng tương tác với engine bên ngoài

OPENRULES có thể kiểm tra engine cục bộ do người dùng chọn mà chưa tạo phiên. Kết quả khớp xác nhận phía kết nối đã tự báo đúng phiên bản được ghim và dùng cấu trúc phản hồi dự kiến. Xác thực, sandbox, thực thi điều kiện phê duyệt và chạy agent cần bằng chứng riêng. Chức năng soạn thảo governance hoạt động ngoại tuyến.

## Khảo sát nguồn

| Gói | Số mục | Revision trong ZIP comment | SHA-256 |
| --- | ---: | --- | --- |
| OpenHands-main.zip | 3.277 | `1a66039ecd20d93e792eec257c7f5abf4d709a10` | `7e48061c8e1b3ff61f5382d427d37573d83c544f4bc517706d56542e297da88d` |
| opencode-dev.zip | 12.377 | `fee1ca5aa9e1d62a7ca42ef7eeaa8c02fe6ed080` | `2d25f85683c60484dfadc362269c3bc777b5bc330e0025ea468ecc14a8ba7f6d` |

Quá trình khảo sát đọc trực tiếp các tệp đã chọn trong hai gói. Giấy phép ở thư mục gốc của cả hai gói là MIT. OPENRULES triển khai tập con giao thức và các mẫu tương tác bằng Rust và Qt. Sản phẩm không nhúng mã nguồn, runtime hoặc tài nguyên giao diện của upstream. Tệp nhập giữ nguồn gốc và điều kiện cấp phép riêng.

| Cơ chế nguồn | Phân loại | Áp dụng và bằng chứng |
| --- | --- | --- |
| OpenHands `docs/architecture.md` và `src/components/features/skills/add-skill-modal.tsx` | REUSE_PATTERN | Tách desktop và engine. Hộp thoại Add Skill của upstream hướng dẫn thiết lập. OPENRULES dùng runtime và database riêng để soạn thảo. |
| OpenHands `src/api/skills-service.ts`, `src/utils/skill-enablement.ts` và `src/utils/skill-scope.ts` | REIMPLEMENT | Tách phạm vi và trạng thái kích hoạt trong thư viện khỏi nội dung. Xem trước khả năng tương thích ánh xạ metadata skill mà chưa kích hoạt. |
| OpenHands `src/api/agent-profiles-service/agent-profiles-service.api.ts` và `profile-behavior.ts` | REUSE_PATTERN | Dùng tham chiếu tài nguyên của profile. Thông tin xác thực và lựa chọn đang lưu không được tự chuyển vào engine đang chạy. |
| OpenHands `src/utils/plugin-launch-preflight.ts` | REUSE_PATTERN | Giữ kiểu dữ liệu và hiện trường chưa hỗ trợ. Fixture kiểm tra giá trị false và trường giữ ở dạng nguồn. |
| OpenHands `config/defaults.json`, `src/api/agent-server-compatibility.ts` và `docs/architecture.md` | CONNECT_VIA_ADAPTER | Kiểm tra Agent Server 1.60.1 với `GET /server_info`. Fixture bao phủ trường phiên bản dự phòng và phản hồi yêu cầu xác thực. |
| OpenHands `src/api/conversation-service/agent-server-conversation-service.api.ts` và `src/api/event-service/event-service.api.ts` | CONNECT_VIA_ADAPTER (bị chặn) | Khởi động phiên cục bộ cần agent profile upstream và workspace đã xác định. Sự kiện và phê duyệt cần quyền theo runtime. Khởi động phiên và streaming dưới governance còn bị chặn. |
| OpenCode `packages/protocol/src/groups/server.ts` và `packages/server/package.json` | CONNECT_VIA_ADAPTER | Kiểm tra OpenCode 2.2.5 V2 với `GET /api/info`. Fixture bao phủ cấu trúc V2, phiên bản sai và phản hồi thiếu cấu trúc. |
| OpenCode `packages/core/src/config/plugin/agent.ts`, `packages/core/src/v1/config/agent.ts` và `packages/schema/src/config/agent.ts` | REIMPLEMENT | Xem trước cấu hình agent profile thuộc V1 hoặc V2 do người dùng chọn. Test kiểm tra bảo toàn kiểu dữ liệu và phát hiện trường trộn giữa hai định dạng. |
| OpenCode `packages/core/src/skill.ts` và `packages/core/src/tool/plugin/skill.ts` | REUSE_PATTERN | Tách metadata khỏi nội dung và trì hoãn tải nội dung tới thao tác được cấp quyền. |
| OpenCode `packages/core/src/config/plugin/reference.ts`, `packages/core/src/config/plugin/skill.ts` và `packages/core/src/permission.ts` | REUSE_PATTERN | Tách phát hiện tệp, tham chiếu theo phạm vi và quyền theo hành động/tài nguyên. Xem trước khả năng tương thích không cấp quyền truy cập. |
| OpenCode `packages/app/src/settings/pages.ts` và `packages/app/src/session/revert-machine.ts` | REUSE_PATTERN | Tách điều hướng, trạng thái trình soạn và chuyển revision. Tua lại phiên khác với rollback governance. |
| OpenCode `packages/protocol/src/groups/session.ts`, `event.ts` và `permission.ts` | CONNECT_VIA_ADAPTER (bị chặn) | V2 tạo phiên với thông tin vị trí và quyền. Luồng có thể mất sự kiện khi ngắt kết nối. Tạo phiên dưới governance, đối soát sự kiện và kiểm soát phê duyệt công cụ còn bị chặn. |

Tài liệu công khai mô tả profile và cấu hình. Revision trong gói xác định tập con giao thức được triển khai. [Agent Profiles của OpenHands](https://docs.openhands.dev/openhands/usage/agent-canvas/agent-profiles) và [Cấu hình OpenCode](https://opencode.ai/docs/config).

## Khả năng và ranh giới tin cậy

| Tính năng | OpenHands | OpenCode | Codex, Claude, Antigravity |
| --- | --- | --- | --- |
| Kiểm tra phiên bản và cấu trúc phản hồi cục bộ | Khả dụng | Khả dụng | Chưa có |
| Xem trước nguồn ngoại tuyến | Frontmatter của SKILL.md | Đối tượng agent profile V1 hoặc V2 được chọn | Chưa ánh xạ |
| Handoff cục bộ | Chưa có adapter handoff | Chưa có adapter handoff | Handoff only |
| Tạo phiên, cấp context, gọi model/công cụ, phê duyệt, streaming, interrupt, resume, cách ly, diff, usage | Bị chặn | Bị chặn | Chặn thực thi native |
| Chạy engine thật đã kiểm chứng | Chưa chạy | Chưa chạy | Chưa chạy |

IPC riêng cung cấp `openrules.integration.list`, `openrules.integration.probe` và `openrules.integration.import_preview`. Probe nhận cổng dạng số và ID engine. Probe gửi GET chỉ đọc tới endpoint phiên bản cố định tại `127.0.0.1`. Worker riêng nhận một tác vụ, giới hạn socket tổng cộng 1,2 giây, header 8 KiB và body 32 KiB. Probe không theo chuyển hướng, dùng proxy, gửi thông tin xác thực hoặc đọc đường dẫn do server trả về. Trạng thái yêu cầu xác thực được báo riêng. Trình kiểm tra JSON chấp nhận phản hồi HTTP/1.x có độ dài nội dung khai báo hoặc body kết thúc khi kết nối đóng.

## Tập con định dạng ngoại tuyến

- Xem trước skill OpenHands yêu cầu frontmatter YAML của `SKILL.md` có `name` và `description`. Có thể trích xuất `license` và `compatibility`. Trường mở rộng được giữ ở nguồn.
- OpenCode chấp nhận JSON nghiêm ngặt hoặc Markdown có frontmatter YAML của một agent profile. Xem trước lựa chọn model dạng chuỗi, mô tả, chế độ và cờ tắt. Cờ `disable` của V1 được ánh xạ sang `disabled`. `permission` của V1 và `permissions` của V2 không cấp quyền OPENRULES. Trường của định dạng khác sinh chẩn đoán. Cấu hình toàn dự án, JSONC, lựa chọn model có cấu trúc, tham số provider và chỉ dẫn upstream nằm ngoài tập ánh xạ này.
- Báo cáo phân loại trường được hỗ trợ, được chuyển đổi, giữ dưới dạng nguồn hoặc bị từ chối. Các trường chính sách, thực thi và thông tin xác thực bị từ chối. Xem trước không tải tham chiếu, kích hoạt skill, lưu cấu hình hoặc khởi động tiến trình. Trình xử lý giới hạn dung lượng và số trường, từ chối alias, anchor, tag và directive YAML, đồng thời chỉ ra vị trí lỗi khi parser cung cấp.

## Trạng thái kiểm chứng

Fixture kiểm tra hai endpoint phiên bản qua loopback, lựa chọn định dạng, trường sai cú pháp hoặc trùng lặp, loại trừ trường nhạy cảm và dispatch stdio trong lúc probe phản hồi chậm. Fixture trả về dữ liệu dựng từ hợp đồng đã đọc. Fixture không chứa engine upstream.

OR-17 có fixture cho metadata và báo cáo tương thích. OR-40 và OR-41 có kiểm tra hợp đồng một phần cho probe và tập con profile đã hỗ trợ. Ánh xạ sự kiện, bắt tay với server thật và thực thi native chưa được xác minh. OR-42 và OR-43 chưa chạy vì môi trường kiểm thử thiếu hai engine đúng phiên bản và thiếu engine cách ly hoặc thông tin xác thực được phép sử dụng. OR-44 có kiểm tra danh mục capability. Hiển thị capability trên desktop được kiểm tra riêng.
