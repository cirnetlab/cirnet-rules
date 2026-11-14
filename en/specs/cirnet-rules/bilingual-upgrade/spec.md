# Bilingual documentation upgrade specification

Date: 2026-11-14. English is the instruction authority; Vietnamese is a human
review translation. This task supersedes the earlier single-language layout
decision, while preserving its scope, product ownership and runtime-free design.

## Requirements

- Write only the existing `rules-common` worktree, initially clean at
  `17f1738ccd4312f284b61372a3c507a65a26588a`. Do not stage, commit or alter global settings.
- Make `en/` canonical for agent instructions and maintained documentation.
  Mirror every English Markdown path under `vi/` for Vietnamese readers.
- Preserve portable root `AGENTS.md` as an exact distribution copy of
  `en/AGENTS.md`, and `CLAUDE.md` as `@AGENTS.md`. The root README is bilingual navigation.
- Add meaningful hooks, policies, plans and specs guidance alongside rules, skills,
  references, templates and docs. Keep exactly three optional executable skill definitions.
- Require agents to update both affected language documents after implementation,
  record verification and limits, and report incomplete translations honestly.
  Do not require unrelated product edits or a spec for every typo.
- Provide an optional repository-local, read-only hook to check matching paths,
  source hashes, Markdown links, portable entrypoints and skill metadata.
  The user explicitly approved this hook. No native/global hook installation.
- Keep old migration plans/specs and verification evidence as dated history.
  Link updates do not turn their recorded test outcomes into current results.
- Preserve LICENSE, existing ignore rules and third-party attribution.

## Design decisions

Use one English authoring source with a checked root distribution mirror.
Vietnamese documents carry source path and SHA-256 metadata. Hashes detect drift,
not translation accuracy: semantic review is required before refreshing a hash.
Agents read English by default and read Vietnamese only to translate or review it.
No bilingual YAML registry, runtime service, database or approval engine is needed.

The local Python hook is an optional maintainer check, not a dependency for reading
the rules. Hook documentation defines preflight, after-change and completion steps;
connecting it to any native client remains a separate user configuration action.

## Acceptance

Paired directories cover hooks, plans, specs, policies, references, rules, skills,
templates and docs. All source facts, commands, status labels and limitations match.
The checker and its negative-case tests pass, internal links resolve, and
`git diff --check` is clean. Native hook auto-execution is `NOT_RUN`.
No product repository is touched; current backup branches remain unchanged.
