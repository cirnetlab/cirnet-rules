<!-- doc-id: CNR-MCP-STDIO -->
<!-- revision: 3 -->
<a id="stdio"></a>

# CIRNETRULES MCP over STDIO

Start the server with canonical absolute roots. Substitute real paths for
the placeholders. Quote a path with spaces as one CLI argument.

```sh
cnr mcp serve --governance-root AUTHORITY --workspace WORKSPACE --access read-only
cnr agent mcp-config preview-export --governance-root AUTHORITY --workspace WORKSPACE --binary CNR_EXECUTABLE --output NEW_BUNDLE --json
cnr agent mcp-config export --governance-root AUTHORITY --workspace WORKSPACE --binary CNR_EXECUTABLE --output NEW_BUNDLE --expect BUNDLE_PLAN_DIGEST --json
cnr agent mcp-config check --governance-root AUTHORITY --bundle NEW_BUNDLE --json
```

The output directory is separate from Git and native configuration locations.
Review the local files and their manifest. Merge the `codex.toml` table into
the desired Codex project configuration, use `claude.mcp.json` as a template
for Claude Code project MCP, or use `antigravity.mcp_config.json` in the
Antigravity MCP editor. The files contain local absolute paths and only
read-only launch arguments. Keep those machine-specific files out of Git.
New manifests bind the launcher to the CNR binary digest, Git checkout
identity, HEAD and governance snapshot. Export again when those inputs change.

Use **Agent setup** in CNR to preview project registration. The automatic
writer requires a confirmed worktree, a reviewed bundle from the running CNR
binary, the current task's exact file grant and operator confirmation.
Confirm native project trust and any server approval in the Agent separately.

| Agent | Project configuration | User scope and fallback |
| --- | --- | --- |
| Codex | `.codex/config.toml` in a trusted project | Merge `codex.toml` in the native client. Its effective config depends on host and `CODEX_HOME`. |
| Claude Code | `.mcp.json` | Merge `claude.mcp.json` through the installed client's MCP configuration. Claude Code controls project approval. |
| Antigravity | `.agents/mcp_config.json` with explicit CLI or IDE surface | Import `antigravity.mcp_config.json` in the installed surface's MCP manager for App, user scope or an unconfirmed surface. |

On the command line, preview with `cnr agent mcp-config install --agent AGENT
--repo REPO --bundle BUNDLE --target WORKTREE --task TASK --actor OPERATOR
--dry-run`. For Antigravity CLI or IDE, include `--surface cli` or
`--surface ide`. Review the exact file and launch template, then replace
`--dry-run` with `--yes --expect PLAN_DIGEST --trust-reviewed`. Inspect a
receipt with `cnr audit agent --receipt RECEIPT`. Preview rollback with
`cnr agent mcp-config rollback --agent AGENT --target WORKTREE --receipt
RECEIPT --task TASK --actor OPERATOR --dry-run` and confirm with the returned
digest. Changes to an existing `cirnetrules` entry or the installed file
block replacement and require review. Registration preserves other file
bytes. Written files and receipts have owner-restricted permissions.
Receipts retain the inserted fragment and file hashes for rollback.

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

<a id="task-context"></a>

## Task context and exact scopes

Read `cirnet_task_status` first. Supply its revision as `expected_revision`
to `cirnet_task_context`. The context requires current read authorization.
The CLI `cnr task context` and MCP return the same task metadata alongside
the routed source files.

- `primary_repo`, `spec_revision`, `descriptor_sha256` and `approval_epoch`
  identify the task and its authority.
- `task_scope` contains the complete `read_scope`, `write_scope`,
  `verification_scope` and `documentation_scope` maps. `acceptance` and
  `allowed_command_ids` come from the same descriptor.
- `execution_bindings` identifies verified bindings by repository, binding
  ID, identity and commit. A binding is marked `frozen` when the task
  recorded it for execution. Absolute checkout paths stay local.
- `authorization_summary` reports declared actions and expiration. The
  write assessment is `WRITE_NOT_AUTHORIZED`, `WRITE_REQUIRES_ACTIVE_TASK`
  or `WRITE_REQUIRES_GRANT_EVALUATION`. Run `cnr task grant-evaluate` for
  the exact path and actor before an edit.

The existing `scope` field remains the string `read-only context`.
The complete metadata counts toward the context byte budget. The CLI
reports this cost in `metrics.task_metadata_bytes`, including in delta
responses. Context returns `BLOCKED` when exact metadata cannot be
disclosed in full. For `cirnet_diff_check`, get the expected digest
from a current `cnr task diff-check` observation after the edit.

`cnr check mcp` probes the running CNR executable through private pipes with
the read-only profile. It checks both lifecycles, the exact six tools and
structured policy and repository calls. A supplied task also exercises
status and context at its reported revision. The probe has a deadline and
cleans up the processes it launched.

`STATIC_CONFIG_VALIDATED` covers generated files, `CONFIG_REGISTERED` covers
the matching project entry, and `MCP_PROTOCOL_VERIFIED` covers the CNR pipe
probe. `NATIVE_CLIENT_VERIFIED` requires a separate observed connection in
each installed Agent at the selected worktree.

[Coordination workflow](shared-task-coordination.md) ·
[Rust SDK](https://docs.rs/rmcp/3.6.0/rmcp/) ·
[MCP specification](https://modelcontextprotocol.io/specification/2026-07-28) ·
[Codex](https://developers.openai.com/codex/mcp) ·
[Claude Code](https://code.claude.com/docs/en/mcp) ·
[Antigravity](https://antigravity.google/docs/mcp)
