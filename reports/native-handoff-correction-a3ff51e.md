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
