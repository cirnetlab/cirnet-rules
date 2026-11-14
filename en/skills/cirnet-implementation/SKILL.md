---
name: cirnet-implementation
description: Use when implementing a scoped feature or behavior change with an agreed outcome.
---

# Implementation

Use for an agreed change. Use debugging first when the cause is unknown.

Minimum inputs: Authorized files, intended behavior, owning interfaces and nearby tests.

1. Inspect the affected flow and direct callers. State observable acceptance checks.
2. Choose the smallest coherent change. Follow local planning rules and generate derived output from its owning input.
3. Add meaningful coverage where warranted. Update affected documentation pairs and run relevant checks.

Report the diff, behavior, checks and limits. Completion needs observed acceptance results or explicit gaps.

Example: A consumer needs a payload field. Inspect its selected contract first and identify an owner-side change if the field is absent. Stop at the write boundary.
