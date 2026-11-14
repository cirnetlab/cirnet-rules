---
name: cirnet-repository-structure
description: Use when reviewing file ownership, repository layout, generated output, or a scoped file move.
---

# Repository structure

Use to locate or adjust actual boundaries. Use refactoring for code-only
restructuring. Keep language-specific layout that already works.

Minimum inputs: Repository purpose, entrypoints, owners, generated/protected paths, test/output policy and permitted files.

1. Map source, tests, docs, generated files and output ownership. Preserve fixtures, licenses and historical evidence.
2. Before a move, locate imports, links, scripts, CI, packaging and case-sensitive consumers. Establish any migration path.
3. Move within scope and update references together. Verify dependencies, entrypoints and output location.

Report the updated path map, references checked, protected artifacts and migration or limits.

Example: Move a generated binding's source only after locating the generator and all consumers. Regenerate output with the owner command.
