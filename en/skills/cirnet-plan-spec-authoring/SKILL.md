---
name: cirnet-plan-spec-authoring
description: Use when drafting or revising a CIRNET architectural specification, implementation plan, or its acceptance criteria.
---

# Author a CIRNET Plan or Spec

Use this workflow with or without Superpowers. Identify the repository owner,
authorized scope, approved requirements and applicable CIRNET rules.

1. For a nontrivial execution task, initialize Task Continuity before edits.
   Capture the original task and keep the existing headings in `task.md`.
   Put the design decisions, ordered steps and dependencies under
   `Plans & Specs`. Include affected files, contracts, constraints and expected
   behavior, including failure cases.
2. Give each milestone a checkable outcome. Map acceptance criteria to owner
   checks and required evidence. Record completed and pending work, blockers,
   next action and observations in `progress.md`. Publish through Task
   Continuity. Review any change in scope or acceptance before applying it.
3. Keep independently useful specifications and decisions with their owners
   and pin the relevant revision and hash. If Superpowers needs a stable
   input, derive it from the existing task outside sealed snapshots and
   reconcile it after approved changes.

Use the applicable revision in the approved checkout.
Task Continuity and Superpowers (`en/docs/distribution.md#task-continuity`) ·
Documentation lifecycle (`en/policies/documentation.md#documentation-lifecycle`)
