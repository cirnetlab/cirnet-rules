# Choosing the smallest coherent implementation

Use this supplement for design or refactoring decisions after reading the affected
code and [baseline](../AGENTS.md).

- Trace inputs through the existing boundary to outputs. Read direct callers,
  contracts and tests to resolve ambiguity. State assumptions that affect
  compatibility, data or safety. If materially different outcomes remain
  plausible, summarize their tradeoffs and ask about the unresolved decision.
  Proceed on stated, reversible assumptions for routine details.
- Extend an established path before adding a second source of truth. Prefer a
  small helper when it solves the requirement. Ground new dependencies,
  configuration or extension points in a current need and weigh maintenance
  costs. Optimize against an observed problem or defined target. Favor readable
  code over a lower line count.
- Preserve validation of untrusted inputs, authorization, timeout, cancellation,
  cleanup, integrity and safety interlocks where required. Keep caller-visible
  errors and other observable behavior stable unless the task calls for a change.
- Tie each group in the diff to the requirement, the observed defect, or checks
  and docs needed for the change. Follow existing style. Leave unrelated naming,
  formatting, architecture and pre-existing dead code alone. Before removing an
  import, variable or helper made unused by your change, check callers, generation
  and dynamic use where applicable.
- Generate derived files from the owner-defined source and command. If the producer
  or generator is unavailable, report the gap instead of editing generated output
  as though it were canonical.
- Update the existing canonical documentation with behavior changes. Search by
  purpose and owner before adding Markdown. Follow the [documentation policy](../policies/documentation.md)
  and local rules for translation, plan/spec placement, dependencies and architecture.
  Keep language and tool version choices within the authorized repository scope.
