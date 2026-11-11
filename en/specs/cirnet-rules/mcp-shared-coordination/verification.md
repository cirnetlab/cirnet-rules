<!-- doc-id: CNR-MCP-COORDINATION-VERIFICATION -->
<!-- revision: 1 -->
<a id="verification"></a>

# Shared coordination verification

## Baseline and preservation

The linked implementation checkout was at
328848abc3dda7e5fce8024d601c4eeeaa83cd83. The archive ref began and ended at
52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c. The task left the Git index empty
and preserved the existing archive zip and concurrent ignore-file edit.

On Windows x64 MSVC, the pinned rustc 1.99.1 and cargo 1.99.1 passed 101
baseline tests. The source audit's SQL observation was reproduced. Both
Git blobs for migration 007 were the 3,283-byte LF variant. The separate
known mixed Windows checksum and LF checksum passed a schema 7 fixture.
An unknown checksum and an altered schema were rejected.

The current and two predecessor default SQLite locations returned
`NOT_FOUND`. Custom locations remain unknown. Real archive, proposal
staging and adoption were `NOT_RUN`.

## Implemented and verified

These commands exited 0 on the linked worktree. The Cargo dependency
cache was isolated for the task.

```text
cargo fmt --all --check
cargo clippy --locked --offline --all-targets -- -D warnings
cargo check --locked --offline --all-targets
cargo build --locked --offline --bins
cargo test --locked --offline --all-targets
cargo build --release --locked --offline --bin cnr
```

The default suite passed 128 tests. The separate offline opt-in `uv`
fixture passed as well. New tests exercised named linked worktrees, stale
binding and branch detection, same-worktree writer exclusion, competing
claim and graph-update processes, fenced expiry and recovery, idempotent
receipts, source drift, report redaction, legacy record migration, and
producer artifact pinning and consumption.

The Windows STDIO suite passed nine tests with `rmcp` 3.6.0 and Tokio
1.54.1. Legacy initialize and current discovery worked through real CNR
processes. The coordinated fixture used CLI and two MCP subprocesses
against a single task store, sealed a changed file, exported portable
evidence and verified the subsequent Git candidate. Other wire fixtures
exercised read-only denial, malformed and oversized frames, pagination,
cancellation, EOF and unread stdout. The static MCP launcher export and
check each exited 0 for the three review templates. They recorded
`STATIC_CONFIG_VALIDATED`. Protocol fixture status is
`MCP_PROTOCOL_VERIFIED`.

CNR help, authority validation and inspection, translation validation,
Agent list, MCP help and named-binding list each exited 0. The authority
catalog retained 32 assets. The task state and audit events share
`.cirnetrules/tasks`. Prepared session credentials stay in the local
session directory. The CI gate returns a JSON receipt linked to the
reviewed bundle, trusted authority and actual candidate. The implementation
patch changed 57 files. Six central EN/VI task documents accompany it.

## Evidence limits

Version probes reported Codex CLI `0.167.0-alpha.3.2` and the Antigravity
IDE launcher `1.107.0`, with OS permission warnings. Claude Code and
the Antigravity CLI were unavailable on PATH. Native client connection
and discovery were `NOT_RUN`. The launch templates are review-only.

WSL inventory returned `Wsl/E_ACCESSDENIED`. Local Linux and hosted CI
execution were `NOT_RUN`. The reusable gate checks one sealed leaf
candidate against an independently supplied review digest and a clean
trusted authority commit. Product build, HIL, release activation,
authenticated independent acceptance and branch-protection deployment
were `NOT_RUN`.
