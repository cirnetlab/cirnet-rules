# CIRNET Rules

[Tiếng Việt](../vi/README.md) · [Engineering baseline](../AGENTS.md) · [Changelog](docs/changelog.md)

## Architecture

- [Ecosystem](../docs/tai-lieu-kien-tuc-he-thong/cirnetlab-architecture-en.md) for organization boundaries and integration.
- [Lab-First](../docs/tai-lieu-kien-tuc-he-thong/cirnet-architecture-lab-first-en.md) for Lab execution and electrical safety.
- [FPGA](../docs/cirnetlab-FPGA-architecture-en.md) for board implementation.
- [Repository ownership](references/architecture-routing.md) and [source SHA-256 pins](../docs/architecture-sources.tsv).

## Local tools

Run the tools from a reviewed checkout. PowerShell 7 runs Task Continuity and
claim verification. Node runs owner check previews, retry decisions, and
static CAD checks. See [operating instructions](docs/operations.md).

## Documentation checks

Use Rust 1.88 or later. The read-only checker validates bilingual source stamps,
local links, version metadata, the baseline and Claude entrypoint, and pinned
architecture sources and owner routing.

```powershell
cargo run --offline --locked --bin check-docs
cargo test --offline --locked
cargo clippy --offline --locked --all-targets -- -D warnings
```

Use `--root PATH` after `--` to select the checkout. Use repeatable
`--new-file PATH` for duplicate review of new Markdown under the root,
`en/` or `vi/`. CI uses [check-changed-docs.ps1](../scripts/check-changed-docs.ps1)
with a local base commit to select added Markdown.

## Migration

Author Work Prompts in the ChatGPT Project with its `SKILL-en.md`. Update
external references to [AGENTS.md](../AGENTS.md), the applicable architecture,
or the retained operating instructions. Existing global client setups and
local checkpoints are managed separately.
