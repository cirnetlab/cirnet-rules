# Repository hooks

These checkpoints support one coding agent; they do not orchestrate agents or
register task state. Read English instructions. Vietnamese counterparts are for
human review, not an additional context layer.

| Checkpoint | Action |
| --- | --- |
| Before a task | Confirm root/branch/status and write scope; read local rules and relevant English context |
| After implementation/build | Identify documentation impact; update English and Vietnamese together; align plan/spec progress and actual results |
| Before completion | Review meaning, refresh reviewed translation metadata, run the check, inspect diff and report limits |

## Optional local executable check

Run these commands from the common-rules repository root with Cargo and Rust
1.88 or later.

```powershell
cargo run --offline --locked --bin check-docs
cargo test --offline --locked
git diff --check
```

The compiled checker defaults to the checkout where it was built. To check a
fixture or a specific common-rules checkout, pass `--root` after `--`.

```powershell
cargo run --offline --locked --bin check-docs -- --root PATH
```

The checker in `hooks/check_docs.rs` runs offline and is read-only. It and the
tests in `tests/check_docs.rs` use only the Rust standard library. Cargo places
build output in ignored `target/` by default. Tests write fixtures only under
this repo's ignored `.superpowers/bilingual-tests/`.

The check covers paired paths, translation source/hash freshness, UTF-8/LF,
whitespace, local Markdown links/fragments, the root baseline mirror, Claude
adapter and the three English skill definitions. Hash matching cannot prove meaning.
The checker exits with 0 for PASS, 1 for a failed check and 2 for invalid input.
A missing Rust toolchain leaves the executable check BLOCKED. Record manual
review separately.

## Connect to an agent later

The command can be run explicitly by Codex, Claude Code or Antigravity when the
task maintains this repository. For native lifecycle automation, consult the
installed client's hook support, preview the exact configuration and preserve
existing settings before the user authorizes installation. This change installs
no native/global hooks; automatic invocation on each client is NOT_RUN.

Do not apply this checker to an arbitrary product repo: it expects the common-rules
en/vi layout and entrypoints. Product checks and test-output allocators remain local.
See the [documentation policy](../policies/documentation.md) for the update contract.
