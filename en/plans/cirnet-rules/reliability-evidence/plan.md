# CIRNET reliability evidence plan

This plan implements the [reliability evidence specification](../../../specs/cirnet-rules/reliability-evidence/spec.md)
in the `cirnet-rules` checkout on `master`. Intake verified baseline
`15175dd1faaf8b7f56f0eb93124417dc200a892e` and a clean working tree.
The operator subsequently authorized direct implementation in that checkout.

## Phase 0 capability audit

Paths below are relative to the owner checkout unless labeled otherwise.
The status describes the inspected baseline. Existing tests named here
identify coverage. The observations recorded below are the executed evidence
for this audit.

| Capability | Baseline implementation and owner | Covered behavior and evidence | Gap and bounded response |
| --- | --- | --- | --- |
| Environment discovery | Rules guidance in `en/docs/context.md`. Owner manifests and verification guides. Interface `package.json` inspected read-only | `GUIDANCE_ONLY` for selecting an owner check. Manifest names are candidates and permissions are separate | Add an explicit bounded preview for synthetic npm, Cargo, CMake and RTL sources. Read only nominated files and emit reviewed metadata without launching commands |
| Build and run | Rules `scripts/task-continuity.ps1 run` and `scripts/continuity/Verification.psm1` | `IMPLEMENTED_VERIFIED` for the synthetic direct executable boundary. The wrapper tracks pins, timeout, cleanup and its observed process exit | Outer exit alone leaves child and assertion coverage open. Keep owner execution and capture separately approved |
| Test evidence | Rules `scripts/benchmark/role.ps1`, `scripts/benchmark/report.mjs` and `tests/benchmark/rtl.tests.ps1`. Assertions remain owner-defined | `PARTIAL`. The reviewer checks command exits and RTL child markers. The direct wrapper probes below show report gaps for ordinary stage evidence | Use pinned owner observations for required/optional evidence. Keep benchmark cases, runner and reports intact |
| Log analysis | Rules `scripts/benchmark/role.ps1`, `scripts/benchmark/inspect-source.mjs` and `tests/benchmark/privacy.tests.ps1` | `IMPLEMENTED_NOT_VERIFIED` for benchmark bounded parsing and canary detection. Source grep establishes the configured scope | Keep raw output with owners. Reuse bounded summaries. Canary findings detect exposure and require separate preventive controls |
| Reproduction | Rules `en/skills/cirnet-debugging/SKILL.md`, `tests/benchmark/checks.test.mjs` and selected fixture contracts | `PARTIAL`. Fixtures exist and the three wrapper limitations below were reproduced in isolation | Turn the missing claim rules into deterministic negative fixtures. Preserve baseline assertions |
| Self-correction | Rules `en/skills/cirnet-debugging/SKILL.md` and `en/skills/cirnet-regression-testing/SKILL.md` | `GUIDANCE_ONLY` for per-failure-cluster correction limits. Deterministic admission for retry history was missing | Evaluate the retry policy from observed history. Leave actions and approval with the owner and protect expected results and permissions |
| UI | Rules `en/skills/cirnet-engineering-workspace-ui/SKILL.md`. Interface `package.json` and owner verification form the product lane | `GUIDANCE_ONLY` in Rules. Interface build, test, boundary and E2E entries were discovered. Product commands were unexecuted | Preserve feature authority and distinguish static review, mock browser and installed product evidence |
| Performance | Rules `scripts/benchmark/role.ps1` records Candidate duration and optional authorized input usage. The product owner controls comparable workloads | `IMPLEMENTED_NOT_VERIFIED` for native measurement. Comparable product baseline is `NOT_MEASURED` | Keep measurement conditional on a pinned owner lane. Report no performance or efficiency improvement from these fixtures |
| Architecture and scope | Rules `en/policies/architecture.md`, `en/references/architecture-routing.md`, `hooks/check_docs.rs` and `scripts/shared/WorkPrompt.psm1` | `IMPLEMENTED_NOT_VERIFIED` at intake for docs integrity and pinned handoff scope. Handoff reconciliation detects net differences | Use owner-approved preflight and evidence paths. The execution environment enforces access and isolation |
| Completion | Rules `scripts/continuity/Verification.psm1`, `scripts/benchmark/report.mjs` and `en/rules/testing.md` | `PARTIAL`. The direct probe observed a failed process, stale inputs and missing tool. Independent Candidate review remains a separate gate | Assess the exact claim and identify every mandatory uncovered stage. Preserve established acceptance models and require fresh scoped evidence |

### Observed gaps

The direct Task Continuity process observation returned `PASS` and outer exit
`0` in three synthetic cases. One launched a nested child that exited `7`,
one declared a required stage skipped for a synthetic reason, and one produced
no child exit evidence. These show the difference between process exit and
claim coverage. Independent direct probes observed `FAIL` for exit `17`,
`NOT_RUN` for timeout with no exit, `BLOCKED` for an unavailable tool, and
`NOT_RUN` after a fingerprint became stale.

### Reuse decision

Owner-native commands and existing CI select and execute approved checks.
Task Continuity records direct execution and freshness. Handoff validation
and Candidate review already own approval and independent review. The
remaining needs are narrow data checks for stage coverage, explicit preview
and correction admission. Their cost is bounded readers, small classifiers
and focused fixtures. A shared executor or orchestrator would duplicate
existing lifecycle and authorization responsibilities and increase the testing
and maintenance surface. Each new check produces a scoped result.

## Implementation and verification

Commands below run from the `cirnet-rules` root.

- [x] **Phase 0.** Inspect current authority and reproduce the three wrapper
  observations. Attribute gaps to the recorded observation scope.
- [x] **Phase 1.** Add `scripts/verify-claim.ps1` using existing bounded
  read, hash and Git helpers. Require pinned owner records, current inputs
  and actual stage evidence. Keep the Continuity engine and schema intact.
  `./tests/verification/verify-claim.tests.ps1` returned exit `1` before the
  entrypoint existed and exit `0` after implementation. The fixture covered
  the nested child exit, required and optional skips, unavailable or timed-out
  stages, stale pins, minimum assertions and malformed observations.
- [x] **Phase 2.** Add `scripts/preview-owner-checks.mjs`. Its synthetic npm,
  Cargo, CMake and RTL checks passed with unchanged fixture files and no
  nominated command execution. The directory-link fixture was skipped by
  Windows permissions. A real read-only Cargo manifest preview selected the
  requested checker and left execution `NOT_RUN`.
- [x] **Phase 3.** Add the advisory retry module. Six tests passed, including
  two observed child exits of `7` followed by refusal of another unchanged
  retry, two launched repairs and refusal when scope, safety or progress
  was missing. The module returned decisions without executing commands.
- [x] **Phase 4.** Add the new tests to the existing Windows/Ubuntu matrix.
  Run the affected lanes locally and verify bilingual docs, the development
  bundle and scope. The table records the observed Windows results.

| Command or lane | Observed local result |
| --- | --- |
| `./tests/verification/verify-claim.tests.ps1` | Exit 0. Stage validation included wrapper zero with a failed child or required skip, missing observations, insufficient assertions, stale pins, unavailable prerequisites and valid PASS. |
| `node --test tests/verification/preview-owner-checks.test.mjs tests/verification/retry-policy.test.mjs tests/benchmark/source.test.mjs tests/benchmark/rtl.test.mjs` | Exit 0. 19 passed and one directory-link fixture skipped. |
| `cargo fmt --all -- --check`, `cargo test --offline --locked`, `cargo clippy --offline --locked --all-targets -- -D warnings` | Exit 0 for each. 61 Rust tests passed and one Windows symlink case was ignored. |
| `cargo run --offline --locked --bin check-docs` with the four new Plan/Spec paths as `--new-file` | Exit 0. EN/VI pairs, hashes, links and all 17 Skill definitions checked. |
| `./tests/continuity/task-continuity.tests.ps1`, `./tests/continuity/bootstrap.tests.ps1`, `./tests/validation/work-prompt.tests.ps1` | Exit 0 for each. Windows skipped the symbolic checkpoint-root fixture. |
| `./tests/distribution/package.Tests.ps1`, `./tests/distribution/install.tests.ps1` | Exit 0 for exercised cases. Windows skipped link and reparse fixtures. |
| `cargo build --offline --locked --bin check-docs`, `scripts/package.ps1 -AllowDirty -Apply`, `scripts/package.ps1 -Verify` | Local build and archive verification passed. The development ZIP contains 17 Skill definitions. |
| `git diff --check HEAD` and protected-path review | Exit 0. Existing engine, benchmark, packaging code and historical paths match HEAD. |

Git HEAD remained `15175dd1faaf8b7f56f0eb93124417dc200a892e`. The scope
review covered tracked and untracked source paths. Run outputs and the local
checkpoint live under the owner's ignored locations. Hosted CI, Linux, native
agent selection, product E2E and physical acceptance remain `NOT_RUN`.
Comparable reliability, context and performance metrics remain `NOT_MEASURED`.
A future owner-approved pilot can test one bounded native task with independent
review of its selected checks and completion evidence.
