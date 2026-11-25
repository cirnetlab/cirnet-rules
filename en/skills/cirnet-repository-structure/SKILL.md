---
name: cirnet-repository-structure
description: Use when reviewing repository or module organization, file ownership, generated output, or a scoped file move.
---

# Repository structure

Use to locate or adjust actual boundaries. Use refactoring for code-only
restructuring. Keep language-specific layout that already works.
Choose AUDIT for review requests or when write scope is unclear.
Choose MODIFY when the requested change is explicitly authorized.

Use the owner's current instructions and applicable architecture. Consult the
proposal at `en/rules/repository-organization.md` and, when useful, the optional
map at `en/templates/repository-map.md`. Read these resources from an approved
`cirnet-rules` checkout. Report unavailable references and continue with the
owner evidence in scope.

## AUDIT

Keep inspection read-only. Map the repository, domain, feature and module
responsible for relevant paths, outputs and evidence. Distinguish architecture
targets from current authority. Search authorized paths, symbols, documents
and tests, then inspect direct callers before recommending reuse or additions.
Report findings in the response with path and revision evidence, the confirmed
owner or unresolved ownership, duplication or dependency risk, and the
proposed action. State the evidence or owner decision needed for any gap.
Keep proposed cleanup, deletion or restructuring in the findings.

## MODIFY

Proceed with explicit authorization for the affected paths and operations.
If ownership or permission is unresolved, report the dependency and hold the
affected change. Before adding paths, use the ownership and search checks
from AUDIT. Review the migration and trace callers, imports, exports,
build files, scripts, tests, CI, packaging, documentation and case-sensitive
consumers. Apply the smallest authorized change and update affected references
together. Preserve user edits, untracked data, generator sources and protected
fixtures, templates, licenses, compatibility versions and historical evidence.
Keep plans and specs in their owner locations.

Run focused checks for dependencies, entrypoints, tests and output locations.
Follow documentation sync for bilingual changes. Report changed paths,
protected artifacts, migration steps and observed check results and limits.

Example: Move a generated binding's source only after locating the generator and all consumers. Regenerate output with the owner command.
