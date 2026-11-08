---
name: hardware-evidence
description: Hardware evidence review workflow for CIRNETLAB
---

<!-- doc-id: SKILL-hardware-evidence -->
<!-- revision: 1 -->
<a id="workflow"></a>

# Hardware evidence review

| Part | Requirement |
| --- | --- |
| Trigger | A board, permit or physical capability is claimed |
| Input | RTL/scaffold scope, safety owner, board/firmware identity and authorized test records |
| Steps | Separate simulation, synthesis, timing and board evidence. Review safe state, watchdog, shutdown and denied peer cases without energizing hardware. |
| Output | Capability decision limited to actual evidence. |
| Failure | Missing hardware authorization or HIL evidence keeps physical acceptance open. |

Apply the [verification rule](../../rules/testing.md).
