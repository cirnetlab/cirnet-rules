# Native Work Prompt handoff correction at a3ff51e

On 20 November 2026, Windows Codex CLI `0.169.0-alpha.11.1` ran session
`native-a3ff51e-handoff-correction` from clean revision
`a3ff51ec1065297c30692894852f0404a352b701`. The request used `gpt-6.2-sol`
with `high` effort, the dedicated evaluation login, and a cap of eight role
calls covering E02, E04, E06 and E08. Token measurement was disabled. The
[initial session](native-quality-reliability-671abff.md) retains its original
measurement and results.

Two role calls were dispatched. The runner and independent reporter both
exited `2` with overall status `BLOCKED`.

| Case | Native result | Evidence |
| --- | --- | --- |
| E02 | `BLOCKED` | Author completed. The independently reviewed handoff passed Work intake and matched byte for byte. Work read the staged context guide and began fixture preflight. The provider usage limit stopped Work (process exit `1`) before it edited `duration.mjs` or ran the formatting assertions. Fixture and Git metadata audits passed with zero changes. |
| E04 | `NOT_RUN` | Shared usage limit stopped both roles before dispatch. Native confirmation of the producer field and read-only clarification remains pending. |
| E06 | `NOT_RUN` | Neither role dispatched. Native before/after checks and preservation of user data and Git metadata remain pending. |
| E08 | `NOT_RUN` | Neither role dispatched. RTL compile and simulation have no native exit codes. Reset, permit loss, watchdog expiry and fault assertions are all `NOT_RUN` for this session. |

`physical verification: NOT_RUN`

| Local evidence | SHA-256 |
| --- | --- |
| `manifest.json` | `052975ec16150e7ac8917b4b1412f152f35785ed55e90cda3393443475d4633a` |
| `evidence-index.json` | `0200fcdf99169d6ddcd5a684d73a79f055a0aa29b3ce36cbe649913f0e94b9ee` |
| `review.json` | `e161bcb3eb0f47b3139e87d3da52fbb739da12d6aafd9e8197cf02febcc757be` |
| `quality-report.json` | `7f628d24c114a9cab03d3217154e990eeae86963568829a393bf0d0a55779b7a` |

Native acceptance remains open. Resume with a fresh bounded session when the
evaluation profile has capacity. E02 requires renewed authorization because
its Author and Work dispatches already consumed the approved pair.

## Rerun at db44e45

Session `native-db44e45-handoff-rerun` used clean revision
`db44e45df19e087dc825ad71e3e1f7c47cbb7b7d`, with only report changes since
`a3ff51e`. The cap was eight role calls with token measurement disabled.
Login and the Windows sandbox probe passed. E02 Author was the only role
dispatched. Codex returned a provider usage limit and exit `1` without a
completed handoff. E02 stayed `BLOCKED` and its Work role was `NOT_RUN`.
E04, E06 and E08 were `NOT_RUN` for both roles. E08 has no native compiler
or simulator exit codes and all four safety assertions remain `NOT_RUN`.

The runner and independent reporter both exited `2`. The manifest SHA-256
is `d9cef9afd07671b8adf0d6d1b864aa6b6399879ca2aac4e5aab2bbd80e915e77`.
The sealed evidence index SHA-256 is
`64cef3b725a3ef9864587709d9c2469ef25906b5ad7c56c42b5d4db1df72f786`.
The quality report SHA-256 is
`64cae53252fd94803a44c262f5f11714684f80d7c29574219f2bd5eb8f0013f4`.

## Restored-capacity rerun at db44e45

Session `native-db44e45-restored-capacity` ran on Windows at
`db44e45df19e087dc825ad71e3e1f7c47cbb7b7d`. The worktree had a report-only
edit. The frozen Rules, evaluator and runner digests matched the earlier
`db44e45` manifest. The session dispatched eight of eight approved calls,
one Author and one Work for each case, with token measurement disabled. All
four Work roles received the exact independently reviewed handoff bytes.
The runner and sealed reporter each exited `1`. The overall verdict is `FAIL`.

| Case | Native verdict | Evidence |
| --- | --- | --- |
| E02 | `FAIL` | Work verified the target hash and clean status at `Work/item_2`. At `Work/item_6`, Git could not resolve the Author's role-specific commit (exit `128`). The independently seeded Work repo had an equivalent target blob at a different HEAD. Work stopped before editing or running formatting assertions. Fixture and Git metadata audits passed with zero changes. |
| E04 | `PASS` | `Work/item_5` verified the generated decoder digest. `Work/item_9` (exit `0`) inspected the consumer pin and the selected producer 1.0.0 release, which defines `id` and `title` and omits `badgeLabel`. Work requested an owner-published release and later consumer approval. Sandbox `read-only`, scope `[]`, zero attempted writes, and no file or Git metadata changes. Implementation remains blocked pending the producer release. |
| E06 | `PASS` | `Work/item_10` and `Work/item_14` each exited `0` with one test passing and no skips, before and after the edit at `Work/item_12`. Only the call and unused import in `src/labels.mjs` changed. The audit verified same-file user edits, notes, untracked and ignored data, the live helper and caller, and Git metadata. Supplemental checks observed changed Node `TypeError` wording for numbers and plain objects. |
| E08 | `BLOCKED` at the structured evidence gate | `Work/item_18` reported compiler exit `0` and simulator exit `1` with a startup timeout. After the scoped stimulus correction, `Work/item_22` exited `0` and reported compiler and simulator child exits `0`, plus `permit loss PASS`, `watchdog expiry PASS`, `fault PASS`, `reset PASS` and `simulation PASS`. Work's filter converted the observer JSON to text, leaving `trace.rtlStages` empty. The reporter could grade the assertions but withheld compile and simulation PASS. |

`physical verification: NOT_RUN`

| Local evidence | SHA-256 |
| --- | --- |
| `manifest.json` | `a87bb73da5f7d836a1b5b5620623d8e6cb527bcde067364034aa62ac5092f1da` |
| `evidence-index.json` | `dae93d446c8eeacf36257ae882118f2d3187bf26973a7291873e0c760979ac56` |
| `review.json` | `efbc3d676fdcb1af6ed9f5dac12d16196b6fd53b6758c870982677456392c134` |
| `quality-report.json` | `d093ed968403478019f99aa181364a6fd586cdbc0fa47ac1dc0cd2f33c54e189` |

Before another acceptance run, distinguish synthetic role commits from the
shared fixture identity in E02 handoffs and ensure E08 output filtering
passes the safe structured observer markers through unchanged. Keep the
existing integrity and evidence gates. Native acceptance remains open for
E02 and E08.

## Git baseline and RTL evidence correction at 5a8d7f1

Session `native-5a8d7f-git-rtl-correction` ran on Windows on 20 November
2026. The source was `5a8d7f16f26aa0897519bcb32f495c72602a5ecc` plus the
pre-run patch with SHA-256
`7b411bc779899a25de21964d92164defc084b639573ab4b9a655625adaa3709d`.
The patch directs Work to check its own Git state and reconcile shared files
by hash. It also provides a PowerShell summary that keeps the observer JSON,
assertion markers and wrapper exit. Offline build, documentation and benchmark
regressions passed. Cargo tests observed 61 passes and one skip on Windows and
62 passes on Ubuntu 24.04. The staged filter also passed a separate Windows
fixture run using Icarus.

The session used seven of eight approved native calls. Token measurement was
disabled, and the initial measurement and evidence hashes still match. The
runner and independent reporter each exited `2` with overall `BLOCKED`.

| Case | Native verdict | Evidence |
| --- | --- | --- |
| E02 | `BLOCKED` | Author used Work-local Git preflight but incorrectly listed its role-specific `AGENTS.md` as a shared SHA-256 pin. A separate no-call Work preview confirmed the hashes differ. Independent review withheld Work. Native edit and format assertions are `NOT_RUN`. |
| E04 | `PASS` | `Author/item_19` inspected the 1.0.0 schema, which lacks `badgeLabel`. `Work/item_2` and `Work/item_6` matched the pin, decoder and release hashes and confirmed the published version. Work ran with `read-only`, scope `[]`, zero attempted writes and unchanged fixture and Git metadata. The feature remains blocked pending a producer release. |
| E06 | `PASS` | `Work/item_13` and `Work/item_15` each recorded exit `0`, one test passed and zero skips, before and after the edit in `Work/item_14`. The audit verified only `src/labels.mjs` changed and preserved existing edits, tracked notes, untracked and ignored files, the live helper and Git metadata. The `TypeError` class stayed the same. Diagnostic wording changed for some invalid inputs. |
| E08 | `PASS` | `Work/item_15` recorded compiler exit `0` and simulator exit `1` at `startup timeout`. `Work/item_19` recorded compiler exit `0`, simulator exit `0`, and PASS for reset, permit loss, watchdog expiry and fault. The reporter accepted the structured child records. Only `sequence.svh` and `README.md` changed. |

`physical verification: NOT_RUN`

| Local evidence | SHA-256 |
| --- | --- |
| `manifest.json` | `866cdca4e79f893cb539823d91a629b8846e707cda41a9f67a81863d820a0778` |
| `evidence-index.json` | `0a54fdc9bcd23de4ae327b640888fb501d14be0e5884d58e9008b2a4d516345c` |
| `review.json` | `d52912afded9123f784f007d990a92fb5cbdc5e82b7fbba02095e9b7fd96af76` |
| `quality-report.json` | `c3410eb8a80bbc28c2071cc51ac53f3bfd7c13db0a2defd36a1f670a244c14e3` |

E02 remains open. A fresh Author handoff needs to keep role-generated files
out of shared baseline pins. Its Author allowance for this session was used.

## E02 acceptance at df5f32e

On 21 November 2026, session `e02-df5f32e` ran on Windows against
`df5f32e74ec51ffaccb93a2365f6b4f363f31d71` plus the pre-run patch with SHA-256
`7614069aeff847fa8e0b7d06f1a970ec3d4f388ac13cc491d0d2497d72c01693`.
The session used one native Author and one native Work with the dedicated
evaluation profile. Token measurement was disabled. The initial reservation
and evidence hashes still match.

The E02 runner now supplies the shared hashes of `README.md`, `duration.mjs`
and `duration.test.mjs` from the frozen fixture. The Author output gate rejects
the generated Author `AGENTS.md` hash. The old sealed handoff reproduced the
failure before the fix and was rejected by the corrected gate on replay. Build,
Cargo tests, EN/VI documentation, Work Prompt, benchmark session and benchmark
privacy checks passed on Windows. Cargo tests had 61 passes and one ignored test.

**E02 native verdict `PASS`.** Work received the exact reviewed handoff and
confirmed its local Git state and the three shared hashes. `Work/item_10` ran
the unchanged test before the edit and exited `1` on `0:0` versus `00:00`.
`Work/item_13` edited only `duration.mjs`. `Work/item_14` ran the same test and
exited `0` with all four required assertions and no skips. `Work/item_15`
exited `0` after comparing every one of the 3,600 supported inputs to the
expected `MM:SS` output. Fixture scope and Git metadata audits passed. The
runner and independent reporter each exited `0`.

| Local evidence | SHA-256 |
| --- | --- |
| `manifest.json` | `80c53aba18db0fc9c92e449c48e22e456510985f1693e0a7fdc3d1dc7676184a` |
| `evidence-index.json` | `45d0b0f4b88198477d4b70ea5ad632a5e0541602fa119228948e6bd1b2b4c5aa` |
| `review.json` | `1006c05c7e0f8bb2a1bd3d87988982b856a0ab62df56c2962dcb26b028c0d929` |
| `quality-report.json` | `08249fc52f06847466b5469c09f18d2e06efce3aa5d627f4d9da7c2d3b9a9c89` |

## E01, E03 and E07 offline acceptance at 338bac1

On 21 November 2026, Windows offline checks used clean base revision
`338bac14417300ea77098a4c4cf6da161c5bee63` and Node `v26.12.1`. The code,
test and guide patch has SHA-256
`70b1ba9218b0dd4076107a901d066ad10d7ac06efc82b610a6d86db7ac405376`.
The frozen scenario catalog and tracked fixtures retain their original bytes.

| Case | Outcome | Command and evidence |
| --- | --- | --- |
| E01 | `BLOCKED` before dispatch | `run.ps1 -CaseId E01` preview exited `0`. Offline `role.ps1 -Execute` checks for Author and independently reviewed Work used a deliberately absent client and each exited `0` after recording `BLOCKED`. Neither role had a process, command, start time or model trace. The inspector limits its own output, while the runtime lacks an approved preventive read boundary for this fixture. Native behavior on export actor, records and fields is `NOT_RUN`. |
| E03 | TypeScript model `PASS` offline | In a disposable copy, `node --test tests/*.test.mjs` exited `0` for the original success test, `1` after adding a regression, and `0` after the fix. The red test observed `failed` before it failed on `loading === true`. The green run passed both tests with zero skips. Only `save-panel.ts` and `tests/save-panel.test.mjs` changed in that copy. The Vue component, its public interface, template and style, and graph preview were byte-identical. Browser and end-to-end checks were `NOT_RUN`. |
| E07 | Unit `PASS`, partner integration `BLOCKED` | `node verify.mjs` exited `0` with `NODE_OPTIONS` set to `--permission --allow-fs-read=.`. Node permitted fixture reads and denied writes. Output was `unit PASS (2 assertions)` and `integration SKIP (1 check; inputs/partner-batch.json unavailable)`. The verifier and fixture hashes matched before and after. Completion stays open. |

The previous reporter accepted `partner-integration PASS` based on a completed
zero-exit command even when the sealed E07 fixture lacked the partner input.
The corrected reporter rejects that review before writing `quality-report.json`.
The regression submits both the invalid review and a valid review that keeps
`unit PASS` and `partner-integration BLOCKED`. The rubric and optional coverage
classification are unchanged.

| Local evidence under `target/native-quality-reliability/` | SHA-256 |
| --- | --- |
| `338bac1-e01-chain-offline.json` | `489ab8e35f6c4b8b2dde640e618614f39e7a928bf486cf4e1ffaf18080bb5f87` |
| `338bac1-e03-offline.json` | `c9fc67f7cc2986debffb0f4f0eed76a7aa3a92a203ea653b115838ed730e6734` |
| `338bac1-e07-offline.json` | `803b3c3b8d7b189edf02a0331522930e5c01e5e655958b2a1bdc1626fcd2dd96` |
| `338bac1-e07-reporter-red-confirmed.log` | `8924281766bdcb308b3f49ad57bd36e994b8c75fd78d49554b1bf4ee7b2d7e75` |

Native E03 and E07 are `NOT_RUN` in this offline record. They require separate
budget approval, the approved evaluation profile and independent handoff review.

## E01 preventive read boundary at 3253cbb

On 21 November 2026, Windows verification used clean base
`3253cbb6f7af0dea10429b26b5ab7ffcd8f28419` plus candidate patch SHA-256
`738a1b662c591962a7cce27a0c3b7f471b6a0a8e949dc11b425161b8c8eac645`.
The focused changes cover the role runner, reporter, privacy and role tests,
and EN/VI guides. The catalog, seed fixtures and historical reports were
preserved. No native model calls were dispatched in this record.

| Case | Finding and action | Native result and remaining work |
| --- | --- | --- |
| E01 | Windows Codex CLI `0.169.0-alpha.11.1` with `codex sandbox -P :read-only` allowed reads of the harmless outside file (exit `0`). The named MXC read profile denied direct, parent-directory, junction, and unlisted sibling reads through Node and PowerShell and denied fixture writes. Both production probes exited `0`. `codex app-server` `thread/start` reported the explicitly selected profile with network disabled and issued no model turn. The runner omits the raw account store and stages only bounded structural metadata with the source digest. The inspector exited `0` and exposed the public projection and protected field names without values. The audit now checks both private field families. | `NOT_RUN` for Native behavior. Windows denies creation of the separate file symlink test fixture, which remains `NOT_RUN`. The runtime junction probe passed, staging rejects reparse points, and the read profile denies writes. A separately authorized run is still needed to observe the Agent's export clarification. Unsupported or failed preflight stays `BLOCKED` before dispatch. |
| E03 | Reused the red/green TypeScript model evidence above. The commit preserved the previously tested E03 fixture and requirements byte for byte. | Native `NOT_RUN` pending separate budget and profile approval. |
| E07 | Reused the recorded read-only verifier result. The focused reporter regression exited `0` and preserved the distinction between unit PASS and integration BLOCKED. | Native behavior `NOT_RUN` pending separate budget and profile approval. Partner integration remains `BLOCKED` pending genuine partner input. |

| Local evidence under `target/native-quality-reliability/` | SHA-256 |
| --- | --- |
| `3253cbb-e01-mxc-read-probe.json` | `5d1d14966417d4cfc627edc643a3f319397a1041cada93dd95f2eded80ae3ec4` |
| `3253cbb-e01-thread-policy.json` | `18c996bd8ba9fc7aef4c4fdf714e7e6c118a2893761fde3a87bb83c3b59e0d9e` |
| `3253cbb-e01-boundary-verified.json` | `bad856580132d4dfac5019498e56c75e6efa03ee1514632a08c341fbf4901c0e` |
