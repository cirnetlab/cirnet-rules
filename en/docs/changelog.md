# Changelog

## 0.2.0 (candidate)

- Clarify export authorization and safe source inspection for E01.
- Fix RTL subprocess scratch paths and standard I/O in MXC. Record actual
  compile and simulation exits and explicit physical status for E08.
- Support isolated native Codex authentication and portable tool paths. Add
  focused harness checks and consolidate [E01–E08 acceptance](e01-e08-followup.md).
- Integrate four engineering behavior principles into the baseline and existing
  skills. Include on-demand Before/After examples and a synthetic Codex
  evaluation suite with offline harness checks.
- Shorten the standalone baseline and load skill bodies and references by task.
- Add 12 namespaced skills for implementation, review, compatibility and maintenance.
- Rename `implementation`, `debugging` and `verification` to their `cirnet-` names.
- Add local bundle packaging, managed installation, verification and backup restore.
- Discover skills dynamically in the Rust checker and validate portable resources,
  translation freshness and toolkit version agreement. Add toolkit CI.

Existing installations need a reviewed migration because the default skill directory
names have changed. See [distribution and upgrade](distribution.md).

## 0.1.0

The repository provided a bilingual baseline, three portable skills and a local
documentation checker. The checker and tests moved from Python to Rust.
