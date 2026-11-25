# CIRNET Rules

[English](../en/README.md) · [Baseline kỹ thuật](../AGENTS.md) · [Nhật ký thay đổi](docs/changelog.md)

## Kiến trúc

- [Ecosystem](../docs/tai-lieu-kien-tuc-he-thong/cirnetlab-architecture-vi.md) cho ranh giới tổ chức và tích hợp.
- [Lab-First](../docs/tai-lieu-kien-tuc-he-thong/cirnet-architecture-lab-first-vi.md) cho thực thi Lab và an toàn điện.
- [FPGA](../docs/cirnetlab-FPGA-architecture-vi.md) cho triển khai bo mạch.
- [Quyền sở hữu repo](references/architecture-routing.md) và [mã SHA-256 của nguồn](../docs/architecture-sources.tsv).

## Công cụ cục bộ

Chạy công cụ từ checkout đã rà soát. PowerShell 7 chạy Task Continuity và
xác minh kết luận kiểm chứng. Node xem trước phép kiểm của bên sở hữu, đánh
giá thử lại và kiểm tra CAD tĩnh. Xem [hướng dẫn vận hành](docs/operations.md).

## Kiểm tra tài liệu

Dùng Rust 1.88 trở lên. Bộ kiểm tra chỉ đọc xác minh dấu nguồn song ngữ,
liên kết cục bộ, metadata phiên bản, baseline, điểm vào Claude, nguồn kiến
trúc đã ghim và bảng định tuyến bên sở hữu.

```powershell
cargo run --offline --locked --bin check-docs
cargo test --offline --locked
cargo clippy --offline --locked --all-targets -- -D warnings
```

Dùng `--root PATH` sau `--` để chọn checkout. Có thể lặp `--new-file PATH`
để kiểm tra trùng lặp cho Markdown mới ở thư mục gốc, `en/` hoặc `vi/`.
CI dùng [check-changed-docs.ps1](../scripts/check-changed-docs.ps1) cùng commit
gốc cục bộ để chọn Markdown mới.

## Di chuyển tham chiếu

Soạn Work Prompt trong ChatGPT Project với `SKILL-en.md` của dự án. Cập nhật
tham chiếu bên ngoài tới [AGENTS.md](../AGENTS.md), kiến trúc phù hợp hoặc
hướng dẫn vận hành còn dùng. Cấu hình client toàn cục và checkpoint cục bộ
hiện có được quản lý riêng.

<!-- source: en/README.md -->
<!-- source-sha256: fac38b328bde7bc8639cbf2b65a9b9f8cebfb4e5b7bc531d4706e4de417b7a20 -->
