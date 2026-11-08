---
name: debugging
description: Debugging workflow for CIRNETLAB
---

<!-- doc-id: SKILL-debugging -->
<!-- revision: 1 -->
<a id="workflow"></a>

# Debugging

| Part | Requirement |
| --- | --- |
| Trigger | Failure or discrepancy |
| Input | Exact fixture, logs, revision and expected result |
| Steps | Reproduce first. Trace the boundary and test one hypothesis. Keep safety and timeout requirements in place. |
| Output | An evidence-backed cause, narrow repair and regression result. |
| Failure | Without reproducible evidence, report observations rather than changing gates. |

Apply the [verification rule](../../rules/testing.md).
