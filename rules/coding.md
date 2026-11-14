# Choosing the smallest coherent implementation

Use this supplement for design or refactoring decisions after reading the affected
code and [baseline](../AGENTS.md).

- Trace inputs through the existing boundary to outputs. Prefer extending an
  established path over adding a second source of truth.
- Separate necessary behavior changes from unrelated cleanup. A refactor should
  preserve observable behavior unless the task explicitly changes it.
- Read direct callers before changing an interface. Preserve error, cancellation,
  timeout and cleanup semantics that callers rely on.
- Generate derived files from the owner-defined source and command. If the producer
  or generator is unavailable, report the gap instead of editing generated output
  as though it were canonical.
- Evaluate a new dependency against existing facilities, maintenance cost and the
  repo's dependency policy. Do not standardize languages or tool versions globally.
- Update affected documentation with behavior changes. Local rules determine
  translation, plan/spec placement and architecture decisions.
