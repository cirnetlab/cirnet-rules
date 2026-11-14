<!-- doc-id: CNR-CODEX-NATIVE-SETUP-SPEC -->
<!-- revision: 1 -->
<a id="scope"></a>

# Codex project configuration scope

Target binding `codex-e2e-20261112-01` in `cirnet-interface` at `f1ff70b9f22554c0680b92679e9e44a5544247ff`. The exact setup read, write and verification paths come from the reviewed instruction preview and the project MCP destination.

- `.agents/skills/implementation/SKILL.md`
- `AGENTS.md`
- `docs/cirnetrules/en/AGENTS.md`
- `docs/cirnetrules/en/commands/README.md`
- `docs/cirnetrules/en/hooks/README.md`
- `docs/cirnetrules/en/policies/governance.md`
- `docs/cirnetrules/en/profiles/developer.md`
- `docs/cirnetrules/en/profiles/product-architecture.md`
- `docs/cirnetrules/en/references/cirnetrules-local.md`
- `docs/cirnetrules/en/routing/README.md`
- `docs/cirnetrules/en/rules/architecture.md`
- `docs/cirnetrules/en/rules/coding.md`
- `docs/cirnetrules/en/rules/documentation.md`
- `docs/cirnetrules/en/rules/security.md`
- `docs/cirnetrules/en/rules/testing.md`
- `docs/cirnetrules/en/skills/implementation/SKILL.md`
- `.codex/config.toml`

Review the `AGENTS.md` managed block and preserve its existing user bytes. The TOML edit is limited to the read-only `mcp_servers.cirnetrules` table. Global settings, approval rules and sandbox configuration are outside this scope. Generated instruction sources include governance boundary language, so the proposed risk is high and independent review remains required for acceptance. Use the registered `interface-diff` verification step and retain both mutation receipts. This spec and the plan document the configuration exercise; no separate post-acceptance documentation target is required.

[Plan](../../../plans/cirnet-interface/codex-native-setup-20261112-01/plan.md)
