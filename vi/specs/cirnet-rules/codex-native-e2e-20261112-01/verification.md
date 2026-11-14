<!-- doc-id: CNR-CODEX-NATIVE-E2E-VERIFICATION -->
<!-- revision: 5 -->
<!-- source-sha256: ae2450af596501f19b2e5ddc2ee21984ac2d68cb4f34c16f4865d5fd3b3682dc -->
<!-- translation-status: synchronized -->
<a id="evidence"></a>

# Kiểm chứng tích hợp Codex Desktop native

Lượt `codex-native-e2e-20261112-01` đã kiểm chứng thao tác sửa tài liệu từ Codex Desktop native. Governance bắt đầu từ `e293c7b39b17b642b0423d9153dc0753627e74ea`. Worktree Interface riêng dùng binding `codex-e2e-20261112-01` ở `f1ff70b9f22554c0680b92679e9e44a5544247ff`. Các đầu ra setup thuộc baseline của smoke. Hai thay đổi mục lục đang để ngoài staging.

| Giai đoạn | Trạng thái | Bằng chứng |
| --- | --- | --- |
| CP1: CNR, repository, build | PASS | Trình hướng dẫn CNR trên Windows hiển thị đúng binding và chạy Check. CNR thoát bình thường; các mục chưa cấu hình vẫn hiện cảnh báo hoặc chưa quan sát. Bản build cuối xác minh governance và binding. |
| CP2: chuẩn bị target và task | PASS | CNR xác minh worktree có tên và ghi phê duyệt của `mrben` cho hai phạm vi setup và sản phẩm riêng. Phiên sửa dùng revision 4 và approval epoch 2. |
| CP3: preview chỉ dẫn và MCP | PASS | Preview sync liệt kê 15 tệp mới và một cập nhật khối quản lý trong `AGENTS.md`. Mẫu MCP chỉ đọc hợp lệ, preview đề xuất tạo mục cấu hình dự án tại `.codex/config.toml`. |
| CP3: áp dụng sync và đăng ký | PASS | CNR áp dụng 16 đầu ra, giữ nội dung `AGENTS.md` ngoài khối quản lý, đăng ký mục MCP cấp dự án và đọc lại cả hai. Kiểm chứng `interface-diff` đạt. Receipt setup đã lưu và đang chờ nghiệm thu độc lập. |
| CP3: probe STDIO từ CNR | PASS | Binary mới khởi chạy server và xác minh giao thức chỉ đọc, gồm status và context của task smoke thật. Kết quả ghi `native_client: NOT_RUN`. |
| CP4: lời gọi MCP và task context qua Desktop | PASS | Nhật ký phiên Local của Interface ghi PASS cho mười lời gọi `cirnetrules` thuộc cả sáu tool chỉ đọc. Phần context được lưu cho thấy revision 4, đúng binding và hai đường dẫn ghi. |
| CP4: tự nạp chỉ dẫn và phát hiện skill | PASS | Trước prompt đầu tiên, nhật ký Desktop chứa toàn văn `AGENTS.md` khớp tệp ở target và metadata của skill `implementation` trong đúng worktree. |
| CP4: gọi skill | NOT_VERIFIED | Phiên đã đọc toàn bộ `SKILL.md` bằng shell. Prompt đầu cũng yêu cầu bằng chứng về skill nên chưa tách được việc chọn ngầm định. Chưa thử chọn skill trực tiếp trong ô soạn. |
| CP5: sửa từ Desktop, verification và audit | PASS | Lệnh host đã duyệt xác nhận grant cho hai mục lục và chặn `AGENTS.md`. Sự kiện sửa tệp của Desktop ghi một dòng thay thế ở mỗi tệp. Cả hai tệp khớp bytes mong đợi và giữ CRLF. Snapshot mới của CNR khớp digest trong lần gọi MCP kiểm tra diff sau sửa. `interface-diff` đạt và audit cục bộ xác nhận receipt ở revision 6. |
| CP6: hồi quy Rust và build | PASS | Format, check, Clippy chặn warning, build debug và release, và test mọi target đều đạt: 266 test qua, một test cần `uv`/Python bên ngoài được bỏ qua. |

Các test hồi quy kiểm tra tạo proposal setup không có grant, xung đột phương án tài liệu, chặn khi drift hoặc thiếu trust, bảo toàn mục cấu hình dự án ngoài phạm vi, scope context chính xác, chặn capability, giới hạn phản hồi, revision cũ và digest của scoped diff. Context dùng chung vẫn yêu cầu grant đọc hiện hành. Trên Windows, Git cách ly chỉ đọc giá trị `core.autocrlf` do trình cài đặt cung cấp khi repo chưa có cấu hình riêng. Thay đổi nội dung thật vẫn được phát hiện và fingerprint dùng bytes gốc.

Shell Desktop mặc định báo lỗi đọc authority khi chạy CLI để kiểm tra grant và diff. Cùng binary đã pin hoàn tất các phép kiểm tra chỉ đọc khi host phê duyệt từng lệnh. Phép thử từ sandbox governance mở được tệp ở Interface nhưng Git bị từ chối khi chuyển thư mục sang Interface. Trên host, Git xác minh được cả hai repo. CNR dùng thông báo stale hoặc unreadable khi không tính lại được authority. Nhật ký shell Interface chưa ghi lệnh con hoặc tệp nào thất bại đầu tiên. Lời gọi audit native diễn ra trước verification. Audit CNR cuối xác nhận receipt và task smoke đang `VERIFIED`, chờ review.

Checkpoint cục bộ tại `target/codex-native-e2e-20261112-01/checkpoint/resume.md` dẫn đến báo cáo có cấu trúc, patch thực tế, trace Desktop đã lọc và receipt kiểm chứng. [Phạm vi setup](../../cirnet-interface/codex-native-setup-20261112-01/spec.md) · [Phạm vi smoke](../../cirnet-interface/codex-native-smoke-20261112-01/spec.md)
