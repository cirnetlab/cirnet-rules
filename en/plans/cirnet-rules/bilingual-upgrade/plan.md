# Bilingual documentation upgrade plan

Date: 2026-11-14.
Spec: [bilingual upgrade](../../../specs/cirnet-rules/bilingual-upgrade/spec.md).

- [x] Confirm exact worktree, clean baseline and existing protected refs.
- [x] Choose English authority, Vietnamese human translations and optional local hook.
- [x] Relocate content to mirrored en/vi paths without losing source evidence.
- [x] Add hooks/policies/plan/spec guides and reusable templates.
- [x] Require paired documentation updates in baseline and implementation/verification skills.
- [x] Implement the read-only checker with failing-case regression tests.
- [x] Review translations and stamp exact English source hashes.
- [x] Run hook, unit tests, skill checks, link/whitespace checks and independent review.
- [x] Record current results separately from historical migration evidence.

Decisions: existing single-language layout is superseded by the new user request.
Keep a standalone root distribution mirror instead of making global use depend on links.
No native hook activation, global settings, products, stage, commit or release.
