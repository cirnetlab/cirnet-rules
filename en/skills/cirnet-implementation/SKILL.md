---
name: cirnet-implementation
description: Use when implementing a scoped feature or behavior change with an agreed outcome.
---

# Implementation

Use for an agreed change. Use debugging first when the cause is unknown.

Minimum inputs: Authorized files, intended behavior, owning interfaces and nearby tests.

1. Inspect the affected flow, direct callers, owning contracts and tests. State
   assumptions that affect behavior, compatibility, data or safety. Resolve
   material ambiguity or missing permission before editing the affected part.
   Proceed with routine, reversible assumptions where the task allows.
2. Write observable acceptance criteria. Follow local planning rules and use
   `Step → check` for nontrivial work.
3. Make the smallest coherent change in the existing flow. Keep required failure
   handling and safety controls. Generate derived output from its owning input.
4. Add meaningful coverage where warranted. Update affected documentation pairs
   and run the checks tied to acceptance.

Report the diff, behavior, checks and limits. Match each acceptance claim to an
observed result and identify any gaps.

Open [Engineering examples](references/engineering-examples.md#engineering-examples)
when a comparable API, Vue, RTL testbench or regression case would guide the change.
