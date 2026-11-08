<!-- doc-id: AUDIT-REMOTE -->
<!-- revision: 1 -->

# cirnet-remote product audit

[Tiếng Việt](../../../vi/specs/cirnet-remote/audit.md) · [Cross-repository evidence](../../references/product-audit.md)

<a id="snapshot"></a>

## Snapshot and identity

| Field | Evidence |
| --- | --- |
| Repository | `https://github.com/cirnetlab/cirnet-remote.git` |
| Time and scope | 2026-11-08 UTC, Windows local read-only audit |
| Local and hosted branch | Local `master` and hosted default `master` matched. Cached `origin/HEAD` points to `master`. |
| Commit SHA | `1a38c1331a07bdc3b5644c70eebb40590ae53452` |
| Root tree SHA | `fbd18a76493861a713e7fdad88fb4628cb9c18bc` |
| Tracked inventory | 629 files. SHA-256 of raw Git tree listing is `cbb87cf38302f9ba9ff6114f9c616516294aba4bef7e0f3f617c53d80bea79ac`. No gitlinks. |
| Working tree | Clean at the audit snapshot. |
| Local tags | v0.1.1, v0.1.2, v0.1.3, v0.1.4 |
| Selected historical hosted tag | `v0.1.4` resolves to `38aea6120920f48cea817827ac2277cce4bef25e`. |
| Target publication | Hosted Git returned no matching `v0.2.4` or `v0.2.2` ref. Public GitHub release lookup for `v0.2.4` returned HTTP 404. Private drafts were outside the observed surface. |

<a id="coverage"></a>

## File coverage

Parsed `integration/release-set.json` and inspected native/version pin passages in `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `integration/contracts-version.json`, `integration/ci-source-dependencies.json`, `native/src/main.rs`, `native/src/openapi.rs`, `native/src/http_experiments.rs`, `native/src/websocket.rs` and `native/src/hardware.rs`. Reviewed `scripts/package-native-windows.mjs`, `scripts/native-release-inputs.mjs`, `scripts/install_release.py`, `scripts/system_release_manifest.py`, `run-cirnet.bat`, `deployment/windows-native/cirnet-native.ps1`, `services/api/pyproject.toml` and relevant CI steps. Consulted the architecture, Lab-First, safety, release-version, HTTP profile and verification documents in `docs/en/` and `docs/vi/`.

The full tracked inventory includes Rust and Python service modules, launchers, deployment templates, screenshots, histories and tests. Other assets, fixtures, source lines and generated dependencies remain inventory only. Neither browser nor service, database, packaging, benchmark, Stop or hardware gates were run in this audit.

<a id="findings"></a>

## Findings and impact

| ID and status | Evidence at this commit | Impact and handling |
| --- | --- | --- |
| AUD-REM-01 · VERIFIED · BLOCKER | `integration/release-set.json` remains the authoritative published system 0.1.4 selection. All 15 retained local historical files matched size and SHA-256. `Cargo.toml` is 0.2.4 but the crate pin is `=0.1.2` at `e640f80b1199779659d3db7817fee16d13cc24c1`. | Preserve historical bytes and select a new manifest only after exact 0.2 family evidence. The version of the native executable does not activate the train. |
| AUD-REM-02 · VERIFIED · HIGH | Static comparison of the frozen document and `native/src/openapi.rs` found 63 unique operation IDs, 17 retired and 46 retained. Native capture uses canonical `application/vnd.cirnet.capture; version=1` and treats legacy-only Accept as 406. `native/src/websocket.rs` and `native/src/hardware.rs` provide distinct WebSocket and local RPC boundaries. | Compare real exported operation metadata against the generated SDK. The body/method/path and operation identity, supported MIME, and session/CSRF behavior require explicit compatibility evidence. A static parse is not a live API test. The [owner's profile table](https://github.com/cirnetlab/cirnet-remote/blob/1a38c1331a07bdc3b5644c70eebb40590ae53452/docs/en/api/http-profiles.md) records each method, replacement and shape change. |
| AUD-REM-03 · OBSERVED · BLOCKER | `services/api/` still contains FastAPI product source and its packaging. `scripts/install_release.py` and `scripts/system_release_manifest.py` implement published-profile install/validation. Native Windows assembly is currently Node in `scripts/package-native-windows.mjs`. | Preserve the published profile as historical evidence. The new release/bootstrap closure must use Rust authority, with any shell launcher thin. Leave Python only in isolated test/benchmark. |
| AUD-REM-04 · OBSERVED · HIGH | `docs/en/architecture/physical-lab-safety-contract.md` and `native/src/main.rs` distinguish safety, executability and lifecycle. `scripts/native-release-inputs.mjs` checks exact input digests and provenance during assembly. | Retain single scheduling authority, no replay, guarded admission, safe-state acknowledgment and quarantine. Neither identity checking nor a software smoke run supplies electrical acceptance. |

<a id="readiness"></a>

## Readiness and handoff

**BLOCKED.** Coordinate Contracts' exact source and SDK export with Interface. Stage clean producer receipts and the reviewed C++/Hardware pair. Verify authenticated HTTP/WebSocket, NCAP, saved-data migration as an explicit offline action, and loaded Stop/rollback against the same artifact set. A valid rollback must be an allowed complete post-cutover bundle. Without one, stop safely and repair forward. The current published lock and all runtime paths were left untouched.
