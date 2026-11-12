<!-- doc-id: CNR-GUIDED-VERIFICATION -->
<!-- revision: 1 -->
<a id="evidence"></a>

# CNR checks and guided setup verification

The 2026-11-12 run used Windows, Rust and Cargo 1.99.1, and the uncommitted working tree at `master` HEAD `f14da9a6a8e41f89570f1ecf1d6d7a5a93511aeb`.

| Command | Exit | Result |
| --- | --- | --- |
| `cargo fmt --all --check` | 0 | Formatting verified |
| `cargo check --locked --all-targets` | 0 | All targets checked |
| `cargo clippy --locked --all-targets -- -D warnings` | 0 | Lint gate passed |
| `cargo build --locked --bins` | 0 | Debug binaries built |
| `cargo test --locked --all-targets` | 0 | 233 passed, 1 ignored |
| `cargo build --locked --release --bins` | 0 | Optimized binaries built |
| `git diff --check` | 0 | Diff whitespace verified |

The baseline reproduced `--target is required` and accepted a supplied quoted Unicode path far enough to reach directory validation. Regression tests exercised structured missing fields, completion, quoting, cancellation, binding freshness, managed entry preservation, named export, grant denial, scoped registration and rollback across the three adapters. The ignored test requires external uv and Python.

The built CLI reported `WARNING` and exit 0 for `cnr check all` in the current workspace, with six binding review findings. `cnr check mcp --agent all` returned exit 0 and protocol evidence from real STDIO. Both supported MCP lifecycles passed the wire tests. A Windows terminal smoke test opened the Vietnamese wizard with `NO_COLOR`, selected an Agent, ran the probe, opened the report details and exited normally.

Codex, Claude Code and Antigravity native discovery and session compliance remain `NOT_RUN`. The project configuration writers were exercised in fixtures. User scope and the Antigravity App use guided import. Ubuntu runtime and release activation were outside this verification.

[Scope](spec.md)
