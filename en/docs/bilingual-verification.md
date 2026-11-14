# Bilingual upgrade verification

Date: 2026-11-14. This report concerns the bilingual upgrade from clean baseline
`17f1738ccd4312f284b61372a3c507a65a26588a`, not the earlier Governance rebuild.

## Implementation

English documentation lives under `en/`; every Markdown file has a matching
Vietnamese document under `vi/`. Root AGENTS is the identical English distribution
copy and CLAUDE imports it. Supporting content includes hooks, policies, plans,
specs, references, rules, three skills and templates. Instructions require paired
updates after implementation/build when documented meaning changes.

The optional `hooks/check_docs.py` is read-only, offline and uses the Python
standard library. It verifies file pairs and technical consistency; semantic
translation review remains a separate responsibility. No global/native hooks,
product changes, staging, commits or release actions were performed.

## Checks

- `python -B hooks/check_docs.py`: PASS on 26 English/Vietnamese document pairs.
  There are 61 maintained files, including 52 paired Markdown documents and four
  root Markdown entry/navigation/attribution files. The baseline has 91 lines.
- `python -B -m unittest discover -s tests -p 'test_*.py' -v`: exit 0;
  28 tests total, 27 PASS and 1 SKIP (NOT_RUN because Windows denied symlink creation).
  Tests cover stale/missing/orphan translations, metadata, links/fragments, traversal,
  excluded trees, skill definitions, entrypoint drift, invalid input and read-only behavior.
- The bundled `quick_validate.py` validated all three English skills: PASS, exit 0.
  `git diff --check`: PASS. Test fixture directories are empty after cleanup.
- Translation authors reviewed meaning and preserved technical paths, commands,
  SHA values, source links and evidence limits. Independent review sampled the
  translations, authority model and hook code; hashes alone do not certify translation.
- Review found a Windows case-variant bypass of excluded directories. It was
  reproduced, covered by two failing regression tests and fixed with consistent
  case-insensitive exclusion checks. The full 28-test suite then passed with the
  stated skip. No other important review findings remain.
- Native hook auto-invocation in Codex/Claude/Antigravity: NOT_RUN.
- Product runtime/build/physical verification: NOT_RUN; products are outside write scope.

HEAD remains `17f1738ccd4312f284b61372a3c507a65a26588a` on `rules-common`.
`master` and `governance-cli-only` remain at
`646afa38516bbfd276b0aecb7a138ab427df5fa9`. The index is unstaged.
LICENSE and existing Git configuration are preserved; the third-party notice now
also identifies the original optional checker without adding bundled dependencies.

The [initial rebuild report](rebuild-verification.md) is historical.
Its file/link counts and test results describe that earlier snapshot only.
