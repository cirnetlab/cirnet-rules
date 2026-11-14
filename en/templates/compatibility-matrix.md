# Compatibility matrix template

Record the owner, version domain, public surface, relevant policy and authorized
scope. Label unknown inputs and keep current source, candidate and selection distinct.

| Producer revision / artifact hash | Consumer revision / selected pin | Change and direction | Source / wire / data / behavior | Test and result | Migration / rollback limit |
| --- | --- | --- | --- | --- | --- |
| Exact identifier | Exact identifier or unknown | Writer to reader, old to new, or API operation | Affected modes | Command, environment, PASS / FAIL / NOT_RUN / BLOCKED | Required action or unknown |

List relevant deprecation windows and outstanding directions. Link evidence
in its owning repo and avoid claiming release selection based on source alone.
