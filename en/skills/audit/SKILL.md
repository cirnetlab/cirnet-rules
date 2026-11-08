---
name: audit
description: Audit workflow for CIRNETLAB
---

<!-- doc-id: SKILL-audit -->
<!-- revision: 1 -->
<a id="workflow"></a>

# Audit

| Part | Requirement |
| --- | --- |
| Trigger | New repository or claimed readiness |
| Input | Exact Git origin/ref, manifests and release evidence |
| Steps | Read the entire tracked inventory without truncation. Inspect relevant authorities and identify unknowns before making proposals. |
| Output | A dated audit with commit/tree distinction, covered paths and blocked gates. |
| Failure | Missing refs or inaccessible sources block claims about them. |

Apply the [verification rule](../../rules/testing.md).
