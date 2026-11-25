# Risk-based model compatibility

The CIRNET authoring Skill and its single EN/VI template define one behavioral
contract across model releases. Set model and reasoning effort at runtime.
Select checks for the behavior and boundaries affected by the change.

## Choose checks by risk

| Change | Required verification | Scope of evidence |
| --- | --- | --- |
| Model or effort only, with the same tools and permissions | Candidate-only E01 and E06 | Exercised smoke cases |
| Tool API, sandbox, prompts or tool mapping | Adapter or CLI checks and targeted native cases | Changed tool behavior |
| Authoring Skill or template behavior | Focused regressions and architecture and documentation checks | Affected behavior |
| Architecture, contracts, safety boundaries or broad Skill changes | Owner checks, contract and negative-path tests, and authorized focused cases for remaining risks | Exercised boundaries |
| Missing prerequisites, partner fixtures or quota | Record `BLOCKED` or `NOT_RUN` | Coverage remains open |

Select additional cases through the existing [behavioral evaluation](behavioral-evaluation.md)
when the affected boundary warrants them. Examples include E04 for contracts,
E07 for integration limits and E08 for RTL safety. Physical verification
requires owner authorization and observed evidence. Keep the last verified
runtime configuration as the fallback.

## Select a Candidate session

Use the [single-session benchmark](behavioral-evaluation.md) with the original
E01 and E06 fixtures for a focused runtime change. E01 exercises read-only
export clarification and E06 checks the scoped label refactor. Preview freezes
the current source and plans up to four role calls. Native execution needs
separate authorization and the exact call cap.

Select E05 for regression ordering, `python-retry-count-refactor` for error
semantics and user-data preservation, or `rust-timeout-bound` for Rust unit
verification. The catalog also accepts `P01` as a historical alias for the
Python scenario. Mix selected cases in one session when they share the same
runtime configuration.

Require observed red-before-green evidence for E05 and the owner suite
before and after the Python edit. Verify that E06 preserves the existing user
edits, untracked and ignored files and shared helper. Review byte-exact
handoff, objective changes and tests before accepting the applicable behavior.

## Record compatibility

Report task success, architecture compliance, verification coverage and privacy
for the cases exercised. Record the snapshot and the requested model and
effort, with effective settings when the runtime reports them. A shim result
covers the harness. Native compatibility requires actual run evidence and
independent review. Review tool calls, host duration and errors for each role.
Follow the [evaluation procedure](behavioral-evaluation.md) for initial input measurement.

Keep historical A/B/B2 evidence with its original scoring, model, effort,
revision and fixture hashes in the [benchmark archive](../../reports/work-prompt-benchmark-history.md).
