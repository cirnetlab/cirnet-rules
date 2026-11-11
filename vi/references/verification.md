<!-- doc-id: VERIFY-LOCAL -->
<!-- revision: 2 -->
<!-- source-sha256: 1a675ac049930a4336b74a7ef8d3ff7088defc9755c4b0543cad179cf171b8b5 -->
<!-- translation-status: synchronized -->
<a id="evidence"></a>

# Kiểm chứng và audit

Chạy các lệnh kiểm tra lõi từ thư mục gốc governance với Rust toolchain
đã pin. Build cả hai binary và đưa `cnr` vào PATH để chạy các lệnh CLI.

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --bins
cnr governance validate --json
cnr governance check-translations --json
```

Với mỗi task, kiểm tra diff theo các binding đã ghi. Với task phối hợp,
kiểm tra thêm bằng chứng phụ thuộc trước khi kiểm chứng. Xem audit của task
để đối chiếu quyết định, biên nhận và trạng thái thẩm quyền hiện tại.

```sh
cnr task diff-check --task ID --json
cnr task dependency check --task ID --json
cnr audit task --task ID --json
```

Theo [quy trình vận hành](cirnetrules-local.md) để kiểm chứng, review và
chấp nhận task. Ghi revision governance và source, lệnh, nền tảng, mã thoát,
digest của kết quả và giới hạn. Dùng `BLOCKED` khi thiếu đầu vào, quyền
hoặc công cụ bắt buộc.

[Quy trình CI cho ứng viên](shared-task-coordination.md) dùng bằng chứng di
động đã review. Việc client phát hiện chỉ dẫn và thực thi hook cần được kiểm
tra ngay trong client đã chọn. Xem [hướng dẫn adapter](../adapters/README.md),
[quy tắc kiểm thử](../rules/testing.md) và [hồ sơ kiểm chứng phối hợp](../specs/cirnet-rules/mcp-shared-coordination/verification.md).

[Audit sản phẩm](product-audit.md) và [biên nhận release bị chặn](../../en/references/evidence/check-release-blocked.json)
ghi lại đầu vào và quyết định sản phẩm tại thời điểm kiểm tra.
