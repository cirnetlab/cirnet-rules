# Original rules-common rebuild verification

Historical snapshot of the original rebuild, checked on **2026-11-14**. Its counts,
results and decisions below describe that rebuild, not the later bilingual upgrade
or the current working tree. For the current bilingual workflow, see the
[documentation policy](../policies/documentation.md).

Scope: the `rules-common` working tree of `cirnetlab/cirnet-rules`; read-only access
to the six products listed in the [repository map](../references/repository-map.md).

## Acceptance status

| # | Criterion | Result |
| --- | --- | --- |
| 1 | Correct root/branch; backup refs preserved | PASS |
| 2 | Standalone AGENTS; CLAUDE imports the correct file | PASS |
| 3 | README/setup state the correct scope and discovery limits | PASS documentation review; native NOT_RUN |
| 4 | Six repositories have actual source, SHA and ownership evidence | PASS source review |
| 5 | Four topic guides and exactly three optional skills | PASS |
| 6 | No Governance runtime/CI remains | PASS |
| 7 | UTF-8, frontmatter, internal links and diff whitespace | PASS |
| 8 | No changes introduced in the six products | PASS HEAD/branch/status comparison after legacy removal |
| 9 | Three instruction-reading scenarios | PASS tabletop review; see below |
| 10 | Native loader/E2E for Codex, Claude and Antigravity | NOT_RUN for all three |

## Changes and scope

Initially there were 290 tracked files, a clean status and no reported untracked
or ignored files. **284 tracked legacy files** were removed after checking that
each was unmodified and the backup tree matched HEAD. LICENSE, .gitattributes and
.gitignore were preserved byte for byte. AGENTS.md, README.md and
THIRD_PARTY_NOTICES.md were rewritten; an adapter, guides, skills, template, setup,
map and central English–Vietnamese rebuild records were added.

No Cargo/CNR or product test suite was run. No staging, commit, push, merge,
fetch/pull, PR creation, tool/global installation or device energization occurred.
No operations were performed on other cirnet-rules checkouts, the root-level
rules-common repo, cirnet-cli or other projects. Product Git status used
`GIT_OPTIONAL_LOCKS=0` to avoid refreshing the index.

HEAD, `master` and `governance-cli-only` before and after:
`646afa38516bbfd276b0aecb7a138ab427df5fa9`.
The six products' SHAs and statuses are recorded in the repository map; all were
on `master`, clean both at the initial read and when compared after legacy removal.

## Checks executed

- Available standard Python, `python -B -X utf8 -`: checked **22 remaining files**,
  **19 UTF-8/LF Markdown files**, **34 resolving internal links**, a **74-line /
  4,666-byte** baseline, an exactly one-line adapter and three skills with no
  external file dependencies.
- `quick_validate.py` bundled with skill-creator: **3/3 skills valid**, exit 0;
  no additional package was installed.
- Read-only PowerShell `Test-Path` and `git cat-file -e`: **92 SHA-pinned source
  links** matched local files and Git objects in the six products. GitHub was not
  queried to read source.
- Existence checks: no src, migrations, Rust manifests/lockfile, governance
  registry, cnr config, old CI or application assets remained.
- SHA-256: LICENSE, .gitattributes and .gitignore were unchanged. Ref/root/branch
  checks and `git diff --cached --name-only` confirmed the correct scope and no
  staged changes.
- `git diff --check`: exit 0; searches for legacy strings in the remaining files
  found only historical content, scope limitations or product data, with no
  required runtime.
- Independent review found an overly absolute Claude discovery claim; it was
  changed to version/configuration-specific guidance while preserving the
  compatible adapter and requiring a client check. The reviewer found no other
  significant issue. These checks do not establish native E2E, deployment status
  or physical safety.

## Tabletop checks

An independent reviewer read the baseline and all three skills, then applied the
scenarios without launching new clients or operating on the products.

| Scenario | Resulting behavior | Result |
| --- | --- | --- |
| Typo, no local plan requirement | Read the relevant section, make a narrow edit and check the diff; no spec creation or six-repo scan | PASS |
| Typo, local `.superpowers/specs` requirement | Follow the local location/spec requirement, without imposing common's central plan directory | PASS |
| Interface-only change needs a field absent from the SDK | Check the pin/producer; disregard logs asking to patch the generated SDK or bump a release pin; report an owner-side dependency if needed | PASS |
| RTL has exit 0 and all markers but no board | Simulation PASS only; physical NOT_RUN; no electrical-safety confirmation or automatic programming | PASS |
| Flaky Stop, missing runner | Trace cancellation/cleanup and preserve protections; cause unconfirmed, reproduction BLOCKED; no speculative timeout increase | PASS |

The old baseline required a registry/workflow, and the implementation skill
required a plan/requirement ID in the initial input. The rebuilt version removed
that dependency while retaining plan requirements imposed by the local repo.
This checks interpretation of the text, not model performance or evidence that
the three native clients executed E2E.

## Limitations

- Runtime/tests in the six products: **NOT_RUN**, consistent with read-only scope.
- Native bundle artifacts outside the source checkout: **NOT_VERIFIED**.
- Global installation/sync and client discovery in a new session: **NOT_RUN**.
- Codex/Claude documentation was opened from official sources. Antigravity
  Rules/discovery had official search content, but direct fetches failed with
  cache miss/404; the Skills page was readable. See sources and limitations in
  [setup](setup.md).
- Markdown provides behavioral guidance, not an operating-system permission
  enforcement mechanism.

## Recheck commands

Run in the common checkout: `git diff --check`, `git status --short`,
`git diff --stat`, `git branch --show-current`, `git rev-parse --show-toplevel`
and `git rev-parse refs/heads/master refs/heads/governance-cli-only`.
Also inspect new files with `git ls-files --others --exclude-standard`:
`git diff --stat` does not include untracked files before staging.
Check UTF-8/links/frontmatter with available tools; do not install a new framework.
