# Bilingual documentation policy

English under `en/` is canonical. The same relative path under `vi/` is the
Vietnamese human translation. Agents read English by default; Vietnamese does not
add requirements or override English. Translation work necessarily reads both.

## Documentation lifecycle

1. **Search before creating.** Identify the purpose and owner. Search within
   the authorized repository by filename, title, heading, topic or issue,
   and follow index and owning-source links. Distinguish current guides, plans,
   specs, changelogs, verification, dated evidence, examples and fixtures.
2. **Update the canonical document.** Reuse the document that serves the same
   purpose and link to it from other pages. Update current behavior, reproduction
   steps, observed results and limits in place. Link dated evidence and preserve
   its original results. Create a file only for a distinct purpose, audience or
   feature, a repository requirement, or an independent evidence record. Report
   the reason and the incoming link for each new file.
3. **Propose consolidation.** Name the candidates, overlap and unique material.
   Explain which source should own the topic. Preserve requirements, decisions,
   examples, commands, versions, schemas, measurements, test status, safety limits and
   provenance. Plan link, index and EN/VI updates and continued access to
   history. Merge only within the task's write scope when the contents can be
   preserved. Review similarity warnings as evidence for investigation.
4. **Verify.** Review affected guides, API examples and plan/spec progress.
   Update English from observed results, then update the Vietnamese counterpart
   in the same change. Compare meaning, especially negation, permissions,
   unavailable capabilities, paths, numbers and
   PASS/FAIL/NOT_RUN/BLOCKED. Keep code identifiers and commands exact and refresh
   source hashes after review. Mirror any baseline edit at the root. Run the
   [local documentation check](../hooks/README.md) and `git diff --check`.
   Resolve errors, review warnings and report changed paths, decisions and limits.

Write from purpose through inputs, steps, verification and limits as needed.
Use links where another document owns the detail. Cut repeated prose while
keeping commands, acceptance criteria, edge cases and safety constraints.

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
