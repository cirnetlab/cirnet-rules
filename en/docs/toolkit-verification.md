# Toolkit upgrade verification

Date: 2026-11-14. Baseline revision `35e677b1264958839a7e0cb883c4477eb24da6df`.
Worktree branch: `rules-common`. The baseline checker passed and Cargo recorded
27 passing tests with one Windows directory symlink test ignored.

## Toolkit context contribution

Measured file bytes as UTF-8, words as whitespace-separated fields, and lines
as physical lines. Skill metadata includes parsed name and description values
only. Claude's adapter imports the baseline once. Its source is 11 bytes.

| Toolkit text | Before | Candidate |
| --- | --- | --- |
| Standalone common baseline | 5,811 bytes / 833 words / 91 lines | 2,444 bytes / 353 words / 42 lines |
| Metadata values for default three skills | 353 bytes | 302 bytes |
| Metadata values for all available skills | 353 bytes / 3 skills | 1,390 bytes / 12 skills |
| Optional fallback index | No dedicated file | 984 bytes / 77 words |

The candidate uses a single English baseline, optional metadata and selected
skill bodies. Walkthroughs inspected the typo, API and FPGA choices in
[context loading](context.md). Client context, tokenizer results and automatic
trigger selection need native observations. The table measures toolkit text.

## Local checks

Local environment was Windows with PowerShell 7.6.9 and Rust 1.99.1.

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` and `cargo clippy --offline --locked --all-targets -- -D warnings` | PASS |
| `cargo test --offline --locked` | 41 passed, 1 Windows symlink case ignored |
| `cargo run --offline --locked --bin check-docs` | PASS, 45 EN/VI pairs and 12 skills, no warnings |
| `pwsh -NoProfile -File tests/package.Tests.ps1` | PASS for exercised fixtures, Windows junction creation blocked |
| `pwsh -NoProfile -File tests/install.tests.ps1` | PASS for exercised fixtures, Windows junction creation blocked. Actual partial-write rollback restored bytes and timestamps |
| `git diff --check` and PowerShell script parsing | PASS |

An additional smoke run packaged a development ZIP, extracted it to a Unicode
path, verified the 15 allowlisted payload files and exercised preview, apply,
verify and restore in separate fake homes for Codex, Claude and Antigravity.
It covered the explicit Codex home option. The test data was allocated in
ignored worktree output. Review and tests also covered invalid receipt state,
bundle overlap, duplicate Markdown references and binary baseline input.

Clean release creation is NOT_RUN on this dirty candidate worktree. Negative
release tests reject missing or incorrect revision and dirty source. No hosted
CI, Linux, actual client discovery or physical/product execution was run.
The candidate has no tag or published release.

See [setup](setup.md) for manual client smoke cases and
[distribution](distribution.md) for bundle commands.

## Markdown hygiene verification

On 2026-11-15, the `rules-common` worktree at `5d6cc26` extended the existing
checker and documentation lifecycle. The first duplicate fixture failed against
the old checker because it emitted no duplicate diagnostic. All eight hygiene
tests subsequently passed, covering duplicate classes, scoped new-file review,
intentional copies, historical evidence and unchanged fixture bytes.

| Command | Observed result |
| --- | --- |
| `cargo fmt --all --check` | Exit 0 |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Exit 0 |
| `cargo test --offline --locked` | Exit 0. 49 passed, one Windows directory symlink test ignored. |
| `cargo run --offline --locked --bin check-docs` | Exit 0. Zero checker warnings and no cross-document candidates on this checkout. |
| `pwsh -NoProfile -File tests/package.Tests.ps1` | Exit 0 for the exercised package and fake-home checks. Windows reparse fixture NOT_RUN because creation was denied. |
| `pwsh -NoProfile -File tests/install.tests.ps1` | Exit 0 for the exercised installer fixtures. |
| `git diff --check` | Exit 0 |

The first parallel Cargo run hit a Windows executable lock and exited 101.
The sequential retry used the same source. The repeatable `--new-file` smoke
check used existing paths to exercise the CLI and exited 0. The policy path
reported `NEW_DOCUMENT_REVIEW` without a candidate, and the spec template
reported `KEEP_SEPARATE`. Both are review guidance.

Similarity measures lexical overlap within the scan. Maintainers decide
meaning and ownership. Evidence is under `.superpowers/markdown-hygiene-5d6cc26/`.
The verification scope was the checker and local bundle fixtures. E01–E08
were NOT_RUN for this change and their retained evidence hashes matched.

## Final hygiene hardening

On 2026-11-15, the `rules-common` change based on `66d2c8a` hardened duplicate
severity, added CI selection and extended the baseline to 491 words and
3,445 UTF-8 bytes. Seven focused regressions reproduced the old behavior.
All 15 hygiene tests passed after the fix. The default checker reported no
warnings on the checkout.

| Check | Observed result on Windows |
| --- | --- |
| `cargo fmt --all --check`, `git diff --check` | Exit 0 |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Exit 0 |
| `cargo test --offline --locked` | Exit 0. 56 passed and one Windows directory symlink test ignored. |
| `cargo run --offline --locked --bin check-docs` | Exit 0. Links, EN/VI hashes and baseline mirror passed. |
| `pwsh -NoProfile -File tests/check-changed-docs.tests.ps1` | Exit 0. The bridge passed committed additions, preserved literal filenames, skipped recognized renames, propagated the duplicate error and reported fallback for unusable bases. |
| `tests/package.Tests.ps1` and `tests/install.tests.ps1` | Exit 0 for exercised fixtures. Reparse and junction creation were denied. |

Evidence is under `.superpowers/markdown-hardening-66d2c8a/`. Hashes of 290
protected input and evidence files and all branch refs matched. Hosted CI,
Linux execution, native agent loading and E01–E08 were NOT_RUN for this change.
The Git bridge requires a confirmed local ancestor. Similarity still needs
review of purpose and ownership.
