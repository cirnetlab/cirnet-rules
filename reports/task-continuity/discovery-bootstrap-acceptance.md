# Independent acceptance of Task Continuity discovery and bootstrap

The review started with a clean Windows checkout of `master` at
`5486a72396f3c3f4fe41a78114a15efe007d60df`. The corrected source is that
revision plus the local changes to `Checkpoint.psm1`, `Discovery.psm1`, and
`bootstrap.tests.ps1`. The SHA-256 of their Git diff is
`0bf1109a3ab0844b2dd319fe72567d4410f1351fb72a4177b7489c8406059d95`.
The review used the [Unified Specification](../../en/specs/cirnet-rules/task-continuity/spec.md),
[distribution guide](../../en/docs/distribution.md), and [repository rules](../../AGENTS.md).
Logs and disposable fixtures are under the ignored
`target/independent-bootstrap-acceptance/` directory.

## Offline functional acceptance

| Criterion | Result on Windows | Reproducible evidence |
| --- | --- | --- |
| Unique eligible task | PASS | Bootstrap suite starts with an empty store, checks `NONE`, then saves one task and checks automatic selection, sealed context and matching Git. |
| Multiple tasks and privacy | PASS | Bootstrap suite adds a second eligible task and checks `AMBIGUOUS` with candidate IDs and no recovered notes. An unknown session key cannot reuse a worktree binding. |
| Durable session binding | PASS | Bootstrap suite tests initialization and explicit replacement. An independent `pwsh -NoProfile` process opened the fixture with `-SessionKey local-acceptance` and reported `SelectedBy = binding`, the correct task, all four pinned references and current syntax evidence. See `native-bootstrap-oracle.json`. |
| Invalid selection and integrity | PASS | Bootstrap suite checks missing, stale, copied and corrupt bindings, damaged or locked checkpoints, conflicting ownership, wrong agent family, separate worktrees and bounded scans. The duplicate-key regression covers both current pointer and manifest. |
| Concurrent selection changes | PASS | Bootstrap suite publishes a second task between selection and revalidation, rejects a superseded checkpoint ID during a separate process publication, and rejects a duplicate ID introduced between selection and revalidation. |
| Git and verification freshness | PASS | Bootstrap and lifecycle suites exercise staged, unstaged and untracked state, Git drift and conflicts, interrupted child processes, stale evidence and changed pinned rules or verification inputs. |
| Recovered context | PASS | Bootstrap compares full and compact sealed task and progress with the original inputs. The lifecycle suite verifies Git inventory, the acceptance mapping, pinned Rules and Plans/Specs, and preserved scope, decisions and next action. |
| Continuation authorization | PASS | Bootstrap requires current scope confirmation, continuation confirmation and a matching reviewed checkpoint ID. Tests with drift, an interrupted process or damaged selection leave `CanContinue` false. The manager records an operator declaration of scope. Runtime permissions and authorization remain with the active session. |
| Compatibility | PASS | Lifecycle covers previous-snapshot recovery, v1/v2 migration, v3 review and closure, and portable handoff. Work Prompt, local benchmark integration, installer, packaging and bilingual checks exercise the corresponding workflows and the 13 Skills. |

### Commands and recorded exits

Windows used PowerShell 7.6.9, Rust 1.99.1 and Node 26.12.1. PowerShell test
paths below were passed to `pwsh -NoProfile -File`.

| Command | Result and exit |
| --- | --- |
| `tests/continuity/bootstrap.tests.ps1` | PASS 0 on the original source with its original tests. FAIL 1 with the added regression on the original engine. PASS 0 on the corrected engine. The Windows symbolic checkpoint-root case was NOT_RUN due to permissions. |
| `tests/continuity/task-continuity.tests.ps1` | PASS 0 on the original and corrected engines. |
| `cargo fmt --all -- --check`, `cargo build --offline --locked`, `cargo clippy --offline --locked --all-targets -- -D warnings` | PASS 0 each on corrected source. |
| `cargo test --offline --locked` | PASS 0 on corrected source, 61 passed and one Windows directory-symlink case ignored. |
| `cargo run --offline --locked --bin check-docs`, `scripts/check-changed-docs.ps1 -Base 10272c6be4c98273dee9445345c6839a00d99a56`, `tests/validation/check-changed-docs.tests.ps1` | PASS 0 each on corrected source, with existing context-budget and architecture-variant advisories. |
| `tests/distribution/package.Tests.ps1`, `tests/distribution/install.tests.ps1` | PASS 0 each on corrected source. The reparse fixture was NOT_RUN due to Windows permissions. The first clean-checkout package run failed with exit 1 because checkout `LICENSE` used CRLF and its blob used LF. The clean LF checkout of the same SHA and a separate package run on the corrected working tree each passed with exit 0. |
| `tests/validation/work-prompt.tests.ps1`, `node --test ./tests/benchmark/source.test.mjs ./tests/benchmark/rtl.test.mjs`, `tests/benchmark/rtl.tests.ps1` | PASS 0 each on corrected source, including nine Node tests. |
| `tests/benchmark/role.tests.ps1`, `tests/benchmark/followup.tests.ps1` | PASS 0 each on corrected source. |
| `tests/benchmark/session.tests.ps1` | PASS 0 on corrected source with the local shim. Its simulated failure and quota cases exercise benchmark handling. |
| `git diff --check` and manual report review | PASS 0, including relative links, title and whitespace. The documentation checker rejected the report as a `--new-file` input with exit 2 because `reports/` is outside its scanned inventory. |
| Ubuntu execution or revision-matched CI | BLOCKED. `wsl -l -v` returned 0 and listed only the stopped `docker-desktop` distro. `docker version` returned 1 because its Linux engine was unavailable. The GitHub Actions exact-SHA query returned HTTP 404. The corrected source is a local diff. Ubuntu-only benchmark checks were NOT_RUN. |

### Confirmed defect and correction

Insert a contradictory `checkpointId` as the first member in the JSON for
either `current.json` or the selected snapshot's `manifest.json`, and keep the
original field and all payloads intact. Run bootstrap for the fixture's verified
binding with the reviewed checkpoint ID and `-ConfirmContinuation`. The
original engine silently used the last field and returned
`ContextRecovered = true` and `CanContinue = true` for each record. The
corrected engine reports `BLOCKED` with both flags false. These commands return
exit 0 because bootstrap reports selection using structured status. See
`duplicate-record-reproduction.json`.

Pointer and manifest reads now share the binding's duplicate-property
validation. Explicit and bound bootstrap recheck both records after live Git
and evidence reconciliation. The fix leaves the checkpoint formats and
publication scheme intact. The focused regression also tests implicit
discovery and the revalidation boundary.

## Native Codex and quota recovery

| Criterion | Result | Evidence |
| --- | --- | --- |
| Fresh Codex session with “Resume unfinished task” | BLOCKED | The initial read-only CLI launch failed to initialize the client with exit 1. An approved launch of a fresh ephemeral session completed with exit 0, but the receiving session's execution policy rejected reading the local manager and running `git status`. It returned no recovered checkpoint context. The CLI events and stderr are retained locally. |
| Fixture and independent recovery | PASS offline | The disposable fixture had an unborn `native-acceptance` branch, a staged and unstaged `count.mjs`, and untracked `fixture-note.txt`. `node --check count.mjs` exited 0. Direct local bootstrap verified `native-count-items` at `45382a4ed5554dbaadbc92007ce155bb`, matching Git, and Next Action “Review count.mjs and add the null guard after operator approval.” Rechecking after the Native attempt confirmed the same checkpoint and Git state. |
| Native Codex account handoff | NOT_RUN | The receiving session used the same account. |
| Actual quota-exhaustion recovery | NOT_RUN | The fixture session encountered an execution policy block. |
| Native Claude Code account handoff | NOT_RUN | Outside this acceptance scope. |

**Recommendation.** Proceed to human review of the exercised Windows offline
scope. Keep cross-platform acceptance open for Ubuntu and the Windows symlink
cases. Keep Native recovery and account handoff open pending evidence from an
authorized receiving session. Final acceptance remains with the user.
