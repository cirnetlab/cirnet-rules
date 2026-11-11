<!-- doc-id: adapters -->
<!-- revision: 4 -->

# Agent bootstrap adapters

[Tiếng Việt](../../vi/adapters/README.md)

<a id="adapter-scope"></a>
## Scope

The Rust bootstrap command prepares a reviewable handoff for one CIRNET
checkout, task and risk level. The context resolver identifies the product
from its Git remote and binds it to full product and governance commit IDs.
The adapter rechecks each selected English document digest before staging.
Its receipt records bytes supplied to the operator. Native discovery, hook
execution and Agent compliance require separate evidence.

Use an existing dedicated absolute session directory outside Git checkouts
and native Agent configuration directories. Only the operator running the
adapter should be able to write to the session or rename its path. The
adapter assumes that no other process can redirect that path while a command
is running. Supply read access to the selected checkout and English sources
through the harness permission system. Set product and governance access to
read-only for product audit tasks. Treat Markdown and hooks as advice layered
on filesystem and CI permissions.

<a id="adapter-lifecycle"></a>
## Isolated lifecycle

From the governance checkout, set `CIRNET_AUDIT_WORKSPACE` to the absolute
directory that contains the product checkouts and `CIRNET_ADAPTER_SESSION`
to an existing isolated session directory. Use `claude` or `antigravity` in
place of `codex` for the other surfaces.

```powershell
cargo run --locked --bin cirnet-governance -- bootstrap --workspace "$env:CIRNET_AUDIT_WORKSPACE" --repo cirnet-remote --task audit --risk normal --agent codex --session "$env:CIRNET_ADAPTER_SESSION" --dry-run
cargo run --locked --bin cirnet-governance -- bootstrap --workspace "$env:CIRNET_AUDIT_WORKSPACE" --repo cirnet-remote --task audit --risk normal --agent codex --session "$env:CIRNET_ADAPTER_SESSION" --action install
cargo run --locked --bin cirnet-governance -- bootstrap --workspace "$env:CIRNET_AUDIT_WORKSPACE" --repo cirnet-remote --task audit --risk normal --agent codex --session "$env:CIRNET_ADAPTER_SESSION" --action uninstall
```

1. Resolve a context for the target checkout. Review the Git identities,
   English document list and risk scope. Run bootstrap in dry-run mode and
   confirm the managed destination and expected filenames.
2. Install only into the approved external session. Review the generated
   entry file, `context.json` and `receipt.json`, then explicitly supply the
   entry and context to the selected Agent. An identical install is
   idempotent. A different or modified active bundle blocks replacement.
3. Uninstall moves an intact managed bundle to a content-addressed backup
   beside the former active directory. Reinstalling the same context
   restores that backup. Unrecognized files, changed receipts and backup
   collisions block the operation so the operator can inspect them.
4. Stop on source drift, a denied read or an unavailable harness. The
   adapter preserves partial stages after an install fails. Review a
   retained stage or stale lock manually and use a fresh isolated session
   to resume while preserving that evidence.

<a id="config-export"></a>
## Repository configuration

Use a separate bundle directory outside Git, with an existing parent.
Select the target worktree explicitly. The examples use path placeholders.

```sh
cnr agent export --repo cirnet-interface --agents codex,claude,antigravity --output NEW_BUNDLE --json
cnr agent check --bundle NEW_BUNDLE --json
cnr agent sync --bundle NEW_BUNDLE --target BOUND_WORKTREE --dry-run --json
cnr agent sync --bundle NEW_BUNDLE --target BOUND_WORKTREE --yes --expect PLAN_DIGEST --task TASK --actor OPERATOR --json
cnr audit agent --receipt RECEIPT --json
cnr agent rollback --target BOUND_WORKTREE --receipt RECEIPT --dry-run --json
cnr agent rollback --target BOUND_WORKTREE --receipt RECEIPT --yes --expect ROLLBACK_DIGEST --task TASK --actor OPERATOR --json
```

Codex and Antigravity share repository instructions in `AGENTS.md` and
skills in `.agents/skills`. Claude Code uses `CLAUDE.md` and
`.claude/skills`. The manifest records byte digests and target capabilities.
Keep bundle output outside Git. Re-export if the selected source or binding
changes. Dry-run reports conflicts with existing user files. Applying changes
requires a current task with exact file-level write grants. Rollback verifies
the managed bytes before reversing the latest receipt. Writes are atomic per
file. After an interruption, inspect the receipt before retrying or rolling
back. Rollback may leave empty parent directories.

Bundle checks provide static evidence. Native discovery requires observing
the Agent load the instructions and skills in the target worktree. Neither a
bundle nor a portable handoff changes native permissions or registers hooks.

<a id="adapter-support"></a>
## Surface support and verification

Official surface documentation was checked on 2026-11-08. The portable Rust
adapter covers generation, dry-run, install, rollback and restoration for
all three entry filenames. Native instruction discovery, live skills, hooks
and model sessions were **NOT_RUN**. Local capability observations are
separate from product verification.

| Surface | Documented discovery and practical handoff | Local observation |
| --- | --- | --- |
| Codex CLI and desktop | Codex discovers scoped `AGENTS.md` files. Skills and hooks have separate discovery and trust rules. Supply the generated `AGENTS.md` and context explicitly. For CLI audit sessions use a read-only sandbox. The documented `--add-dir` grants extra write scope, so keep it out of this audit flow. | CLI `0.167.0-alpha.3.2` reported its version. The probe encountered sandbox-denied temporary alias preparation. Native session loading was **NOT_RUN**. |
| Claude Code | Native `CLAUDE.md`, skills and hooks have distinct settings and discovery. `--add-dir` exposes additional paths and loading their instructions is separately controlled. Supply the generated `CLAUDE.md` and context explicitly. In a compatible CLI use the documented `--append-system-prompt-file` flag with the generated entry and confirm file permissions. | CLI executable unavailable on `PATH`. Native loading was **NOT_RUN**. |
| Antigravity Desktop, IDE and CLI | Google documents workspace rules in `AGENTS.md` or `GEMINI.md`, skills and hooks by surface. Explicitly supply the generated `GEMINI.md` and context in the relevant session. Current CLI docs describe `agy` and `--add-dir`. Review permissions before adding a directory. | Installed Desktop binary reports `1.23.2`. The `antigravity` launcher is the Electron editor CLI and `agy` was unavailable on `PATH`. Native loading was **NOT_RUN**. |

Codex references. [Instructions](https://learn.chatgpt.com/docs/agent-configuration/agents-md) ·
[Skills](https://learn.chatgpt.com/docs/build-skills) · [Hooks](https://learn.chatgpt.com/docs/hooks) ·
[CLI reference](https://learn.chatgpt.com/docs/cli/reference).

Claude Code references. [Instructions and additional directories](https://code.claude.com/docs/en/memory) ·
[Skills](https://code.claude.com/docs/en/skills) · [Hooks](https://code.claude.com/docs/en/hooks) ·
[CLI reference](https://code.claude.com/docs/en/cli-reference).

Antigravity references. [Rules](https://antigravity.google/docs/rules/) ·
[Customization discovery](https://antigravity.google/docs/customization-discovery?tab=cli) ·
[Hooks](https://antigravity.google/docs/hooks?tab=ide) ·
[Permissions](https://antigravity.google/docs/permissions?tab=cli).

For cloud sessions provision the approved source checkouts in the remote
environment and resolve context there. Local paths identify only the machine
that generated them. Managed user-wide rules and hook installations require
a separate authorized rollout and an isolated native discovery test.

<a id="mcp-launch"></a>

## MCP launch templates

The separate [MCP export](../references/mcp.md) produces reviewable launcher
templates. Instruction and skill sync keeps its existing ownership contract.
