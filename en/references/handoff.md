<!-- doc-id: HANDOFF -->
<!-- revision: 2 -->
<a id="decision"></a>

# Governance and product handoff

**GOVERNANCE_IMPLEMENTATION** covers the registry, paired docs, executable
checks and portable adapters in this checkout. See [verification evidence](verification.md).

**PRODUCT_0.2.4_CUTOVER_READINESS** is blocked. The [read-only audit](product-audit.md)
records the six exact snapshots, selected published inputs and unresolved gates.
The published combination remains separate from the proposed native train.

Order the work after product authorization.

1. Agree on contract family, reviewed representations and negative cases. Freeze the reviewed Remote API export and rebuild the Contracts SDK before selecting Interface inputs.
2. Build the bounded C++ Simulator worker and FPGA source scaffold with immutable receipts. Join Hardware to the exact Contracts and worker inputs and verify fail-closed lifecycle behavior.
3. Build Interface with the matching SDK. Verify the Remote runtime, clean system install, browser, Stop and rollback. Keep physical HIL separate.
4. Record the exact owner-side selection, verify evidence, then request explicit publication and activation approval.

Use [adapter guidance](../adapters/README.md) for scoped bootstrap, and the
[tools guide](../tools/README.md) for release and drift checks.

<a id="record-templates"></a>

## Shared record templates

| Record | Fields to complete before handoff |
| --- | --- |
| Design decision | ID, problem, constraints, alternatives, selected proposal, rationale, affected paths and contracts, owner, approval reference, rollback. |
| Product task | Requirement or ADR, task ID, owner, repo and paths, dependency task IDs, planned revision and artifact, acceptance check, evidence identity, release gate, next action. |
| Scoped exception proposal | Rule ID, precise scope and expiry, rationale, compensating evidence, owner, approver and actual approval reference. See the closed initial policy. |
| Verification evidence | Source and governance commits, command, tool versions, platform, UTC time, exit code, evidence kind, result path and digest, limits. |

Keep proposal, implementation and verification states separate. Follow the
[policy](../policies/governance.md) for approvals and the
[evidence formats](../schemas/release-evidence.md) for machine input.
