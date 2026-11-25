# CIRNET reliability evidence

## Scope and ownership

Add three small checks for scoped engineering work in `cirnet-rules`. Use the
[owner's workflows and architecture](../../../policies/architecture.md) for
product commands, contracts, output allocation and acceptance. Use
[Task Continuity](../../../docs/distribution.md#task-continuity) where applicable.
Preserve its schemas and execution behavior, the package contract and sealed
Candidate evidence.

The operator approves scope, effects and sources. Owners supply command
observations and assertion coverage. The agent verifies the claim against
that evidence and reports its scope. Filesystem and network restrictions
come from the execution environment. Scope reconciliation measures net changes.
The checks below complement those controls at separate points in the workflow.

## Expected behavior

1. **Assess a claim.** Read independently pinned expectation and owner
   observation JSON. Bind the claim to the approved repository root, current
   Git revision and declared input digests. Verify exact stage IDs, observed
   integer exit codes, executed and failed assertion counts, and each required
   minimum. Evaluate actual stages independently of an outer wrapper exit.
   Expose stale or incomplete evidence separately from executed failures.
   Keep explicitly skipped stages, their reasons and requiredness in the result.
2. **Preview owner checks.** Read explicitly nominated npm, Cargo, CMake and
   RTL declarations. Return candidate IDs, source digests, selection reasons,
   and prerequisite and side-effect limitations. Keep selection explicit.
   Review the owner's actual scripts, effects, tools and output paths before
   separate authorization and execution. Keep command bodies and raw evidence
   out of the summary.
3. **Assess a correction.** Use the current failed check and the observed
   attempt history for one failure cluster. The default permits at most two
   launched repairs per cluster and one unchanged retry for a check identity
   with a documented transient reason. A repair requires a distinct hypothesis.
   Authorization, safety and progress are prerequisites. Stop on a repeated
   hypothesis, budget exhaustion or missing evidence. Treat the decision as
   advisory at intake. The authorized owner workflow performs any action.

## Acceptance

- Show `PASS`, `FAIL`, `BLOCKED` and `NOT_RUN` for claims and retain
  `SKIPPED` at stage level. A required skip or missing observation prevents
  overall PASS. Disclose any optional skipped scope. Verify failures under a
  successful wrapper, insufficient assertions, missing child exits, stale
  revisions and inputs, and unavailable prerequisites with controlled fixtures.
- Preview explicitly selected checks across supported synthetic owner layouts.
  Verify execution stays `NOT_RUN`. Reject missing or unsafe inputs and
  unknown selections. Leave undeclared owner layouts for owner review.
- Show a failing executable observation and a narrow guard with a passing
  regression. Verify the stop decisions after repeated unchanged failures and
  two repairs, and when scope, safety or progress is missing.
- Preserve EN/VI pairing and hashes, scope, CI and bundle compatibility.
  Report native agent discovery, product E2E, hardware evidence and comparative
  performance according to the exercised scope.

Record the audit, reuse decision and observed results in the
[implementation plan](../../../plans/cirnet-rules/reliability-evidence/plan.md).
