<!-- doc-id: CUT-INTERFACE -->
<!-- revision: 1 -->
<a id="cutover"></a>

# cirnet-interface cutover proposal

`CUT-INTERFACE` · `REL-01` · `PLANNED` · [Audit snapshot](../../specs/cirnet-interface/audit.md)

## Dependency and boundary

Prerequisites [CUT-CONTRACTS](../cirnet-contracts/cutover.md).
Affected source `package.json`, `package-lock.json`, `integration/compatibility.json`, `scripts/bootstrap-contracts.mjs`, `src/services/api/client.ts`.

Keep Vue/TypeScript/Vite. Build from exact reviewed Contracts SDK and native Remote export. Remove active old response/route compatibility once data handling is approved.

## Acceptance and rollback

Verify stable operation IDs, scope and browser workflows, deny old peers, and reconcile failures without replaying writes. Owner gate `browser_native`. Record the exact source, contract revision, artifact digest and tool versions. Producer changes require product owner authorization. Restore only an approved complete combination and matching data. See [release policy](../../policies/governance.md) and [ADR 0001](../../references/adr-0001.md).
