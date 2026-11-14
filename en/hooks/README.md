# Repository hooks

These checkpoints support one coding agent; they do not orchestrate agents or
register task state. Read English instructions. Vietnamese counterparts are for
human review, not an additional context layer.

| Checkpoint | Action |
| --- | --- |
| Before a task | Confirm root/branch/status and write scope; read local rules and relevant English context |
| After implementation/build | Identify documentation impact; update English and Vietnamese together; align plan/spec progress and actual results |
| Before completion | Review meaning, refresh reviewed translation metadata, run the check, inspect diff and report limits |

## Optional local executable check

From the common-rules repository root, using an already installed Python 3.10+:

```powershell
python -B hooks/check_docs.py
python -B -m unittest discover -s tests -p 'test_*.py' -v
git diff --check
```

The script resolves its own repository root. A read-only `--root PATH` override
exists for validation fixtures or an explicitly chosen common-rules checkout.
It does not install tools, translate content, stamp hashes, modify Git or contact
the network. Unit tests write fixtures only under this repo's ignored
`.superpowers/bilingual-tests/`.

The check covers paired paths, translation source/hash freshness, UTF-8/LF,
whitespace, local Markdown links/fragments, the root baseline mirror, Claude
adapter and the three English skill definitions. Hash matching cannot prove meaning.
Exit 0 is PASS, 1 is a failed check, 2 is invalid input. A missing Python runtime
means the executable check is BLOCKED; manual review is not its executed PASS.

## Connect to an agent later

The command can be run explicitly by Codex, Claude Code or Antigravity when the
task maintains this repository. For native lifecycle automation, consult the
installed client's hook support, preview the exact configuration and preserve
existing settings before the user authorizes installation. This change installs
no native/global hooks; automatic invocation on each client is NOT_RUN.

Do not apply this checker to an arbitrary product repo: it expects the common-rules
en/vi layout and entrypoints. Product checks and test-output allocators remain local.
See the [documentation policy](../policies/documentation.md) for the update contract.
