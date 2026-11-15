# Bilingual documentation upgrade specification

Start on 2026-11-14 from clean `rules-common` at
`17f1738ccd4312f284b61372a3c507a65a26588a`.

- Keep `en/` canonical and `vi/` paired for human readers. Review meaning before
  stamping English paths and SHA-256. Update both affected languages after edits
  and record verification limits or incomplete translations.
- Keep the standalone root `AGENTS.md` mirror, `CLAUDE.md` adapter, bilingual
  navigation and three optional skills for this phase. Pair guides, hooks,
  policies, plans, specs, references, rules, skills, templates and docs.
- Verify source pairs, links, entrypoints and skill metadata using the approved
  optional, read-only Python hook for this phase. Run negative tests and
  `git diff --check`. Native hook activation remains NOT_RUN.

Keep dated migration evidence, backup refs, LICENSE, ignore rules and attribution.
Scope covers the existing worktree. Product edits, global settings, Git staging
and commit, native hook installation and runtime infrastructure are excluded.
