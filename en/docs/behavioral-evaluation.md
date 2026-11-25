# Verify a single Candidate with Codex

Use a focused benchmark when a change affects engineering behavior, tool
access, verification or privacy. Select scenarios by [risk](model-compatibility.md)
and use product-owner checks for actual product changes. The benchmark freezes
the current checkout's Rules, Skills, evaluator and selected synthetic fixtures.
It runs the original task directly in one isolated fixture per selected scenario.

## Preview and execute selected scenarios

The [scenario catalog](../../tests/benchmark/cases.json) is shared by all cases.
JavaScript, TypeScript, Python, Rust and SystemVerilog use the same fixture
layout and evidence format. E01–E08 retain their established identifiers.
`P01` is an alias for `python-retry-count-refactor`. Select the Rust case as
`rust-timeout-bound`. Use descriptive IDs for new scenarios.

Run from the rules checkout with PowerShell 7.2+, Git and Node 22.18+.
Native execution uses an authorized Codex CLI with noninteractive JSONL.
The default selection E01,E06 allows at most two scenario calls. Supply
comma-separated `-CaseId` values to select a different scope. The exact
`-MaxNativeCalls` equals the selected scenario count. Use a new session
name and an existing output directory under OS temp outside the checkout.

```powershell
$root = (New-Item -ItemType Directory -Path (Join-Path ([IO.Path]::GetTempPath()) "cirnet-native-$([guid]::NewGuid())")).FullName
./scripts/benchmark/run.ps1 -Session candidate-preview -Model '<supported-model-id>' -Effort high -OutputRoot $root
# Use the approved evaluation login, call cap and reviewed fixture scope.
./scripts/benchmark/run.ps1 -Session candidate-run -Model '<supported-model-id>' -Effort high -OutputRoot $root -Execute -MaxNativeCalls 2 -AuthHome $authHome -MeasureInitialInput
```

Preview makes zero model calls. Compare its source and file hashes with the
execution manifest. The manifest records HEAD, working-tree bytes, settings and
the scenario cap. Missing tools block the affected scenario. A shared
authentication, quota, network or sandbox blocker stops further dispatch.
`-TimeoutSeconds` sets the per-scenario limit.

Use `-MeasureInitialInput` only for the initial authorized evaluation. It
reserves the dedicated output root before dispatch and ties the reservation to
the frozen manifest. Keep that reservation and the sealed initial evidence.
Use the same root with a fresh session name for affected correction cases and
omit the switch, even if the initial run was interrupted. The session summary
records provider input usage for that initial run when available. Missing
usage and unmeasured runs show `NOT_AVAILABLE`.

Use a dedicated file-backed Codex login under OS temp with `-AuthHome`, or an
authorized `-UseApiKey`. For a ChatGPT login, run
`codex -c 'cli_auth_credentials_store="file"' login --device-auth` with a
separate temporary profile for the login process. Keep authentication outside
the session, snapshot and model environment. `-ToolPath` accepts one verified
portable tool directory under OS temp, separate from the checkout and login.
Python uses `python` or `python3`, Rust requires `rustc`, and E08 requires
`iverilog` and `vvp`. The model shell has an isolated home and no network.
Inspect the trace for reads beyond the fixture because OS read isolation varies.

## Grade the evidence

- Review `compatibility-summary.json` for execution and objective checks.
  Exit 0 covers preview or objective success, exit 1 a failure, and exit 2 a
  blocked run. An objective result remains pending independent review.
- The runner gives the model the frozen catalog task and stages the applicable
  Skills and context. The reporter verifies the input bytes against the catalog.
  Native E01 uses the Windows MXC read profile only after direct, traversal,
  junction, alternate-reader and write probes pass. It stages only verified
  account structure. Failed or unavailable preflight blocks model access.
  E01 and E04 run read-only for clarification. E07 can run read-only checks,
  and the reporter rejects integration PASS without the pinned partner input.
  The runner rechecks its owned fixture and Git metadata before execution.
  A change to the Git index fails the scope audit even for an allowed file.
- Find the scenario's `run.json`, `events.jsonl`, `final.md` and diff under
  `trials/<case-id>/Direct/`. Inspect them and the sealed evidence index
  against the frozen rubric. Keep E05's actual red-before-green order and the
  before/after owner checks for E06 and the Python refactor. Cite completed
  direct commands and observed exits for verification PASS.
- E08 requires real compile and simulator exits plus reset, permit loss,
  watchdog expiry and fault assertions. The observer command is
  `node --import ./toolkit/rtl-observer.mjs simulate.mjs`. On PowerShell, use
  the summary command in the direct fixture's `AGENTS.md`. It keeps safe JSON
  `CIRNET_RTL_STAGE ` lines unchanged and preserves the wrapper exit. The parser
  also accepts historical `Child result ` JSON markers.
  Numeric exit 0 with a process error or signal is insufficient. Report
  `physical verification: NOT_RUN` unless there is separate board evidence.

An independent reviewer copies `review-template.json` to `review.json` inside
the owned session. Keep the evidence-index SHA-256, identify the reviewer and
give a status, reason and case-relative locator for each applicable criterion.
Use paths such as `Direct/run.json` or completed command IDs such as `Direct/item_3`.
Submit the review without running the model again.

```powershell
$review = Join-Path $root 'candidate-run/review.json'
./scripts/benchmark/run.ps1 -Session candidate-run -OutputRoot $root -ReviewFile $review
```

The reporter checks the frozen inputs, run provenance, scenario input hashes and
indexed artifacts. Objective success requires a completed run and a passing
fixture scope audit. The reporter writes one `quality-report.json` with Task Success,
Architecture Compliance, Verification Coverage and Privacy for each scenario.
PASS applies to the selected scope. BLOCKED or NOT_RUN remains explicit where
evidence is missing. A shim can exercise the harness and review controls.
Native quality PASS requires actual model evidence. Review scenario tool
calls, duration, errors and verification evidence for the pinned session.
Record failures and blocked runs separately.

The [benchmark archive](../../reports/work-prompt-benchmark-history.md) links
historical comparisons and their original revisions. Review historical Author/Work evidence with the reporter and schema from that revision.

## Privacy inspection and evidence controls

The direct fixture includes the bounded structural inspector. In the
offline E01 fixture, `node toolkit/inspect-source.mjs accounts.mjs --symbol readPreferences`
returns a source digest, field names and function property identifiers. It
does not execute the source or return literals, expressions or code excerpts.
The inspector rejects linked, oversized, outside-checkout and unsupported
inputs with a fixed diagnostic. Its lexical metadata is not semantic proof;
use a safer access path when it cannot establish the needed fact.
The read is capped at 64 KiB. Regex/division syntax, interpolated templates and
unclosed literals are unsupported and block that inspection.

This helper filters before its output reaches the model. It does not intercept
other shell tools or enforce OS confidentiality. In protected Native E01, the
runner calls it outside the Agent's read boundary and stages verified structural
metadata at `toolkit/e01-account-structure.json`. The export actor, records
and allowed fields still require authorization.

The independent canary detector audits literal, Unicode, nested Unicode and
hex character escapes in scenario output and evidence. It withholds detected
values while retaining event IDs. Detection is after model exposure and cannot
undo it; arbitrary encodings and unknown secrets are not guaranteed detectable.
Any known exposure remains FAIL. Run `tests/benchmark/privacy.tests.ps1` and
`node --test tests/benchmark/privacy.test.mjs` for the focused negative checks.
Offline/shim checks establish harness behavior, not native engineering quality.
