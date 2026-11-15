# Engineering behavior upgrade

Start on 2026-11-15 at `a24c94aeb43d95dbf05f8d53f8676676dccba2e2`.

- Add clarification, simplicity, scoped changes and verifiable outcomes to the
  baseline, coding rule and six skills. Package four original synthetic
  Before/After examples in the implementation skill. Keep 12 skill IDs,
  portable resources, EN/VI and safety.
- Evaluate eight synthetic cases against their rubrics and write scopes.
  Freeze inputs and use fresh fixtures and isolated profiles. Match settings
  when comparing runs. For scoped candidate reruns, use Attempt 2 and explicit
  case IDs. Keep native evidence separate from offline checks and record context size.
- **E01.** Clarify who may export which records and fields when authorization
  is unclear. Editing permission does not authorize export. Inspect only
  relevant source, filter sensitive values before printing and preserve the fixture.
- **E08.** Verify the portable toolchain in MXC with workspace-write and
  approval never. Use owned TEMP/TMP/TMPDIR and pipes for subprocess I/O.
  Run simulation only after successful compilation. Preserve the DUT,
  assertions and deadlines. Require observed exits for both stages,
  all four safety assertions and `physical verification: NOT_RUN` in native evidence.

Limit writes to `rules-common` and dedicated test output. Protect prior
evidence, product repositories, real client settings and Git refs.

[Plan](../../../plans/cirnet-rules/engineering-behavior/plan.md) ·
[Acceptance report](../../../docs/e01-e08-followup.md)
