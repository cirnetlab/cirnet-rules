# E01/E08 recovery plan

Scope and acceptance: [specification](../../../specs/cirnet-rules/e01-e08-recovery/spec.md).

- [x] Confirm `rules-common` at `b8981de` and preserve historical inputs/evidence hashes.
- [x] Add conditional export authorization guidance and review the EN/VI pairs.
- [x] Diagnose Icarus using the runner's profile/toolchain and model-free MXC probes.
- [x] Freeze candidate bytes and evaluate E01/E08. Preserve both first failures; E01 passed after specific inspection guidance. A justified E08 retry after the demonstrated standard-handle fix reached all safety markers within scope.
- [x] Finish focused toolkit checks and the bilingual [follow-up report](../../../docs/e01-e08-followup.md).
- [ ] Meet all E08 native acceptance criteria: explicit physical NOT_RUN is missing and the green simulator exit was not captured separately. Keep the final behavioral FAIL; no further model retry.

E02–E07, full behavioral suites, real Agent settings, product repositories and
toolkit Git mutations are outside this follow-up. Preserve unsuccessful probes.
