<!-- doc-id: AUDIT-PRODUCTS -->
<!-- revision: 2 -->
# Six-product audit for release train 0.2.4

<a id="method"></a>

## Scope and reproducibility

The local evidence snapshot was taken on 2026-11-08 UTC on Windows with Git 2.52.0.windows.1 and PowerShell 7.6.9. All six local worktrees reported clean and used branch `master`. A public Git query independently identified `master` as the default branch and returned the same commits. No public Git refs for `v0.2.4` or `v0.2.2` appeared in that query. The public release endpoint for `v0.2.4` returned HTTP 404 for each repository. Private or draft release records and private CI evidence were outside the observed scope.

| Repository | Commit | Tracked paths | Selected public historical tag |
| --- | --- | ---: | --- |
| [cirnet-contracts](../specs/cirnet-contracts/audit.md#snapshot) | `4bfd0950f564aaf24abaf6b8b82399c2d54d0e39` | 334 | `v0.1.2` |
| [cirnet-remote](../specs/cirnet-remote/audit.md#snapshot) | `1a38c1331a07bdc3b5644c70eebb40590ae53452` | 629 | `v0.1.4` |
| [cirnet-hardware](../specs/cirnet-hardware/audit.md#snapshot) | `2139cccc8b36dc403c2f7182dfb670a97513858a` | 91 | `v0.1.2` |
| [cirnet-simulator](../specs/cirnet-simulator/audit.md#snapshot) | `c05be23bef71de44a9027319af945d3e635d0882` | 73 | `v0.1.2` |
| [cirnet-interface](../specs/cirnet-interface/audit.md#snapshot) | `f1ff70b9f22554c0680b92679e9e44a5544247ff` | 609 | `v0.1.4` |
| [cirnet-fpga](../specs/cirnet-fpga/audit.md#snapshot) | `bd9d37ee3a3e1a526fb32b00c5ef71f2e8008cd0` | 56 | `v0.1.2` |

Each repository audit records the real root tree ID and raw inventory digest. The six initial lookup IDs are commit objects. The root tree records paths and entry identities; a blob identifies one Git object. An artifact SHA-256 is computed over artifact bytes independently of those Git IDs.

Run these commands inside a product repository. For the inventory digest, hash the unmodified byte stream from the final Git command, including NUL separators, before parsing path records. Use the pinned commit instead of `HEAD` to reproduce this audit later. Do not hash a PowerShell-decoded pipeline.

```text
git --no-optional-locks status --porcelain=v1 --untracked-files=all
git rev-parse HEAD HEAD^{tree}
git --no-optional-locks -c credential.helper= ls-remote --symref origin HEAD refs/heads/master refs/tags/v0.2.4 refs/tags/v0.2.2 refs/tags/v0.1.4 refs/tags/v0.1.2
git --no-optional-locks ls-tree -r -z --full-tree HEAD
```

All 1,792 tracked paths were inventoried. The six audits list human-reviewed source, documentation and verification passages and classes that were only inventoried. Reviewed `README.md`, `AGENTS.md`, changelog and relevant Git history for each checkout. FPGA's Agent entry is scoped under `fpga/AGENTS.md`. No product build, clean-room run, cloud CI or physical test was executed in this governance audit. Retained artifact bytes and receipt hashes were checked separately.

<a id="identity"></a>

## Repository routing

Use the [registry](../../governance.yaml) for repository ownership, task
routing and typed dependencies. The table above pins the sources audited
for the recorded release decision. Check the actual Git worktree binding
when resolving a new task.

<a id="contract-inventory"></a>

## Dependency evidence

Each repository audit identifies its producer and consumer boundaries and
the conformance still required for the recorded family `0.2`. Record exact
source and governance revisions, reviewed contract inputs, artifact digests
and receipts. Product owners maintain implementation details in their
repositories.

<a id="historical-evidence"></a>

## Recorded selection

The [published selection as inspected](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/integration/release-set.json)
had SHA-256 `90a8ef4d30ddc19d3024dcb3790fef5d5e041f45fa47922bfd5e34c2fd19c419`.
Rehashing the 15 retained assets matched the recorded selection.

<a id="candidate-evidence"></a>

## Verification decision

The audit recorded **PRODUCT_0.2.4_CUTOVER_READINESS · BLOCKED**. The exact
candidate needed clean source inputs, conformance receipts, reproducible
packaging and its own functional, safety and rollback evidence. Earlier
functional receipts apply to their original artifacts. See the
[blocked release receipt](evidence/check-release-blocked.json) and the
[original audit and evidence details](https://github.com/cirnetlab/cirnet-rules/blob/52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c/en/references/product-audit.md#candidate-evidence).
