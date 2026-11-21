---
name: cirnet-plan-spec-authoring
description: Use when drafting or revising a CIRNET architectural specification, implementation plan, or its acceptance criteria.
---

# Author a CIRNET Plan or Spec

Keep each document in the repository that owns its requirements or work.

| Record | Owns |
| --- | --- |
| Spec | Requirements, observable behavior, invariants, constraints and measurable acceptance criteria |
| Plan | Implementation sequence, dependencies, milestones and verification steps |
| Work Prompt | A specific authorized execution assignment for a Coding Agent |
| Task Continuity | Progress, decisions, evidence and recovery state |

1. **Find the owner and authoritative documents.** Check applicable local
   instructions, ownership guidance and existing Plans, Specs and incoming
   links. Identify the relevant document versions and approved scope before
   drafting. Update the authoritative document in place when its purpose
   matches. Use the owner's document conventions for a distinct deliverable.
2. **Write at the right level.** In the Spec, define what must hold, including
   negative cases and acceptance criteria with observable expected outcomes.
   In the Plan, link to the Spec and order the work by dependency. Give each
   milestone a verification step and map each acceptance criterion to the
   owner check and expected result that can establish it. Use
   `cirnet-work-prompt-authoring` for a requested execution assignment and
   `cirnet-task-continuity` for an interrupted task or checkpoint handoff.
3. **Review the change.** Compare the draft with approved requirements,
   architecture, scope and acceptance criteria. Obtain authorization for any
   proposed change to those decisions before applying it. Keep verification
   results traceable to the criterion, source revision and observed evidence.
   Synchronize applicable translations under the owner's policy.

Use the applicable revision in the approved checkout.
Plan conventions (`en/plans/README.md`) ·
Spec conventions (`en/specs/README.md`) ·
Architecture policy (`en/policies/architecture.md`) ·
Documentation lifecycle (`en/policies/documentation.md#documentation-lifecycle`)
