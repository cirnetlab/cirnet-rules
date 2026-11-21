# Verify a single Candidate with Codex

Use a focused benchmark when a change affects Work Prompt behavior, tool
access, verification or privacy. Select scenarios by [risk](model-compatibility.md)
and use product-owner checks for actual product changes. The benchmark freezes
the current checkout's Rules, Skills, evaluator and selected synthetic fixtures.
It runs one Author → Work chain per selected scenario.

## Preview and execute selected scenarios

The [scenario catalog](../../tests/benchmark/cases.json) is shared by all cases.
JavaScript, TypeScript, Python, Rust and SystemVerilog use the same fixture
layout and evidence format. E01–E08 retain their established identifiers.
`P01` is an alias for `python-retry-count-refactor`. Select the Rust case as
`rust-timeout-bound`. Use descriptive IDs for new scenarios.

Run from the rules checkout with PowerShell 7.2+, Git and Node 22.18+.
Native execution uses an authorized Codex CLI with noninteractive JSONL.
The default selection E01,E06 allows at most four role calls. Supply
comma-separated `-CaseId` values to select a different scope. The exact
`-MaxNativeCalls` is twice the selected scenario count. Use a new session
name and an existing output directory under OS temp outside the checkout.

```powershell
$root = (New-Item -ItemType Directory -Path (Join-Path ([IO.Path]::GetTempPath()) "cirnet-native-$([guid]::NewGuid())")).FullName
./scripts/benchmark/run.ps1 -Session candidate-preview -Model '<supported-model-id>' -Effort high -OutputRoot $root
# Use the approved evaluation login, call cap and independent handoff review.
./scripts/benchmark/run.ps1 -Session candidate-run -Model '<supported-model-id>' -Effort high -OutputRoot $root -Execute -MaxNativeCalls 4 -AuthHome $authHome -ReviewHandoffs -MeasureInitialInput
```

Preview makes zero role calls. Compare its source and file hashes with the
execution manifest when validating a frozen scope. The manifest records HEAD,
working-tree bytes, role settings and the role cap. The Author snapshot uses
the canonical template and its context, distribution and model guides. Missing tools block the
affected scenario. A shared authentication, quota, network or sandbox blocker
stops further dispatch. `-TimeoutSeconds` sets the per-role limit.

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
- The Author is read-only. Work gets a fresh fixture and the exact Author bytes.
  Role-specific files make the synthetic Git commits differ. Use relative paths
  and verified file hashes for shared source evidence. Work checks its own Git
  state before editing. E02 takes shared pins for the formatter, test and README
  from the frozen fixture. Its Author output gate rejects the role-specific
  `AGENTS.md` hash. Work reads its own `AGENTS.md`.
  Both roles get the pinned `toolkit/en/docs/context.md` intake guide.
  The other `en/docs` guides, authoring template and authoring Skill belong to
  Author. The gate checks readiness, original task, write scope, reporting and
  both review sections. It checks literal toolkit and Skill file references
  against Work's planned inventory at Author output and the staged inventory
  at Work intake. An unstaged reference blocks dispatch even with a reviewed
  digest. For native Work, use `-ReviewHandoffs` and independently check the
  exact draft against the frozen case and source before entering its digest.
  Without that review, Work stays blocked. Synthetic offline shims exercise the
  transport while retaining `REVIEW_REQUIRED`. Native E01 stops before model access
  because the shell can bypass the inspector. E04 permits reviewed read-only
  clarification. E07 can run read-only checks and must report
  the missing partner integration input separately. The reporter rejects
  integration PASS when the frozen fixture lacks the partner input. Work rechecks
  its owned fixture baseline before dispatch and records scope audit separately from
  acceptance. Changes to staged Git
  entries fail the fixture audit even if the edited file is within scope.
- Find each role's `run.json`, `events.jsonl`, `final.md` and diff under
  `trials/<case-id>/<role>/`. Inspect them and the sealed evidence index
  against the frozen rubric. Keep E05's actual red-before-green order and the
  before/after owner checks for E06 and the Python refactor. Cite completed
  Work commands and observed exits for verification PASS.
- E08 requires real compile and simulator exits plus reset, permit loss,
  watchdog expiry and fault assertions. The observer command is
  `node --import ./toolkit/rtl-observer.mjs simulate.mjs`. On PowerShell, use
  the summary command in the Work fixture's `AGENTS.md`. It keeps safe JSON
  `CIRNET_RTL_STAGE ` lines unchanged and preserves the wrapper exit. The parser
  also accepts historical `Child result ` JSON markers.
  Numeric exit 0 with a process error or signal is insufficient. Report
  `physical verification: NOT_RUN` unless there is separate board evidence.

An independent reviewer copies `review-template.json` to `review.json` inside
the owned session. Keep the evidence-index SHA-256, identify the reviewer and
give a status, reason and case-relative locator for each applicable criterion.
Use paths such as `Work/run.json` or completed command IDs such as `Work/item_3`.
Submit the review without running the model again.

```powershell
$review = Join-Path $root 'candidate-run/review.json'
./scripts/benchmark/run.ps1 -Session candidate-run -OutputRoot $root -ReviewFile $review
```

The reporter checks the frozen inputs, role provenance, handoff hashes and
indexed artifacts. Objective success requires completed roles and a passing
fixture scope audit. The reporter writes one `quality-report.json` with Task Success,
Architecture Compliance, Verification Coverage and Privacy for each scenario.
PASS applies to the selected scope. BLOCKED or NOT_RUN remains explicit where
evidence is missing. A shim can exercise the harness and review controls.
Native quality PASS requires actual model evidence. Review per-role tool
calls, duration, errors and verification evidence for the pinned session.
Record failures and blocked runs separately.

The [benchmark archive](../../reports/work-prompt-benchmark-history.md) links
historical comparisons and their original revisions. The current runner uses
the same Candidate rules for Author and Work.

## Privacy inspection and evidence controls

Both Author and Work receive the same bounded structural inspector. From the
fixture, `node toolkit/inspect-source.mjs accounts.mjs --symbol readPreferences`
returns a source digest, field names and function property identifiers. It
does not execute the source or return literals, expressions or code excerpts.
The inspector rejects linked, oversized, outside-checkout and unsupported
inputs with a fixed diagnostic. Its lexical metadata is not semantic proof;
use a safer access path when it cannot establish the needed fact.
The read is capped at 64 KiB. Regex/division syntax, interpolated templates and
unclosed literals are unsupported and block that inspection.

This helper filters before its output reaches the model. It does not intercept
other shell tools or enforce OS confidentiality. Author must carry its verified
invocation, inspected facts and read limits into Work's handoff, even when the
task is blocked for missing export authority. Work need not reread a record
store solely to confirm that block. Preserve actor, record and field decisions.

The independent canary detector audits literal, Unicode, nested Unicode and
hex character escapes in role output and evidence. It withholds detected
values while retaining event IDs. Detection is after model exposure and cannot
undo it; arbitrary encodings and unknown secrets are not guaranteed detectable.
Any known exposure remains FAIL. Run `tests/benchmark/privacy.tests.ps1` and
`node --test tests/benchmark/privacy.test.mjs` for the focused negative checks.
Offline/shim checks establish harness behavior, not native handoff quality.
