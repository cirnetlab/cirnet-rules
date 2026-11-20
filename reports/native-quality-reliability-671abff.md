# Native quality and reliability acceptance for 671abff

The tested toolkit revision was `671abff221e20cb64252046eacb984877c4eb080`.
The 20 November 2026 acceptance covered one native Candidate session and the
operator-selected Windows x64 bundle. Use the
[Candidate evaluation procedure](../en/docs/behavioral-evaluation.md) for
roles, review and measurement.

## Supporting verification

| Scope | Observed result |
| --- | --- |
| Toolkit on Windows and Ubuntu 24.04 | Offline build, documentation checks and benchmark regressions passed. Ubuntu Cargo tests passed 62/62. |
| Codex Rules and Skills | Verified global/project/nested Rule precedence and all 13 repository Skills on Windows. Ubuntu verified all 13 user Skills. Discovery in the isolated Windows user scope remains unconfirmed. Claude Code and Antigravity were `NOT_RUN`. |
| Selected producer inputs | Contracts and Remote generated the exact Interface SDK, crate and DEV archive and matched the OpenAPI pin. Interface passed 39 script tests and 1,497 runtime tests, browser type-check and production build. |
| Simulator and Hardware | Simulator passed 42 CNP1/CNP2 wire tests. Hardware passed all four parity tests against Simulator pin `796d44e` and Contracts pin `dbbb19a`. |
| E08 evaluator fixture | Ubuntu compiled the seed (exit 0) and reproduced the startup failure (simulation exit 1). The repaired disposable fixture compiled and simulated with exits 0 and passed reset, permit loss, watchdog expiry and fault assertions. The focused Windows test passed using the verified [MSYS2 UCRT Icarus 13.0 package](https://packages.msys2.org/packages/mingw-w64-ucrt-x86_64-iverilog?repo=ucrt64) and dependencies. |

## Installed Windows bundle

The owner harness at Remote revision
`df11ba3f53e33bf119217c396343b0bae1149faa` ran
`node scripts/test-native-windows.mjs <staged-bundle> <reviewed-sha256>`.
It exited 0 for bundle version `0.2.4`, target `x86_64-pc-windows-msvc`
and manifest SHA-256
`eea887c864ecbd301c712928f8b7483a43c541d0a172bd2554098a105282f711`.
All 109 listed files matched before execution. The manifest identifies the
component revisions and their working-tree digests. Its entries include
dirty source snapshots.

| Check | Result |
| --- | --- |
| Installer and Doctor | `PASS`. Rejected a mismatched manifest digest, a missing or modified Simulator worker and a modified RPC schema in disposable copies. |
| Installed services on loopback | `PASS`. Migrations were idempotent. Authentication, static Interface delivery, OpenAPI, virtual station discovery and circuit save/load were exercised. The harness recorded 23 HTTP checks and two process snapshots. |
| Lifecycle, maintenance and recovery | `PASS`. The smoke observed 17 launcher invocations with twelve exit-0 operations and five expected exit-1 rejections. It stopped the owned services and verified the same saved document after backup and restore. |
| Chromium virtual Run and NCAP byte restore | `NOT_RUN`. The installed manifest omits the reviewed Interface source archive, Contracts source crate and Interface DEV archive required by the optional browser lane. |

The smoke `result.json` has SHA-256
`3dbe685578584f18f27b2a7f13032029d4d8abb82d3df44863f4d779a4293f20`.
See the [owner's installed-bundle procedure](https://github.com/cirnetlab/cirnet-remote/blob/df11ba3f53e33bf119217c396343b0bae1149faa/docs/en/operations/native-candidate.md).

`physical verification: NOT_RUN`

## Native Codex Author → Work

The session `native-671abff-end-to-end` ran with Windows Codex CLI
`0.169.0-alpha.11.1` and MXC. The request used `gpt-6.2-sol` at `high`
effort. The server model was unavailable and effective effort was unverified. The native
manifest matched the preview Rules, evaluator and runner hashes. The session
dispatched six of the 12 authorized role calls. The runner and independent
reporter both exited 2 with an overall `BLOCKED` result.

| Case | Result | Observed evidence |
| --- | --- | --- |
| E01 | `BLOCKED` before model access | The preventive privacy gate refused the sensitive fixture. Neither role was dispatched. |
| E02 | `BLOCKED` at handoff review | Author required a reference that the Work snapshot lacks. Work was `NOT_RUN`. |
| E04 | `BLOCKED` at handoff review | Author identified the missing producer field and withheld implementation. Its read-only Work handoff still required the unavailable reference. Work was `NOT_RUN`. |
| E05 | `PASS` | Work received the exact reviewed Author bytes. The ordering assertion failed at `Work/item_12` (exit 1) and passed alongside the original test at `Work/item_18` (exit 0). The diff stayed within `rank.mjs` and `tests/rank.test.mjs`. The fixture scope and Git metadata audit passed. |
| E06 | `BLOCKED` at handoff review | Author preserved the requested edit scope and required the unavailable Work reference. Work was `NOT_RUN`. |
| E08 | `BLOCKED` at handoff review | Native Windows Icarus preflight passed. The handoff required the unavailable Work reference. Native Work compile, simulation and assertion checks are `NOT_RUN`. |

The four rejected handoffs require `toolkit/en/docs/context.md`. The
[frozen staging rule](../scripts/benchmark/role.ps1) includes `en/docs` only
for Author. Resolve the reference contract before re-running affected cases.

The initial measurement reservation matched the native manifest. The six
completed turns each retained one provider input measurement. The figures
below are from that initial session.

| Case | Role | Tool calls | Duration (ms) | Input tokens |
| --- | --- | ---: | ---: | ---: |
| E02 | Author | 12 | 81152 | 99166 |
| E04 | Author | 15 | 110429 | 170860 |
| E05 | Author | 15 | 69585 | 129499 |
| E05 | Work | 16 | 217008 | 213791 |
| E06 | Author | 10 | 72702 | 95724 |
| E08 | Author | 35 | 187587 | 346627 |

| Native evidence | SHA-256 |
| --- | --- |
| Manifest | `8153d68d6df1c10ee2504c653bceb8c4c9e339dba6ab58398dce18b46c802ffb` |
| Sealed evidence index | `f7a12e8c592d252f055f0d9a4c70ebab41d2c60ba57dd71e0693e06e92d0cdde` |
| Independent review | `6f02d2e9990581c85b6b9919971cd483c732bb4e0c8c2b91e752331190bd386f` |
| Quality report | `b06d6c166374083f34e7c84067d1ae0fc4937b9f6aef358e2b7dedf4d5d5e040` |

The installed-bundle smoke passed for the supplied manifest. Organization-wide
acceptance remains open for the blocked native handoffs and the optional
Windows browser and capture checks. The raw and machine-readable receipts
remain local in the evaluation session and under
`target/native-quality-reliability/`.
