<!-- doc-id: CNR-CLI-ONLY-VERIFICATION -->
<!-- revision: 1 -->
<a id="verification"></a>

# CLI-only verification

## Baseline

The archive and implementation branches began at 52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c. Windows x64 MSVC reported rustc 1.99.1 (58cf8f9c3) and cargo 1.99.1 (5f94df478). Baseline cargo test exited 1 after three legacy integration loopback failures and one legacy Windows ACL expectation in the private runtime. Tests later in that run were unobserved. The ignored zip and output directories were excluded from source changes. The current and two predecessor default SQLite database paths and their sidecars were absent. Custom data locations were not inventoried.

## Replaced tests

| Prior runtime fixture | Coverage in the CLI-only architecture |
| --- | --- |
| Draft/restart/publish/rollback, interrupted SQL publication | Lossless archive preserves revisions and drafts; staging and approval digest guard proposal application. Git retains approved source history. |
| Traversal, stale writes, dependencies, structured content and elevated publish | Catalog validation, content freshness, migration archive checks and the unchanged scoped task gates. |
| Profile and binding pinning for conversations | Archive preserves pinned snapshots for review; active resolution validates the Git binding and current task scope. |
| Published export and exact import preview | Archive inventories every revision and the import compares the reviewed digest. |
| Attachment aliases and executable attachments | Legacy review reports unsafe assets; Agent bundles guard managed paths and executable metadata. |
| Private wire schema and desktop interaction | These interfaces are retired with the runtime. Existing CLI compatibility tests retain the maintenance envelope. |
| Engine probes, conversation and scheduling, preferences, UI and IPC | Removed with their associated services. The immutable archive preserves the prior implementations and tests. |

## Windows results

All of these commands exited 0 with the pinned toolchain.

```text
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo check --locked --all-targets
cargo build --locked --bins
cargo test --locked --all-targets
cargo build --release --locked --bin cnr
```

The default suite passed 101 tests with one opt-in test ignored. The separate
offline opt-in `uv` fixture passed. Compatibility validation and translation
checks each verified 109 EN/VI pairs. CNR help, JSON version, authority
inspection and context, and tooling audit ran successfully. Missing explicit
SQLite fixture input returned `NOT_FOUND` with exit 2.

The migration suite passed seven tests. The schema 7 WAL fixture preserved
four assets, three revisions and one draft. It also retained assignments,
defaults, audit records and pinned historical snapshots. The source database,
WAL and SHM bytes matched their pre-export values. Preview, resume, rollback
and path-conflict tests used disposable fixtures. All seven historical SQL
files matched their preflight byte digests.

Combined Agent export and check each exited 0 and recorded
`STATIC_EXPORT_VALIDATED`. The staged bundle had 18 managed files plus its
manifest. Repeated export returned `UNCHANGED`. Agent sync and rollback used
bound Git fixtures with per-file task grants.

## Evidence limits

The three default database locations were `NOT_FOUND`. Custom locations
remain unknown. Real migration was `NOT_RUN`, so there is no real backup
or import receipt. The archive Git ref still points to the baseline.

Version probes observed Codex `0.167.0-alpha.3.2` and Antigravity IDE launcher
`1.107.0`. Claude Code was unavailable on PATH. Native discovery, hook
execution, authenticated acceptance and release activation were `NOT_RUN`.
Local Linux testing was unavailable due to `Wsl/E_ACCESSDENIED`. Hosted CI,
Unix symlink fixtures and live Windows junction tests were `NOT_RUN`.
