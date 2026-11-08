<!-- doc-id: schema-release-evidence -->
<!-- revision: 1 -->
# Native release input and evidence formats

<a id="authority"></a>
## Authority and activation

These proposed schemas define a local verification gate for the Windows MSVC
software baseline. Remote owns `integration/release-set.json` and
`integration/compatibility-matrix.json`. Governance stores digests for the
approved matrix, evidence index and published owner manifest. The historical
schema v1 remains readable as blocked cutover input.
The `0.2.4` release train and the string family `"0.2"` are separate from
contract revisions, toolchain versions and the governance Git commit.

A locally consistent candidate without a reviewed evidence index records
`integrity_validation: PASS` with `status: BLOCKED`. With the index approved,
`candidate_validation: PASS` remains `BLOCKED` until the baseline is activated
and the selected owner manifest is published and approved. Release owners
verify hosted tag and artifact provenance before pinning the SHA-256 of the
exact published manifest bytes. The offline verifier checks this digest,
committed release-set bytes and exact local tag targets.

<a id="selection"></a>
## Exact selection

The [release schema](native-release.schema.json) identifies six component
commits, exact versioned artifact names, targets, sizes, SHA-256 digests,
toolchain identities, build inputs, capabilities and contract bytes. The
selection digest hashes compact UTF-8 JSON with recursively sorted object keys,
preserved array order and these fields only: `contracts`, `family`,
`releaseTrain`, `repositories` and `rollback`. Arrays of artifacts are part of
the reviewed selection. The [compatibility matrix](compatibility-matrix.schema.json)
allowlists exact selection digests. Its original JSON bytes must match the
approved digest in the registry.

Git supplies build input bytes from each selected commit. The lock input is
`Cargo.lock` for Rust, `package-lock.json` for Interface, `CMakeLists.txt` for
Simulator and `integration/toolchain-lock.json` for FPGA. The verifier checks
Rust Contracts source pins against the selected Contracts commit and the
Interface SDK selection against the selected Contracts npm archive bytes,
including artifact SHA-256 and npm lock SHA-512 integrity. Native
producer targets must agree. The web and FPGA source targets are
`web-static` and `portable-source`.

<a id="evidence"></a>
## Evidence

Keep reviewed evidence in the product workspace, outside the repository
checkouts, under `cirnet-release-evidence/<selectionSha256>/`.

- `artifacts/<repository>/<filename>` contains selected component bytes.
- `contracts/<domain>/<filename>` contains selected contract bytes.
- `gates/<gate>.json` contains the exact [gate record](release-evidence.schema.json).
- `results/<resultFile>` contains the corresponding evidence result.

Each gate record binds a repository commit, governance commit, selection
digest, command, tool versions, platform, UTC time, result digest, exit code
and limits. Producer tools create these records in an authorized verification
session. Following review, `release_policy.approved_evidence` records the
policy `governance_commit` used for verification and the `index_sha256` of
the manifest `evidence` map serialized as compact UTF-8 JSON with recursively
sorted object keys. The records must use that policy commit, which must be an
ancestor of the current governance checkout or the current commit itself.
Changing a receipt or its result changes the approved hash chain and requires
a new review. Active Python or legacy paths found in a
selected checkout block the gate, while tests, benchmark and historical
documentation are excluded from the Python path scan. The separate
`legacy_rejection` gate verifies behavioral rejection. Simulation evidence
covers the reviewed software capability scope. Physical and firmware
availability require separate HIL review outside this verifier.
