---
name: release-readiness
description: Release readiness workflow for CIRNETLAB
---

<!-- doc-id: SKILL-release-readiness -->
<!-- revision: 1 -->
<a id="workflow"></a>

# Release readiness

| Part | Requirement |
| --- | --- |
| Trigger | An artifact is proposed for a release train |
| Input | Immutable producer receipts, owner release-set, approved selection and gates |
| Steps | Verify exact identity and digest, clean refs and all producer/consumer gates. Verify independent rollback. Separate candidate from published evidence and request authorization. |
| Output | A readiness result plus all blockers. |
| Failure | Missing approval, mismatch or unverified capability prevents activation. |

Apply the [verification rule](../../rules/testing.md).
