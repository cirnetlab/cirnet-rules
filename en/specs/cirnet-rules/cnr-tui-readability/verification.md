<!-- doc-id: CNR-TUI-READABILITY-VERIFICATION -->
<!-- revision: 1 -->
<a id="evidence"></a>

# CNR check and editor verification

The 2026-11-12 run used Windows, Rust and Cargo 1.99.1, and the working tree of `master` at HEAD `aca6afdbbb81f4996c98a0b22ab8d09e29aef58b`.

| Command | Final exit | Evidence |
| --- | --- | --- |
| `cargo fmt --all --check` | 0 | Format gate |
| `cargo clippy --locked --all-targets -- -D warnings` | 0 | Lint gate |
| `cargo check --locked --all-targets` | 0 | All targets checked |
| `cargo build --locked --bins` | 0 | Debug binaries in the dedicated target directory |
| `cargo test --locked --all-targets` | 0 | 258 passed, 1 ignored |
| `cargo build --locked --release --bins` | 0 | Optimized binaries |
| `git diff --check` | 0 | Diff whitespace gate |

The baseline Check used a four-column line with a maximum of 201 characters in the current workspace. The new Vietnamese plain report reached 84 characters on that fixture, with no ANSI. The same check reported 41 findings with 16 PASS, 6 BLOCKED and 19 NOT_RUN. Overall WARNING and exit 0 agreed with the schema 1 JSON. Bindings returned BLOCKED with exit 2, agent-config returned three NOT_RUN findings with exit 2, and the MCP check returned protocol PASS with native evidence pending.

Regression tests exercised 40x12, 60x20, 80x24, 120x30 and 160x40 frames, resize and 135 Unicode findings. They also covered status and repository filters, finding details, raw JSON exclusion from clipboard actions, grapheme selection, a bounded undo stack, oversized or multiline paste, clipboard failure, mouse hit testing and cancellation. The ignored test requires external uv and Python.

Windows Terminal 1.24.12741.0 ran the release binary headlessly with `WT_SESSION` present. `cnr check registry --json --non-interactive` returned PASS, schema 1 and exit 0. A separate Windows console PTY with `NO_COLOR` exercised the Vietnamese list, details, status filter, JSON mode, search and terminal restoration on exit. The native clipboard API was tested with codec cases and mock failures. Delivery of desktop clipboard shortcuts, physical right-click, host selection, IME input and Windows Terminal Preview behavior was not observed. Linux and WSL runtime were not run.

The first baseline test pass reached the document validator while the new plan hash was still being written. The final suite used synchronized files. Windows denied replacement of the existing default debug executable, so the successful debug build and full test run used a fresh target directory.

[Scope and layout decision](spec.md)
