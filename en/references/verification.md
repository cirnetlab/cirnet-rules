<!-- doc-id: VERIFY-LOCAL -->
<!-- revision: 1 -->
<a id="evidence"></a>

# Verification evidence

The checkout base is `a21d372c667ecdb1963dac4dbabc30df816c2e87`. Governance
changes are local, so that commit identifies the base, not an immutable
release of this implementation. Windows x64, Rust and Cargo 1.99.1 and
Git 2.52.0 were used. Product checks were read-only.

## Local runs on 2026-11-08 UTC

Run from the governance root. The local toolchain alias was `stable`,
verified to resolve to 1.99.1 for both Rust and Cargo. The repository and
CI pin the numbered distribution. On the local host, the process-scoped
`RUSTUP_TOOLCHAIN` override selected the installed alias.

| Command or verification | Exit and observation |
| --- | --- |
| `cargo +stable fmt --all --check` | 0. Formatting matched. |
| `cargo +stable clippy --locked --offline --all-targets -- -D warnings` | 0. Strict lint passed. |
| `cargo +stable build --locked --offline` | 0. CLI built. |
| `cargo +stable test --locked --offline` | 0. 16 unit and 3 CLI integration tests passed. |
| `validate` and `check-translations` | 0. Valid registry, 49 pairs, local links, anchors, versions and source hashes. |
| `context --workspace .. --repo cirnet-hardware --task audit --risk hardware` | 0. Scoped English context, including hardware safety review. |
| `check-drift --workspace ..` | 0. The six clean snapshots, audit references and product Git identities agreed. |
| `check-versions --workspace ..` | 2, `BLOCKED`. Observed source and the selected release require cutover. |
| `check-release --workspace ..` | 2, `BLOCKED`. See [JSON output](evidence/check-release-blocked.json), SHA-256 `0d4cd3f52cbfe410e0e7a16588fc26dc1ae749dbfcc912a27fed5a58934d9a4a`. |
| `check-release` with an empty workspace | 2, `BLOCKED`. |
| `doctor --workspace ..` | 0 with the installed 1.99.1 alias. 2 when the pinned numbered name was unavailable in the local shell. Tool discovery does not start Agent sessions. |
| `bootstrap` in an isolated session | 21 CLI invocations succeeded, seven per Agent. Dry-run, install, idempotence, rollback, repeated uninstall, restore and final rollback. Three backups remained, with no active bundles. |

See the [product audit](product-audit.md) for its snapshot, commands and limits,
and the [adapter matrix](../adapters/README.md) for the distinction between
portable tests and native harness discovery. GitHub-hosted CI, board testing,
product builds, semantic approval and release activation are independent gates.
