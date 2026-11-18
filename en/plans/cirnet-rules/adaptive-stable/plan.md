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
