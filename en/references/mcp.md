<!-- doc-id: CNR-MCP-STDIO -->
<!-- revision: 1 -->
<a id="stdio"></a>

# CIRNETRULES MCP over STDIO

Start the server with canonical absolute roots. Substitute real paths for
the placeholders. Quote a path with spaces as one CLI argument.

```sh
cnr mcp serve --governance-root AUTHORITY --workspace WORKSPACE --access read-only
cnr agent mcp-config export --governance-root AUTHORITY --workspace WORKSPACE --binary CNR_EXECUTABLE --output NEW_BUNDLE --json
cnr agent mcp-config check --governance-root AUTHORITY --bundle NEW_BUNDLE --json
```

The output directory is separate from Git and native configuration locations.
Review the local files and their manifest. Merge the `codex.toml` table into
the desired Codex project configuration, use `claude.mcp.json` as a template
for Claude Code project MCP, or use `antigravity.mcp_config.json` in the
Antigravity MCP editor. The files contain local absolute paths and only
read-only launch arguments. Keep those machine-specific files out of Git.

The server uses the official Rust MCP SDK `rmcp` 3.6.0 with Tokio 1.54.1. The
transport supports STDIO discovery for `2026-07-28` and the legacy
`2025-11-25` lifecycle. The tools are:

| Profile | Tools |
| --- | --- |
| Read-only | `cirnet_repositories_discover`, `cirnet_policy_lookup`, `cirnet_task_context`, `cirnet_task_status`, `cirnet_diff_check`, `cirnet_audit_read` |
| Coordinated adds | `cirnet_task_claim`, `cirnet_task_renew`, `cirnet_task_release`, `cirnet_task_report` |

STDOUT is reserved for protocol frames. Roots and the access profile are
fixed at launch. The service pins the Git admin and common directories, HEAD, registry and schemas at startup. Restart the server if they change. Tool calls
use bounded schemas and a fixed worker pool. A local session file prepared
by an authorized operator is required for coordinated access. Pass its
path with `--session-file` when launching the server. Keep the file private.
Use one file per authorized task session. The tool schema never carries its
token or lets a caller select an actor or an alternate root.

`STATIC_CONFIG_VALIDATED` describes generated files. `MCP_PROTOCOL_VERIFIED`
describes process-level wire tests. `NATIVE_CLIENT_VERIFIED` requires an
observed connection in the installed Agent. File ownership, OS write access
and native client sandbox settings remain separate controls.

[Coordination workflow](shared-task-coordination.md) ·
[Rust SDK](https://docs.rs/rmcp/3.6.0/rmcp/) ·
[MCP specification](https://modelcontextprotocol.io/specification/2026-07-28) ·
[Codex](https://developers.openai.com/codex/mcp) ·
[Claude Code](https://code.claude.com/docs/en/mcp) ·
[Antigravity](https://antigravity.google/docs/mcp)
