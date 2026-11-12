<!-- doc-id: INDEX -->
<!-- revision: 5 -->
<a id="index"></a>

# CIRNET RULES

[Tiếng Việt](../vi/README.md) · [Agent entrypoint](AGENTS.md) · [Registry](../governance.yaml)

CIRNETRULES provides a Rust governance core, the `cnr` CLI and an MCP
server over STDIO. Run coding sessions independently in Codex Desktop,
Claude Code or Antigravity. Use the English registry and policies to
resolve task authority and the local task store to track work and evidence.

Open `cnr` in a terminal. Use `cnr check all` to inspect readiness and
`cnr setup agent` to select an Agent, repository and worktree. Review each
proposed write and confirm it with the required task authorization. Check
reports distinguish launch configuration, protocol evidence and native client
observations. [CNR operator guide](references/cnr.md)

| Area | Start here |
| --- | --- |
| Governance registry, policies and rules | [Registry](../governance.yaml), [policy and transitions](policies/governance.md), [architecture](rules/architecture.md), [coding](rules/coding.md), [security](rules/security.md), [documentation](rules/documentation.md) |
| Skills and profiles | [Skills](skills/), [role profiles](profiles/), [repository configuration export](adapters/README.md) |
| References and hooks | [References](references/), [command index](commands/README.md), [hook guidance](hooks/README.md), [tooling](tools/README.md) |
| Repository routing | [Workspace and bindings](routing/README.md), [repository ownership](profiles/product-architecture.md) |
| Task and permission control | [Operator workflow](references/cirnetrules-local.md), [shared coordination](references/shared-task-coordination.md), [agent handoff](references/handoff.md) |
| Verification and audit | [Verification guide](references/verification.md), [Rust tests](tests/README.md), [schemas](schemas/README.md) |
| CNR CLI and MCP server | [CLI reference](references/cnr.md), [MCP STDIO and launcher export](references/mcp.md) |

[Core plans](plans/cirnet-rules/) · [Core specs](specs/cirnet-rules/) ·
[Changelog](CHANGELOG.md) · [Legacy data recovery](references/cirnetrules-identity.md)
