<!-- doc-id: CGACP-V3-VERIFICATION -->
<!-- revision: 1 -->
<a id="verification"></a>

# CGACP v3 local verification

The implementation starts from governance commit
`a83dd8b710d45a5ace1aabe9477ebd887c7f825e` and remains a local change. Windows x64 MSVC used Rust and Cargo
1.99.1 through the existing stable alias. Fixture checkouts were disposable.
The previous release and historical audit remain separate evidence.

The Windows command set used `cargo +stable fmt --all --check`,
`cargo +stable clippy --locked --offline --all-targets -- -D warnings`,
`cargo +stable check --locked --offline --all-targets`,
`cargo +stable build --locked --offline` and
`cargo +stable test --locked --offline`. CLI verification used the built
binary for `validate`, scoped `check-translations`, and
`workspace-discover`. Git diff whitespace and staging were checked separately.

| Acceptance | Test or evidence | Scope |
| --- | --- | --- |
| CGACP-01–04, CGACP-07 | Workspace and CLI fixtures | Offline discovery, explicit binding, an alternate linked Git worktree, one UI repo and exact producer/consumer context. |
| CGACP-05–06, CGACP-20 | Runner and workspace fixtures | Boundary and out-of-scope diff, dirty/rename/delete/untracked input, session delta, context bounds, lease timeout and cancellation. |
| CGACP-08–10 | Task and runner tests | Task and initiative links, CAS revisions, overlapping grants, expiry, authority changes and unsafe paths. |
| CGACP-11–14 | Process, task and CLI fixtures | Unsupported command denial, no raw secret output, authorized tool checks, stale test rejection and closure without release. High-risk authenticated acceptance remains blocked. |
| CGACP-15 | CLI fixture and Markdown tests | Independent worktree binding, no writes on preview, partial conflict, resume, repeat apply, heading boundary and final verification. |
| CGACP-16–17 | Release and workspace fixtures | Configurable train/family, exact matrix approval, rejected repackaging and explicit release separation. Product release was not run. |
| CGACP-18 | Cargo and validators | Binary and tests under src, registry validation, scoped and full EN/VI checks. |
| CGACP-19 | Portable adapter and CLI fixtures | Adapter lifecycle and task handoff preview. Native discovery and hooks remain NOT_RUN. |

## Resource observations

These scoped fixture measurements count UTF-8 source bodies, apart from the
JSON envelope. The timing is a local sample. A hit avoids resending source
content while the selected bytes are reread to detect changes.

| Packet | Product repos | Source files | Bytes read | Bytes emitted | Hits | Elapsed ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| UI cold | 1 | 4 | 60 | 60 | 0 | 92 |
| UI warm | 1 | 4 | 60 | 0 | 4 | 95 |
| API | 2 | 5 | 126 | 126 | 0 | 158 |
| RELEASE context | 2 | 6 | 154 | 126 | 0 | 149 |

The RELEASE context withheld one fixture secret and performed context
selection only. The token estimate divides emitted bytes by four and rounds
up. It measures neither token billing nor model retention. Context bytes,
source counts, output size and command time are bounded. The managed runner
uses one active run per task and zero automatic retries. CPU and RAM isolation
are external.

## Enforcement and review

The controlled runner denies unsupported executables, argument sets and
paths. Docs apply checks authorization, accepted binding and base bytes.
Git diff guards detect scoped changes after external edits. They cannot
observe arbitrary ignored files, unreported changes outside the tracked
scope or every semantic boundary. Owner-exclusive directories and the
authorized harness enforce isolation and identity outside managed calls.

A different reviewing Agent inspected lifecycle and release scope, reproduced
the Markdown heading issue and reported receipt and worktree gaps. Tests
cover the resulting guard changes. Low-risk acceptance requires a separate
authorization actor and fresh tool evidence. A local actor label is an
assertion. Native Agent discovery, hosted CI and hardware checks were not run.

[Operator workflow](cgacp-local.md) · [Historical verification](verification.md)
