# CIRNET Common Rules & Agent Skills Toolkit

[English](en/README.md) · [Tiếng Việt](vi/README.md) · [Changelog](CHANGELOG.md)

One short baseline, 13 task-specific skills, and a local bundle for one chosen
client. Start with [AGENTS.md](AGENTS.md), then choose relevant English guidance
using [context loading](en/docs/context.md).

Một baseline ngắn, 13 skill theo nhiệm vụ và bundle local cho một client được
chọn. Xem [nạp context](vi/docs/context.md) và [thiết lập](vi/docs/setup.md).

- Setup / Thiết lập — [EN](en/docs/setup.md) · [VI](vi/docs/setup.md)
- Chat-to-Work Prompt / Prompt bàn giao — [EN](en/templates/chat-to-work-prompt.md) · [VI](vi/templates/chat-to-work-prompt.md)
- Package and restore / Đóng gói và restore — [EN](en/docs/distribution.md) · [VI](vi/docs/distribution.md)
- Architecture and ownership / Kiến trúc và chủ sở hữu — [EN](en/policies/architecture.md) · [VI](vi/policies/architecture.md)
- Verification / Kiểm chứng — [EN](en/docs/toolkit-verification.md) · [VI](vi/docs/toolkit-verification.md)

```powershell
cargo run --offline --locked --bin check-docs
cargo test --offline --locked
```

[LICENSE](LICENSE) · [Third-party notices](THIRD_PARTY_NOTICES.md) ·
[Historical Governance branch](https://github.com/cirnetlab/cirnet-rules/tree/governance-cli-only)
