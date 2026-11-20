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
