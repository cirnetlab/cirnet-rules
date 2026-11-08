---
name: refactoring
description: Refactoring workflow for CIRNETLAB
---

<!-- doc-id: SKILL-refactoring -->
<!-- revision: 1 -->
<a id="workflow"></a>

# Refactoring

| Part | Requirement |
| --- | --- |
| Trigger | Change of internal structure |
| Input | Current callers, public interfaces and coverage |
| Steps | Map old and new paths. Keep normal entry points and behavior, run boundary regression checks, then remove obsolete authority only once successors are verified. |
| Output | Migration map and verified behavior. |
| Failure | Unknown consumers or missing parity keep cleanup pending. |

Apply the [verification rule](../../rules/testing.md).
