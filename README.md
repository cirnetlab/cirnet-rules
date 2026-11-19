# CIRNET Common Rules & Agent Skills Toolkit

[English](en/README.md) · [Tiếng Việt](vi/README.md) · [Changelog](CHANGELOG.md)

Use one common [baseline](AGENTS.md) and 13 task-specific Skills to turn user
requirements into scoped Work Prompts and verified code changes. Start with the
target repository’s applicable instructions and the CIRNET Lab-First architecture.

Dùng một [baseline chung](AGENTS.md) và 13 Skill theo nhiệm vụ để chuyển yêu cầu
người dùng thành Work Prompt có phạm vi và thay đổi mã đã kiểm chứng. Bắt đầu từ
chỉ dẫn áp dụng của repo đích và kiến trúc CIRNET Lab-First.

| Task / Nhiệm vụ | English | Tiếng Việt |
| --- | --- | --- |
| Read the task and choose relevant context / Đọc nhiệm vụ và chọn context | [Context](en/docs/context.md) | [Context](vi/docs/context.md) |
| Find the architecture and repository owner / Tìm kiến trúc và repo sở hữu | [Policy](en/policies/architecture.md) · [Routing](en/references/architecture-routing.md) | [Chính sách](vi/policies/architecture.md) · [Định tuyến](vi/references/architecture-routing.md) |
| Write a LIGHT, STANDARD or DEEP Work Prompt / Soạn Work Prompt phù hợp | [Authoring Skill](en/skills/cirnet-work-prompt-authoring/SKILL.md) · [Template](en/templates/chat-to-work-prompt.md) | [Skill biên soạn](vi/skills/cirnet-work-prompt-authoring/SKILL.md) · [Mẫu](vi/templates/chat-to-work-prompt.md) |
| Implement and verify within scope / Triển khai và kiểm chứng trong phạm vi | [Skills](en/skills/README.md) · [Testing](en/rules/testing.md) | [Skills](vi/skills/README.md) · [Kiểm thử](vi/rules/testing.md) |
| Set up Codex, Claude Code or Antigravity / Thiết lập client | [Setup](en/docs/setup.md) · [Distribution](en/docs/distribution.md) | [Thiết lập](vi/docs/setup.md) · [Phân phối](vi/docs/distribution.md) |

Run the core toolkit checks offline / Chạy phép kiểm cốt lõi ở chế độ offline

```powershell
cargo run --offline --locked --bin check-docs
cargo test --offline --locked
```

## Historical evidence / Bằng chứng lịch sử

The [Candidate benchmark](en/docs/behavioral-evaluation.md) verifies selected
scenarios in one session. Historical comparisons can be reproduced from their
pinned revisions. / [Benchmark Candidate](vi/docs/behavioral-evaluation.md)
kiểm chứng các ca đã chọn trong một phiên. Các phép so sánh lịch sử có thể
được tái lập từ revision đã pin.

[Benchmark history / Lịch sử benchmark](reports/work-prompt-benchmark-history.md) ·
[Historical Governance branch](https://github.com/cirnetlab/cirnet-rules/tree/governance-cli-only)

[License](LICENSE) · [Third-party notices](THIRD_PARTY_NOTICES.md)
