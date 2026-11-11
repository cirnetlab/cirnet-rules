# CIRNET RULES

[English](en/README.md) · [Tiếng Việt](vi/README.md)

**CIRNETRULES** provides a Rust governance core, the CNR CLI and an MCP server over
STDIO. Run coding sessions independently in Codex Desktop, Claude Code or
Antigravity. Use CNR to bind the worktree, resolve the authorized task context,
verify changes and record evidence.
Use the binaries from `target/debug` or make them available on your path.
On Windows the executable is `cnr.exe`.

## Các lệnh kiểm tra

```sh
# kiểm tra định dạng mã nguồn
cargo fmt --all --check
# kiểm tra chất lượng code
cargo clippy --locked --all-targets -D warnings
# kiểm tra lỗi biên dịch
cargo check --locked --all-targets
# kiểm tra khả năng tạo tài liệu API
cargo doc --locked --no-deps
# kiểm thử với tất cả feature flags
cargo test --locked --all-features
# kiểm tra build phiên bản Release
cargo build --locked --release --bins
```

## Các lệnh xóa & build & kiểm thử chương trình

```sh
# xóa các artifact build
cargo clean
# build chương trình
cargo build --locked --bins
# chạy kiểm thử
cargo test --locked --all-targets
```

```powershell
# --locked: Không cho phép Cargo thay đổi dependency lockfile
# --all-targets: Kiểm tra tất cả targets như library, binary, tests, examples, benchmarks
# -D warnings: Chuyển tất cả warnings thành errors
# --bins: Chỉ build các binary executable được khai báo trong project
```

```sh
cnr governance validate
cnr governance inspect
cnr workspace discover
cnr agent list
```

[Operator workflow](en/references/cirnetrules-local.md) ·
[Hướng dẫn CNR](vi/references/cnr.md) ·
[Agent configuration](en/adapters/README.md) ·
[Legacy data recovery](en/references/cirnetrules-identity.md) ·
[Governance registry](governance.yaml)

[MCP STDIO](en/references/mcp.md) · [Shared task coordination](en/references/shared-task-coordination.md) · [Phối hợp task dùng chung](vi/references/shared-task-coordination.md)
