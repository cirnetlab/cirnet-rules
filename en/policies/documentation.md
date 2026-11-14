# Bilingual documentation policy

English under `en/` is canonical. The same relative path under `vi/` is the
Vietnamese human translation. Agents read English by default; Vietnamese does not
add requirements or override English. Translation work necessarily reads both.

## After implementation or build

1. Inspect the change's documentation impact: behavior, API, setup, commands,
   examples, limitations and current plan/spec progress. Read only affected context.
2. Update the English source from observed implementation and verification results.
   Translate the affected Vietnamese counterpart in the same change.
3. Compare meaning, particularly negation, permissions, unavailable capabilities,
   paths, numbers and PASS/FAIL/NOT_RUN/BLOCKED. Keep code identifiers and commands
   exact; translate explanatory text. Do not turn historical evidence into new PASS.
4. Refresh source metadata only after semantic review. Update the root baseline
   copy if `en/AGENTS.md` changed. Do not change the root Claude adapter.
5. Run the [local documentation check](../hooks/README.md), inspect the final diff,
   and report both updated paths plus verification and remaining limitations.

If a build changes no documented meaning, record that assessment and the actual
test result in the task report; do not manufacture edits merely to touch two files.
A translation-only typo may change just Vietnamese when English meaning remains
correct. Review the pair and retain its matching English source hash.

Missing required English/Vietnamese content is unfinished work. If access or scope
prevents synchronization, label the affected step BLOCKED, identify the missing
counterpart and continue independent authorized work. A passing build cannot
substitute for required documentation updates.

## Pair metadata

Every Vietnamese Markdown document ends with these two comments; replace the
example path and hash with the corresponding current English file and digest:

```text
<!-- source: en/policies/documentation.md -->
<!-- source-sha256: <64 lowercase hexadecimal characters> -->
```

Compute SHA-256 over the exact UTF-8/LF English bytes. For example, from repo root:

```powershell
(Get-FileHash -Algorithm SHA256 -LiteralPath .\en\policies\documentation.md).Hash.ToLowerInvariant()
```

The hook checks freshness, not translation quality. Never refresh a hash merely
to silence a stale-translation failure. There is no automatic stamping/translation
mode; edit the translation, review it, then update its metadata.

## Sources and exceptions

- `en/AGENTS.md` is the editable baseline; root `AGENTS.md` is an identical,
  self-contained distribution copy. Root `CLAUDE.md` contains `@AGENTS.md`.
- Root README is a bilingual navigation page. LICENSE and root third-party notices
  retain their legal/attribution role; do not duplicate or translate legal terms
  as though the translation replaced them. Git files and checker code are shared.
- All Markdown under en/vi, including skill docs and task records, is paired.
  Install only `en/skills/` definitions; `vi/skills/` is for humans.
- Move or retire a pair together and update its incoming links. Preserve historical
  evidence; clearly separate its original scope/date from current verification.
- Product repositories keep their own language and output policies. Common guidance
  grants no permission to create a bilingual tree or alter another repo.
