---
name: implementation
description: Implementation workflow for CIRNETLAB
---

<!-- doc-id: SKILL-implementation -->
<!-- revision: 1 -->
<a id="workflow"></a>

# Implementation

| Part | Requirement |
| --- | --- |
| Trigger | Approved scoped task |
| Input | Requirement ID, plan, owners and acceptance checks |
| Steps | Review current behavior, reproduce the risk, edit only authorized files, verify direct and negative paths. |
| Output | Small diff and recorded evidence with explicit limits. |
| Failure | Scope drift, unavailable checks or unexpected side effects block completion. |

Apply the [verification rule](../../rules/testing.md).
