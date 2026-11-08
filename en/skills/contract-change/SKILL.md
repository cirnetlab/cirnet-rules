---
name: contract-change
description: Contract change workflow for CIRNETLAB
---

<!-- doc-id: SKILL-contract-change -->
<!-- revision: 1 -->
<a id="workflow"></a>

# Contract change

| Part | Requirement |
| --- | --- |
| Trigger | Request shape, encoding, IDs or semantics change |
| Input | Producer authority, current schema, consumers and frozen digests |
| Steps | Compare required fields, enums, numeric bounds, units, defaults and errors. Review reserved protobuf numbers, frame limits and wire evidence. Generate once from the authoritative source. |
| Output | ADR, exact contract and generator revisions, cross-consumer positive and rejection tests. |
| Failure | An unreviewed break or mismatched generated output blocks selection. |

Apply the [verification rule](../../rules/testing.md).
