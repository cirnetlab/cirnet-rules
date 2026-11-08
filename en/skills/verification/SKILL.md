---
name: verification
description: Verification workflow for CIRNETLAB
---

<!-- doc-id: SKILL-verification -->
<!-- revision: 1 -->
<a id="workflow"></a>

# Verification

| Part | Requirement |
| --- | --- |
| Trigger | Before completion, approval or release |
| Input | Exact inputs, expected exit codes and test environment |
| Steps | Run owner-defined checks in isolation. Read full output and distinguish sample scope, skipped checks and real failures. |
| Output | A receipt linking inputs, commands, results and gates. |
| Failure | Missing evidence blocks the associated gate. |

Apply the [verification rule](../../rules/testing.md).
