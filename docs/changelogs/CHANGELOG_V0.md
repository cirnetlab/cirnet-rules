# CIRNET Rules Toolkit version 0 changelog

[Changelog index](README.md)

## Unreleased

### Changed

- Document canonical architecture ownership and the English-only changelog policy.
- Verify major-version changelog structure and stale Remote architecture references offline.

## v0.2.0 — Candidate

### Changed

- Resolve local continuity and verification storage from the canonical repository identity.
- Retire internal Work Prompt authoring and run Candidate engineering scenarios directly. Work Prompt authoring now uses the ChatGPT Project’s `SKILL-en.md`.
- Add [owner-check preview, pinned stage evidence and bounded correction](../../en/docs/distribution.md#verification-evidence)
  for the existing verification workflow.

- Add the [proposed repository and module organization standard](../../en/rules/repository-organization.md)
  and reference it from the existing repository structure Skill.

- Add an [engineering workspace UI Rule](../../en/rules/engineering-workspace-ui.md)
  and [opt-in Skill](../../en/skills/cirnet-engineering-workspace-ui/SKILL.md) for
  artifact-centered editors and workbenches.

- Add three opt-in Skills for Task Continuity, Plan/Spec authoring and Single
  Candidate evaluation. Document how to install them alongside the defaults.

- Use one current Candidate session and one catalog for JavaScript, TypeScript,
  Python, Rust and SystemVerilog. Seal role evidence for independent quality
  review and keep previous A/B tooling at its recorded revisions.

- Center the toolkit on scoped Work Prompts, Lab-First ownership and verified
  code changes. Freeze checkpoint and large A/B tooling for manual use.
  Keep role diagnostics and retire benchmark performance scoring.

- Preserve safe verification stage protocols in filtered output; accept historical
  marker lines with numeric child statuses and reject error/signal false PASS.
  Cover both newline formats and keep old grading evidence immutable.

- Recover supplied original tasks inside authoring requests before declaring a
  missing goal; retain failed frozen candidates as separate benchmark evidence.
- Preserve explicit handoff operations and per-role context reuse. Prefer bounded
  structural inspection and retain privacy limits in blocked handoffs.
- Separate repaired candidate identities from historical B2 evidence. Extend
  escaped-value audit coverage and tie native acceptance to reviewed evidence.
- Add the opt-in Work Prompt authoring Skill and a bilingual template for
  handing scoped tasks to another agent.
- Add Lab-First architecture selection, repository ownership routing and an
  evidence gate for parallel development. Verify pinned EN/VI snapshots and
  routing with the existing offline checker and CI.
- Search for the owning document before creating Markdown. Review additions
  and fail unprotected confirmed duplicates through `check-docs` and the
  CI Git diff bridge.
- Clarify export authorization and safe source inspection for E01.
- Fix RTL subprocess scratch paths and standard I/O in MXC. Record actual
  compile and simulation exits and explicit physical status for E08.
- Support isolated native Codex authentication and portable tool paths. Add
  focused harness checks and consolidate [E01–E08 acceptance](../../en/docs/e01-e08-followup.md).
- Integrate four engineering behavior principles into the baseline and existing
  skills. Include on-demand Before/After examples and a synthetic Codex
  evaluation suite with offline harness checks.
- Shorten the standalone baseline and load skill bodies and references by task.
- Add 12 namespaced skills for implementation, review, compatibility and maintenance.
- Rename `implementation`, `debugging` and `verification` to their `cirnet-` names.
- Add local bundle packaging, managed installation, verification and backup restore.
- Discover skills dynamically in the Rust checker and validate portable resources,
  translation freshness and toolkit version agreement. Add toolkit CI.

Existing installations need a reviewed migration because the default skill directory
names have changed. See [distribution and upgrade](../../en/docs/distribution.md).

## v0.1.0 — Historical baseline

### Added

- The repository provided a bilingual baseline, three portable skills and a local documentation checker. The checker and tests moved from Python to Rust.

**Historical sources**

- [Earlier English candidate history](https://github.com/cirnetlab/cirnet-rules/blob/49e54ad0c392bfe01fb5a2c8a0aa6f1505c020ef/en/docs/changelog.md)
- [Earlier translation](https://github.com/cirnetlab/cirnet-rules/blob/49e54ad0c392bfe01fb5a2c8a0aa6f1505c020ef/vi/docs/changelog.md)
