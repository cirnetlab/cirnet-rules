<!-- doc-id: CNR-CLI-ONLY-VERIFICATION -->
<!-- revision: 2 -->
<a id="verification"></a>

# CLI-only verification

## Verification scope

This record covers the CLI-only implementation on Windows x64 MSVC with
Rust and Cargo 1.99.1. The implementation baseline was
`52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c`. Test counts and receipts below
describe that run. Use the [current verification guide](../../../references/verification.md)
for the active checklist.

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

Migration used disposable SQLite and Git fixtures. Real data migration,
native agent discovery, hook execution, authenticated acceptance and release
activation were `NOT_RUN`. Hosted CI, local Linux, Unix symlink fixtures and
live Windows junction checks were `NOT_RUN` during this run.
