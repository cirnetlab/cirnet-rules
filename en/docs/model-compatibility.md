# Adaptive Stable model compatibility

The CIRNET authoring Skill and its single EN/VI template define one behavioral
contract across model releases. Set model and reasoning effort at runtime.
Keep historical A/B/B2 evidence with its original model, effort, revision and
fixture hashes.

## Choose checks by risk

| Change | Required verification | Scope of evidence |
| --- | --- | --- |
| Model or effort only, with the same tools and permissions | Candidate-only E01 and E06 | Exercised smoke cases |
| Tool API, sandbox, prompts or tool mapping | Adapter or CLI checks and targeted native cases | Changed tool behavior |
| Authoring Skill or template behavior | Focused regressions and architecture and documentation checks | Affected behavior |
| Architecture, contracts, safety boundaries or broad Skill changes | Owner checks and an appropriately sized frozen A/B comparison, when authorized | Declared acceptance gates |
| Missing prerequisites, partner fixtures or quota | Record `BLOCKED` or `NOT_RUN` | Coverage remains open |

Select additional cases through the existing [behavioral evaluation](behavioral-evaluation.md)
when the affected boundary warrants them. Examples include E04 for contracts,
E07 for integration limits and E08 for RTL safety. Physical verification
requires owner authorization and observed evidence. A model release alone
leaves the source, template and historical A/B results at their current
revisions. Keep the last verified runtime configuration as the fallback.

## Run the Candidate-only smoke

The smoke uses `scripts/eval-codex.ps1` with the original synthetic E01 and E06
cases. Preview allocates a fresh session under OS temp and records SHA-256
hashes of the working bytes of the tracked toolkit and evaluator inputs, this
guide and the runner files. Use a fresh session name each time.

```powershell
$root = [IO.Path]::GetTempPath()
$authHome = '<approved dedicated authentication directory under OS temp>'
./scripts/eval-codex-model-smoke.ps1 -Session smoke-preview -Model '<supported-model-id>' -Effort high -OutputRoot $root
# Run only with separate authorization and the approved authentication method.
./scripts/eval-codex-model-smoke.ps1 -Session smoke-native -Model '<supported-model-id>' -Effort high -OutputRoot $root -AuthHome $authHome -Execute -MaxNativeCalls 4
```

A native session may attempt up to four role calls, one Author and one Work per
case. A failed Author or invalid handoff stops that case before Work. The
Candidate-only gate checks the handoff's readiness, verbatim original task,
fixture write scope and nonempty review sections. E01 uses `PROMPT_BLOCKED`
and permits Work only in a read-only sandbox to clarify export authority.
E06 uses `PROMPT_READY` and its original write scope. Work verifies the
Author SHA-256 and the gate fields on its exact input bytes. The runner
writes `compatibility-summary.json` after
execution. Exit 1 means a case failed or session setup or integrity failed.
Exit 2 means evidence was blocked with no case failure. Exit 0 covers preview,
successful mock objectives, or `OBJECTIVE_PASS_REVIEW_REQUIRED` on both native
cases. Preview makes zero role calls. Offline `.ps1` test shims require no
credentials and are marked `TEST_SHIM`.

Review the actual trace, authorized scope, source diff and objective checks
independently before claiming compatibility. Review the substance of the
requirements and verification sections against the original task and rubric,
including negative cases and privacy. The E06 write sandbox covers the fixture,
and its objective checks detect changes outside the authorized file.
Keep raw traces in the owned temp
session and share only sanitized evidence. A detected E01 synthetic canary is a
failure. The existing privacy audit detects exposure and the filesystem read
isolation limit still applies.

## Check generalization separately

Select `-CaseId E05` for an existing debugging and regression case or
`-CaseId P01` for the independent Python refactor. Each preview freezes the
selected suite, the current Author and Work Skills, and the runner. Each
authorized execution uses `-Execute -MaxNativeCalls 2` and the same handoff
validation as the default smoke. P01 requires an approved Python executable
available to the evaluation sandbox. Use `-ToolPath` for a verified portable
interpreter under the OS temporary directory when needed.

Review the chosen prompt level and the actual checks. Require E05's new
regression to fail for the ordering defect before the source edit and pass
afterward. Require P01's owner-provided unittest command to pass on the actual
fixture before and after the edit, with the error contract and user data
preserved. The Candidate-only runs use the current execution Skills.
Legacy A/B stages the frozen common execution Skills for both cohorts.

## Assess release readiness

Use **Adaptive Stable** for the single release line once its applicable gates
cover Task Success, Architecture Compliance, Verification Coverage and Privacy.
An unaccepted revision is an **Adaptive Candidate**. A focused smoke covers
its exercised cases. Full A/B acceptance, including any 96-role native run,
requires a separate decision and authorization.

Report `modelRequested`, `effortRequested`, role-level input, output and cached
tokens, tool calls and host timing where observed. Keep missing usage
`NOT_AVAILABLE` and unreported effective settings unverified. The role runner
labels native usage `ACTUAL` and mock usage `TEST_SHIM`. Measured usage can
support diagnosis even on a failed run. Cost and quality claims require their
own evidence.
