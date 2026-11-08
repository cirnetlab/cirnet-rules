<!-- doc-id: AUDIT-INTERFACE -->
<!-- revision: 1 -->

# cirnet-interface product audit

[Tiếng Việt](../../../vi/specs/cirnet-interface/audit.md) · [Cross-repository evidence](../../references/product-audit.md)

<a id="snapshot"></a>

## Snapshot and identity

| Field | Evidence |
| --- | --- |
| Repository | `https://github.com/cirnetlab/cirnet-interface.git` |
| Time and scope | 2026-11-08 UTC, Windows local read-only audit |
| Local and hosted branch | Local `master` and hosted default `master` matched. Cached `origin/HEAD` points to `master`. |
| Commit SHA | `f1ff70b9f22554c0680b92679e9e44a5544247ff` |
| Root tree SHA | `1f7e59f4b3e219fea6fa27b0183337c6d63a15c6` |
| Tracked inventory | 609 files. SHA-256 of raw Git tree listing is `f2180cae375c887b79817ca25fc3c270ca8bf044afd9cf8677f3e68bb3bdb9e0`. No gitlinks. |
| Working tree | Clean at the audit snapshot. |
| Local tags | v0.1.1, v0.1.2, v0.1.3, v0.1.4 |
| Selected historical hosted tag | `v0.1.4` resolves to `f1fe0c4738e105727aff1a331327caf7172cdf5f`. |
| Target publication | Hosted Git returned no matching `v0.2.4` or `v0.2.2` ref. Public GitHub release lookup for `v0.2.4` returned HTTP 404. Private drafts were outside the observed surface. |

<a id="coverage"></a>

## File coverage

Parsed `package.json`, the relevant dependency entries in `package-lock.json` and both `integration/*.json` descriptors. Inspected `src/main.ts`, `src/services/api/client.ts`, `src/services/api/captures.ts`, WebSocket and NCAP source excerpts, `scripts/bootstrap-contracts.mjs`, `scripts/build-release.mjs`, `.github/workflows/frontend.yml`, and the English/Vietnamese deployment, namespace and verification guides. The tracked tree includes browser/e2e suites, assets, binary model files, migration records and generated declarations. Most were inventoried only. No bundle, browser, desktop/mobile app, build or integration endpoint was executed.

<a id="findings"></a>

## Findings and impact

| ID and status | Evidence at this commit | Impact and handling |
| --- | --- | --- |
| AUD-INT-01 · VERIFIED · BLOCKER | JSON identifies `@cirnet/web` 0.2.4 with SDK file `file:.artifacts/cirnet-contracts-0.1.2.tgz`. `integration/compatibility.json` declares `UNRELEASED_NATIVE_HANDOFF`, target SDK 0.2.2 and actual 0.1.2. The selected SDK archive SHA-256 is `b963cf4990fee8b5a9294a6aaf85cfd803ed7724efc9ad2b3ee1ec302646b3c8`. | Package version is ahead of pinned handoff. Preserve reviewed published inputs. Regenerate the SDK from the exact accepted Remote export and verify a new clean Interface build against that artifact. |
| AUD-INT-02 · OBSERVED · HIGH | `scripts/build-release.mjs` distinguishes verification/candidate artifacts from release inputs, requires a clean committed export outside candidate mode and verifies SDK integrity and producer metadata. `.github/workflows/frontend.yml` exports verification assets with Node 26. | The workflow name or asset filename does not establish release readiness. Require producer receipt, target-aligned SDK, browser gate and Remote consumer acceptance. Keep existing Vue/TypeScript/Vite/Three.js/Pinia. |
| AUD-INT-03 · OBSERVED · HIGH | Source configures HTTP with the same-origin default `/api` and uses separate WebSocket and NCAP modules. Write requests use redirect error handling. The compatibility descriptor lists graph 1.0, 1.1 and 1.2 and Web as implemented, desktop/mobile proposed. | Preserve operation identity, credentials, reconnect reconciliation without replay, and unavailable capability semantics. Decide the target stored-data handling explicitly before removing legacy paths. Browser proof cannot stand in for hardware availability. |

<a id="readiness"></a>

## Readiness and handoff

**BLOCKED.** Depend on the reviewed Contracts SDK and native Remote handoff. Update the exact SDK pin and lock in the product repo under approval. Verify legacy profile rejection, no mutation replay, bounded WebSocket/NCAP and browser/authority cases on the same bundle. Preserve the current frontend stack and use the approved offline migration route or fresh data.
