# Changelog

## 0.2.0 (candidate)

- Clarify export authorization and safe source inspection. Fix RTL subprocess
  scratch paths and inherited standard handles in MXC; add focused E01/E08 checks.
- Support isolated native Codex authentication, portable tool paths and Windows
  MXC execution. Record matched evaluation outcomes and correct related-docs grading.
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
