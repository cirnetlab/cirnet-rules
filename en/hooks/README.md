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
It also validates the six architecture snapshots against
[the source manifest](../../docs/architecture-sources.tsv) and checks EN/VI
[ownership routing](../references/architecture-routing.md) against the
canonical Ecosystem and Lab-First tables. It reports superseded architecture
links and warns about divergent same-named sources without editing them.
Snapshot checks preserve the reviewed source bytes; the regular formatting
and new-file inventories still cover root Markdown and the `en/` and `vi/`
trees. These are documentation checks. See
[architecture verification](../docs/toolkit-verification.md#architecture-compliance)
for product-owner tests and limits.

The portable frontmatter subset has exact `---` fences and one unindented
`name` and `description` each. Use a one-line plain string or single-quoted
literal with no escape syntax. Link required files in Markdown within the same
skill directory. Paths require exact casing and forward slashes. HTTP, HTTPS
and mailto citations are allowed. File URIs and out-of-skill paths fail validation.

Exit codes are 0 for PASS, 1 for ERROR diagnostics and 2 for invalid invocation.
WARN includes context budgets and similarity findings. INFO includes
measurements and file review guidance. A missing Rust toolchain leaves the
executable check BLOCKED. Record manual review separately.

To check a product checkout, pass `--product --root PATH`. This mode checks
root Markdown and the existing `docs/en`, `docs/vi` and `docs/changelogs`
trees with the same link, formatting and duplicate checks. It checks exact
target path casing and pairs files when the product uses bilingual trees.
Product files can use CRLF and Markdown hard line breaks made with two
spaces. Products can adopt translation hashes under their own policies. Existing
source metadata is verified when present. The default toolkit mode retains
its full Rules, Skills, version and architecture checks.

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
Exact matches compare bytes. Normalization accounts for CRLF, blank lines at
the end outside open code fences and valid Vietnamese source footers. LF
validation still applies. Matching Vietnamese text with distinct paired
English source hashes remains advisory. Near matches compare distinct
three-word prose sequences using Jaccard similarity, excluding fenced examples
and comments. Each document needs at least 40 words and 24 distinct sequences.
The threshold is 60%, or 40% for documents in the same folder with a shared
title, issue ID or filename family after trimming revision suffixes.

The checker reports paths, method, score, confidence, counts of unmatched
headings and word sequences, and detected incoming Markdown links. Review
`REVIEW_MERGE` and `UPDATE_EXISTING` candidates against the owner's purpose, unique content and
link targets. `KEEP_SEPARATE` flags paths recognized as historical evidence,
verification reports or release notes for preservation.
`NEW_DOCUMENT_REVIEW` asks the maintainer to decide when no suitable existing
candidate was found. Confirm a distinct purpose even when the checker suggests
no candidate. Byte-identical or conservatively normalized duplicates trigger
`DOC_NEW_DUPLICATE` (exit 1) when either or both paths are listed and neither
document is protected. Near matches remain advisory. The checker leaves all
files untouched. Reviewers determine purpose, ownership and consolidation scope.

CI runs `scripts/check-changed-docs.ps1` with the pull request base or the
pre-push commit. The script compares a verified local ancestor with checkout
HEAD and passes added Markdown as literal `--new-file` arguments. It skips
recognized renames, unsupported filenames and paths outside the checker's scan.
A missing or unusable base produces a warning and the ordinary scan. Locally,
pass a full commit SHA with `-Base` or use explicit `--new-file` for files
outside the committed diff.

## Connect to an agent later

The command can be run explicitly by Codex, Claude Code or Antigravity when the
task maintains this repository. For native lifecycle automation, consult the
installed client's hook support, preview the exact configuration and preserve
existing settings before the user authorizes installation. This change installs
no native/global hooks; automatic invocation on each client is NOT_RUN.

Do not apply this checker to an arbitrary product repo: it expects the common-rules
en/vi layout and entrypoints. Product checks and test-output allocators remain local.
See the [documentation policy](../policies/documentation.md) for the update contract.
