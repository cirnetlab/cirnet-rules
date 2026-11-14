# Manual setup

Use **one chosen agent** in the target repository. The common repo is not read
automatically from GitHub or another checkout. Instructions take effect when
placed in the appropriate scope or explicitly read in a session with access.
There is no need to download rules over the network for every task.

The client documentation was checked on **2026-11-14**. Discovery can vary by
version, App/CLI/IDE, overrides and local/remote environment. These are instructions
for later installation; the original rebuild installed nothing into user scope.
Native loader/E2E for all three clients: **NOT_RUN**. See the
[original rebuild verification](rebuild-verification.md).

Within this repository, author the baseline at `en/AGENTS.md` and keep root
`AGENTS.md` as its identical distribution copy. User/global installation continues
to use that root copy. Supporting agent context lives under `en/`; `vi/` contains
translations for human review, not additional agent instructions. For the current
bilingual workflow and repository documentation hook, see the
[documentation policy](../policies/documentation.md). The hook is a repository
check; these instructions do not install it into a client's native lifecycle.

## A. Controlled copy/sync into user scope

1. Choose a reviewed checkout and revision. Read the common diff and local changes
   before use. Do not automatically pull or update from an unreviewed branch.
2. Inspect existing destination files, overrides, preferences and policies. Create
   a separately named backup with a date/time; confirm it is readable before editing.
3. Prepare a merged version in a temporary file and inspect its diff. Preserve
   personal preferences and remove superseded common content to avoid conflicting
   baselines. Apply only after review.
4. Preserve the product repo's `AGENTS.md`/adapter. Do not overwrite it with common.
5. Record the source commit, date and SHA-256 of the reviewed file in personal
   installation notes. For uncommitted source, explicitly record the working-tree
   hash, not just HEAD.
6. Open a new session in the target repo, inspect the instruction sources loaded
   and try a read-only task. If loading fails, use method B and check the
   documentation for the exact client.

The following commands are **read-only**, run from the common checkout. They do
not install configuration:

```powershell
git rev-parse HEAD
git status --short
git diff -- en/AGENTS.md AGENTS.md
Get-FileHash -Algorithm SHA256 -LiteralPath .\AGENTS.md
```

On Linux/macOS, use `sha256sum AGENTS.md` or `shasum -a 256 AGENTS.md` if already
available. There is no need to install a tool just to inspect a file.

### Codex

The default user scope is `~/.codex/AGENTS.md`; `CODEX_HOME` can change its location.
A nonempty `AGENTS.override.md` in the same scope takes precedence. Repo-local
instructions load along the path from the root to the working directory; inspect
any required nested instructions too. See the
[official AGENTS.md guide](https://developers.openai.com/codex/guides/agents-md)
and [prompting guide](https://developers.openai.com/cookbook/examples/gpt-5/codex_prompting_guide).

### Claude Code

Use `CLAUDE.md` as an explicit entry point; do not rely on `AGENTS.md` alone without
checking discovery in the version/configuration in use. Loader documentation can
vary between versions. The adapter still shares one content source.
In this checkout, [CLAUDE.md](../../CLAUDE.md) imports `@AGENTS.md`.
For user scope, place the reviewed common copy at `~/.claude/AGENTS.md`,
then merge this line into `~/.claude/CLAUDE.md`:

```text
@AGENTS.md
```

Relative imports resolve from the file containing the import. **AGENTS.md must be
beside CLAUDE.md**; copying the adapter alone points to a missing file. Preserve
existing preferences alongside the import without duplicating the entire common
baseline. When authorized to edit an adapter in the target repo, its import must
point to that repo's correct local instructions. No symlink is required.
See [Claude memory/imports](https://code.claude.com/docs/en/memory).

### Antigravity

The documentation checked specifies `~/.gemini/AGENTS.md` for global scope and
`AGENTS.md` for workspace scope. Do not create a duplicate `GEMINI.md` when the
client supports AGENTS. Check the exact surface and the client's Rules list; do
not assume all older versions have the same discovery. According to the discovery
documentation, opening the repo root does not automatically load nested
`fpga/AGENTS.md`: read it explicitly for FPGA work.
See [Rules](https://antigravity.google/docs/rules/) and
[Customization discovery](https://antigravity.google/docs/customization-discovery/).
During this check, content was readable through official search results; direct
fetches of the Rules/discovery pages returned cache miss/404. Native behavior has
not been verified.

### Operating-system locations

| Client | Windows, default home | Linux/macOS |
| --- | --- | --- |
| Codex | `%USERPROFILE%\.codex\AGENTS.md` | `~/.codex/AGENTS.md` |
| Claude | `%USERPROFILE%\.claude\CLAUDE.md` and adjacent `AGENTS.md` | `~/.claude/CLAUDE.md` and `~/.claude/AGENTS.md` |
| Antigravity | `%USERPROFILE%\.gemini\AGENTS.md` | `~/.gemini/AGENTS.md` |

In PowerShell, `$env:USERPROFILE` references the home directory; `%USERPROFILE%`
in the table is Windows path notation, not PowerShell syntax.
For WSL/container/remote use, identify the home of the environment running the
agent; do not assume it uses the Windows host configuration. Respect the active
client's overrides.

## B. Read a local checkout in the session

Do not change global configuration. Replace placeholders with actual local paths:

> Read `<common-checkout>/AGENTS.md` as the baseline, then read relevant local
> instructions in `<product-checkout>`. Edit only task files in the authorized
> product repo; the common checkout is read-only. Report any access failure.

In a layout with multiple worktrees, common may be at
`<repo-root>/Branch/cirnet-rules/rules-common/AGENTS.md`.
Grant file-read access through the client if necessary; read permission does not
grant write permission. Do not rely on a fixed PC's drive path or username.
This method must be repeated in a new session if common is not already in an
automatically loaded scope.

## Optional skills

The three directories `en/skills/implementation`, `en/skills/debugging` and
`en/skills/verification` each contain a single self-contained `SKILL.md`. Read the
path directly when needed, or review and manually install **the selected skill
directory**; installing all three is not required. Install only English skills
from `en/skills/`. Their `vi/skills/` counterparts are translations for human
review, not additional installable agent skills.

| Client | Documented user scope | Project scope |
| --- | --- | --- |
| [Codex skills](https://developers.openai.com/codex/skills/) | `~/.agents/skills/<name>/SKILL.md` | `.agents/skills/<name>/SKILL.md` |
| [Claude skills](https://code.claude.com/docs/en/skills) | `~/.claude/skills/<name>/SKILL.md` | `.claude/skills/<name>/SKILL.md` |
| [Antigravity skills](https://antigravity.google/docs/skills?app=antigravity) | `~/.gemini/config/skills/<name>/SKILL.md` | `.agents/skills/<name>/SKILL.md` |

Check support in the actual version/surface before installing. Generic names may
collide with existing skills: preview, back up and choose which version to use;
do not overwrite automatically. `en/skills/` in the common checkout does not
automatically appear in the product workspace. The baseline does not require
installing `en/rules/`, references or task records into global scope.

## Three usage scenarios

- **Fix a typo:** read the loaded baseline, local rules and relevant document
  passage; make a narrow edit and check links/diff. Do not scan six repos or create
  a separate plan unless local rules require a plan/spec in a specific location.
- **Change an API:** inspect the API source, canonical Contracts and the SDK version
  selected by the consumer. Do not edit schemas or generated SDK output in the
  consumer to bypass the owner; if another repo must change, report the dependency
  and write-permission boundary.
- **Change FPGA:** read `fpga/AGENTS.md`, verification guidance and relevant RTL.
  RTL PASS establishes only the simulated behavior tested; it does not certify
  electrical safety, board programming or physical firmware. Do not infer permission
  to apply power.
