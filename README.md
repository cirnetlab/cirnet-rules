# CIRNET RULES

[English](en/README.md) · [Tiếng Việt](vi/README.md)

**CIRNETRULES** provides a Rust governance core, the CNR CLI and an MCP server over
STDIO. Run coding sessions independently in Codex Desktop, Claude Code or
Antigravity. Use CNR to bind the worktree, resolve the authorized task context,
verify changes and record evidence.

```sh
cargo build --locked --bins
cnr governance validate
cnr governance inspect
cnr workspace discover
cnr agent list
```

Use the binaries from `target/debug` or make them available on your path.
On Windows the executable is `cnr.exe`.

[Operator workflow](en/references/cirnetrules-local.md) ·
[Hướng dẫn CNR](vi/references/cnr.md) ·
[Agent configuration](en/adapters/README.md) ·
[Legacy data recovery](en/references/cirnetrules-identity.md) ·
[Governance registry](governance.yaml)

[MCP STDIO](en/references/mcp.md) · [Shared task coordination](en/references/shared-task-coordination.md) · [Phối hợp task dùng chung](vi/references/shared-task-coordination.md)
