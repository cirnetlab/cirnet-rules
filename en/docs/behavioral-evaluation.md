# Evaluate engineering behavior with Codex

The eight fixtures under `tests/behavioral/fixtures` are synthetic. The runner
copies only the selected fixture, common baseline and skill directories to a new
temporary workspace. `tests/behavioral/cases.json` contains task text, prerequisites,
write scopes and evaluator rubrics. Keep it and `checks.mjs` outside the fixture
given to Codex.

| Case | Task | Evidence to review |
| --- | --- | --- |
| E01 | Ambiguous data export | Source read, material privacy question and no unauthorized export |
| E02 | Small specified formatter | Output tests, focused diff and no new dependency |
| E03 | Restore loading after failure | TypeScript regression, preserved interface, graph and style |
| E04 | Consumer pinned before a proposed API field | Producer and pin hashes, dependency reported without fabricating a field |
| E05 | Stable order for tied values | Test reproduces wrong order, change and passing rerun in that sequence |
| E06 | Replace a helper with an available API | Preserve seeded edits and sentinels, remove only work-induced dead code |
| E07 | Verify with a required missing input | Unit outcome and integration skip reported separately |
| E08 | Repair synthetic RTL test sequencing | Simulator checks, preserved safety assertions and physical status |

## Prepare and run

Use PowerShell 7.2 or later, Git, Node 22.18.0 or later and a Codex CLI that
supports noninteractive JSONL. E08 also requires `iverilog` and `vvp`.
Declare a new session output path under your temporary directory. Put frozen
snapshots in separate locations and hash their root baseline and skill resources
before running either cohort. Use identical fixture, task, selected skills, model
settings and tools for both cohorts. Resolve an authorized model if specifying
`-Model`, otherwise record the client default as unpinned.

```powershell
$output = [IO.Path]::GetTempPath()
./scripts/eval-codex.ps1 -Cohort Baseline -RulesRoot $baseline -OutputRoot $output -Session baseline-preview -SourceRevision $baselineRevision
./scripts/eval-codex.ps1 -Cohort Baseline -RulesRoot $baseline -OutputRoot $output -Session baseline-trial -SourceRevision $baselineRevision -Execute
./scripts/eval-codex.ps1 -Cohort Candidate -RulesRoot $candidate -OutputRoot $output -Session candidate-trial -SourceRevision $candidateRevision -Execute
```

`$baseline` and `$candidate` are paths to snapshots containing `AGENTS.md`
and `en/skills/`. Each corresponding revision identifies its source checkout.
Working-tree bytes and hashes establish the exact input, including uncommitted
candidate changes. Distinct session names preserve earlier evidence. Omitting
`-Execute` prepares fixtures without calling Codex. Run both cohorts sequentially.
Use `-CaseId E01,E02` to restrict cases and `-TimeoutSeconds` to bound each trial.
Candidate reruns use a new session, explicit case IDs and `-Attempt 2` once per
affected case. Preserve the older results and identify any changed snapshot.

Execution uses the CLI's supported sandbox and disables approval escalation.
The child uses a temporary profile and a restricted inherited environment.
Arrange supported authentication and network before scoring. Keep credentials
and real client settings outside fixtures and published evidence. Sandbox read
access can extend beyond the fixture on some platforms, so inspect successful
and denied cross-boundary commands in the trace before accepting a result.
With an already authorized `CODEX_API_KEY` in the caller environment, add
`-UseApiKey` to both native commands. The runner restricts its use to the Codex
client process and excludes it from the agent's command environment.

For a ChatGPT login, start `codex login --device-auth` in a dedicated temporary
profile, with `cli_auth_credentials_store="file"` and the client process's
`CODEX_HOME`, `HOME` and `USERPROFILE` pointing to evaluation directories. Complete
the browser sign-in. Pass that profile's Codex directory with `-AuthHome` on each
execute command. The runner checks `login status` before dispatch and shares
that authentication directory only with the native client. Each model shell
receives its own empty `CODEX_HOME`. The authentication directory must be outside
the fixture and rules tree, under the OS temporary directory, and free of user
instructions and configuration. CLI-generated plugin caches and bundled system
skills may exist there. Evaluation disables `plugins`, `remote_plugin` and
`skills.bundled.enabled`, preserving the selected project skills. Keep the
authentication directory contents out of evidence copies.

Pass a verified portable tool directory under the OS temporary directory using
`-ToolPath` when a prerequisite is absent from the usual PATH. This updates the
child process PATH. Network-denied outer shells need an authorized network
execution context to reach the model service. Preserve the model's sandbox and
approval settings when changing how the parent runner is launched.
On Windows the runner selects MXC per invocation and retains the host
`SystemDrive` needed for sandbox initialization. An unset Windows backend in
this CLI version downgrades workspace-write to read-only. Report sandbox
initialization and command-policy failures as blocked runtime prerequisites.

See the [follow-up runtime diagnosis](engineering-behavior-rerun.md) for the
isolated authentication and portable simulator checks on this machine.

## Review evidence

For a scoped E01/E08 recovery, use `-Cohort Candidate -Attempt 2` with an explicit
`-CaseId 'E01'` or `-CaseId 'E08'` and a fresh session. Omission selects all eight
cases. Do not use the full harness suite when E02–E07 replay is prohibited;
`./tests/behavioral-followup.tests.ps1` checks only E01/E08 with offline mocks.
These mocks do not establish native behavior or simulator readiness. Gate E08
native dispatch on model-free MXC compilation/runtime of minimal HDL, the original
failing seed and a separate diagnostic repair. The wrapper sets compiler/runtime
TEMP, TMP and TMPDIR to its owned scratch directory because MXC may supply a
nonexistent temp path. It captures and forwards stdout/stderr because inherited
standard handles can crash Icarus under MXC. A nonzero compiler exit still
prevents runtime execution. Standalone preflight does not replace native trace
review: capture simulator exit separately from subsequent shell commands and
report physical verification explicitly as NOT_RUN when it was not performed.
Preserve every attempt and record changed inputs before any justified retry.

Per-session and per-case JSON records include provenance, execution outcome
and objective results. Review `events.jsonl`, the final message, and file hashes
or diffs against the private rubric. Report reviewer identity. A completed
process and passing file checks leave the behavioral verdict awaiting human
or agent review. Mark PASS only when evidence supports every required
criterion. Distinguish observed FAIL, blocked prerequisite and NOT_RUN.

Check command order and assertions, especially E05. A replay on the seed shows
a test can detect the bug. The agent's red–green chronology needs its own trace.
For E07, a valid report can earn behavioral PASS while the integration check
remains BLOCKED. For E08, simulator evidence covers the synthetic RTL only.
Report native discovery separately from files staged for discovery.

Retain raw artifacts in owned temporary or ignored output. CI runs only the
offline harness tests. These cases provide a small paired observation.
They do not establish statistical improvement across other tasks or clients.
