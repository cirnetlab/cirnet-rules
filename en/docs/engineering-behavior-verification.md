# Engineering behavior verification

Date: 2026-11-15. Worktree: `rules-common`. Starting revision:
`a24c94aeb43d95dbf05f8d53f8676676dccba2e2`. The candidate consists of the
working-tree bytes on that revision. The implementing Codex session reviewed
the toolkit and authored this report.

## Guidance and startup context

| Common baseline | Words | UTF-8 bytes | Lines |
| --- | ---: | ---: | ---: |
| Frozen starting revision | 353 | 2,444 | 42 |
| Candidate | 456 | 3,192 | 53 |
| Budget | 500 | 4,000 | |

Candidate root and English baselines match byte for byte. The 12 skill IDs are
unchanged. Six skill bodies and the coding rule now cover relevant engineering
principles. The implementation skill links to four original Before/After
examples in one optional resource. The exported bundle contains 12 skill
entrypoints and the one example resource alongside its three root files. Vietnamese
translations are maintained for readers. Evaluation materials stay outside the
installed skill directories.

Baseline SHA-256: `7dd5148b986e0e48ccb0635cf83024ffd94cc301781a3268a63c836cdd4a8309`.
Candidate SHA-256: `841f567ab11bc35d9c3227397b664fbdde1d64c4678576e65065e9fe7f13a0f9`.

## Toolkit checks

- Rust format and clippy passed. Rust tests passed 41, with one Windows directory
  symlink test ignored because of the required permission.
- Archive generation, extracted resource closure, SHA-256 verification and
  install/verify/restore in a fake Codex home passed. The archive reparse fixture
  and installer destination junction fixture were skipped because Windows denied
  creation. The installer passed the exercised fake-home and rollback checks.
- The Rust documentation checker passed EN/VI pairing, source hashes, local
  links and skill resource checks.
- The offline evaluator self-tests passed 11 checks. The RTL simulation test
  was skipped because `iverilog` and `vvp` were unavailable. The PowerShell
  harness passed preview, dirty-state and denied-write evidence, unavailable
  client, nonzero exit, incomplete JSONL and timeout checks with a test shim.
- Claude Code static review covered the existing root import and packaged skill
  resource. Antigravity static review covered the existing user baseline and
  skill paths. Native startup and discovery for both clients are NOT_RUN.
- Physical hardware, product repositories and product execution are NOT_RUN.

## Codex native behavior

Codex CLI `0.169.0-alpha.11.1` launched E01 from each frozen cohort in a new
temporary fixture and profile. Both E01 turns started, then timed out at the
18-second limit with repeated Windows DNS error `11001` and exit code `-1`.
An isolated login probe returned `Not logged in` and this run had no authorized
API key. The runner blocked dispatch of E02–E08 in both cohorts after detecting
the common network failure. E08 also lacked `iverilog` and `vvp`.

The case definitions and objective evaluator have matching hashes across
cohorts. The runner hash differs because evidence validation was tightened
between attempts. Both attempts used the same default model request, timeout,
`workspace-write` sandbox and `never` approval policy. No model ID was observed.

| Case | Baseline | Candidate | Task checks | Evidence in each cohort | Limit |
| --- | --- | --- | --- | --- | --- |
| E01 | BLOCKED | BLOCKED | No fixture changes in either attempt | `E01/events.jsonl`, `E01/changes.json` | No final response or privacy decision to grade |
| E02 | BLOCKED | BLOCKED | Offline format assertions failed on the seed and passed after a test fix | `E02/run.json`, offline self-tests | Model process not launched |
| E03 | BLOCKED | BLOCKED | Offline TypeScript failure test detected the bug and passed after a test fix | `E03/run.json`, offline self-tests | No native task or browser coverage |
| E04 | BLOCKED | BLOCKED | Offline checks rejected producer, lock and generated file changes. Shim retained a denied write in the trace | `E04/run.json`, offline self-tests | No native contract decision |
| E05 | BLOCKED | BLOCKED | Offline ID-order assertion failed on the seed and passed with a test fix | `E05/run.json`, offline self-tests | No native red–green chronology |
| E06 | BLOCKED | BLOCKED | Fresh seed contains tracked, untracked and ignored changes. Offline checks rejected damage to user edits | `E06/run.json`, offline self-tests | No native preservation behavior |
| E07 | BLOCKED | BLOCKED | Offline unit PASS and partner integration SKIP correctly differentiated | `E07/run.json`, offline self-tests | No native report or partner input |
| E08 | BLOCKED | BLOCKED | Offline checks caught RTL and safety assertion edits. Simulation skipped | `E08/run.json`, offline self-tests | Missing simulator. Physical evaluation NOT_RUN |

The retained local output is `.superpowers/engineering-eval-a24c94a`, ignored
by Git. See `baseline-identity.json` and `candidate-identity.json` for the frozen
inputs and `auth-preflight.json` with `native-preflight*` for the isolated client
probe. `native-baseline/session.json` and `native-candidate/session.json`
index the per-case artifacts in the table.

No native model response completed and no input, cached-input or output tokens
could be measured. Staging project instructions and skill files for discovery
does not confirm their native loading. Filesystem read isolation remains
unverified. No behavioral comparison can be drawn until the paired cases
complete on matching prerequisites and model settings.

Use the [evaluation procedure](behavioral-evaluation.md) with an authorized
isolated Codex authentication route, working network access, Node 22.18 or
later for E03, and Icarus Verilog's `iverilog` and `vvp` for E08.
