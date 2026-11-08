<!-- doc-id: CUT-CONTRACTS -->
<!-- revision: 1 -->
<a id="cutover"></a>

# cirnet-contracts cutover proposal

`CUT-CONTRACTS` · `REL-01` · `PLANNED` · [Audit snapshot](../../specs/cirnet-contracts/audit.md)

## Dependency and boundary

First product dependency. Agree on the Remote OpenAPI and SDK generation sequence before freezing producer artifacts.
Affected source `contracts/manifest.json`, `contracts/api/openapi.json`, `native/src/main.rs`, `setup.py`, `contracts/codegen/generate_contracts.py`.

Lock exact family representations and SDK provenance from the reviewed Remote export. Remove production dependency on Python generation/packaging, retaining comparison in tests.

## Acceptance and rollback

Regenerate clean Rust-led schemas and bindings, preserve frozen bytes, prove operation ID uniqueness and cross-language positive/negative wire checks. Owner gate `contract_conformance`. Record the exact source, contract revision, artifact digest and tool versions. Producer changes require product owner authorization. Restore only an approved complete combination and matching data. See [release policy](../../policies/governance.md) and [ADR 0001](../../references/adr-0001.md).
