# Changelog

## 0.2.0 (candidate)

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
