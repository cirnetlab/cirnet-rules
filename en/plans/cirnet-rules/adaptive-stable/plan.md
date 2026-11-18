# Adaptive Stable integration plan

Scope: the CIRNET rules checkout and the supplied read-only overlay. The
[specification](../../../specs/cirnet-rules/adaptive-stable/spec.md) sets the
behavior and acceptance boundaries.

| Step | Check |
| --- | --- |
| Review the overlay and checkout | Compare anchor edits with current source and freeze the starting Git state. |
| Update the authoring Skill, template and guides | Review each EN/VI pair, verify exact source hashes and count 13 Skills. |
| Wire the Candidate-only smoke to the role runner | Preview and test the Author to Work path and its failure handling with an offline shim. |
| Review the diff and verification | Run the documentation, Cargo and offline regression checks. Verify historical inputs and evidence against the starting state. |

## Verification record

The integration started from a clean `master` at `fdfe0949`. The overlay
was reconciled manually so both context guides received the lifecycle section.
The offline smoke passed preview, byte-exact handoff, synthetic privacy guards,
objective failure and blocked-evidence checks. The source tests confirmed
13 Skills, one Work Prompt template pair and matching source hashes.

Cargo formatting, tests and Clippy, the changed-document selector, package and
install fixtures, and the offline behavioral, follow-up and A/B shim checks passed.
The architecture snapshots and 5,525 historical files under `.superpowers`
matched the starting checkout. RTL simulation lacked `iverilog` and `vvp`.
Windows skipped link cases that require symlink or reparse-point permissions.

Native smoke, independent behavioral review and the full native benchmark
remain `NOT_RUN`. No native usage was measured. Native execution requires
separate authorization, and compatibility requires independent review of
the resulting evidence.

## CI exit handling follow-up

On `a7e14202`, the Windows GitHub Actions PowerShell wrapper printed the smoke
suite's `PASS` message and exited `2` because `$LASTEXITCODE` still held the
simulated blocked run's status. The test now exits `0` after all assertions
pass. It still checks the smoke runner's expected failure and blocked exits
of `1` and `2`.

The workflow checks passed locally on Windows and in an offline Ubuntu 24.04
container using the GitHub Actions checkout layout. Both smoke steps exited
`0`. The Ubuntu checkout layout matters because the role runner derives the
workspace parent from the checkout path. Hosted CI for the patch is pending
publication.

## Native Candidate-only follow-up

One Windows session requested `gpt-6.2-sol` at `xhigh` and used three of the
four allowed roles. The service did not report the effective model or effort.
Independent review of the role traces, handoff bytes and fixture changes
confirmed that E01 met the exercised Task Success, Architecture Compliance,
Verification Coverage and Privacy criteria. Author used the structural
inspector and requested the missing export authority. Work matched the store
hash, reused those findings and made no changes. The privacy audit reported
zero canary or credential hits in the observed roles.

E06 Author timed out at 300 seconds without completing a handoff. Its trace
records a rejected structural inspection, a PowerShell constrained-language
error and repeated read-only probes. Work was `NOT_RUN`, so E06 implementation
and before/after verification are `BLOCKED`. The release remains Candidate.

| Role | Actual input tokens | Host runtime (s) | Observed tool calls |
| --- | ---: | ---: | ---: |
| E01 Author | 137,671 | 119.699 | 13 |
| E01 Work | 63,854 | 37.825 | 8 |
| E06 Author | `NOT_AVAILABLE` | 300.032 | 26 |

E01 consumed 201,525 input tokens, including 161,792 cached. The complete
session input total is `NOT_AVAILABLE`. The session took 766.333 seconds
wall time including setup and checks. The two E01 inspector invocations
returned equivalent JSON. E06 generated additional probes after inspection
failed. These traces show repeated context, while the isolated token and
latency effects of a source change remain unmeasured. The CI correction was
the only code change.

## Author recovery follow-up

| Step | Check |
| --- | --- |
| Reproduce the trace boundary | Review the recorded commands and compare inspector results for the original E06 and E01 sources. |
| Add recovery to the authoring Skill and template | Verify both languages preserve the supplied goal, stop unnecessary discovery and retain the privacy boundary. |
| Exercise the original smoke cases offline | Verify the E06 handoff carries the original operation through inspector refusal, and Work preserves unrelated data. Run the E01 privacy checks. |
| Prepare native validation | Verify preview with runtime model and effort and record the separate authorization requirement. |

The original native E06 Author trace recorded a timeout after 300.032 s and
26 observed tool calls. The inspector refused template interpolation in
`src/labels.mjs` and the Author continued with repeated read-only probes.
PowerShell rejected `[pscustomobject]` in Constrained Language mode. Offline
reproduction returned inspector exit `2` on E06 and `0` on E01 and reproduced
the PowerShell error with exit `1`. The inspector and the role runner keep
their existing safety behavior.

The authoring pair now drafts the original goal before discovery, stops when
enough evidence supports a safe handoff and delegates unread details to a
scoped Work preflight. Retries require essential evidence, a verified
alternative and the existing read scope, output limits and shell policy.

On Windows and Ubuntu 24.04, the Candidate-only shim exercised the real E06
inspector refusal, passed the original request byte-for-byte to Work with
`NOT_INSPECTED`, ran the label tests before and after the edit and passed the
existing data-preservation checks. The E01 guard detected and withheld a
synthetic canary. The behavioral harness, offline A/B shim checks, package
tests, Cargo and EN/VI checks passed. Windows skipped symlink/reparse-point
cases requiring permissions and the E08 simulation requiring Icarus. The
historical `.superpowers` inventory matched its recorded SHA-256 across
5,525 files.

The Windows smoke took 108.160 s and the standalone preview took 9.793 s.
The Ubuntu run of the smoke, Cargo, source and privacy checks took 42.653 s.
Preview recorded E01/E06 and the four-role cap with runtime model and effort,
and made zero model calls. Follow the [compatibility procedure](../../../docs/model-compatibility.md)
once native execution has separate authorization. Native behavior, token
usage and latency for this revision are `NOT_RUN` or `NOT_AVAILABLE`. Keep
Candidate pending independent E01/E06 review.

## Candidate handoff validation

| Step | Check |
| --- | --- |
| Reproduce the dispatch gap | Use byte-valid shim handoffs with a broadened E01 scope and an incomplete E06 goal. Confirm whether each reaches Work. |
| Validate Candidate-only handoffs | Require the frozen task and scope, explicit readiness and review sections. Check the exact bytes again at Work intake. |
| Verify sandbox selection | Expect read-only clarification in E01 and the existing write sandbox in E06. Retest privacy, protected data and the legacy harness. |
| Prepare native validation | Verify a fresh preview and source hashes. Preserve the native control configuration and wait for separate authorization. |

On `5c705e9`, both the E01 handoff that broadened scope and the E06 handoff
that dropped the requested operation reached Work with matching SHA-256 and
passed the shim objective checks. The Candidate-only gate now rejects both
before dispatch. The offline smoke also verifies read-only E01 clarification,
the existing E06 write sandbox, a wrong Work input digest and an empty handoff
verification section. E06 recovery still runs the label tests before and
after the scoped edit. The canary and objective failure checks still run.

The Candidate-only smoke passed on Windows and Ubuntu 24.04. The Windows
legacy A/B shim suite, Cargo, doc and source SHA checks passed. The Ubuntu
legacy continuation hit the same `.gitignore` lookup error at
`scripts/eval-codex-ab.ps1:70` on the base commit and the patch. An isolated
probe confirmed the dotfile exists and requires `Get-Item -Force` on Ubuntu.
The historical evidence inventory still matches 5,525 files.

The old native E06 timeout evidence remains 300.032 seconds and 26 tool calls.
The current authoring rules already say when to reuse evidence, defer an unread
source to Work and stop discovery. The new gate leaves that guidance intact.
Claims about fewer retries or lower token usage require native evidence.
Preview verified 96 SHA-256 entries with `gpt-6.2-sol` and `xhigh` and invoked
zero native roles. Run E01/E06 at the same settings, capped at four roles,
after separate authorization. Keep Candidate pending independent review.

## Ubuntu continuation and native readiness

On `1725e8c`, the legacy A/B shim exited 1 on Ubuntu 24.04.5 with
PowerShell 7.7.0. Continuation found the frozen E06 `.gitignore` during
enumeration, then failed to read its metadata without `Get-Item -Force`.
With `-Force` on that lookup, the same shim exited 0. Continuation still
validates the path, rejects hard links and checks the copied SHA-256.

The Candidate preview verified 96 input SHA-256 entries against
`1725e8c70417f89e662cbbcb7b88570da0bef8c2`. Use `gpt-6.2-sol`, `xhigh`,
300 seconds per role and at most four role calls. After separate authorization,
verify the execution input hashes against the preview and review both cases
independently. Native validation for this revision is `NOT_RUN`.
