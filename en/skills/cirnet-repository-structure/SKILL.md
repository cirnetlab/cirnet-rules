---
name: cirnet-repository-structure
description: Use when reviewing file ownership, repository layout, generated output, or a scoped file move.
---

# Repository structure

Use to locate or adjust actual boundaries. Use refactoring for code-only
restructuring. Keep language-specific layout that already works.

Minimum inputs: Repository purpose, entrypoints, owners, generated/protected paths, test/output policy and permitted files.

1. Map ownership of source, tests, docs, generated files and output. Before
   adding a path, find the existing owner and location for its purpose. Search
   the authorized repository by path, heading, topic, issue ID and incoming links.
2. Before moving or consolidating, trace imports, inbound links, scripts, CI,
   packaging and case-sensitive consumers. Identify any migration needed to
   preserve fixtures, templates, licenses, compatibility versions, historical
   evidence and access to it. Keep plans and specs with distinct purposes separate.
3. Apply authorized changes and update references together. Verify dependencies,
   entrypoints and output location. Use documentation sync for bilingual text.

Report the updated path map, references checked, protected artifacts and migration or limits.

Example: Move a generated binding's source only after locating the generator and all consumers. Regenerate output with the owner command.
