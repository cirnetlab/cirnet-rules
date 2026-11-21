# Set up one client

Pick a reviewed local bundle and a single client. Keep the product repository's
applicable local instructions. The English baseline is portable and each
[Skill](../skills/README.md) is individually installable. Vietnamese copies are for
translation and review. See [context loading](context.md) for what to read on a task.

## Install from an extracted local bundle

Use PowerShell 7 and the scripts from the reviewed toolkit checkout. Set
`$bundle` to the extracted directory containing `bundle.json` and choose
`$agent` as `Codex`, `Claude` or `Antigravity`. The first command previews the
exact paths. After reviewing, the user can run the apply command.

```powershell
./scripts/install.ps1 -Source $bundle -Agent $agent
./scripts/install.ps1 -Source $bundle -Agent $agent -Apply
./scripts/verify-install.ps1 -Source $bundle -Agent $agent
```

The default selection is `cirnet-implementation`, `cirnet-debugging` and
`cirnet-verification`. Use the same `-Skills` array when previewing, applying
and verifying a different selection. Add optional names explicitly. Installer
preview and `-Apply -WhatIf` leave the destination untouched. For tests or custom
locations, pass `-HomeRoot` explicitly, plus `-CodexHome` when needed.
Use `-Skills @()` in PowerShell to select only the baseline and adapter.

`-Skills` sets the full selection. To keep the defaults and add Task Continuity,
Plan/Spec authoring and Single Candidate evaluation, preview, install and verify
with the same list.

```powershell
$skills = @(
    'cirnet-implementation', 'cirnet-debugging', 'cirnet-verification',
    'cirnet-task-continuity', 'cirnet-plan-spec-authoring', 'cirnet-candidate-evaluation'
)
./scripts/install.ps1 -Source $bundle -Agent $agent -Skills $skills
./scripts/install.ps1 -Source $bundle -Agent $agent -Skills $skills -Apply
./scripts/verify-install.ps1 -Source $bundle -Agent $agent -Skills $skills
```

| Client | User baseline destination | User skill directory |
| --- | --- | --- |
| Codex | `$CODEX_HOME/AGENTS.md` or `~/.codex/AGENTS.md` | `~/.agents/skills/` |
| Claude Code | `~/.claude/CLAUDE.md` imports `@cirnet/AGENTS.md` | `~/.claude/skills/` |
| Antigravity | `~/.gemini/AGENTS.md` | `~/.gemini/config/skills/` |

For Codex, explicit `-CodexHome` wins. A supplied `-HomeRoot` uses its
`.codex` by default and isolates the test or custom home from the ambient
`CODEX_HOME`. With neither option, the script uses the environment override
or the default. Codex's nonempty `AGENTS.override.md` takes precedence.
See [Codex instructions](https://developers.openai.com/codex/guides/agents-md)
and [Codex skills](https://developers.openai.com/codex/skills).

Claude resolves the managed import relative to the importing file. Check the
client's discovery behavior for your version and surface. See
[Claude imports](https://code.claude.com/docs/en/memory) and
[Claude skills](https://code.claude.com/docs/en/skills).

For Antigravity, check the App, CLI or IDE discovery and precedence. The cited
documentation describes the user destinations above and the workspace discovery
boundary. Read an FPGA nested instruction explicitly when the chosen surface
does not load it. See [Antigravity rules](https://antigravity.google/docs/rules/)
and [Antigravity skills](https://antigravity.google/docs/skills?app=antigravity).

Resolve home in the environment where the client runs, including WSL, container
and remote sessions. For project scope, adapt the
[local instructions template](../templates/AGENTS.md) with the repository owner
and use that client version's project skill conventions.

## Read an authorized local checkout

Ask the client to read the local checkout's root `AGENTS.md` followed by the
target repository's applicable instructions. Supply the path and grant read
access as needed. Select an English skill explicitly if useful. Repeat in new
sessions that lack a configured baseline.

For Codex or Claude Code tasks using Task Continuity, give the session the
approved toolkit checkout, owning worktree, Task ID and checkpoint location.
Use [Task Continuity setup and commands](distribution.md#task-continuity) to
select an ignored local store and recover a task after interruption.

## Check actual discovery

File verification checks selected destinations, managed content, references and
known user-scope conflicts. Other sources such as plugins and project overrides
need review in the actual client. Native discovery and task behavior have
separate results. Start a fresh session
in the client's actual surface, confirm baseline and skill discovery, then
try the read-only portion of a scoped task.

| Client | Manual observation |
| --- | --- |
| Codex | Check instruction chain and skill metadata in the actual CLI/App/IDE. Test override and nested scope in a fixture. |
| Claude Code | Confirm the single import resolves and the chosen skill is available in the intended project. |
| Antigravity | Confirm user rules and Skills in the chosen surface and request relevant nested context explicitly if needed. |

For all three, choose one relevant skill, check the local/nested rules, and
report the checks actually run. This task observed file behavior only in fake
homes. Native discovery and behavior are NOT_RUN. For update and restore, see
[distribution](distribution.md).
