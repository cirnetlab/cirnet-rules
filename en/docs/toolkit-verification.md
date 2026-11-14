# Toolkit upgrade verification

Date: 2026-11-14. Baseline revision `35e677b1264958839a7e0cb883c4477eb24da6df`.
Worktree branch: `rules-common`. The baseline checker passed and Cargo recorded
27 passing tests with one Windows directory symlink test ignored.

## Toolkit context contribution

Measured file bytes as UTF-8, words as whitespace-separated fields, and lines
as physical lines. Skill metadata includes parsed name and description values
only. Claude's adapter imports the baseline once. Its source is 11 bytes.

| Toolkit text | Before | Candidate |
| --- | --- | --- |
| Standalone common baseline | 5,811 bytes / 833 words / 91 lines | 2,444 bytes / 353 words / 42 lines |
| Metadata values for default three skills | 353 bytes | 302 bytes |
| Metadata values for all available skills | 353 bytes / 3 skills | 1,390 bytes / 12 skills |
| Optional fallback index | No dedicated file | 984 bytes / 77 words |

The candidate uses a single English baseline, optional metadata and selected
skill bodies. Walkthroughs inspected the typo, API and FPGA choices in
[context loading](context.md). Client context, tokenizer results and automatic
trigger selection need native observations. The table measures toolkit text.

## Local checks

Local environment was Windows with PowerShell 7.6.9 and Rust 1.99.1.

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` and `cargo clippy --offline --locked --all-targets -- -D warnings` | PASS |
| `cargo test --offline --locked` | 41 passed, 1 Windows symlink case ignored |
| `cargo run --offline --locked --bin check-docs` | PASS, 45 EN/VI pairs and 12 skills, no warnings |
| `pwsh -NoProfile -File tests/package.Tests.ps1` | PASS for exercised fixtures, Windows junction creation blocked |
| `pwsh -NoProfile -File tests/install.tests.ps1` | PASS for exercised fixtures, Windows junction creation blocked. Actual partial-write rollback restored bytes and timestamps |
| `git diff --check` and PowerShell script parsing | PASS |

An additional smoke run packaged a development ZIP, extracted it to a Unicode
path, verified the 15 allowlisted payload files and exercised preview, apply,
verify and restore in separate fake homes for Codex, Claude and Antigravity.
It covered the explicit Codex home option. The test data was allocated in
ignored worktree output. Review and tests also covered invalid receipt state,
bundle overlap, duplicate Markdown references and binary baseline input.

Clean release creation is NOT_RUN on this dirty candidate worktree. Negative
release tests reject missing or incorrect revision and dirty source. No hosted
CI, Linux, actual client discovery or physical/product execution was run.
The candidate has no tag or published release.

See [setup](setup.md) for manual client smoke cases and
[distribution](distribution.md) for bundle commands.
