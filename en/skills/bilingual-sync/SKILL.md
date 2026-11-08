---
name: bilingual-sync
description: Bilingual sync workflow for CIRNETLAB
---

<!-- doc-id: SKILL-bilingual-sync -->
<!-- revision: 1 -->
<a id="workflow"></a>

# Bilingual sync

| Part | Requirement |
| --- | --- |
| Trigger | An English document changes |
| Input | EN source, VI counterpart, glossary and critical IDs |
| Steps | Update the English revision and Vietnamese wording together. Preserve identifiers and anchors. Record the normalized source digest. Check links and review semantics. |
| Output | Paired docs with passing structural checks and explicit semantic review status. |
| Failure | Stale hash, orphan doc or altered safety language blocks acceptance. |

Apply the [verification rule](../../rules/testing.md).
