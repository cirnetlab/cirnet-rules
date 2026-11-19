# Adaptive Stable source integration

The Work Prompt authoring Skill and its EN/VI template define one contract.
Handoffs preserve the task objective, authority, privacy and verification
requirements. Model and effort are evaluation runtime settings. The
[compatibility guide](../../../docs/model-compatibility.md) selects checks
based on the changed behavior, capability or risk.

The Candidate-only smoke uses `scripts/eval-codex.ps1` with the original E01
and E06 synthetic cases. Preview records the frozen input hashes. A separately
authorized execution may attempt up to four role calls. A failed Author or
invalid handoff stops that case before Work. Compare the exact handoff bytes
with Work's input and use the existing objective evaluator and privacy audit.
Role evidence stays in the owned temp session. Report observed usage with its
provenance and mark missing telemetry `NOT_AVAILABLE`.

Offline acceptance covers preview, mock Author and Work, privacy and objective
failure, all 13 Skills, EN/VI parity and source hashes. Native compatibility
requires actual role evidence and independent review. The applicable release
gates cover Task Success, Architecture Compliance, Verification Coverage and
Privacy. Historical fixtures, grading rules, snapshots and results are fixed.

Author recovery preserves the supplied action, target, transformation and
scope before source discovery. A failed read can use an independently verified
alternative within the same access and output limits only when evidence is
essential to a safe handoff. Otherwise hand off the uninspected detail to a
scoped Work preflight when the objective, owner and safe scope are established.
Existing source inspection and sandbox controls apply to both roles.

For Candidate-only E01/E06, validate the Author handoff against the frozen
case before dispatch. Its first three lines declare readiness, the original
task verbatim and the exact fixture write scope. Require nonempty requirements
and verification sections for independent review. E01 requires
`PROMPT_BLOCKED` and an empty scope. Work may clarify authority in a read-only
sandbox. E06 requires `PROMPT_READY` and the original file allowlist. An
invalid handoff ends the case before dispatch. Work verifies the expected
Author SHA-256 and header on the same bytes it receives.

The gate checks structure and trusted case inputs. Review functional details,
negative cases, privacy and verification against the actual trace and diff.
The E06 sandbox remains fixture-wide. Its objective and Git evidence checks
detect scope violations. The gate uses the existing runner. Historical roles
run under their pinned protocol and evidence.

## Current maintenance scope

The day-to-day workflow uses the common baseline, repository-owned architecture
and contracts, scoped Work Prompts and risk-based verification. The default
toolkit checks cover the handoff and essential regressions offline.
Existing client setup and isolated fixture execution remain maintenance work.
The Governance branch serves as historical reference.

The checkpoint coordinator and large A/B comparisons are maintained as manual
tools. Native execution requires separate authorization. New reviews cover task success,
architecture compliance, verification coverage and privacy. Measured role
usage and timing serve diagnosis. Historical comparisons keep their original
inputs, reports, scoring and source revision.
