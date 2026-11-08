<!-- doc-id: AUDIT-CONTRACTS -->
<!-- revision: 1 -->

# cirnet-contracts product audit

[Tiếng Việt](../../../vi/specs/cirnet-contracts/audit.md) · [Cross-repository evidence](../../references/product-audit.md)

<a id="snapshot"></a>

## Snapshot and identity

| Field | Evidence |
| --- | --- |
| Repository | `https://github.com/cirnetlab/cirnet-contracts.git` |
| Time and scope | 2026-11-08 UTC, Windows local read-only audit |
| Local and hosted branch | Local `master` and hosted default `master` matched. Cached `origin/HEAD` points to `main`. |
| Commit SHA | `4bfd0950f564aaf24abaf6b8b82399c2d54d0e39` |
| Root tree SHA | `5365983a29060c009e07d504bc1c84616e517f2e` |
| Tracked inventory | 334 files. SHA-256 of raw Git tree listing is `acfcb4331358571052686329f3f0b0c45c86d9f9ab4e633eabefda52d7414980`. No gitlinks. |
| Working tree | Clean at the audit snapshot. |
| Local tags | v0.1.1 |
| Selected historical hosted tag | `v0.1.2` resolves to `524cc7a5e2e65f39077feefe70c7fb2ae3a773d6`. |
| Target publication | Hosted Git returned no matching `v0.2.4` or `v0.2.2` ref. Public GitHub release lookup for `v0.2.4` returned HTTP 404. Private drafts were outside the observed surface. |

<a id="coverage"></a>

## File coverage

Parsed the checked-in `contracts/api/openapi.json` and `contracts/manifest.json`, and inspected the capture layout metadata. Reviewed release and protocol passages in `docs/en/api/compatibility.md`, `docs/en/contract-reference.md`, `docs/en/fpga-protocol.md`, `docs/en/verification.md` and their Vietnamese counterparts. Inspected dependency and generation paths in `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `pyproject.toml`, `native/src/main.rs`, `native/build.rs`, `setup.py`, `scripts/package-native.mjs`, `contracts/codegen/generate_contracts.py` and both release workflows. Protocol references include `contracts/hardware/v2/rpc_v2.proto`, `contracts/public/realtime.schema.json`, `contracts/circuit-schema/` and `contracts/fpga-protocol/`.

The inventory also includes generated model/binding sources, catalogs, frozen compatibility snapshots, migration tests and device examples. Their presence was recorded. The audit did not review every generated line, validate schema subtyping, run tests, rebuild wheels/SDKs or attest third-party code.

<a id="findings"></a>

## Findings and impact

| ID and status | Evidence at this commit | Impact and handling |
| --- | --- | --- |
| AUD-CON-01 · OBSERVED · BLOCKER | Cargo crate `cirnet-contracts-native`, Python package and npm SDK prepare 0.2.2. Remote's selected published system uses Contracts 0.1.2. `rust-version` is 1.90 and the toolchain pin is 1.99.1. | Version domains differ legitimately. Product selection for 0.2.4 needs the clean selected revision and exact artifacts. Keep toolchain and MSRV independent. |
| AUD-CON-02 · VERIFIED · HIGH | Parsed frozen OpenAPI bytes. OAS is 3.1.0, `info.version` is 0.1.2, 63 operation IDs are unique and SHA-256 is `dc39c874386eba2b58f8b5c631cbee0d63ded86cf287aea63f54e4a237ff8f08`. | Remote owns API operation semantics. Contracts owns the preserved handoff, schema and SDK generation. Generate SDK bytes against the exact reviewed Remote export, then bind raw and packed digests. |
| AUD-CON-03 · OBSERVED · BLOCKER | `setup.py` generates Python RPC bindings for wheel/sdist builds. The Python/Node standalone CI also runs schema and catalog generation and produces installable Python products. `native/src/main.rs` has a Rust generator entry point and `native/build.rs` compiles the canonical descriptor. | Separate the old generator/wheel lane as comparison evidence. Prove that release generation and distribution have a Rust-led closure with Python confined to test/benchmark. |
| AUD-CON-04 · OBSERVED · HIGH | Canonical Hardware protobuf package is `cirnet.hardware.v2`. CircuitGraph 1.0, 1.1 and 1.2, NCAP wire v1/v2 and FPGA logical `1.0-scaffold` / register map `1.0-reserved` are independent of package and OAS versions. | Establish family `0.2` by reviewed representation, revision and positive/negative conformance evidence. Keep original byte anchors historical. Do not infer wire compatibility from numeric order. |

<a id="readiness"></a>

## Readiness and handoff

**BLOCKED.** First reconcile Remote's exported API and publishable Contracts inputs in the review process. Produce exact versioned Rust/resource/SDK receipts from a clean source revision. Demonstrate generator parity, reserved Protobuf field numbering, malformed input rejection, bounds, canonical credentials/MIME and consumer conformance. Remote and Hardware must then verify the selected Rust crate; Interface verifies the exact SDK digest. Existing frozen bytes and historical verification are comparison inputs.
