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
adapter, toolkit version agreement and discovered English skill definitions.
The portable frontmatter subset has exact `---` fences and one unindented
`name` and `description` each. Use a one-line plain string or single-quoted
literal with no escape syntax. Link required files in Markdown within the same
skill directory. Paths require exact casing and forward slashes. HTTP, HTTPS
and mailto citations are allowed. File URIs and out-of-skill paths fail validation.

Exit codes are 0 for PASS, 1 for ERROR diagnostics and 2 for invalid invocation.
WARN includes context budgets and similarity findings. INFO includes
measurements and file review guidance. A missing Rust toolchain leaves the
executable check BLOCKED. Record manual review separately.

## Review new Markdown

After searching for the owning document, list each new Markdown path explicitly
to review it against discovered files. Paths are relative to the selected root
and use the spelling and case seen by the checker.

```powershell
cargo run --offline --locked --bin check-docs -- --new-file 'en/docs/New guide.md' --new-file 'vi/docs/New guide.md'
```

`--new-file` is repeatable and also accepts `--new-file=PATH`. Its scope is
existing root Markdown and the `en/` and `vi/` trees, including untracked files
when listed. Supply the paths yourself. The checker uses the same read-only
inventory and exits 2 for an invalid or undiscovered path.

The default check also reports similarity findings. It compares documents by
role within each of the root, English and Vietnamese groups. It skips intentional
entrypoints, templates, examples, test fixtures, standard license and attribution
files, empty or heading-only stubs, and README or index files under 40 words.
Exact matches compare bytes. Normalized matches account for CRLF,
terminal empty lines and the valid paired Vietnamese source footer, preserving
code, punctuation, numbers and status text. LF validation still applies.
Near matches use Jaccard similarity of distinct three-word sequences.
Each document needs at least 40 words and
24 distinct sequences. The threshold is 60%, or 40% with a shared title, issue
ID or same-directory filename family after trimming revision suffixes.

The checker reports paths, method, score, confidence, counts of unmatched
headings and word sequences, and detected incoming Markdown links. Review
`REVIEW_MERGE` and `UPDATE_EXISTING` candidates against the owner's purpose, unique content and
link targets. `KEEP_SEPARATE` flags paths recognized as historical evidence,
verification reports or release notes for preservation.
`NEW_DOCUMENT_REVIEW` asks the maintainer to decide when no suitable existing
candidate was found. Confirm a distinct purpose even when the checker suggests
no candidate. An exact match between a listed path and a matching
unlisted, unprotected document triggers `DOC_NEW_DUPLICATE` (exit 1).
Pairs with both paths listed remain advisory. The checker leaves all files
untouched. Human review establishes meaning, intent and consolidation scope.

## Connect to an agent later

The command can be run explicitly by Codex, Claude Code or Antigravity when the
task maintains this repository. For native lifecycle automation, consult the
installed client's hook support, preview the exact configuration and preserve
existing settings before the user authorizes installation. This change installs
no native/global hooks; automatic invocation on each client is NOT_RUN.

Do not apply this checker to an arbitrary product repo: it expects the common-rules
en/vi layout and entrypoints. Product checks and test-output allocators remain local.
See the [documentation policy](../policies/documentation.md) for the update contract.
