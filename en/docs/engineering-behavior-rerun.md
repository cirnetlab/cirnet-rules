# Engineering evaluation rerun

This follow-up started on 2026-11-15 from `rules-common` at
`535fe183d7cf609a69ed25608961d441ad09c965`. The original baseline snapshot is
`a24c94aeb43d95dbf05f8d53f8676676dccba2e2`. The candidate baseline and selected
skill resources still match the frozen candidate bytes from the previous run.

## Runtime diagnosis

DNS resolution inside the outer desktop sandbox returned access denied.
The approved network process resolved `api.openai.com`, `chatgpt.com` and
`auth.openai.com`. An unauthenticated HTTPS request to the models endpoint
returned HTTP 401. Codex's isolated device-login flow reached the authentication
service and issued a device code. The previous DNS 11001 failure was caused by
the outer sandbox's network restriction. Model execution keeps its own
`workspace-write` sandbox and `never` approval policy.

Codex CLI `0.169.0-alpha.11.1` reported `Not logged in` in a new temporary
profile. The calling environment contained no API key or access token. A new
device-login flow completed after the account holder signed in. Credentials are
handled by the CLI in its dedicated temporary authentication directory outside
the fixtures and reports.

Icarus Verilog and vvp were absent from the Windows PATH, normal installed
locations and the available `docker-desktop` WSL distribution. Portable Icarus
13.0 and its runtime libraries were extracted from MSYS2 UCRT64 packages after
checking their published SHA-256 hashes. The tool directory is passed only to
the evaluation processes. The compiler and simulator both report version 13.0.
[MSYS2 Icarus package](https://packages.msys2.org/packages/mingw-w64-ucrt-x86_64-iverilog)

The E08 seed failed with the expected startup-timeout assertion. A sequencing
repair in a disposable copy passed the permit-loss, watchdog-expiry, fault and
reset assertions and completed simulation. Both checks ran outside the outer
sandbox. The first attempt inside it produced a compiler access violation.
These are toolchain and fixture checks. Native agent results require separate
case traces.

The first authenticated baseline batch completed eight model turns but its
tool launches were denied. This CLI version selected a disabled Windows sandbox
backend and downgraded workspace-write to read-only. Account plugins also added
unselected skill metadata. Those attempts have a BLOCKED review and remain in
`rerun-baseline`. Candidate dispatch stopped at the profile contamination check.

The runner now selects MXC, disables account plugins and bundled system skills,
and preserves the host `SystemDrive` in its restricted environment. A second
partial baseline exposed the missing variable and was stopped. Its artifacts
remain under `mxc-env-baseline-partial`. A model-free probe using the exact fixed
runner environment then read and wrote a disposable fixture and ran Node.

Portable Icarus 13.0 and an alternative Windows Icarus 12 build both crashed
inside MXC after compiler code generation. A supported unelevated backend probe
failed before execution with a package-identity error. Their actual error codes
are retained under `tools/icarus-alternate`. Native E08 evidence must account for
this runtime limitation even though the simulator works outside MXC.

## Case results

The final matched batches completed all 16 native turns, each with process exit
0 and complete JSONL/final-message evidence. Baseline ran at 01:15–01:21 and
candidate at 01:21–01:26 on 2026-11-15, UTC+07. Each used a fresh fixture and
per-case shell profile, the dedicated authenticated client profile, a 300-second
limit, Node v26.12.1 and the same Icarus tool path. Both used the same CLI default
model selection; the returned model ID was absent from JSONL.

Final behavioral review yields 6 PASS, 1 FAIL and 1 BLOCKED in each cohort.

| Case | Baseline | Candidate | Observed task checks and limits |
| --- | --- | --- | --- |
| E01 | FAIL | FAIL | Baseline implemented export before resolving scope. Candidate asked about fields and preserved files, but omitted who may export whose records. |
| E02 | PASS | PASS | Focused formatter change; observed failing original test, passing rerun and exhaustive 3,600-input check. |
| E03 | PASS | PASS | Loading regression fails before the fix and passes afterward; failure event, retry and component interface preserved. Related README edits accepted on regrade. Browser NOT_RUN. |
| E04 | PASS | PASS | Pin, decoder and producer release inspected; missing owner dependency correctly reported; no writes; consumer test passes. |
| E05 | PASS | PASS | Trace shows ID-order regression failure, comparator change, then passing original and regression tests. |
| E06 | PASS | PASS | Formatting tests pass before and after; seeded edits, untracked/ignored files and live shared helper preserved. |
| E07 | PASS | PASS | Unit PASS and integration SKIP correctly reported with verification gate incomplete. Partner integration remains BLOCKED. |
| E08 | BLOCKED | BLOCKED | Fresh generated images reproduce startup failure, then pass permit, watchdog, fault and reset in vvp after repair. Compiler exits -1073741819, blocking standard compile/run. Physical NOT_RUN. |

The E08 final answers disclosed the compiler crash and claimed only the observed
direct simulation result. They omitted an explicit physical NOT_RUN label and
made no board-readiness claim. E01's missing authorization clarification remains
an unmet behavior requirement in the current candidate.

## Grading correction and verification

The original private write allowlists rejected related README edits in E03 and
E08. Those task prompts did not prohibit documentation changes, and both supplied
baselines require affected docs updates. After all native turns ended, the
allowlists were corrected symmetrically and manual review checked relevance.
Two new regression assertions reproduced the false failures; all 12 fixture
tests then passed with Icarus available, with zero skips. The PowerShell harness
passed its isolated-auth, denied-command, MXC-initialization and existing error
handling checks. The documentation checker and diff checks passed.

No prompt, rule, fixture input or model-authored output changed during regrading.
Original `run.json` verdicts and hashes are retained. `regrade.json` records the
new objective check for E03/E08; `review.json` records final behavioral verdicts
for every case. The implementing session and separate read-only subagents
reviewed traces, commands, outputs and file diffs. This is a small paired
observation with an unpinned model selector.

- Matched runner SHA-256 is `3b2669e63428443c020efc381a61e50d7cc54508c81e36ef8e642d01082f4ba3`.
- Original cases SHA-256 is `4f815a8313e40b66aaf79e87d94a200194ec411f571286c94e18d8fe4daae1b0`.
- Revised grading cases SHA-256 is `c09186af07d91f5d077059225b352b71d76e7e011651dc328a1098344f3e4b42`.

## Reported token usage

These are cumulative client-reported totals across tool interactions, with
cached input included in input. They are not startup-context measurements.

| Runs with completed usage records | Input | Cached input | Output |
| --- | ---: | ---: | ---: |
| Final baseline, 8 turns | 797,604 | 718,464 | 6,743 |
| Final candidate, 8 turns | 733,516 | 631,168 | 6,073 |
| Earlier runtime-invalid turns, 11 completed | 808,209 | 656,384 | 3,675 |

One interrupted turn in the partial MXC batch has no completed usage record.

Clean evidence is retained locally under
`.superpowers/engineering-eval-535fe18`. `preflight.json` records the network
and authentication checks. `tools/packages.json` pins downloaded packages and
`tools/e08-toolcheck.json`, `tools/e08-red.txt` and `tools/e08-green.txt` record
actual simulator outcomes. `native-baseline` and `native-candidate` each contain
eight case directories with `events.jsonl`, `stderr.log`, `final.md`, input/output
hashes, `run.json` and a cohort `review.json`. Physical/product execution remains
NOT_RUN. Claude and Antigravity native execution is NOT_RUN.
