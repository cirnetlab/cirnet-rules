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
2. For export/disclosure work, use trusted requirements, code and contracts to
   establish who acts, whose records they may export and the permitted fields.
   Distinguish own, other-user, tenant/group and bulk scope. Coding permission,
   login, an ID or a public projection does not establish export authorization.
   Resolve recipient/destination, download/storage and redaction where they affect
   disclosure. Ask specifically or identify missing actor/record or field decisions
   before editing the affected part; reuse decisions already established. Continue
   targeted inspection or a short unimplemented proposal without creating an export
   endpoint, migration or artifact while authorization is unresolved. Inspect
   data-store function/projection code and property names first; filter sensitive
   value literals before emitting source excerpts. A full-file read into command
   output discloses stored values even when the final response omits them. Keep
   those values out of examples and new logs.
3. Write observable acceptance criteria. Follow local planning rules and use
   `Step → check` for nontrivial work.
4. Make the smallest coherent change in the existing flow. Keep required failure
   handling and safety controls. Generate derived output from its owning input.
5. Add meaningful coverage where warranted. Update affected documentation pairs
   and run the checks tied to acceptance.

Report the diff, behavior, checks and limits. Match each acceptance claim to an
observed result and identify any gaps.

Open [Engineering examples](references/engineering-examples.md#engineering-examples)
when a comparable API, Vue, RTL testbench or regression case would guide the change.
